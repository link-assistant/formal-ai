//! Registry-backed method execution for the universal solver.
//!
//! The meta core owns method selection: an impulse is formalized, its
//! `route:`/`handler:` relevants are resolved through [`MethodRegistry`], and the
//! ordered method names are executed here. Handler functions remain ordinary Rust
//! implementations, but they are no longer selected by a separate hardcoded loop
//! in `solver.rs`.

use std::fmt::Write as _;

use crate::engine::{answer_links_notation, SymbolicAnswer};
use crate::event_log::{build_evidence_links, EventLog};
use crate::intent_formalization::IntentFormalization;
use crate::method_registry::MethodRegistry;
use crate::proof_engine::ProofRenderConfig;
use crate::solver::{ConversationTurn, SolverConfig, UniversalSolver};
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_dispatch::{
    handler_for_method, try_contextual_override, ContextualOutcome, ContextualRuntime,
};
use crate::solver_handlers::{
    try_behavior_rules_with_runtime, try_concept_lookup_with_response_language,
    try_definition_merge_by_default, try_explicit_repository_lookup, try_feature_capability,
    try_natural_language_tool_request, try_playwright_script, try_project_lookup,
    try_project_lookup_with_response_language, CapabilityRuntime, SelfAwarenessRuntime,
};

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
    let method_names = registry.ordered_method_names_for_relevants(&intent_formalization.relevants);
    let runtime = MethodRuntime::new(solver.config);

    // Issue #556: a response-language follow-up replays the previous request
    // through the whole solver with this language forced onto every localizable
    // answer family, so the retarget generalizes beyond a single handler.
    let forced_response_language = solver.config.forced_response_language;
    for name in method_names {
        if matches!(name.as_str(), "feature_capability" | "capabilities") {
            if let Some(answer) = try_explicit_repository_lookup(
                prompt,
                &normalized,
                log,
                solver.config.associative_project_promotion,
                intent_formalization.route.as_deref() == Some("identity"),
                forced_response_language,
            ) {
                return Some(record_method_answer(prompt, log, answer, "project_lookup"));
            }
        }
        if let Some(answer) = try_prelude_method(solver, &name, prompt, &normalized, log, runtime) {
            return Some(answer);
        }
        if solver.config.definition_fusion_by_default && name == "concept_lookup" {
            if let Some(answer) = try_definition_merge_by_default(prompt, log) {
                return Some(record_method_answer(
                    prompt,
                    log,
                    answer,
                    "definition_merge_by_default",
                ));
            }
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
        if name == "concept_lookup" {
            if let Some(language) = forced_response_language {
                if let Some(answer) =
                    try_concept_lookup_with_response_language(prompt, log, Some(language))
                {
                    return Some(record_method_answer(prompt, log, answer, "concept_lookup"));
                }
            }
        }
        if let Some(handler) = handler_for_method(&name) {
            if let Some(answer) = handler(prompt, &normalized, log) {
                return Some(record_method_answer(prompt, log, answer, &name));
            }
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
