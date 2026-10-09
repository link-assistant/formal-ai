use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter};
use formal_ai::composition::{compose, compose_with_ir};
use formal_ai::concept_discovery::{
    CandidatePart, ConceptMap, ConceptRequirement, StructuralMeaning, structural_meanings,
};
use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::needs::{NeedKind, NeedState};
use formal_ai::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};

fn parameter(name: &str, annotation: Option<&str>) -> Parameter {
    Parameter {
        name: name.to_owned(),
        annotation: annotation.map(str::to_owned),
    }
}

fn example(arguments: &[&str], expected: &str) -> Example {
    Example {
        arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
        expected: expected.to_owned(),
    }
}

fn spec(name: &str, parameters: Vec<Parameter>, examples: Vec<Example>) -> CodingTaskSpec {
    CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Function,
        callable_binding_origin: formal_ai::coding_task_spec::CallableBindingOrigin::Declared {
            signature: String::new(),
        },
        name: name.to_owned(),
        parameters,
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: Vec::new(),
        examples,
        expected_stdout: None,
        prose_language: "en".to_owned(),
    }
}

fn program(requirement: &str, expected_stdout: Option<&str>) -> CodingTaskSpec {
    CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Program,
        callable_binding_origin: formal_ai::coding_task_spec::CallableBindingOrigin::Declared {
            signature: String::new(),
        },
        name: "main".to_owned(),
        parameters: Vec::new(),
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: vec![requirement.to_owned()],
        examples: Vec::new(),
        expected_stdout: expected_stdout.map(str::to_owned),
        prose_language: "en".to_owned(),
    }
}

fn candidate(id: &str, kind: &str, code: Option<&str>) -> CandidatePart {
    CandidatePart {
        id: id.to_owned(),
        kind: kind.to_owned(),
        label: id.to_owned(),
        language: Some("python".to_owned()),
        code: code.map(str::to_owned),
        callable_name: None,
        callable_contract: None,
        source_tests: Vec::new(),
        license: if kind == "stdlib" {
            "PSF-2.0"
        } else {
            "Apache-2.0"
        }
        .to_owned(),
        source_url: format!("https://source.invalid/{id}"),
        sha256: "e".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
        score: 1.0,
    }
}

fn map(structure_ids: &[&str], candidates: Vec<CandidatePart>) -> ConceptMap {
    let registry = structural_meanings();
    let structures = structure_ids
        .iter()
        .map(|id| {
            registry
                .iter()
                .find(|meaning| meaning.id == *id)
                .cloned()
                .unwrap_or_else(|| StructuralMeaning {
                    id: (*id).to_owned(),
                    idiom: String::new(),
                    grounding: String::new(),
                })
        })
        .collect();
    ConceptMap {
        needs: vec![ConceptRequirement::new(
            "fixture need",
            "en",
            NeedState::Satisfied,
            structures,
            candidates,
        )],
        evidence: Vec::new(),
    }
}

#[test]
fn passing_drafts_are_executed_and_least_action_selects_the_stdlib_gcd() {
    let task = spec(
        "greatest_common_divisor",
        vec![parameter("a", Some("int")), parameter("b", Some("int"))],
        vec![example(&["3", "5"], "1"), example(&["25", "15"], "5")],
    );
    let concepts = map(
        &[],
        vec![
            candidate("math.gcd", "stdlib", None),
            candidate(
                "Z14857",
                "wikifunctions_implementation",
                Some("import math\nreturn math.gcd(a, b)"),
            ),
            candidate(
                "Z13642",
                "wikifunctions_implementation",
                Some("while b:\n    a, b = b, a % b\nreturn a"),
            ),
        ],
    );
    let outcome = compose(&task, &concepts);
    let selected = outcome.selected.expect("a GCD draft passes");
    assert_eq!(selected.id, "stdlib:math.gcd");
    assert!(selected.source.contains("return math.gcd(a, b)"));
    assert!(
        outcome
            .attempts
            .iter()
            .any(|attempt| attempt.id == "part:Z14857")
    );
    assert_eq!(selected.assertion_count, 2);
}

