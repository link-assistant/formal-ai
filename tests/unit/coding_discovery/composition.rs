use formal_ai::coding_task_spec::{CodingTaskSpec, Example, Parameter};
use formal_ai::composition::compose;
use formal_ai::concept_discovery::{
    CandidatePart, ConceptMap, ConceptNeed, StructuralMeaning, structural_meanings,
};

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
        name: name.to_owned(),
        parameters,
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: Vec::new(),
        examples,
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
        needs: vec![ConceptNeed {
            phrase: "fixture need".to_owned(),
            structures,
            candidates,
            status: "satisfied".to_owned(),
        }],
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
