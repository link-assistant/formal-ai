//! Failed attempts identify real IR and preserve the exact rejection stage.

use super::*;
use formal_ai::composition_search::{SearchBounds, search_with_structures};

fn sequence_program(catalog: &FragmentCatalog) -> ProgramIr {
    let fragment = catalog.get("python_materialize_list").unwrap();
    ProgramIr {
        name: String::new(),
        parameters: Vec::new(),
        result: fragment.result.clone(),
        body: IrNode::Apply {
            fragment: fragment.id.clone(),
            arguments: vec![IrNode::Literal {
                text: "[]".to_owned(),
                ty: fragment.signature[0].clone(),
            }],
        },
        fragments: vec![fragment.id.clone()],
        source_urls: vec![fragment.grounding.clone()],
        source_licenses: vec![fragment.license.clone()],
        reuse: ReuseMode::ShapeOnly,
    }
}

fn assert_rejected(
    outcome: &formal_ai::composition::CompositionOutcome,
    identifier: &str,
    stage: &str,
) {
    assert!(outcome.selected.is_none());
    assert!(outcome.unverified.is_empty());
    assert_eq!(outcome.attempts.len(), 1, "{outcome:#?}");
    let attempt = &outcome.attempts[0];
    assert_eq!(attempt.id, identifier);
    assert!(!attempt.passed);
    assert!(attempt.detail.starts_with(stage), "{attempt:#?}");
    assert!(
        outcome
            .research_trail
            .contains(&format!("{identifier}:failed"))
    );
}

#[test]
fn missing_target_realization_retains_the_actual_ir_and_lowering_gap() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = sequence_program(&catalog);
    assert!(ir.type_check(&catalog).is_ok());
    let identifier = ir.content_id();
    let mut task = program("materialize an iterable as a list", None);
    task.language = "javascript".to_owned();
    let expected = formal_ai::ir_lowering::lowering_for(&task.language)
        .unwrap()
        .lower(&ir, &catalog)
        .expect_err("the Python-only fragment has no JavaScript realization");
    let outcome = compose_with_ir(&task, &ConceptMap::default(), &catalog, [ir]);
    assert_rejected(&outcome, &identifier, "lowering:");
    assert_eq!(outcome.attempts[0].detail, format!("lowering:{expected:?}"));
    assert!(
        outcome.attempts[0]
            .detail
            .contains("python_materialize_list")
    );
}

#[test]
fn an_absent_backend_is_a_failure_of_the_supplied_ir() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = sequence_program(&catalog);
    let identifier = ir.content_id();
    let mut task = program("materialize an iterable as a list", None);
    task.language = "unsupported_fixture_language".to_owned();
    let outcome = compose_with_ir(&task, &ConceptMap::default(), &catalog, [ir]);
    assert_rejected(&outcome, &identifier, "backend:");
    assert_eq!(
        outcome.attempts[0].detail,
        "backend:unsupported_fixture_language"
    );
}

#[test]
fn missing_catalog_parts_keep_both_the_failed_ir_and_the_blocked_need() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = sequence_program(&catalog);
    let identifier = ir.content_id();
    let outcome = compose_with_ir(
        &program("materialize an iterable as a list", None),
        &ConceptMap::default(),
        &FragmentCatalog::default(),
        [ir],
    );
    assert_rejected(&outcome, &identifier, "fragment:python_materialize_list");
    assert_eq!(outcome.blocked_needs.len(), 1);
    assert!(
        outcome.blocked_needs[0]
            .subject
            .contains("python_materialize_list")
    );
}

#[test]
fn a_type_error_is_recorded_before_lowering() {
    let catalog = FragmentCatalog::bootstrap();
    let mut ir = sequence_program(&catalog);
    let IrNode::Apply { arguments, .. } = &mut ir.body else {
        panic!("the fixture is a real fragment application");
    };
    arguments.clear();
    let expected = ir
        .type_check(&catalog)
        .expect_err("the fragment needs one argument");
    let identifier = ir.content_id();
    let outcome = compose_with_ir(
        &program("materialize an iterable as a list", None),
        &ConceptMap::default(),
        &catalog,
        [ir],
    );
    assert_rejected(&outcome, &identifier, "type_check:");
    assert_eq!(outcome.attempts[0].detail, format!("type_check:{expected}"));
}

#[test]
fn an_empty_frontier_does_not_fabricate_a_failed_program() {
    let outcome = compose_with_ir(
        &program("no catalog operation applies", None),
        &ConceptMap::default(),
        &FragmentCatalog::default(),
        Vec::<ProgramIr>::new(),
    );
    assert!(outcome.selected.is_none());
    assert!(outcome.unverified.is_empty());
    assert!(outcome.attempts.is_empty());
    assert!(!outcome.research_trail.contains("program_ir_"));
}

#[test]
fn bounded_search_delivers_unlowerable_candidates_to_the_failure_ledger() {
    let catalog = FragmentCatalog::bootstrap();
    let mut task = program("materialize iterable as list", None);
    task.language = "javascript".to_owned();
    let candidates = search_with_structures(
        &task,
        &catalog,
        SearchBounds {
            max_candidates: 8,
            max_depth: 2,
        },
        &["python_materialize_list".to_owned()],
    );
    assert!(
        !candidates.is_empty(),
        "the closed sequence base case opens the typed frontier"
    );
    assert!(candidates.iter().all(|ir| ir.type_check(&catalog).is_ok()));
    let identifiers: Vec<_> = candidates.iter().map(ProgramIr::content_id).collect();
    let outcome = compose_with_ir(&task, &ConceptMap::default(), &catalog, candidates);
    assert!(outcome.selected.is_none());
    assert!(outcome.unverified.is_empty());
    assert_eq!(
        outcome
            .attempts
            .iter()
            .map(|attempt| &attempt.id)
            .collect::<Vec<_>>(),
        identifiers.iter().collect::<Vec<_>>()
    );
    assert!(
        outcome
            .attempts
            .iter()
            .all(|attempt| !attempt.passed && attempt.detail.starts_with("lowering:"))
    );
}
