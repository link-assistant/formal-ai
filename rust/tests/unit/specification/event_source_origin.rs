//! Append-site identity augments provenance without changing evidence contracts.

use formal_ai::event_log::EventLog;
use formal_ai::server::describe_turn;
use formal_ai::{ThinkingStep, solve};

#[test]
fn append_origin_preserves_raw_event_and_thinking_identity() {
    let mut plain = EventLog::new();
    let mut recorded = EventLog::new();
    let payload = "answered(calculation)=true";
    let before = plain.append("validation", payload);
    let after = recorded.append_with_origin("validation", "validation:answer", payload);
    assert_eq!(before, after);
    assert_eq!(plain.events(), recorded.events());
    assert_eq!(plain.evidence_links(), recorded.evidence_links());
    assert_eq!(plain.steps_block(), recorded.steps_block());
    let expected = plain.thinking_steps();
    let mut actual = recorded.thinking_steps();
    assert_eq!(
        actual[0].source_origin.as_deref(),
        Some("validation:answer")
    );
    actual[0].source_origin = None;
    assert_eq!(actual, expected);
    recorded.append("response", "4");
    assert!(recorded.thinking_steps()[1].source_origin.is_none());
}

#[test]
fn legacy_serialized_thinking_steps_keep_origin_optional() {
    let plain = ThinkingStep::new(0, "rule_verification", "", "detailed", "validation");
    let legacy = serde_json::to_value(&plain).unwrap();
    assert!(legacy.get("source_origin").is_none());
    assert_eq!(
        serde_json::from_value::<ThinkingStep>(legacy).unwrap(),
        plain
    );
    let mut recorded = plain;
    recorded.source_origin = Some("validation:answer".to_owned());
    let value = serde_json::to_value(&recorded).unwrap();
    assert_eq!(value["source_origin"], "validation:answer");
    assert_eq!(
        serde_json::from_value::<ThinkingStep>(value).unwrap(),
        recorded
    );
}

#[test]
fn executed_validation_identifies_answer_and_candidate_appenders() {
    for (prompt, origin, rust_symbol, js_symbol) in [
        (
            "What is 2 + 2?",
            "validation:answer",
            "record_answer_test",
            "solverEventLog",
        ),
        (
            "Choose a prime between 8 and 12",
            "validation:candidate",
            "record_validation",
            "solverPrimeIntervalValidation",
        ),
    ] {
        let answer = solve(prompt);
        let index = answer
            .thinking_steps
            .iter()
            .position(|step| step.source_event == "validation")
            .unwrap();
        assert_eq!(
            answer.thinking_steps[index].source_origin.as_deref(),
            Some(origin)
        );
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link.starts_with("validation:"))
        );
        let view = describe_turn(&answer.thinking_steps);
        assert_eq!(
            view.stages[index].rust.as_ref().unwrap().symbol,
            rust_symbol
        );
        assert_eq!(view.stages[index].js.as_ref().unwrap().symbol, js_symbol);
    }
}

#[test]
fn unknown_origin_does_not_fall_back_to_a_considered_appender() {
    let mut log = EventLog::new();
    log.append_with_origin("validation", "validation:unregistered", "executed=true");
    let view = describe_turn(&log.thinking_steps());
    assert!(view.stages[0].rust.is_none());
    assert!(view.stages[0].js.is_none());
}
