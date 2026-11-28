//! Issue #1138, plan 02 L7 — bounded typed enumeration replaces the forty
//! authored composition blocks.
//!
//! The point of the search is that a combination nobody wrote down composes,
//! because the enumeration is over fragment *type signatures* rather than over
//! authored shapes. Determinism is part of the contract: the same prompt and the
//! same catalog must yield the same candidates in the same order.

use formal_ai::coding_task_spec::recognise;
use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter};
use formal_ai::composition::compose;
use formal_ai::composition_search::{SearchBounds, search, search_with_structures};
use formal_ai::concept_discovery::ConceptMap;
use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::ir_lowering::lowering_for;
use formal_ai::program_ir::ProgramIr;

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
    // More task inputs read first, then cheapest first: a candidate that
    // ignores an input agrees with the examples only by coincidence.
    let reads = |program: &ProgramIr| {
        let shape = format!("{:?}", program.body);
        program
            .parameters
            .iter()
            .filter(|(name, _)| shape.contains(&format!("Parameter {{ name: {name:?}")))
            .count()
    };
    assert!(
        found.windows(2).all(|pair| {
            let (left, right) = (reads(&pair[0]), reads(&pair[1]));
            left > right || (left == right && pair[0].action_cost() <= pair[1].action_cost())
        }),
        "candidates are enumerated by inputs read, then cheapest first"
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
        "same prompt, same catalog, same order — enumeration is by inputs read, cost, then id"
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
            .any(|candidate| candidate.source.contains("functools.reduce")),
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
        callable_binding_origin: formal_ai::coding_task_spec::CallableBindingOrigin::Declared {
            signature: String::new(),
        },
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
    let programs = search_with_structures(&spec, &catalog, SearchBounds::default(), &structures);
    let recursive = programs
        .iter()
        .find(|program| {
            matches!(
                program.body,
                formal_ai::program_ir::IrNode::RecursiveReduce { .. }
            )
        })
        .unwrap_or_else(|| panic!("typed search omitted recursive reduction: {programs:#?}"));
    let source = lowering_for("python")
        .expect("Python lowering exists")
        .lower(recursive, &catalog)
        .expect("the discovered recursive reduction lowers");

    assert!(source.contains("lambda self"));
    assert!(recursive.fragments.iter().any(|id| id == "reduce_min"));
}

/// PR #1188 dogfooding: "their sum" of two scalar parameters is their binary
/// sum, not the reduction of the first one. The seeded `integer_add` /
/// `integer_multiply` fragments realize `reduce_sum` / `reduce_product` for two
/// values, and the ranking prefers the candidate that reads both parameters.
#[test]
fn a_reduction_over_two_scalar_parameters_reads_both() {
    let catalog = FragmentCatalog::bootstrap();
    for (prompt, structure, fragment, source) in [
        (
            "Write a Python function add(a, b) that returns their sum.",
            "reduce_sum",
            "integer_add",
            "a + b",
        ),
        (
            "Write a Python function multiply(a, b) that returns a times b.",
            "reduce_product",
            "integer_multiply",
            "a * b",
        ),
    ] {
        let spec = recognise(prompt).expect("the request is a coding task");
        let programs = search_with_structures(
            &spec,
            &catalog,
            SearchBounds::default(),
            &[structure.to_owned()],
        );
        let first = programs.first().expect("the reduction composes");
        let reads_both = matches!(
            &first.body,
            formal_ai::program_ir::IrNode::Apply { fragment: applied, arguments }
                if applied == fragment
                    && matches!(
                        arguments.as_slice(),
                        [
                            formal_ai::program_ir::IrNode::Parameter { name: left, .. },
                            formal_ai::program_ir::IrNode::Parameter { name: right, .. },
                        ] if left == "a" && right == "b"
                    )
        );
        assert!(reads_both, "{prompt}: {:#?}", first.body);
        let lowered = lowering_for("python")
            .expect("Python lowering exists")
            .lower(first, &catalog)
            .expect("the binary reduction lowers");
        assert!(lowered.contains(&format!("return {source}\n")), "{lowered}");
    }
}

/// A single sequence parameter keeps the reduction itself.
#[test]
fn a_reduction_over_one_sequence_parameter_stays_a_reduction() {
    let catalog = FragmentCatalog::bootstrap();
    let spec = recognise("Write a Python function total(items) that returns the sum of the items.")
        .expect("the request is a coding task");
    let programs = search_with_structures(
        &spec,
        &catalog,
        SearchBounds::default(),
        &["reduce_sum".to_owned()],
    );
    let first = programs.first().expect("the reduction composes");
    assert_eq!(first.fragments, ["reduce_sum"]);
}

/// PR #1188 dogfooding: a reduction over three scalar parameters reads all
/// three (`a + b + c`), not the first two.
#[test]
fn a_reduction_over_three_scalar_parameters_reads_all_three() {
    let catalog = FragmentCatalog::bootstrap();
    let spec = recognise("Write a Python function add3(a, b, c) that returns their sum.")
        .expect("the request is a coding task");
    let programs = search_with_structures(
        &spec,
        &catalog,
        SearchBounds::default(),
        &["reduce_sum".to_owned()],
    );
    let first = programs.first().expect("the reduction composes");
    let lowered = lowering_for("python")
        .expect("Python lowering exists")
        .lower(first, &catalog)
        .expect("the reduction lowers");
    assert!(lowered.contains("return a + b + c\n"), "{lowered}");
}

