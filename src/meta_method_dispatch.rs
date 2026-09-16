//! Registry-backed method execution for the universal solver.
//!
//! The meta core owns method selection: an impulse is formalized, its
//! `route:`/`handler:` relevants are resolved through [`MethodRegistry`], and the
//! ordered method names are executed here. Handler functions remain ordinary Rust
//! implementations, but they are no longer selected by a separate hardcoded loop
//! in `solver.rs`.

use std::fmt::Write as _;

use crate::capability_routing::{ObjectType, RoutingOutcome};
use crate::engine::{SymbolicAnswer, answer_links_notation};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::IntentFormalization;
use crate::method_registry::MethodRegistry;
use crate::proof_engine::ProofRenderConfig;
use crate::solver::{ConversationTurn, SolverConfig, UniversalSolver};
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_dispatch::{
    ContextualOutcome, ContextualRuntime, PRELUDE_METHOD_NAMES, handler_for_method,
    try_contextual_override,
};
use crate::solver_handlers::{
    CapabilityRuntime, SelfAwarenessRuntime, finalize_simple, try_behavior_rules_with_runtime,
    try_concept_lookup_with_response_language, try_explicit_repository_lookup,
    try_feature_capability, try_natural_language_tool_request,
    try_pattern_inference_with_response_language, try_playwright_script, try_project_lookup,
    try_project_lookup_with_response_language, try_response_language_followup,
    try_routed_calendar_create_event, try_routed_http_fetch_with_offline,
};
use crate::translation::detect_response_language;

/// Execute the single registry-backed method-selection path.
pub fn try_dispatch(
    solver: &UniversalSolver,
    prompt: &str,
    intent_formalization: &IntentFormalization,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let normalized = prompt.to_lowercase();
    let registry = MethodRegistry::from_dispatch();
    let promoted_method = intent_formalization.relevants.iter().find_map(|relevant| {
        let route = relevant
            .strip_prefix("route:")
            .or_else(|| relevant.strip_prefix("handler:"))?;
        registry
            .method_for_route(route)
            .map(|method| method.name.clone())
    });
    let method_names = registry.ordered_method_names_for_relevants(&intent_formalization.relevants);
    let runtime = MethodRuntime::new(solver.config);
    let mut capability_route_checked = false;

    // Issue #556: a response-language follow-up replays the previous request
    // through the whole solver with this language forced onto every localizable
    // answer family, so the retarget generalizes beyond a single handler.
    let forced_response_language = solver.config.forced_response_language;
    for name in method_names {
        // Prelude methods keep first refusal: they include explicit natural-
        // language tool requests and policy-like assistant capabilities that
        // already own complete recipes.  The shared capability table then
        // chooses the first regular method, before a promoted web-search
        // handler can reinterpret a URL, path, clock, language or local scope.
        if !capability_route_checked && !PRELUDE_METHOD_NAMES.contains(&name.as_str()) {
            capability_route_checked = true;
            if let Some(answer) =
                try_capability_route(solver, prompt, history, promoted_method.as_deref(), log)
            {
                return Some(answer);
            }
        }
        if matches!(name.as_str(), "feature_capability" | "capabilities")
            && let Some(answer) = try_explicit_repository_lookup(
                prompt,
                &normalized,
                log,
                solver.config.associative_project_promotion,
                intent_formalization.route.as_deref() == Some("identity"),
                forced_response_language,
            )
        {
            return Some(record_method_answer(prompt, log, answer, "project_lookup"));
        }
        if let Some(answer) = try_prelude_method(solver, &name, prompt, &normalized, log, runtime) {
            return Some(answer);
        }
        if solver.config.definition_fusion_by_default
            && name == "concept_lookup"
            && let Some(answer) = crate::definition_merge::merge_definitions_by_default(prompt, log)
        {
            return Some(record_method_answer(
                prompt,
                log,
                answer,
                "definition_merge_by_default",
            ));
        }
        match try_contextual_override(
            &name,
            prompt,
            &normalized,
            history,
            ContextualRuntime::new(
                runtime.proof_render_config,
                runtime.self_awareness_runtime,
                solver.config,
            ),
            log,
        ) {
            ContextualOutcome::Answer(answer) => {
                return Some(record_contextual_method_answer(prompt, log, answer, &name));
            }
            ContextualOutcome::Skip => continue,
            ContextualOutcome::NotHandled => {}
        }
        // Issue #556: when a language is forced, route the concept-lookup family
        // through its response-language variant so a replayed definitional
        // request re-renders in the requested language before the plain handler
        // (which localizes only to the detected prompt language) can claim it.
        if name == "concept_lookup"
            && let Some(language) = forced_response_language
            && let Some(answer) =
                try_concept_lookup_with_response_language(prompt, log, Some(language))
        {
            return Some(record_method_answer(prompt, log, answer, "concept_lookup"));
        }
        // Issue #531/#556: when a language is forced, render the pattern-inference
        // report in that language so a replayed "find the pattern" request no
        // longer strands its answer in English.
        if name == "pattern_inference"
            && let Some(language) = forced_response_language
            && let Some(answer) =
                try_pattern_inference_with_response_language(prompt, &normalized, log, language)
        {
            return Some(record_method_answer(
                prompt,
                log,
                answer,
                "pattern_inference",
            ));
        }
        // Issue #1138 B7, plan 07 leaf 6: a learned name is executed as the
        // recipe program its operations project onto, instead of through
        // `handler_for_method` -- there is no compiled handler behind it. This
        // is `MethodRegistry::learned_method`'s first production caller.
        if let Some(answer) =
            try_learned_method(solver, &registry, &name, prompt, intent_formalization, log)
        {
            return Some(answer);
        }
        if let Some(handler) = handler_for_method(&name)
            && let Some(answer) = handler.call(prompt, &normalized, log)
        {
            return Some(record_method_answer(prompt, log, answer, &name));
        }
        if name == "concept_lookup" {
            let answer = if let Some(language) = forced_response_language {
                try_project_lookup_with_response_language(
                    prompt,
                    prompt,
                    log,
                    solver.config.associative_project_promotion,
                    intent_formalization.route.as_deref() == Some("identity"),
                    language,
                )
            } else {
                try_project_lookup(
                    prompt,
                    &normalized,
                    log,
                    solver.config.associative_project_promotion,
                    intent_formalization.route.as_deref() == Some("identity"),
                )
            };
            if let Some(answer) = answer {
                return Some(record_method_answer(prompt, log, answer, "project_lookup"));
            }
        }
    }
    None
}

