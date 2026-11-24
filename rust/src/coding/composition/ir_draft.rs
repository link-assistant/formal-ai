//! Lower actual searched IR and retain failures before executable verification.

use super::{
    CodingTaskSpec, Draft, DraftAttempt, FragmentCatalog, FragmentKind, MissingFragment, ProgramIr,
    constant_bounded_domain, echo_declared_surface,
};

pub(super) fn ir_draft(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    program: ProgramIr,
    search_rank: usize,
    attempts: &mut Vec<DraftAttempt>,
) -> Result<Option<Draft>, MissingFragment> {
    if let Some(id) = program
        .fragments
        .iter()
        .find(|fragment| catalog.get(fragment).is_none())
    {
        record_rejected(&program, "fragment", id, attempts);
        return Err(MissingFragment::new(id));
    }
    if let Err(error) = program.type_check(catalog) {
        record_rejected(&program, "type_check", &error, attempts);
        return Ok(None);
    }
    let Some(lowering) = crate::coding::ir_lowering::lowering_for(&spec.language) else {
        record_rejected(&program, "backend", &spec.language, attempts);
        return Ok(None);
    };
    let source = match lowering.lower(&program, catalog) {
        Ok(source) => source,
        Err(error) => {
            record_rejected(&program, "lowering", &format!("{error:?}"), attempts);
            return Ok(None);
        }
    };
    // The prompt's declared surface travels with the derived artifact: a
    // signature the task itself spelled out (annotations, return, imports)
    // is echoed verbatim, never reconstructed from the unified types — the
    // reconstruction would drop the spelling's own import (`List[float]`
    // needs `from typing import List`) and with it the artifact's
    // executability (issue #1085 upstream transfer, HumanEval/0).
    let source = echo_declared_surface(spec, source);
    let content_id = program.content_id();
    let mut action_cost = program.action_cost();
    // A draft whose driving iteration runs over a constant performs
    // input-independent action: its work is bounded by the constant, not by
    // the task's input, so agreement with the examples is coincidence. Such
    // a draft owes the unexamined input as extra action.
    action_cost += 4 * constant_bounded_domain(&program, catalog);
    // A literal the search introduced is an assumption no oracle justified:
    // each one owes extra action, so among verifying drafts the composition
    // that reads the task's inputs outranks an equally cheap one that
    // guesses a constant.
    action_cost += crate::coding::composition_search::literal_leaves(&program.body);
    let parameter_names: Vec<&str> = spec
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect();
    let referenced_params =
        crate::coding::composition_search::parameter_reads(&program.body, &parameter_names);
    let composition = format!(
        "typed_search({})",
        program.composition_notation(|id| {
            catalog
                .get(id)
                .is_some_and(|fragment| fragment.kind == FragmentKind::Meaning)
        })
    );
    Ok(Some(Draft {
        id: content_id,
        source,
        callable_name: program.name,
        source_urls: program.source_urls,
        source_licenses: program.source_licenses,
        composition,
        action_cost,
        typed_ir: true,
        search_rank,
        referenced_params,
    }))
}

fn record_rejected(
    program: &ProgramIr,
    stage: &str,
    error: &str,
    attempts: &mut Vec<DraftAttempt>,
) {
    let identifier = program.content_id();
    crate::diagnostic!("trace"; "coding_composition: rejected_ir={identifier} stage={stage} error={error}");
    attempts.push(DraftAttempt {
        id: identifier,
        passed: false,
        detail: format!("{stage}:{error}"),
    });
}