#[test]
fn closed_program_sequence_slots_have_typed_base_cases() {
    let catalog = FragmentCatalog::bootstrap();
    let spec = CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Program,
        callable_binding_origin: formal_ai::coding_task_spec::CallableBindingOrigin::Declared {
            signature: String::new(),
        },
        name: "main".to_owned(),
        parameters: Vec::new(),
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: vec!["materialize iterable as list".to_owned()],
        examples: Vec::new(),
        expected_stdout: None,
        prose_language: "en".to_owned(),
    };
    let bounds = SearchBounds {
        max_candidates: 8,
        max_depth: 2,
    };
    let structures = ["python_materialize_list".to_owned()];
    let first = search_with_structures(&spec, &catalog, bounds, &structures);
    assert!(
        !first.is_empty(),
        "closed programs can apply ranked sequence operations"
    );
    assert!(first.len() <= bounds.max_candidates);
    assert!(
        first
            .iter()
            .all(|ir| ir.parameters.is_empty() && ir.type_check(&catalog).is_ok())
    );
    assert!(
        first.iter().any(|ir| matches!(
            &ir.body,
            formal_ai::program_ir::IrNode::Apply { fragment, arguments }
                if fragment == "python_materialize_list" && matches!(arguments.as_slice(),
                    [formal_ai::program_ir::IrNode::Literal { text, .. }] if text == "[]")
        )),
        "the empty sequence is an actual typed argument, not an invented input"
    );
    let lower = lowering_for("python").unwrap();
    assert!(first.iter().all(|ir| lower.lower(ir, &catalog).is_ok()));
    assert_eq!(
        first.iter().map(ProgramIr::content_id).collect::<Vec<_>>(),
        search_with_structures(&spec, &catalog, bounds, &structures)
            .iter()
            .map(ProgramIr::content_id)
            .collect::<Vec<_>>()
    );
    assert!(
        search_with_structures(&spec, &FragmentCatalog::default(), bounds, &structures).is_empty()
    );
}

/// Every `pairwise_distinct` application nested inside another one.
fn nested_pairwise_applications(node: &formal_ai::program_ir::IrNode, inside: bool) -> usize {
    use formal_ai::program_ir::IrNode;
    let children: Vec<&IrNode> = match node {
        IrNode::Parameter { .. } | IrNode::Literal { .. } => Vec::new(),
        IrNode::Apply { arguments, .. } => arguments.iter().collect(),
        IrNode::Each {
            items,
            body,
            predicate,
            ..
        } => [items.as_ref(), body.as_ref()]
            .into_iter()
            .chain(predicate.as_deref())
            .collect(),
        IrNode::Fold {
            items,
            initial,
            body,
            ..
        } => vec![items.as_ref(), initial.as_ref(), body.as_ref()],
        IrNode::Repeat { from, to, body, .. } => vec![from.as_ref(), to.as_ref(), body.as_ref()],
        IrNode::Recurrence {
            base,
            transition,
            index,
            ..
        } => base
            .iter()
            .chain([transition.as_ref(), index.as_ref()])
            .collect(),
        IrNode::RecursiveReduce {
            target,
            items,
            next,
            admissible,
            base_test,
            base,
            local,
            ..
        } => target
            .iter()
            .chain(next)
            .chain([
                items.as_ref(),
                base_test.as_ref(),
                base.as_ref(),
                local.as_ref(),
            ])
            .chain(admissible.as_deref())
            .collect(),
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => vec![test.as_ref(), then_branch.as_ref(), else_branch.as_ref()],
        IrNode::Bind { value, body, .. } => vec![value.as_ref(), body.as_ref()],
        IrNode::Emit { value } | IrNode::Return { value } => vec![value.as_ref()],
    };
    let pairs = matches!(node, IrNode::Apply { fragment, .. } if fragment == "pairwise_distinct");
    usize::from(pairs && inside)
        + children
            .into_iter()
            .map(|child| nested_pairwise_applications(child, inside || pairs))
            .sum::<usize>()
}

/// Issue #1639: `pairwise_distinct` maps a sequence of `n` items to `n(n-1)/2`
/// pairs, and its result still fits its own input slot. Nested six and seven
/// deep over four numbers it needs about 1.1 GB and then ~10^14 tuples, which
/// is the 15 GB the Benchmark Corpus Gate hit on `near_pair`. The search must
/// keep applying the fragment once while never feeding an amplifying fragment
/// its own output.
#[test]
fn an_amplifying_fragment_is_never_applied_to_its_own_output() {
    let spec = recognise(concat!(
        "Implement Python function near_pair(numbers, threshold). Return whether ",
        "at least one distinct pair has a difference that is less than the threshold."
    ))
    .expect("the request is a coding task");
    let structures = [
        "quantifier_any",
        "pairwise_distinct",
        "predicate_abs_diff_lt",
    ]
    .map(str::to_owned);
    let programs = search_with_structures(
        &spec,
        &FragmentCatalog::bootstrap(),
        SearchBounds::default(),
        &structures,
    );
    assert!(
        programs.iter().any(|ir| ir
            .fragments
            .iter()
            .any(|fragment| fragment == "pairwise_distinct")),
        "the pairwise structure must still be enumerated"
    );
    let nested: Vec<String> = programs
        .iter()
        .filter(|ir| nested_pairwise_applications(&ir.body, false) > 0)
        .map(ProgramIr::to_links_notation)
        .collect();
    assert!(
        nested.is_empty(),
        "pairwise_distinct was applied to its own output: {nested:#?}"
    );
}