/// Capabilities the symbolic solver can execute without a client-owned
/// workspace tool.  Filesystem capabilities are deliberately absent: when the
/// table selects one, its `HonestGap` is surfaced instead of allowing a later
/// web-search handler to claim the request.
const SOLVER_CAPABILITIES: &[&str] = &[
    "web_fetch",
    "web_search",
    "calendar_create_event",
    "response_language_demonstration",
    "concept_measurement_lookup",
    "compose_from_sources",
    "explain_previous_turn",
    "report_issue",
    "ask_user",
];

/// Execute the object/act/locus decision on the non-agent solver surface.
///
/// This is intentionally a dispatcher, not another recognizer.  All lexical
/// interpretation happened in `capability_routing`; this function records the
/// selected axes and either invokes the matching runtime capability or reports
/// that the chat surface lacks the client-owned tool it needs.
fn try_capability_route(
    solver: &UniversalSolver,
    prompt: &str,
    history: &[ConversationTurn],
    promoted_method: Option<&str>,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !crate::capability_routing::table_routing_enabled() {
        return None;
    }
    let decision = crate::capability_routing::route_decision(prompt, SOLVER_CAPABILITIES);
    if !solver_route_is_authoritative(prompt, &decision) {
        return None;
    }

    // A specifically promoted interpreter keeps the request it already
    // grounded.  The table replaces generic fetch/search/clarification
    // precedence that caused #745/#1138's cross-tool misroutes; it does not
    // interrupt translation, text manipulation, installation conversion, or
    // any other complete recipe merely because that recipe mentions a path or
    // a language.
    if let Some(method) = promoted_method {
        if !matches!(
            method,
            "http_fetch"
                | "url_navigate"
                | "web_search"
                | "calendar_create_event"
                | "clarification"
                | "conversation_memory"
                | "response_language_followup"
        ) {
            return None;
        }
    }

    let (capability, unavailable) = match &decision.outcome {
        RoutingOutcome::Routed { capability } => (capability.as_str(), None),
        RoutingOutcome::Lowered {
            preferred,
            capability,
        } => (capability.as_str(), Some(preferred.as_str())),
        RoutingOutcome::HonestGap { needed, missing } => {
            record_capability_route(log, decision.object, decision.act, decision.locus, needed);
            return Some(capability_gap(prompt, log, needed, missing));
        }
        RoutingOutcome::Ask { .. } => return None,
    };
    record_capability_route(
        log,
        decision.object,
        decision.act,
        decision.locus,
        capability,
    );
    if let Some(preferred) = unavailable {
        log.append(
            "capability_route:lowered",
            format!("preferred={preferred} capability={capability}"),
        );
    }

    let normalized = prompt.to_lowercase();
    let live_fetch = std::env::var("FORMAL_AI_LIVE_FETCH").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    });
    match capability {
        "web_fetch" => {
            try_routed_http_fetch_with_offline(prompt, log, solver.config.offline || !live_fetch)
        }
        "calendar_create_event" => try_routed_calendar_create_event(prompt, &normalized, log),
        "response_language_demonstration" => {
            if let Some(answer) =
                try_response_language_followup(prompt, &normalized, log, history, solver.config)
            {
                return Some(record_contextual_method_answer(
                    prompt,
                    log,
                    answer,
                    "response_language_followup",
                ));
            }
            response_language_demonstration(prompt, &normalized, log)
        }
        "explain_previous_turn" => Some(explain_previous_turn(prompt, log)),
        // These capabilities already have general registry methods.  Recording
        // the table decision here changes their precedence without duplicating
        // their execution or interrupting a multi-step research recipe.
        "web_search"
        | "compose_from_sources"
        | "concept_measurement_lookup"
        | "report_issue"
        | "ask_user" => None,
        _ => Some(capability_gap(prompt, log, capability, capability)),
    }
}