#[test]
fn structural_compositions_cover_tuple_pairwise_and_vowel_counting() {
    let sum_product = compose(
        &spec(
            "sum_product",
            vec![parameter("numbers", Some("list[int]"))],
            vec![
                example(&["[]"], "(0, 1)"),
                example(&["[1, 2, 3, 4]"], "(10, 24)"),
            ],
        ),
        &map(
            &["tuple_of", "reduce_sum", "reduce_product"],
            vec![
                candidate("sum", "stdlib", None),
                candidate("math.prod", "stdlib", None),
            ],
        ),
    );
    let sum_product = sum_product
        .selected
        .expect("sum/product composition passes");
    assert!(
        sum_product
            .source
            .contains("return (sum(numbers), math.prod(numbers))")
    );

    let close = compose(
        &spec(
            "has_close_elements",
            vec![
                parameter("numbers", Some("list[float]")),
                parameter("threshold", Some("float")),
            ],
            vec![
                example(&["[1.0, 2.0, 3.0]", "0.5"], "False"),
                example(&["[1.0, 2.8, 3.0]", "0.3"], "True"),
            ],
        ),
        &map(
            &[
                "quantifier_any",
                "pairwise_distinct",
                "predicate_abs_diff_lt",
            ],
            Vec::new(),
        ),
    );
    let close = close.selected.expect("pairwise composition passes");
    assert!(close.source.contains(".combinations(numbers, 2)"));
    assert!(close.source.contains("abs(left - right) < threshold"));

    let vowels = compose(
        &spec(
            "count_vowels",
            vec![parameter("text", Some("str"))],
            vec![example(&["'hello'"], "2"), example(&["'sky'"], "0")],
        ),
        &map(&["reduce_count", "vowel_character_class"], Vec::new()),
    );
    let vowels = vowels.selected.expect("vowel composition passes");
    assert!(vowels.source.contains("sum(1 for character in text"));
    assert!(vowels.composition.starts_with("typed_search("));
}

#[test]
fn failed_parts_return_no_answer_and_name_the_failed_example() {
    let task = spec(
        "transform_value",
        vec![parameter("value", Some("int"))],
        vec![example(&["1"], "2")],
    );
    let outcome = compose(&task, &map(&[], vec![candidate("abs", "stdlib", None)]));
    assert!(outcome.selected.is_none());
    assert!(
        outcome.research_trail.contains("abs"),
        "{}",
        outcome.research_trail
    );
    assert!(
        outcome.research_trail.contains("transform_value(1) == 2"),
        "{}",
        outcome.research_trail
    );
}

