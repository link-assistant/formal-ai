//! Research-backed Python synthesis for structurally recognized coding tasks.

use std::fmt::Write as _;

use crate::coding::composition;
use crate::coding::concept_discovery::discover;
use crate::coding::synthesis_runtime::{
    discovery_catalog, extend_with_sequence_programs, live_fetch_enabled, procedure_ledger,
    render_answer, research_trail_line, response_language,
};
use crate::coding::task_spec::recognise;
use crate::meta_algorithm_builder::{CodingSurface, MetaAlgorithmBuilder};
use crate::{
    engine::{ExecutionRecipe, SymbolicAnswer},
    event_log::EventLog,
};

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
    let mut catalog = discovery_catalog(&spec, log, online);
    let mut concepts = discover(&spec, &catalog);
    let mut outcome = composition::compose(&spec, &concepts);
    if outcome.selected.is_none() {
        catalog = extend_with_sequence_programs(catalog, &spec, log, online);
        if !catalog.source_candidates.is_empty() {
            concepts = discover(&spec, &catalog);
            outcome = composition::compose(&spec, &concepts);
        }
    }
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
        log.append("synthesis:verification", "tests_failed".to_owned());
        log.append("synthesis:research_trail", outcome.research_trail.clone());
        log.append(
            "skill_gap",
            crate::program_skill_gap::gap_name(Some(&spec.name), Some(&spec.language)),
        );
        let response_language = response_language(&spec);
        let mut body = crate::program_skill_gap::render(
            Some(&spec.name),
            Some(&spec.language),
            response_language,
        );
        let _ = write!(
            body,
            "\n\n{}",
            research_trail_line(response_language, &outcome.research_trail)
        );
        return Some(finalize_simple(
            prompt,
            log,
            "write_program_skill_gap",
            "response:write_program:skill_gap",
            &body,
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
    let mut answer = finalize_simple(
        prompt,
        log,
        "write_program",
        "response:write_program:synthesized:python",
        &body,
        1.0,
    );
    // A verified artifact keeps its typed source when it crosses into agent
    // mode. Protocol adapters can therefore select the client's advertised
    // write capability without scraping Markdown or knowing which discovery
    // producer built the program.
    let program = spec.artifact_shape == crate::coding::task_spec::ArtifactShape::Program;
    let language = crate::coding::program_language_by_slug(&spec.language);
    let path = language.map_or_else(
        || format!("main.{}", spec.language),
        |language| language.save_as.to_owned(),
    );
    let commands = language.map_or_else(Vec::new, |language| {
        language
            .execution
            .check_command
            .into_iter()
            .chain(program.then_some(language.execution.run_command))
            .map(str::to_owned)
            .collect()
    });
    answer.execution_recipe = Some(Box::new(ExecutionRecipe {
        language: spec.language.clone(),
        source: selected.source,
        path,
        supporting_files: Vec::new(),
        commands,
    }));
    Some(answer)
}

/// Does the prompt carry a structural coding-task specification?
///
/// Kept as the shared gate used by the minimal-script handler. The semantic
/// recognizer owns all prose vocabulary; this function adds no task names.
#[must_use]
pub fn looks_like_python_function_request(prompt: &str, _normalized: &str) -> bool {
    recognise(prompt).is_some()
}
