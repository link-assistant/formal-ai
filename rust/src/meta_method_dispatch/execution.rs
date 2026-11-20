use super::{
    CapabilityRuntime, EventLog, IntentFormalization, MethodExecution, MethodRegistry,
    MethodRuntimeKind, ProofRenderConfig, SelfAwarenessRuntime, SolverConfig, SymbolicAnswer,
    UniversalSolver, answer_links_notation, append_diagnostic_trace, build_evidence_links,
    try_behavior_rules_with_runtime, try_feature_capability, try_natural_language_tool_request,
    try_playwright_script,
};
use std::fmt::Write as _;

/// Execute one adopted learned method as the recipe program its learned
/// operations project onto (issue #1138 B7, plan 07 leaf 6).
///
/// A learned abstraction has no compiled handler: its `operations` are the
/// recorder event kinds `crate::recipe_interpreter` already dispatches on, so
/// the method runs by executing that program rather than by calling Rust that
/// was written for it. A name that is not a learned record, an operation that
/// binds to no recorder, and a misordered program are all reported in the trace
/// under `method:learned` and then declined, never silently skipped.
pub(super) fn try_learned_method(
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
pub(super) struct MethodRuntime {
    pub(super) proof_render_config: ProofRenderConfig,
    capability_runtime: CapabilityRuntime,
    pub(super) self_awareness_runtime: SelfAwarenessRuntime,
}

impl MethodRuntime {
    pub(super) const fn new(config: SolverConfig) -> Self {
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

pub(super) fn try_attributed_runtime(
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
        MethodRuntimeKind::Diagnostic => try_diagnostic(solver, prompt, log),
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

fn try_diagnostic(
    solver: &UniversalSolver,
    prompt: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (marker, _) = crate::rule_interpreter::handler_table_rows("diagnostic_marker")
        .iter()
        .find(|(marker, _)| !marker.is_empty() && prompt.contains(marker.as_str()))?;
    log.append("diagnostic_mode", "active".to_owned());
    let stripped = prompt.replace(marker.as_str(), "").trim().to_owned();
    let inner_solver = UniversalSolver::new(solver.config);
    let inner = inner_solver.solve(&stripped);
    let mut decorated = inner.answer.clone();
    let _ = writeln!(decorated, "\n\n{marker}");
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

pub(super) fn record_method_answer(
    prompt: &str,
    log: &mut EventLog,
    answer: SymbolicAnswer,
    name: &str,
) -> SymbolicAnswer {
    record_method(log, name);
    refresh_answer_projection(prompt, log, answer)
}

pub(super) fn record_contextual_method_answer(
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