/// Whether the table should preempt regular symbolic-handler precedence.
///
/// A typed/named object is authoritative.  A bare term is not: definitions and
/// other local symbolic answers keep their turn unless the prompt explicitly
/// names the open web, lands in the workspace, or asks to compose from sources.
fn solver_route_is_authoritative(
    prompt: &str,
    decision: &crate::capability_routing::RoutingDecision,
) -> bool {
    match decision.object {
        ObjectType::BareTerm => {
            decision.locus == crate::capability_routing::Locus::Workspace
                || decision.act == crate::capability_routing::Act::Compose
                || crate::capability_routing::names_open_web(prompt)
        }
        ObjectType::None => false,
        _ => true,
    }
}

fn record_capability_route(
    log: &mut EventLog,
    object: ObjectType,
    act: crate::capability_routing::Act,
    locus: crate::capability_routing::Locus,
    capability: &str,
) {
    log.append(
        "capability_route",
        format!(
            "object={} act={} locus={} capability={capability}",
            object.slug(),
            act.slug(),
            locus.slug(),
        ),
    );
}

fn capability_gap(prompt: &str, log: &mut EventLog, needed: &str, missing: &str) -> SymbolicAnswer {
    log.append(
        "capability_gap",
        format!("needed={needed} missing={missing}"),
    );
    let body = format!(
        "This request routes to the `{needed}` capability, but this chat surface does not \
         expose the required `{missing}` tool. Use an agent client that advertises it."
    );
    finalize_simple(
        prompt,
        log,
        "capability_gap",
        "response:capability_gap",
        &body,
        1.0,
    )
}

