//! Research-backed Python synthesis for structurally recognized coding tasks.

use crate::coding::synthesis_runtime::{
    attach_execution_recipe, discover_and_compose, discover_recurrence_and_compose,
    live_fetch_enabled, procedure_ledger, render_answer, research_trail_line, response_language,
    trace_synthesis_stage,
};
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec, recognise};
use crate::meta_algorithm_builder::{CodingSurface, MetaAlgorithmBuilder};
use crate::{engine::SymbolicAnswer, event_log::EventLog};

use super::finalize_simple;

pub fn try_program_synthesis(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_program_synthesis_with_online(prompt, normalized, log, live_fetch_enabled())
}

pub fn try_program_synthesis_with_online(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
    online: bool,
) -> Option<SymbolicAnswer> {
    let spec = recognise(prompt)?;
    trace_synthesis_stage("general_synthesis_started");
    let answer = synthesize(prompt, spec, log, online, false);
    trace_synthesis_stage("general_synthesis_finished");
    answer
}

pub(crate) fn try_bound_publisher_recurrence(
    prompt: &str,
    route: Option<&str>,
    config: crate::solver::SolverConfig,
    language: crate::language::Language,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let specification = publisher_recurrence_specification(prompt, route)?;
    if let Some(answer) =
        super::policy_gates::try_policy_gates(config.execution_surface, prompt, log, language)
    {
        return Some(answer);
    }
    trace_synthesis_stage("publisher_recurrence_started");
    let online = !config.offline && crate::solver_config::read_network_allowed();
    let answer = synthesize(prompt, specification, log, online, true);
    trace_synthesis_stage(if answer.is_some() {
        "publisher_recurrence_verified"
    } else {
        log.append(
            crate::program_skill_gap::UNVERIFIED_RECURRENCE,
            language.slug(),
        );
        "publisher_recurrence_unverified"
    });
    answer
}

fn publisher_recurrence_specification(prompt: &str, route: Option<&str>) -> Option<CodingTaskSpec> {
    if !matches!(route, Some("write_program" | "program_synthesis"))
        || crate::solver_dispatch::code_artifact_task_claims(prompt)
        || super::page_query_text::page_query_text_claims(prompt)
    {
        return None;
    }
    recognise(prompt)
        .filter(|spec| spec.artifact_shape == ArtifactShape::Function && spec.language == "python")
}

fn synthesize(
    prompt: &str,
    spec: CodingTaskSpec,
    log: &mut EventLog,
    online: bool,
    recurrence_only: bool,
) -> Option<SymbolicAnswer> {
    MetaAlgorithmBuilder::for_surface(CodingSurface::ProgramSynthesis).record(log);
    log.append("synthesis:spec", spec.to_links_notation());
    let procedure_ledger = procedure_ledger();
    let recalled = procedure_ledger
        .as_ref()
        .and_then(|ledger| match ledger.recall(&spec) {
            Ok(hit) => hit,
            Err(error) => {
                log.append("synthesis:procedure_ledger_error", error.to_string());
                None
            }
        });
    if let Some(procedure) = &recalled {
        log.append("cache_hit", procedure.id.clone());
        log.append(
            "synthesis:procedure_replay",
            format!("{} composition={}", procedure.id, procedure.composition),
        );
    }
    let (concepts, outcome) = if recurrence_only {
        discover_recurrence_and_compose(&spec, log, online)?
    } else {
        discover_and_compose(&spec, log, online)
    };
    log.append("synthesis:concept_map", concepts.to_links_notation());
    for attempt in &outcome.attempts {
        log.append(
            "synthesis:draft_comparison",
            format!(
                "draft={};status={};detail={}",
                attempt.id,
                if attempt.passed { "passed" } else { "failed" },
                attempt.detail
            ),
        );
    }
    let Some(selected) = outcome.selected else {
        if recurrence_only {
            return None;
        }
        log.append("synthesis:verification", "tests_failed".to_owned());
        log.append("synthesis:research_trail", outcome.research_trail.clone());
        log.append(
            "skill_gap",
            crate::program_skill_gap::gap_name(Some(&spec.name), Some(&spec.language)),
        );
        let response_language = response_language(&spec);
        let gap = crate::program_skill_gap::render(
            Some(&spec.name),
            Some(&spec.language),
            response_language,
        );
        let trail = research_trail_line(response_language, &outcome.research_trail);
        return Some(finalize_simple(
            prompt,
            log,
            "write_program_skill_gap",
            "response:write_program:skill_gap",
            &format!("{gap}\n\n{trail}"),
            1.0,
        ));
    };
    log.append("synthesis:composition", selected.composition.clone());
    if let Some(procedure) = recalled
        && procedure.composition != selected.composition
    {
        log.append(
            "synthesis:procedure_rediscovered",
            format!(
                "id={};previous={};current={}",
                procedure.id, procedure.composition, selected.composition
            ),
        );
    }
    log.append(
        "synthesis:cst_tree",
        crate::coding::validated_program_cst("python", &selected.source)?.links_notation(),
    );
    log.append(
        "synthesis:verification",
        format!("tests_passed assertion_count={}", selected.assertion_count),
    );
    log.append("execution_status", "tests passed".to_owned());
    for source in &selected.source_urls {
        log.append("synthesis:source", source.clone());
    }
    if let Some(ledger) = &procedure_ledger {
        match ledger.remember(&spec, &concepts, &selected) {
            Ok(procedure) => {
                log.append("synthesis:procedure_recorded", procedure.id);
            }
            Err(error) => {
                log.append("synthesis:procedure_ledger_error", error.to_string());
            }
        }
    }
    let body = render_answer(&selected, response_language(&spec));
    let answer = finalize_simple(
        prompt,
        log,
        "write_program",
        "response:write_program:synthesized:python",
        &body,
        1.0,
    );
    Some(attach_execution_recipe(answer, &spec, selected))
}

/// Does the prompt carry a structural coding-task specification?
///
/// Kept as the shared gate used by the minimal-script handler. The semantic
/// recognizer owns all prose vocabulary; this function adds no task names.
#[must_use]
pub fn looks_like_python_function_request(prompt: &str, _normalized: &str) -> bool {
    recognise(prompt).is_some()
}
