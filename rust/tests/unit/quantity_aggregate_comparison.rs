//! Shared whole-solver vectors. Authored parity controls, not native execution evidence.

use lino_objects_codec::format::parse_indented;
use serde_json::Value;

use formal_ai::engine::SymbolicAnswer;
use formal_ai::solver::{SolverConfig, UniversalSolver};

fn solve(question: &str) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
    .solve(question)
}

fn vectors() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/quantity-aggregate-comparison.json"
    ))
    .expect("shared quantity vectors")
}

fn frame(answer: &SymbolicAnswer) -> Option<Value> {
    answer.evidence_links.iter().find_map(|value| {
        value
            .strip_prefix("quantity_constraint:frame:")
            .map(|payload| serde_json::from_str(payload).expect("actual solver frame payload"))
    })
}

fn slice<'a>(question: &'a str, span: &Value) -> &'a str {
    let start = usize::try_from(span["start"].as_u64().expect("UTF-8 start")).expect("usize start");
    let end = usize::try_from(span["end"].as_u64().expect("UTF-8 end")).expect("usize end");
    question.get(start..end).expect("original UTF-8 boundary")
}

fn raw_spans(question: &str, value: &Value) {
    assert_eq!(value["source"].as_str(), Some(question));
    assert_eq!(
        value["sourceUtf8Bytes"].as_u64(),
        Some(question.len() as u64)
    );
    for proof in ["aggregationProof", "comparisonProof"] {
        assert_eq!(
            slice(question, &value[proof]["span"]),
            value[proof]["surface"].as_str().expect("proof surface")
        );
    }
    for term in value["terms"]
        .as_array()
        .expect("terms")
        .iter()
        .chain(std::iter::once(&value["bound"]))
    {
        assert_eq!(
            slice(question, &term["span"]),
            term["literal"].as_str().expect("literal")
        );
        assert_eq!(
            slice(question, &term["unitSpan"]),
            term["unitSurface"].as_str().expect("unit surface")
        );
    }
}

fn dimension_decline(answer: &SymbolicAnswer, id: &str, question: &str) -> bool {
    if id != "incompatible-units"
        || answer.intent != "unit_incompatibility"
        || !answer
            .answer
            .contains("different physical dimensions and cannot be converted into each other")
        || answer
            .answer
            .chars()
            .any(|character| character.is_ascii_digit() || character == '=')
        || !answer
            .evidence_links
            .iter()
            .any(|value| value.starts_with("unit_incompatibility:"))
    {
        return false;
    }
    let Ok((_, record)) = parse_indented(&answer.links_notation) else {
        return false;
    };
    if record.get("prompt").map(String::as_str) != Some(question)
        || record.get("intent").map(String::as_str) != Some(answer.intent.as_str())
        || record.get("answer").map(String::as_str) != Some(answer.answer.as_str())
    {
        return false;
    }
    record.get("steps").is_some_and(|steps| {
        steps.split("; ").any(|step| {
            let mut parts = step.splitn(3, ' ');
            let numbered = parts
                .next()
                .and_then(|value| value.strip_prefix("step_"))
                .is_some_and(|digits| {
                    !digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit())
                });
            numbered
                && parts.next() == Some("unit_incompatibility")
                && parts.next() == Some("kg:mass vs ms:time")
        })
    })
}

#[test]
fn all_shared_aggregate_vectors_pass_through_the_full_solver() {
    let document = vectors();
    for vector in document["cases"].as_array().expect("cases") {
        let question = vector["question"].as_str().expect("question");
        let answer = solve(question);
        let expected = &vector["expected"];
        let id = vector["id"].as_str().expect("vector id");
        if expected["kind"] == "comparison" {
            let holds = expected["holds"].as_bool().expect("verdict");
            let intent = if holds {
                "quantity_constraint_within"
            } else {
                "quantity_constraint_exceeds"
            };
            assert_eq!(answer.intent, intent, "{id}: {}", answer.answer);
            assert!(
                answer
                    .evidence_links
                    .contains(&format!("quantity_constraint:verdict:{holds}")),
                "{id}"
            );
            for key in ["total", "bound"] {
                assert!(
                    answer.answer.contains(&expected[key].to_string()),
                    "{id}: {key}"
                );
            }
            assert!(
                answer
                    .answer
                    .contains(expected["unit"].as_str().expect("unit")),
                "{id}"
            );
            let actual =
                frame(&answer).expect("full solver preserves raw frame JSON, not an event ID");
            assert_eq!(actual["status"], "supported", "{id}");
            assert_eq!(actual["aggregation"], "sum", "{id}");
            assert_eq!(actual["relation"], "le", "{id}");
            raw_spans(question, &actual);
        } else {
            assert!(
                !answer
                    .evidence_links
                    .iter()
                    .any(|value| value.starts_with("quantity_constraint:verdict:")),
                "{id}"
            );
            let decline = matches!(
                answer.intent.as_str(),
                "quantity_constraint_unknown" | "calculation_error" | "unknown"
            );
            let dimensions = dimension_decline(&answer, id, question);
            assert!(
                decline || dimensions,
                "{id}: {} / {}",
                answer.intent,
                answer.answer
            );
            assert!(!answer.answer.is_empty(), "{id}");
            if answer.intent == "quantity_constraint_unknown" {
                let actual = frame(&answer).expect("actual Unknown raw frame");
                assert_eq!(actual["status"], "unknown", "{id}");
                assert_eq!(actual["source"].as_str(), Some(question), "{id}");
                assert!(
                    actual["reason"]
                        .as_str()
                        .is_some_and(|reason| !reason.is_empty()),
                    "{id}"
                );
            }
        }
    }
}