fn response_language_demonstration(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let target = detect_response_language(normalized)?;
    let canonical = crate::language::language_name(target).unwrap_or(target);
    let surface = crate::seed::lexicon()
        .meanings_with_role(crate::seed::ROLE_RESPONSE_LANGUAGE_MARKER)
        .find(|meaning| {
            meaning.defined_by.iter().any(|slug| {
                crate::language::language_for_concept_slug(slug)
                    == crate::language::from_slug(target)
            })
        })
        .and_then(|meaning| meaning.words().find(|word| normalized.contains(word)))
        .unwrap_or(canonical);
    let demonstration =
        crate::seed::response_for("greeting", target).unwrap_or_else(|| canonical.to_owned());
    log.append("language_to", target.to_owned());
    let body = format!("{canonical} ({surface}): {demonstration}");
    Some(finalize_simple(
        prompt,
        log,
        "response_language_demonstration",
        "response:response_language_demonstration",
        &body,
        1.0,
    ))
}

fn explain_previous_turn(prompt: &str, log: &mut EventLog) -> SymbolicAnswer {
    let body = crate::solver_helpers::last_assistant_turn(log).map_or_else(
        || String::from("There is no previous assistant turn available to explain yet."),
        |previous| format!("Here is the previous answer in a simpler form:\n\n{previous}"),
    );
    finalize_simple(
        prompt,
        log,
        "explain_previous_turn",
        "response:explain_previous_turn",
        &body,
        0.9,
    )
}

/// Execute one adopted learned method as the recipe program its learned
/// operations project onto (issue #1138 B7, plan 07 leaf 6).
///
/// A learned abstraction has no compiled handler: its `operations` are the
/// recorder event kinds `crate::recipe_interpreter` already dispatches on, so
/// the method runs by executing that program rather than by calling Rust that
/// was written for it. A name that is not a learned record, an operation that
/// binds to no recorder, and a misordered program are all reported in the trace
/// under `method:learned` and then declined, never silently skipped.
fn try_learned_method(
    solver: &UniversalSolver,
    registry: &MethodRegistry,
    name: &str,
    prompt: &str,
    intent_formalization: &IntentFormalization,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let learned = registry.learned_method(name)?;
    let program = match learned.to_recipe_program() {
        Ok(program) => program,
        Err(reason) => {
            log.append("method:learned", format!("{name}:{reason}"));
            return None;
        }
    };
    let trace = match program.execute(
        intent_formalization,
        solver.config.max_decomposition_depth,
        solver.config.recursion_mode,
        solver.config.selection_mode,
        solver.config.skill_mode,
    ) {
        Ok(trace) => trace,
        Err(reason) => {
            log.append("method:learned", format!("{name}:{reason}"));
            return None;
        }
    };
    log.append("method:learned", name.to_owned());
    let mut rendered = program.to_links_notation();
    for id in &trace.executed {
        let _ = write!(rendered, "\n  executed {id}");
    }
    for id in &trace.skipped {
        let _ = write!(rendered, "\n  skipped {id}");
    }
    let intent = format!("learned_method:{name}");
    log.append("intent", intent.clone());
    let response_link = format!("response:learned_method:{name}");
    log.append("response", response_link.clone());
    let trace_id = log.append("trace", intent.clone());
    let evidence_links = build_evidence_links(prompt, log, &response_link);
    let links_notation = answer_links_notation(prompt, &intent, &rendered, log, &trace_id);
    let thinking_steps = log.thinking_steps_for_answer(&rendered);
    Some(record_method_answer(
        prompt,
        log,
        SymbolicAnswer {
            intent,
            answer: rendered,
            execution_recipe: None,
            confidence: solver.config.guess_probability,
            evidence_links,
            thinking_steps,
            links_notation,
        },
        name,
    ))
}

#[derive(Clone, Copy)]
struct MethodRuntime {
    proof_render_config: ProofRenderConfig,
    capability_runtime: CapabilityRuntime,
    self_awareness_runtime: SelfAwarenessRuntime,
}

