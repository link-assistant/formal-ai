use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter};
use formal_ai::composition::compose;
use formal_ai::concept_discovery::{
    CandidatePart, ConceptMap, ConceptRequirement, StructuralMeaning, structural_meanings,
};
use formal_ai::needs::NeedState;

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
    assert!(close.source.contains("itertools.combinations(numbers, 2)"));
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
    assert!(vowels.id.contains("structure"));
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
    assert!(range.source.contains("for number in range(1, 5 + 1):"));
    assert!(range.source.contains("print(number)"));
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
    source_candidate.callable_name = Some("accumulated_total".to_owned());
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
    assert!(selected.composition.contains("orthogonal_or_diagonal"));
    assert!(selected.source.contains("last_row + 1"));
    assert_eq!(
        selected.source_urls,
        ["https://competitive-programming.cs.princeton.edu/files/lec_f22_w4.pdf"]
    );
}
