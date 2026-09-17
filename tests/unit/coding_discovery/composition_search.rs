//! Issue #1138, plan 02 L7 — bounded typed enumeration replaces the forty
//! authored composition blocks.
//!
//! The point of the search is that a combination nobody wrote down composes,
//! because the enumeration is over fragment *type signatures* rather than over
//! authored shapes. Determinism is part of the contract: the same prompt and the
//! same catalog must yield the same candidates in the same order.

use formal_ai::coding_task_spec::recognise;
use formal_ai::composition::compose;
use formal_ai::composition_search::{SearchBounds, search, search_with_structures};
use formal_ai::concept_discovery::ConceptMap;
use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::ir_lowering::lowering_for;
use formal_ai::program_ir::ProgramIr;
use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter};

/// A held-out request whose shape is in none of the deleted authored blocks.
const UNSEEN_COMBINATION: &str = concat!(
    "Write a Python function run_length(text) that returns a list of ",
    "(character, count) pairs for each run of equal characters in text."
);

fn candidates() -> Vec<ProgramIr> {
    let spec = recognise(UNSEEN_COMBINATION).expect("the request is a coding task");
    search(
        &spec,
        &FragmentCatalog::bootstrap(),
        SearchBounds::default(),
    )
}

#[test]
fn an_unseen_combination_of_seeded_meanings_composes() {
    let found = candidates();
    assert!(
        !found.is_empty(),
        "a combination absent from the authored blocks must still compose"
    );

    let first = &found[0];
    assert!(
        first.type_check(&FragmentCatalog::bootstrap()).is_ok(),
        "every enumerated candidate is well-typed"
    );
    assert!(
        found
            .windows(2)
            .all(|pair| pair[0].action_cost() <= pair[1].action_cost()),
        "candidates are enumerated cheapest first"
    );
    assert!(
        !first.fragments.is_empty(),
        "a composed program names the fragments it was built from"
    );
}

#[test]
fn search_is_deterministic_across_runs() {
    let first: Vec<String> = candidates().iter().map(ProgramIr::content_id).collect();
    let second: Vec<String> = candidates().iter().map(ProgramIr::content_id).collect();

    assert_eq!(
        first, second,
        "same prompt, same catalog, same order — enumeration is by cost then id"
    );
    let mut sorted = first.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        first.len(),
        "the enumeration never yields the same candidate twice"
    );
}

#[test]
fn language_neutral_structures_produce_the_same_search_frontier() {
    let english = recognise(UNSEEN_COMBINATION).expect("the request is a coding task");
    let mut russian = english.clone();
    russian.requirement_sentences =
        vec!["Верни серии одинаковых символов строки как пары символа и количества.".to_owned()];
    russian.prose_language = "ru".to_owned();
    // Discovery has normalized both source languages to this semantic id;
    // search must not reopen the original prose and choose a different
    // frontier for the Russian spelling.
    let structures = vec!["extend_run".to_owned()];
    let catalog = FragmentCatalog::bootstrap();
    let frontier = |spec| {
        search_with_structures(spec, &catalog, SearchBounds::default(), &structures)
            .into_iter()
            .map(|program| program.fragments)
            .collect::<Vec<_>>()
    };

    let english = frontier(&english);
    let russian = frontier(&russian);
    assert_eq!(english, russian);
    assert!(
        english
            .iter()
            .any(|fragments| fragments.iter().any(|id| id == "extend_run")),
        "the discovered reduction identity opens the same typed neighborhood"
    );
}

#[test]
fn a_typed_lowering_without_an_oracle_stays_explicitly_unverified() {
    let spec = recognise(UNSEEN_COMBINATION).expect("the request is a coding task");
    let outcome = compose(&spec, &ConceptMap::default());

    assert!(
        outcome.selected.is_none(),
        "loading generated source is not equivalent to checking its semantics"
    );
    assert!(
        outcome
            .unverified
            .iter()
            .any(|candidate| candidate.source.contains("functools').reduce")),
        "a well-typed and lowered candidate remains available with an honest status: {outcome:#?}"
    );
    assert!(
        outcome
            .attempts
            .iter()
            .any(|attempt| attempt.detail.contains("unverified:no_executable_oracle")),
        "the missing oracle is explicit in the research trail"
    );
}

#[test]
fn recursive_reduction_is_discovered_and_lowered_from_typed_fragments() {
    let spec = CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Function,
        name: "least_weight_to_coordinate".to_owned(),
        parameters: vec![
            Parameter {
                name: "weights".to_owned(),
                annotation: Some("list[list[int]]".to_owned()),
            },
            Parameter {
                name: "last_row".to_owned(),
                annotation: Some("int".to_owned()),
            },
            Parameter {
                name: "last_column".to_owned(),
                annotation: Some("int".to_owned()),
            },
        ],
        return_annotation: Some("int".to_owned()),
        imports: Vec::new(),
        requirement_sentences: vec![
            "Return the minimum path cost to a grid coordinate using allowed predecessors."
                .to_owned(),
        ],
        examples: vec![Example {
            arguments: vec!["[[1]]".to_owned(), "0".to_owned(), "0".to_owned()],
            expected: "1".to_owned(),
        }],
        expected_stdout: None,
        prose_language: "en".to_owned(),
    };
    let catalog = FragmentCatalog::bootstrap();
    let structures = vec!["grid_minimum_cost_path".to_owned()];
    let programs = search_with_structures(
        &spec,
        &catalog,
        SearchBounds::default(),
        &structures,
    );
    let recursive = programs
        .iter()
        .find(|program| matches!(program.body, formal_ai::program_ir::IrNode::RecursiveReduce { .. }))
        .unwrap_or_else(|| panic!("typed search omitted recursive reduction: {programs:#?}"));
    let source = lowering_for("python")
        .expect("Python lowering exists")
        .lower(recursive, &catalog)
        .expect("the discovered recursive reduction lowers");

    assert!(source.contains("lambda self"));
    assert!(recursive.fragments.iter().any(|id| id == "reduce_min"));
}
