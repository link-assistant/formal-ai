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
use crate::solver_dispatch::{handler_for_method, try_contextual_override, ContextualOutcome};
use crate::solver_handlers::{
    try_behavior_rules_with_runtime, try_definition_merge_by_default, try_feature_capability,
    try_natural_language_tool_request, try_playwright_script, try_project_lookup,
    CapabilityRuntime, SelfAwarenessRuntime,
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

    for name in method_names {
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
            runtime.proof_render_config,
            runtime.self_awareness_runtime,
            log,
        ) {
            ContextualOutcome::Answer(answer) => {
                // `try_contextual_override` records the compatibility
                // `specialized_handler` event for contextual paths.
                return Some(record_contextual_method_answer(prompt, log, answer, &name));
            }
            ContextualOutcome::Skip => continue,
            ContextualOutcome::NotHandled => {}
        }
        if let Some(handler) = handler_for_method(&name) {
            if let Some(answer) = handler(prompt, &normalized, log) {
                return Some(record_method_answer(prompt, log, answer, &name));
            }
        }
        if name == "concept_lookup" {
            if let Some(answer) = try_project_lookup(
                prompt,
                &normalized,
                log,
                solver.config.associative_project_promotion,
                intent_formalization.route.as_deref() == Some("identity"),
            ) {
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
        confidence: inner.confidence,
        evidence_links,
        thinking_steps,
        links_notation,
    })
}

fn record_method(log: &mut EventLog, name: &str) {
    log.append("method", name.to_owned());
    log.append("specialized_handler", name.to_owned());
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