#[test]
fn a_missing_catalog_fragment_is_a_blocked_need_and_never_an_answer() {
    let task = spec(
        "unknown_operation",
        vec![parameter("value", Some("int"))],
        vec![example(&["1"], "1")],
    );
    let missing_id = "rediscover_this_operation";
    let program = ProgramIr {
        name: task.name.clone(),
        parameters: vec![("value".to_owned(), IrType::Integer)],
        result: IrType::Integer,
        body: IrNode::Apply {
            fragment: missing_id.to_owned(),
            arguments: vec![IrNode::Parameter {
                name: "value".to_owned(),
                ty: IrType::Integer,
            }],
        },
        fragments: vec![missing_id.to_owned()],
        source_urls: Vec::new(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::ShapeOnly,
    };

    let outcome = compose_with_ir(
        &task,
        &map(&[], Vec::new()),
        &FragmentCatalog::default(),
        [program],
    );

    assert!(outcome.selected.is_none());
    assert_eq!(
        outcome.unverified,
        [] as [formal_ai::composition::UnverifiedDraft; 0]
    );
    assert_eq!(outcome.blocked_needs.len(), 1);
    assert_eq!(outcome.blocked_needs[0].kind, NeedKind::Part);
    assert_eq!(outcome.blocked_needs[0].state, NeedState::Unsatisfiable);
    assert_eq!(
        outcome.blocked_needs[0].subject,
        format!("fragment:{missing_id}")
    );
    assert!(outcome.research_trail.contains(missing_id));
}

#[test]
fn legacy_composition_cannot_bypass_the_callers_empty_catalog() {
    let task = spec(
        "unique_count",
        vec![parameter("values", Some("list[int]"))],
        vec![example(&["[1, 1, 2]"], "2")],
    );
    let outcome = compose_with_ir(
        &task,
        &map(&["distinct_elements", "reduce_len"], Vec::new()),
        &FragmentCatalog::default(),
        [],
    );

    assert!(outcome.selected.is_none());
    assert!(
        outcome
            .blocked_needs
            .iter()
            .any(|need| need.subject.starts_with("fragment:")),
        "the injected empty catalog must be authoritative: {outcome:#?}"
    );
}

#[test]
fn runnable_programs_are_composed_from_output_and_range_observations() {
    let literal = compose(
        &program("Print Alpha, beta!", Some("Alpha, beta!")),
        &map(&["print_stdout"], Vec::new()),
    )
    .selected
    .expect("an exact stdout observation should verify a runnable program");
    assert_eq!(literal.source, "print(\"Alpha, beta!\")");
    assert_eq!(literal.assertion_count, 1);

    let range = compose(
        &program("Count to five inclusive.", None),
        &map(&["range_inclusive"], Vec::new()),
    )
    .selected
    .expect("a seeded cardinal and inclusive range should derive stdout");
    assert!(range.source.contains("for number in range(1, 5 + 1)"));
    assert!(range.source.contains("print(number)"));
    assert!(range.composition.starts_with("typed_search("));
}

#[test]
fn source_recurrence_candidates_are_verified_with_discovered_examples() {
    let mut source_candidate = candidate(
        "source-abstract-17",
        "wikifunctions_recurrence",
        Some("def accumulated_total(n):\n    return 0 if n == 0 else n + accumulated_total(n - 1)"),
    );
    source_candidate.source_tests = vec![
        example(&["0"], "0"),
        example(&["4"], "10"),
        example(&["7"], "28"),
    ];
    let source_parameter_expression =
        formal_ai::coding_recurrence::Expression::Parameter("n".to_owned());
    let contract = formal_ai::coding_recurrence::SourceCallableContract {
        name: "accumulated_total".to_owned(),
        parameter: "n".to_owned(),
        expression: formal_ai::coding_recurrence::Expression::Apply(
            formal_ai::coding_recurrence::Operation::Conditional,
            vec![
                formal_ai::coding_recurrence::Expression::Apply(
                    formal_ai::coding_recurrence::Operation::Equal,
                    vec![
                        source_parameter_expression.clone(),
                        formal_ai::coding_recurrence::Expression::Literal(0),
                    ],
                ),
                formal_ai::coding_recurrence::Expression::Literal(0),
                formal_ai::coding_recurrence::Expression::Apply(
                    formal_ai::coding_recurrence::Operation::Add,
                    vec![
                        source_parameter_expression.clone(),
                        formal_ai::coding_recurrence::Expression::Recur(Box::new(
                            formal_ai::coding_recurrence::Expression::Apply(
                                formal_ai::coding_recurrence::Operation::SubtractOne,
                                vec![source_parameter_expression],
                            ),
                        )),
                    ],
                ),
            ],
        ),
    };
    source_candidate.code = Some(contract.render_python());
    source_candidate.callable_name = Some(contract.name.clone());
    source_candidate.callable_contract = Some(contract);
    let outcome = compose(
        &spec(
            "accumulated_total",
            vec![parameter("n", Some("int"))],
            Vec::new(),
        ),
        &map(&[], vec![source_candidate]),
    );
    let selected = outcome
        .selected
        .expect("source recurrence should pass its discovered tests");
    assert_eq!(selected.id, "recurrence:source-abstract-17");
    assert_eq!(selected.assertion_count, 3);
}

#[test]
fn existing_collection_meanings_compose_into_held_out_programs() {
    let filtered = compose(
        &spec(
            "retain_matching_values",
            vec![parameter("values", None), parameter("fragment", None)],
            vec![
                example(&["['cabin', 'mint', 'cab']", "'cab'"], "['cabin', 'cab']"),
                example(&["[]", "'x'"], "[]"),
            ],
        ),
        &map(&["filter_only", "membership"], Vec::new()),
    )
    .selected
    .expect("filter and membership should compose");
    assert!(
        filtered
            .source
            .contains("[item for item in values if fragment in item]")
    );

    let rolling = compose(
        &spec(
            "prefix_high_water_marks",
            vec![parameter("samples", None)],
            vec![
                example(&["[]"], "[]"),
                example(&["[5, 2, 7, 1]"], "[5, 5, 7, 7]"),
            ],
        ),
        &map(&["running_prefix", "reduce_max"], Vec::new()),
    )
    .selected
    .expect("running maximum should compose");
    assert!(
        rolling
            .source
            .contains("itertools.accumulate(samples, max)")
    );

    let normalized = compose(
        &spec(
            "normalized_symbol_cardinality",
            vec![parameter("symbols", None)],
            vec![example(&["'AaBbA'"], "2"), example(&["''"], "0")],
        ),
        &map(
            &["distinct_elements", "reduce_count", "case_insensitive"],
            Vec::new(),
        ),
    )
    .selected
    .expect("normalization should compose before distinct counting");
    assert!(normalized.source.contains("len(set(symbols.lower()))"));

    let windows = compose(
        &spec(
            "count_shifted_matches",
            vec![parameter("haystack", None), parameter("needle", None)],
            vec![
                example(&["'zzzzz'", "'zz'"], "4"),
                example(&["'abc'", "'x'"], "0"),
            ],
        ),
        &map(&["count_overlapping"], Vec::new()),
    )
    .selected
    .expect("overlapping windows should compose");
    assert!(windows.source.contains("haystack.startswith(needle, i)"));
}

#[test]
fn held_out_grid_examples_select_the_supported_predecessor_relation() {
    let outcome = compose(
        &spec(
            "least_weight_to_coordinate",
            vec![
                parameter("weights", Some("list[list[int]]")),
                parameter("last_row", Some("int")),
                parameter("last_column", Some("int")),
            ],
            vec![
                example(
                    &[
                        "[[1, 100, 100, 100], [100, 2, 100, 100], [100, 100, 3, 4]]",
                        "2",
                        "3",
                    ],
                    "10",
                ),
                example(&["[[4, 8], [7, 1]]", "1", "1"], "5"),
            ],
        ),
        &map(&["grid_minimum_cost_path"], Vec::new()),
    );
    let selected = outcome.selected.unwrap_or_else(|| {
        panic!(
            "examples should select diagonal as an allowed predecessor: {:#?}",
            outcome.attempts
        )
    });
    assert!(
        selected.source.contains("lambda self, state_0, state_1"),
        "the selected candidate is a general recursive-reduction IR: {}",
        selected.source
    );
    assert!(selected.source.contains("(-1, -1)"));
    assert!(
        selected
            .source_urls
            .iter()
            .any(|url| url
                == "https://competitive-programming.cs.princeton.edu/files/lec_f22_w4.pdf")
    );
    assert!(selected.composition.starts_with("typed_search("));
}

fn competing_product_program(task: &CodingTaskSpec, catalog: &FragmentCatalog) -> ProgramIr {
    let fragments = vec!["reduce_product".to_owned(), "range_inclusive".to_owned()];
    ProgramIr {
        name: task.name.clone(),
        parameters: vec![(task.parameters[0].name.clone(), IrType::Unknown(0))],
        result: IrType::Integer,
        body: IrNode::Apply {
            fragment: "reduce_product".to_owned(),
            arguments: vec![IrNode::Apply {
                fragment: "range_inclusive".to_owned(),
                arguments: vec![IrNode::Parameter {
                    name: task.parameters[0].name.clone(),
                    ty: IrType::Unknown(0),
                }],
            }],
        },
        source_urls: fragments
            .iter()
            .map(|identifier| catalog.get(identifier).unwrap().grounding.clone())
            .collect(),
        source_licenses: fragments
            .iter()
            .map(|identifier| catalog.get(identifier).unwrap().license.clone())
            .collect(),
        fragments,
        reuse: ReuseMode::ShapeOnly,
    }
}

#[test]
fn source_owned_body_reads_compete_with_actual_typed_ir_under_a_provisional_signature() {
    let task = formal_ai::coding_task_spec::recognise(
        "Write a Python function that returns the factorial of n",
    )
    .unwrap();
    assert_eq!(
        task.callable_binding_origin,
        formal_ai::coding_task_spec::CallableBindingOrigin::Provisional
    );
    assert_eq!(task.parameters[0].name, "input");
    let source_candidate =
        formal_ai::coding_recurrence::cache::source_recurrence_candidate(&task).unwrap();
    let contract = source_candidate.callable_contract.as_ref().unwrap();
    assert_eq!(contract.parameter, "n");
    assert_eq!(contract.read_positions(), Some(vec![0]));
    let source = source_candidate.code.clone().unwrap();
    let selected_identifier = format!("recurrence:{}", source_candidate.id);
    let source_url = source_candidate.source_url.clone();
    let catalog = FragmentCatalog::bootstrap();
    let competing = competing_product_program(&task, &catalog);
    assert_eq!(competing.action_cost(), 3);
    let competing_identifier = competing.content_id();
    let outcome = compose_with_ir(
        &task,
        &map(&[], vec![source_candidate]),
        &catalog,
        [competing],
    );
    assert!(
        outcome
            .attempts
            .iter()
            .any(|attempt| attempt.id == competing_identifier && attempt.passed)
    );
    assert!(
        outcome
            .attempts
            .iter()
            .any(|attempt| attempt.id == selected_identifier && attempt.passed)
    );
    let selected = outcome.selected.unwrap();
    assert_eq!(selected.id, selected_identifier);
    assert_eq!(selected.source, source);
    assert_eq!(selected.source_urls, vec![source_url]);
    assert_eq!(selected.assertion_count, 4);
}

#[test]
fn declaration_only_source_names_cannot_outrank_an_input_reading_program() {
    let mut task = spec(
        "discovered_function",
        vec![parameter("input", None)],
        Vec::new(),
    );
    task.callable_binding_origin = formal_ai::coding_task_spec::CallableBindingOrigin::Provisional;
    let contract = formal_ai::coding_recurrence::SourceCallableContract {
        name: "constant".to_owned(),
        parameter: "n".to_owned(),
        expression: formal_ai::coding_recurrence::Expression::Literal(1),
    };
    assert_eq!(contract.read_positions(), Some(Vec::new()));
    let mut source_candidate = candidate("source-constant", "wikifunctions_recurrence", None);
    source_candidate.code = Some(contract.render_python());
    source_candidate.callable_name = Some(contract.name.clone());
    source_candidate.callable_contract = Some(contract);
    source_candidate.source_tests = vec![example(&["0"], "1"), example(&["1"], "1")];
    let catalog = FragmentCatalog::bootstrap();
    let competing = competing_product_program(&task, &catalog);
    let identifier = competing.content_id();
    let outcome = compose_with_ir(
        &task,
        &map(&[], vec![source_candidate]),
        &catalog,
        [competing],
    );
    assert!(
        outcome
            .attempts
            .iter()
            .any(|attempt| attempt.id == "recurrence:source-constant" && attempt.passed)
    );
    let selected = outcome.selected.unwrap();
    assert_eq!(selected.id, identifier);
    assert_eq!(
        selected.source,
        "import math\n\ndef discovered_function(input):\n    return math.prod(range(1, input + 1))"
    );
    assert_eq!(selected.assertion_count, 2);
    assert!(selected.composition.starts_with("typed_search("));
    assert!(
        !selected
            .source_urls
            .iter()
            .any(|url| url == "https://source.invalid/source-constant")
    );
}

#[test]
fn source_ast_binding_keeps_explicit_signature_and_self_recursion() {
    let task = formal_ai::coding_task_spec::recognise("def discovered_function(input: int) -> int:\n    \"\"\"Return the factorial of input.\"\"\"").unwrap();
    assert!(matches!(
        &task.callable_binding_origin,
        formal_ai::coding_task_spec::CallableBindingOrigin::Declared { .. }
    ));
    let candidate =
        formal_ai::coding_recurrence::cache::source_recurrence_candidate(&task).unwrap();
    let outcome = compose_with_ir(
        &task,
        &map(&[], vec![candidate]),
        &FragmentCatalog::bootstrap(),
        [],
    );
    let selected = outcome.selected.unwrap();
    assert_eq!(
        selected.source,
        "def discovered_function(input: int) -> int:\n    return 1 if (input == 0) else (input * discovered_function(input - 1))"
    );
    assert_eq!(selected.assertion_count, 4);
    assert!(selected.composition.starts_with("source_recurrence("));
}

#[test]
fn unsupported_explicit_source_bindings_are_rejected_before_execution() {
    for signature in [
        "chosen(x, y)",
        "chosen(*x)",
        "chosen(x=3)",
        "chosen(x, /)",
        "chosen(chosen)",
    ] {
        let task = formal_ai::coding_task_spec::recognise(&format!(
            "def {signature}:\n    \"\"\"Return the factorial of x.\"\"\""
        ))
        .unwrap();
        let candidate =
            formal_ai::coding_recurrence::cache::source_recurrence_candidate(&task).unwrap();
        assert!(
            candidate
                .callable_contract
                .as_ref()
                .unwrap()
                .bound_to(&task)
                .is_none(),
            "{signature}"
        );
        let outcome = compose_with_ir(
            &task,
            &map(&[], vec![candidate]),
            &FragmentCatalog::bootstrap(),
            [],
        );
        assert!(outcome.selected.is_none(), "{signature}");
        assert!(outcome.attempts.is_empty(), "{signature}");
    }
}

#[test]
fn observed_calls_bind_identity_without_using_literal_arguments_as_formal_names() {
    let task =
        formal_ai::coding_task_spec::recognise("Return the factorial.\nassert chosen(3) == 6")
            .unwrap();
    assert_eq!(
        task.callable_binding_origin,
        formal_ai::coding_task_spec::CallableBindingOrigin::Observed
    );
    assert_eq!(task.parameters[0].name, "arg1");
    let candidate =
        formal_ai::coding_recurrence::cache::source_recurrence_candidate(&task).unwrap();
    let bound = candidate
        .callable_contract
        .as_ref()
        .unwrap()
        .bound_to(&task)
        .unwrap();
    assert_eq!(bound.name, "chosen");
    assert_eq!(bound.parameter, "n");
    assert_eq!(bound.read_positions(), Some(vec![0]));
    let source = bound.render_python();
    let outcome = compose_with_ir(
        &task,
        &map(&[], vec![candidate]),
        &FragmentCatalog::bootstrap(),
        [],
    );
    let selected = outcome.selected.unwrap();
    assert_eq!(selected.source, source);
    assert_eq!(selected.assertion_count, 1);
}

#[test]
fn source_body_contract_must_match_the_callable_and_evaluated_program() {
    let task = formal_ai::coding_task_spec::recognise(
        "Write a Python function that returns the factorial of n",
    )
    .unwrap();
    let original = formal_ai::coding_recurrence::cache::source_recurrence_candidate(&task).unwrap();
    let mut missing = original.clone();
    missing.callable_contract = None;
    let mut changed_body = original.clone();
    changed_body.callable_contract.as_mut().unwrap().expression =
        formal_ai::coding_recurrence::Expression::Literal(1);
    let mut changed_identity = original;
    changed_identity.callable_name = Some("different_callable".to_owned());
    for candidate in [missing, changed_body, changed_identity] {
        let outcome = compose_with_ir(
            &task,
            &map(&[], vec![candidate]),
            &FragmentCatalog::bootstrap(),
            [],
        );
        assert!(outcome.selected.is_none());
        assert!(outcome.attempts.is_empty());
    }
}