impl MethodRuntime {
    const fn new(config: SolverConfig) -> Self {
        Self {
            proof_render_config: ProofRenderConfig {
                guess_probability: config.guess_probability,
                follow_up_probability: config.follow_up_probability,
            },
            capability_runtime: CapabilityRuntime::new(
                config.offline,
                config.agent_mode,
                config.diagnostic_mode,
                config.definition_fusion_by_default,
            ),
            self_awareness_runtime: SelfAwarenessRuntime::new(
                config.execution_surface,
                config.offline,
                config.agent_mode,
                config.diagnostic_mode,
                config.definition_fusion_by_default,
                config.blueprint_composition,
            ),
        }
    }
}

fn try_prelude_method(
    solver: &UniversalSolver,
    name: &str,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: MethodRuntime,
) -> Option<SymbolicAnswer> {
    let answer = match name {
        "diagnostic" => try_diagnostic(solver, prompt, normalized, log),
        "nl_tool" => {
            try_natural_language_tool_request(prompt, normalized, log, solver.config.agent_mode)
        }
        "behavior_rules" => {
            try_behavior_rules_with_runtime(prompt, normalized, log, runtime.self_awareness_runtime)
        }
        "feature_capability" => {
            try_feature_capability(prompt, normalized, log, runtime.capability_runtime)
        }
        "playwright_script" => {
            try_playwright_script(prompt, normalized, log, solver.config.guess_probability)
        }
        _ => return None,
    }?;
    Some(record_method_answer(prompt, log, answer, name))
}

fn try_diagnostic(
    solver: &UniversalSolver,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("[diagnostic]") {
        return None;
    }
    log.append("diagnostic_mode", "active".to_owned());
    let stripped = prompt.replace("[diagnostic]", "").trim().to_owned();
    let inner_solver = UniversalSolver::new(solver.config);
    let inner = inner_solver.solve(&stripped);
    let mut decorated = inner.answer.clone();
    decorated.push_str("\n\n[diagnostic]\n");
    decorated.push_str(inner.links_notation.trim_end());
    decorated.push('\n');
    for link in &inner.evidence_links {
        let _ = writeln!(decorated, "evidence: {link}");
    }
    let _ = writeln!(decorated, "trace: {}", inner.intent);
    log.append("intent", inner.intent.clone());
    let response_link = format!("response:diagnostic:{}", inner.intent);
    log.append("response", response_link.clone());
    let trace_id = log.append("trace", inner.intent.clone());
    let evidence_links = build_evidence_links(prompt, log, &response_link);
    let links_notation = answer_links_notation(prompt, &inner.intent, &decorated, log, &trace_id);
    let thinking_steps = log.thinking_steps_for_answer(&inner.answer);
    let answer = append_diagnostic_trace(solver.config.diagnostic_mode, decorated, &links_notation);
    Some(SymbolicAnswer {
        intent: inner.intent,
        answer,
        execution_recipe: inner.execution_recipe,
        confidence: inner.confidence,
        evidence_links,
        thinking_steps,
        links_notation,
    })
}

fn record_method(log: &mut EventLog, name: &str) {
    log.append("method", name.to_owned());
}

fn record_method_answer(
    prompt: &str,
    log: &mut EventLog,
    answer: SymbolicAnswer,
    name: &str,
) -> SymbolicAnswer {
    record_method(log, name);
    refresh_answer_projection(prompt, log, answer)
}

fn record_contextual_method_answer(
    prompt: &str,
    log: &mut EventLog,
    answer: SymbolicAnswer,
    name: &str,
) -> SymbolicAnswer {
    log.append("method", name.to_owned());
    refresh_answer_projection(prompt, log, answer)
}

fn refresh_answer_projection(
    prompt: &str,
    log: &EventLog,
    mut answer: SymbolicAnswer,
) -> SymbolicAnswer {
    let Some(response_link) = log.last_of("response").map(|event| event.payload.clone()) else {
        return answer;
    };
    let Some(trace_id) = log.last_of("trace").map(|event| event.id.clone()) else {
        return answer;
    };
    answer.evidence_links = build_evidence_links(prompt, log, &response_link);
    answer.links_notation =
        answer_links_notation(prompt, &answer.intent, &answer.answer, log, &trace_id);
    answer.thinking_steps = log.thinking_steps_for_answer(&answer.answer);
    answer
}
