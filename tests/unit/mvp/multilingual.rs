//! Multilingual chat surface tests.
//!
//! `VISION.md` asks for chat in English, Russian, Hindi, Chinese, and later
//! other languages. These tests pin down the user-visible expectations.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectation: prototype English greeting.
// ---------------------------------------------------------------------------

#[test]
fn english_greeting_is_handled_today() {
    assert_eq!(answer("Hi").intent, "greeting");
    assert_eq!(answer("Hello").intent, "greeting");
    assert_eq!(answer("Hey").intent, "greeting");
}

// ---------------------------------------------------------------------------
// MVP expectations: Russian, Hindi, Chinese baseline greetings and identity.
// ---------------------------------------------------------------------------

#[test]
fn russian_greeting_returns_greeting_intent() {
    let response = answer("Привет");
    assert_eq!(response.intent, "greeting");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "Russian answers should tag the detected language"
    );
}

#[test]
fn russian_greeting_reply_is_in_russian() {
    let response = answer("Привет");
    assert!(
        response.answer.contains("Здравствуйте") || response.answer.contains("Привет"),
        "Russian greeting should be answered in Russian, got: {}",
        response.answer
    );
}

#[test]
fn hindi_greeting_returns_greeting_intent() {
    let response = answer("नमस्ते");
    assert_eq!(response.intent, "greeting");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "language:hi"));
}

#[test]
fn chinese_greeting_returns_greeting_intent() {
    let response = answer("你好");
    assert_eq!(response.intent, "greeting");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "language:zh"));
}

#[test]
fn russian_identity_question_returns_identity_intent() {
    let response = answer("Кто ты?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn hindi_identity_question_returns_identity_intent() {
    let response = answer("तुम कौन हो?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn chinese_identity_question_returns_identity_intent() {
    let response = answer("你是谁?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn every_multilingual_answer_declares_detected_language_link() {
    for prompt in ["Hi", "Привет", "你好", "नमस्ते"] {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("language:")),
            "missing language tag for prompt {prompt:?}"
        );
    }
}

#[test]
fn unknown_language_prompts_fall_back_to_english_with_unknown_language_link() {
    let response = answer("لطفاً سلام بگو");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:unknown"),
        "answers in unsupported languages should record an unknown-language link"
    );
    assert!(response.answer.contains("English"));
}

// ---------------------------------------------------------------------------
// Issue #16: "What is X?" style prompts must work in Russian, Hindi, Chinese.
// ---------------------------------------------------------------------------

#[test]
fn russian_concept_question_returns_concept_lookup_intent() {
    let response = answer("Что такое Википедия?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Russian concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
    assert!(
        response.answer.to_lowercase().contains("wikipedia")
            || response.answer.to_lowercase().contains("encyclopedia")
            || response.answer.to_lowercase().contains("википед"),
        "Russian Wikipedia answer should reference the concept, got: {}",
        response.answer
    );
}

#[test]
fn hindi_concept_question_returns_concept_lookup_intent() {
    let response = answer("विकिपीडिया क्या है?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Hindi concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
}

#[test]
fn chinese_concept_question_returns_concept_lookup_intent() {
    let response = answer("维基百科是什么?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Chinese concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
}
