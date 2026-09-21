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
    "ого, чето начал соображать:)",
    "ого, что-то начал соображать",
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

const fn language_courtesy_response_fixtures() -> [(&'static str, &'static [&'static str]); 4] {
    [
        ("English", ENGLISH_COURTESY_RESPONSES),
        ("Russian", RUSSIAN_COURTESY_RESPONSES),
        ("Hindi", HINDI_COURTESY_RESPONSES),
        ("Chinese", CHINESE_COURTESY_RESPONSES),
    ]
}

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
            .contains("cannot answer that from local links rules"),
        "courteous follow-up should not return the unknown fallback: {}",
        response.answer
    );
}

#[test]
fn courtesy_response_matrix_is_classified_across_languages() {
    for (_language, prompts) in language_courtesy_response_fixtures() {
        assert_intent_for_each(prompts, "courtesy_response");
    }
}

#[test]
fn courtesy_response_matrix_never_falls_through_to_unknown() {
    for (_language, prompts) in language_courtesy_response_fixtures() {
        assert_intent_not(prompts, "unknown");
    }
}

// Issue #262: a Russian praise/acknowledgement after a correct answer should
// keep the same courtesy flow as the existing multilingual follow-ups.
#[test]
fn reported_russian_praise_is_a_courtesy_response() {
    assert_intent_for_each(&["ого, чето начал соображать:)"], "courtesy_response");
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
