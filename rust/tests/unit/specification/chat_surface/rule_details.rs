use super::*;

#[test]
fn self_facts_answer_includes_model_id_and_strategy() {
    let response = answer("List all facts you know about yourself");
    assert_eq!(response.intent, "self_facts");
    assert_eq!(response.answer, SELF_FACTS_ANSWER);
    assert!(
        response.answer.to_lowercase().contains("formal-ai")
            || response.answer.to_lowercase().contains("symbolic"),
        "self facts should describe the model identity; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_uses_describe_prefix() {
    let response = answer("describe behavior rule unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(response.answer, ENGLISH_UNKNOWN_RULE_DETAIL);
    assert!(
        response.answer.contains("rule_unknown"),
        "describe prefix must surface the same detail as Show behavior rule; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_uses_read_rule_prefix() {
    let response = answer("Read rule unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(response.answer, ENGLISH_UNKNOWN_RULE_DETAIL);
    assert!(
        response.answer.contains("rule_unknown"),
        "Read rule prefix must surface the same detail as Show behavior rule; got: {}",
        response.answer
    );
}