#[test]
fn ordinary_arithmetic_stays_on_its_existing_full_solver_route() {
    let document = vectors();
    let question = document["arithmeticRegression"]["question"]
        .as_str()
        .expect("arithmetic question");
    let answer = solve(question);
    assert_eq!(answer.intent, "calculation");
    assert_eq!(
        answer.answer,
        document["arithmeticRegression"]["expectedContent"]
            .as_str()
            .expect("arithmetic answer")
    );
    assert!(frame(&answer).is_none());
}

#[test]
fn raw_utf8_spans_survive_nbsp_through_the_full_solver() {
    let document = vectors();
    let vector = document["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|vector| vector["id"] == "explicit-total-renamed-amounts")
        .expect("renamed case");
    let question = vector["question"]
        .as_str()
        .expect("question")
        .replace(' ', "\u{00a0}");
    let answer = solve(&question);
    assert_eq!(answer.intent, "quantity_constraint_within");
    let actual = frame(&answer).expect("actual raw frame");
    raw_spans(&question, &actual);
    let byte_start = actual["bound"]["span"]["start"]
        .as_u64()
        .expect("bound offset") as usize;
    assert!(byte_start > question[..byte_start].chars().count());
}

#[test]
fn concept_questions_are_not_claimed_as_aggregate_constraints() {
    for question in [
        "Is the sum operation associative?",
        "Is their combined total a useful column name?",
    ] {
        let answer = solve(question);
        assert!(frame(&answer).is_none());
        assert!(!answer.intent.starts_with("quantity_constraint_"));
        assert!(
            !answer
                .evidence_links
                .iter()
                .any(|value| value.starts_with("quantity_constraint:verdict:"))
        );
    }
}

#[test]
fn owned_unknown_constraints_cannot_enter_numeric_unit_conversion() {
    let document = vectors();
    for id in [
        "conditional-inclusion",
        "mixed-compatible-units",
        "explicit-not-within",
    ] {
        let vector = document["cases"]
            .as_array()
            .expect("cases")
            .iter()
            .find(|vector| vector["id"] == id)
            .expect("case");
        let question = vector["question"].as_str().expect("question");
        assert_eq!(
            formal_ai::capability_routing::claim_evidence_holds(
                "calculation_expression",
                question,
                question
            ),
            Some(true)
        );
        assert_eq!(
            formal_ai::capability_routing::claim_evidence_holds(
                "measured_quantity",
                question,
                question
            ),
            Some(false)
        );
        assert_eq!(solve(question).intent, "quantity_constraint_unknown");
    }
}

#[test]
fn unsafe_integer_operands_decline_without_a_numeric_verdict() {
    for question in [
        "The bound is 9007199254740992 ms. The amounts are 1 ms and 2 ms. Is their combined total within the bound?",
        "The bound is 10 ms. The amounts are -1 ms and 2 ms. Is their combined total within the bound?",
    ] {
        let answer = solve(question);
        assert_eq!(answer.intent, "quantity_constraint_unknown");
        assert!(
            !answer
                .evidence_links
                .iter()
                .any(|value| value.starts_with("quantity_constraint:verdict:"))
        );
        assert_eq!(
            frame(&answer).expect("source frame")["source"].as_str(),
            Some(question)
        );
    }
}

#[test]
fn dimension_refusal_requires_source_and_exact_observed_dimension_trace() {
    let document = vectors();
    let vector = document["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|vector| vector["id"] == "incompatible-units")
        .expect("incompatible case");
    let question = vector["question"].as_str().expect("question");
    let answer = solve(question);
    if answer.intent == "unit_incompatibility" {
        assert!(dimension_decline(&answer, "incompatible-units", question));
    } else {
        assert_eq!(answer.intent, "quantity_constraint_unknown");
        assert_eq!(
            frame(&answer).expect("Unknown source")["source"].as_str(),
            Some(question)
        );
    }
    let mut fake = answer.clone();
    fake.intent = "unit_incompatibility".to_owned();
    fake.links_notation.clear();
    assert!(!dimension_decline(&fake, "incompatible-units", question));
    fake = answer.clone();
    fake.evidence_links.clear();
    assert!(!dimension_decline(&fake, "incompatible-units", question));
    fake = answer.clone();
    fake.links_notation = fake
        .links_notation
        .replace("kg:mass vs ms:time", "kg:length vs ms:time");
    assert!(!dimension_decline(&fake, "incompatible-units", question));
    assert!(!dimension_decline(
        &answer,
        "incompatible-units",
        "a different source question"
    ));
    assert!(!dimension_decline(&answer, "unrelated-control", question));
}
