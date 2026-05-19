use formal_ai::FormalAiEngine;

const ENGLISH_COURTESY_RESPONSES: &[&str] = &[
    "I am fine, thank you",
    "I'm fine, thanks",
    "fine thanks",
    "thank you",
    "thanks",
];
const RUSSIAN_COURTESY_RESPONSES: &[&str] = &[
    "спасибо",
    "благодарю",
    "у меня все хорошо спасибо",
    "всё хорошо спасибо",
    "хорошо спасибо",
];
const HINDI_COURTESY_RESPONSES: &[&str] = &[
    "धन्यवाद",
    "शुक्रिया",
    "मैं ठीक हूँ धन्यवाद",
    "ठीक हूँ धन्यवाद",
    "मैं अच्छा हूँ धन्यवाद",
];
const CHINESE_COURTESY_RESPONSES: &[&str] =
    &["谢谢", "我很好谢谢", "我很好 谢谢", "好的谢谢", "好的 谢谢"];

// Issue #160: a polite small-talk follow-up after the assistant greeting was
// falling through to the learnable-rule fallback.
#[test]
fn courteous_follow_up_is_recognized() {
    let response = FormalAiEngine.answer("I am fine, thank you");

    assert_eq!(response.intent, "courtesy_response");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:courtesy_response"),
        "courteous follow-up should cite response:courtesy_response",
    );
    assert!(
        !response
            .answer
            .contains("cannot answer that from local Links Notation rules"),
        "courteous follow-up should not return the unknown fallback: {}",
        response.answer
    );
}

#[test]
fn courtesy_response_matrix_is_classified_across_languages() {
    for prompts in [
        ENGLISH_COURTESY_RESPONSES,
        RUSSIAN_COURTESY_RESPONSES,
        HINDI_COURTESY_RESPONSES,
        CHINESE_COURTESY_RESPONSES,
    ] {
        assert_intent_for_each(prompts, "courtesy_response");
    }
}

#[test]
fn courtesy_response_matrix_never_falls_through_to_unknown() {
    for prompts in [
        ENGLISH_COURTESY_RESPONSES,
        RUSSIAN_COURTESY_RESPONSES,
        HINDI_COURTESY_RESPONSES,
        CHINESE_COURTESY_RESPONSES,
    ] {
        assert_intent_not(prompts, "unknown");
    }
}

fn assert_intent_for_each(prompts: &[&str], expected_intent: &str) {
    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, expected_intent,
            "prompt {prompt:?} should yield intent {expected_intent:?}, got intent={} answer={}",
            response.intent, response.answer,
        );
    }
}

fn assert_intent_not(prompts: &[&str], forbidden_intent: &str) {
    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        assert_ne!(
            response.intent, forbidden_intent,
            "prompt {prompt:?} should not yield intent {forbidden_intent:?}, got answer={}",
            response.answer,
        );
    }
}
