//! Registry-backed method execution for the universal solver.
//!
//! The meta core owns method selection: an impulse is formalized, its
//! `route:`/`handler:` relevants are resolved through [`MethodRegistry`], and the
//! ordered method names are executed here. Handler functions remain ordinary Rust
//! implementations, but they are no longer selected by a separate hardcoded loop
//! in `solver.rs`.

use std::fmt::Write as _;

use crate::capability_routing::{Act, ObjectType, RoutingOutcome};
use crate::engine::{SymbolicAnswer, answer_links_notation};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::IntentFormalization;
use crate::method_registry::{
    MethodExecution, MethodRegistry, MethodRuntimeKind, ProjectLookupPhase, ResponseLanguageVariant,
};
use crate::proof_engine::ProofRenderConfig;
use crate::solver::{ConversationRole, ConversationTurn, SolverConfig, UniversalSolver};
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
    let registry = MethodRegistry::shared();
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
    //
    // Plan 10 leaf 15 (issue #724): a language the conversation has already
    // established binds the same way. The user names it once -- a routed
    // demonstration, a retarget, a plain request -- and the conversation keeps
    // speaking it until they name another; an explicit per-run forcing in the
    // config is the stronger statement and wins.
    let forced_response_language = solver
        .config
        .forced_response_language
        .or_else(|| established_response_language(history));
    for name in method_names {
        let execution = registry.execution_for(&name);
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
        if let Some(answer) = crate::family_method::try_family_method_preempting(
            prompt,
            &normalized,
            history,
            log,
            &name,
            solver.config,
        ) {
            return Some(answer);
        }
        if execution.project_lookup == Some(ProjectLookupPhase::BeforeRuntime)
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
        if let Some(answer) =
            try_attributed_runtime(solver, execution, &name, prompt, &normalized, log, runtime)
        {
            return Some(answer);
        }
        if solver.config.definition_fusion_by_default
            && execution.definition_fusion
            && let Some(answer) = crate::definition_merge::merge_definitions_by_default(prompt, log)
        {
            return Some(record_method_answer(
                prompt,
                log,
                answer,
                "definition_merge_by_default",
            ));
        }
        // Family interpreters replace the generic unknown/concept fallback,
        // not the grounded methods ordered ahead of it.  This data-selected
        // boundary preserves richer history, source, policy, and execution
        // semantics while still letting a family claim requests none of those
        // methods could solve.
        if execution.project_lookup == Some(ProjectLookupPhase::Fallback)
            && let Some(answer) = crate::family_method::try_family_method(
                prompt,
                &normalized,
                history,
                log,
                solver.config,
            )
        {
            return Some(answer);
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
        if let Some(language) = forced_response_language
            && let Some(answer) = try_response_language_variant(
                execution.response_language,
                prompt,
                &normalized,
                log,
                language,
            )
        {
            return Some(record_method_answer(prompt, log, answer, &name));
        }
        // Issue #1138 B7, plan 07 leaf 6: a learned name is executed as the
        // recipe program its operations project onto, instead of through
        // `handler_for_method` -- there is no compiled handler behind it. This
        // is `MethodRegistry::learned_method`'s first production caller.
        if let Some(answer) =
            try_learned_method(solver, registry, &name, prompt, intent_formalization, log)
        {
            return Some(answer);
        }
        if let Some(handler) = handler_for_method(&name)
            && let Some(answer) = handler.call(prompt, &normalized, log)
        {
            return Some(record_method_answer(prompt, log, answer, &name));
        }
        if execution.project_lookup == Some(ProjectLookupPhase::Fallback) {
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
    // A request for unbounded autonomy is the bounded-autonomy policy's to
    // answer, whatever object/act/locus triple the table reads off it.
    // Before the word-boundary fix (issue #1138 plan 03 wave F) such a
    // prompt reached that policy only through a substring false positive
    // ("improve" embedding "prove") promoting a proof handler; with the
    // false positive gone, the table must decline on its own instead of
    // answering a grep capability gap to "improve my codebase forever".
    // An explicit agent opt-in is likewise the agent flow's to serve: the
    // table routes chat-surface requests, and an opted-in agent request
    // never becomes a chat capability gap (#1138).
    let normalized = prompt.to_lowercase();
    if crate::solver_helpers::is_unbounded_autonomy(&normalized)
        && !crate::solver_helpers::is_agent_opt_in(&normalized)
    {
        return None;
    }
    if crate::solver_helpers::is_agent_request(&normalized) {
        return None;
    }
    // A conversion between units the lexicon places in distinct physical
    // dimensions (meters of length, kilobytes of data storage) is the unit
    // specialist's to answer, whatever object/act/locus triple the table reads
    // off it. The quantity rows exist so a magnitude question reaches its
    // measured property; a pair of units that cannot convert has none, and
    // routing it to a web capability reports a capability gap where the honest
    // answer is the recorded incompatibility. Same shape as the autonomy
    // policy above: the table declines on its own instead of a specialist
    // having to win a race it no longer runs (issue #1138).
    if crate::solver_handler_units::names_incompatible_unit_pair(&normalized) {
        return None;
    }
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
    if let Some(method) = promoted_method
        && !capability_route_preempts(method, &decision)
    {
        return None;
    }

    let (capability, unavailable) = match &decision.outcome {
        RoutingOutcome::Routed { capability } => (capability.as_str(), None),
        RoutingOutcome::Lowered {
            preferred,
            capability,
        } => (capability.as_str(), Some(preferred.as_str())),
        RoutingOutcome::HonestGap { needed, missing } => {
            // An agent client advertises the workspace tools itself (issue #671:
            // `read the file alpha.txt` must reach the planner, never dead-end in
            // a refusal). The gap refusal is the plain chat surface's honest
            // answer, and only its.
            if solver.config.agent_mode {
                return None;
            }
            if let Some(answer) = crate::family_method::try_family_method_preempting(
                prompt,
                &normalized,
                history,
                log,
                "capability_gap",
                solver.config,
            ) {
                return Some(answer);
            }
            record_capability_route(log, decision.object, decision.act, decision.locus, needed);
            return Some(capability_gap(prompt, log, needed, missing));
        }
        RoutingOutcome::Ask { .. } => return None,
    };
    // Capability routing and handler-family routing are both data-selected
    // interpretations of the same request.  Let the family catalog arbitrate
    // a routed capability by its declared `preempts` relation before the
    // capability executes.  Without this shared boundary, adding a capability
    // can silently steal an older family's requests merely because capability
    // routing runs earlier in the dispatcher.
    if let Some(answer) = crate::family_method::try_family_method_preempting(
        prompt,
        &normalized,
        history,
        log,
        capability,
        solver.config,
    ) {
        return Some(answer);
    }
    record_capability_route(
        log,
        decision.object,
        decision.act,
        decision.locus,
        capability,
    );
    if let Some(preferred) = unavailable {
        log.append_fields(
            "capability_route:lowered",
            &[("preferred", preferred), ("capability", capability)],
        );
    }

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
        "explain_previous_turn" => {
            // Issue #556 first: re-rendering the previous turn must not steal
            // an explicit response-language retarget OF that same previous
            // turn ("I don't understand English, write in Hindi" is both a
            // comprehension failure and a retarget; the retarget is the
            // request). Without a marker the act re-renders as usual.
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
            Some(explain_previous_turn(prompt, log))
        }
        "compose_from_sources" | "concept_measurement_lookup" => {
            // An elaboration of a procedure this conversation already answered
            // ("give me the exact steps") is that procedure's continuation
            // (issue #444). The rebind is recovered from the prior turns, not
            // from this prompt's own words, so neither a promoted method nor a
            // cue-matched family can claim the turn before the table routes
            // the bare words to a fresh compose — the same blind spot the
            // fetch arms solve for the response-language retarget below. Offer
            // the follow-up its turn before composing from scratch.
            if let Some(answer) =
                crate::solver_handler_how::try_procedural_how_to_followup(prompt, &normalized, log)
            {
                return Some(record_contextual_method_answer(
                    prompt,
                    log,
                    answer,
                    "procedural_how_to_followup",
                ));
            }
            Some(crate::source_capability::execute(
                capability,
                prompt,
                solver.config,
                log,
            ))
        }
        // The fresh-period digest reading (issue #1138 news_07:
        // `relative_period` + `retrieve` routed here) summarises web events a
        // plain chat surface cannot fetch, so the honest answer names the
        // capability the surface lacks instead of falling through to the
        // unknown opener. An agent client that advertises `web_search`
        // executes the digest itself and keeps the fall-through. Weekday and
        // date questions stay `time_expression`, which has no such row, so
        // calendar reasoning keeps answering them.
        "web_search" if decision.object == ObjectType::RelativePeriod => {
            if solver.config.agent_mode {
                None
            } else {
                Some(capability_gap(prompt, log, capability, capability))
            }
        }
        // These capabilities already have general registry methods. Recording
        // the table decision here changes their precedence without duplicating
        // their execution or interrupting a multi-step research recipe; they
        // fall through with everything else.
        // An advertised capability the dispatcher has no dedicated executor
        // for still reaches the planner (issue #671: `read the file alpha.txt`
        // must be answered, not dead-ended). A surface that genuinely lacks
        // the tool never gets here -- that refusal is the `HonestGap` arm
        // above, which names the missing tool instead of the routed one.
        _ => None,
    }
}

/// Whether a grounded capability decision is more specific than a promoted
/// handler's reading of the same prompt.
///
/// Generic retrieval/clarification handlers and concept lookup must yield to a
/// typed URL, path, clock, language or self-surface. Translation is different:
/// a complete translation recipe owns its transform even though it names a
/// target language. Only the `(language_name, demonstrate, dialogue)` reading
/// is a request to render this response in that language.
fn capability_route_preempts(
    promoted_method: &str,
    decision: &crate::capability_routing::RoutingDecision,
) -> bool {
    matches!(
        promoted_method,
        "http_fetch"
            | "url_navigate"
            | "web_search"
            | "calendar_create_event"
            | "clarification"
            | "concept_lookup"
            | "software_project"
            | "conversation_memory"
            | "response_language_followup"
    ) || (promoted_method == "translation"
        && decision.object == ObjectType::LanguageName
        && decision.act == Act::Demonstrate)
}

/// Whether the table should preempt regular symbolic-handler precedence.
///
/// A typed/named object is authoritative.  A bare term is not: definitions and
/// other local symbolic answers keep their turn unless the prompt explicitly
/// names the open web, lands in the workspace, or asks to compose from sources.
/// A prompt that is *only* a seeded clarification utterance is not either: the
/// table reads its short words as a self-surface question, and the seed's own
/// class keeps the turn (issue #29).
fn solver_route_is_authoritative(
    prompt: &str,
    decision: &crate::capability_routing::RoutingDecision,
) -> bool {
    if crate::capability_routing::is_bare_clarification(prompt) {
        return false;
    }
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
    log.append_fields(
        "capability_route",
        &[
            ("object", object.slug()),
            ("act", act.slug()),
            ("locus", locus.slug()),
            ("capability", capability),
        ],
    );
}

fn capability_gap(prompt: &str, log: &mut EventLog, needed: &str, missing: &str) -> SymbolicAnswer {
    log.append_fields(
        "capability_gap",
        &[("needed", needed), ("missing", missing)],
    );
    let mut body = seeded_runtime_text(
        "capability_gap",
        &[("needed", needed), ("missing", missing)],
    );
    let anchors = request_anchors(prompt);
    if !anchors.is_empty() {
        let rendered = anchors
            .iter()
            .map(|anchor| format!("`{anchor}`"))
            .collect::<Vec<_>>()
            .join(", ");
        log.append("capability_gap:request_anchors", anchors.join("|"));
        body.push_str(&seeded_runtime_text(
            "capability_gap_anchors",
            &[("anchors", &rendered)],
        ));
    }
    finalize_simple(
        prompt,
        log,
        "capability_gap",
        "response:capability_gap",
        &body,
        1.0,
    )
}

/// Stable machine-addressable values that must survive an honest capability
/// gap. A chat surface cannot open the requested workspace, but it must not
/// discard an exact commit, path or code identifier before handing the request
/// to a capable client.
fn request_anchors(prompt: &str) -> Vec<&str> {
    let mut anchors = Vec::new();
    for token in prompt.split(|character: char| {
        !(character.is_ascii_alphanumeric()
            || matches!(character, '/' | '.' | '_' | '-' | ':' | '@'))
    }) {
        let token = token.trim_matches(['.', ',', ':', ';']);
        let exact_commit =
            token.len() == 40 && token.chars().all(|character| character.is_ascii_hexdigit());
        let named_path =
            token.contains('/') && !token.starts_with("http://") && !token.starts_with("https://");
        let code_identifier = token.contains('_')
            && token
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_');
        if (exact_commit || named_path || code_identifier) && !anchors.contains(&token) {
            anchors.push(token);
        }
    }
    anchors
}

/// The response language the conversation has already established, read from
/// the user's own turns (plan 10 leaf 15, issue #724).
///
/// The marker is the seed-grounded response-language role
/// ([`detect_response_language`]), so this holds no phrase table of its own and
/// reads the request the same way the demonstration route does. Only user turns
/// speak: an assistant turn merely obeyed.
pub fn established_response_language(history: &[ConversationTurn]) -> Option<&'static str> {
    history
        .iter()
        .rev()
        .filter(|turn| turn.role == ConversationRole::User)
        .find_map(|turn| detect_response_language(&turn.content.to_lowercase()))
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
    let body = seeded_runtime_text(
        "response_language_demonstration",
        &[
            ("canonical", canonical),
            ("surface", surface),
            ("demonstration", &demonstration),
        ],
    );
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
        || seeded_runtime_text("explain_previous_turn_empty", &[]),
        |previous| seeded_runtime_text("explain_previous_turn_answer", &[("previous", previous)]),
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
    let execution = match learned.execute(intent_formalization, solver.config) {
        Ok(execution) => execution,
        Err(reason) => {
            log.append("method:learned", format!("{name}:{reason}"));
            return None;
        }
    };
    log.append("method:learned", name.to_owned());
    log.append(
        "method:learned:operations_verified",
        execution.operations_verified.to_string(),
    );
    let rendered = execution.answer;
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

fn try_attributed_runtime(
    solver: &UniversalSolver,
    execution: MethodExecution,
    method_name: &str,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: MethodRuntime,
) -> Option<SymbolicAnswer> {
    let kind = execution.runtime?;
    let answer = match kind {
        MethodRuntimeKind::Diagnostic => try_diagnostic(solver, prompt, normalized, log),
        MethodRuntimeKind::NaturalLanguageTool => {
            try_natural_language_tool_request(prompt, normalized, log, solver.config.agent_mode)
        }
        MethodRuntimeKind::BehaviorRules => {
            try_behavior_rules_with_runtime(prompt, normalized, log, runtime.self_awareness_runtime)
        }
        MethodRuntimeKind::FeatureCapability => {
            try_feature_capability(prompt, normalized, log, runtime.capability_runtime)
        }
        MethodRuntimeKind::PlaywrightScript => {
            try_playwright_script(prompt, normalized, log, solver.config.guess_probability)
        }
    }?;
    Some(record_method_answer(prompt, log, answer, method_name))
}

fn try_response_language_variant(
    variant: Option<ResponseLanguageVariant>,
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    language: &str,
) -> Option<SymbolicAnswer> {
    match variant? {
        ResponseLanguageVariant::ConceptLookup => {
            try_concept_lookup_with_response_language(prompt, log, Some(language))
        }
        ResponseLanguageVariant::PatternInference => {
            try_pattern_inference_with_response_language(prompt, normalized, log, language)
        }
    }
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

fn seeded_runtime_text(intent: &str, values: &[(&str, &str)]) -> String {
    crate::seed::render_response(intent, "en", values).unwrap_or_else(|| intent.to_owned())
}
