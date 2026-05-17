//! Translation-via-Links tests.
//!
//! `VISION.md` argues that Links Notation is the language of meaning: every
//! human language is a surface form, and translation between languages
//! happens by first projecting both sentences into the same link network.
//! These tests pin down that contract for the full-scope.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: implementation already returns a Links Notation trace.
// ---------------------------------------------------------------------------

#[test]
fn every_answer_publishes_a_links_notation_trace() {
    let response = answer("Hi");
    assert!(!response.links_notation.is_empty());
}

// ---------------------------------------------------------------------------
// full-scope expectations.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "tracked requirement: translation between human languages should preserve the meaning link id"]
fn translation_preserves_meaning_id_across_languages() {
    let english = answer("Translate 'Hello, how are you?' to Russian");
    let russian = answer("Переведи 'Hello, how are you?' на русский");
    assert!(
        english.intent.starts_with("translate_") && russian.intent.starts_with("translate_"),
        "both prompts should be classified as translation requests"
    );
    let english_meaning_id = english
        .evidence_links
        .iter()
        .find(|link| link.starts_with("meaning:"));
    let russian_meaning_id = russian
        .evidence_links
        .iter()
        .find(|link| link.starts_with("meaning:"));
    assert!(english_meaning_id.is_some() && english_meaning_id == russian_meaning_id);
}

#[test]
#[ignore = "tracked requirement: translation request must return the target-language surface form"]
fn translation_request_returns_target_surface_form() {
    let response = answer("Translate 'Hello' to Russian");
    assert!(
        response.answer.contains("Привет") || response.answer.contains("Здравствуйте"),
        "Russian translation should return a Russian surface form, got: {}",
        response.answer
    );
}

#[test]
#[ignore = "tracked requirement: synonyms across languages should hash to the same meaning link"]
fn synonyms_across_languages_share_meaning() {
    let a = answer("Define 'hello' as a Links Notation record");
    let b = answer("Опиши 'привет' как запись Links Notation");
    let a_meaning = a
        .evidence_links
        .iter()
        .find(|link| link.starts_with("meaning:"));
    let b_meaning = b
        .evidence_links
        .iter()
        .find(|link| link.starts_with("meaning:"));
    assert_eq!(
        a_meaning, b_meaning,
        "synonyms across languages should collapse to a single meaning link"
    );
}

#[test]
#[ignore = "tracked requirement: translation must declare both source and target language tags"]
fn translation_declares_source_and_target_language_tags() {
    let response = answer("Translate 'Hello' from English to Russian");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "language_from:en"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "language_to:ru"));
}

#[test]
#[ignore = "tracked requirement: translation traces must include the intermediate Links Notation meaning record"]
fn translation_trace_includes_intermediate_meaning() {
    let response = answer("Translate 'Hello' to Russian");
    assert!(
        response.links_notation.contains("meaning") && response.links_notation.contains("surface"),
        "translation trace must show projection into Links Notation and back to a surface form"
    );
}

#[test]
#[ignore = "tracked requirement: cross-language code translation must preserve runnable semantics"]
fn cross_language_code_translation_preserves_semantics() {
    let response = answer("Translate `def add(a, b): return a + b` from Python to Rust");
    assert!(response.intent.starts_with("translate_"));
    assert!(response.answer.contains("fn add"));
    assert!(response.answer.contains("a + b"));
}

#[test]
#[ignore = "tracked requirement: untranslatable concepts must be flagged rather than approximated silently"]
fn untranslatable_concepts_are_flagged() {
    let response = answer("Translate 'тоска' to English in one word");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("translation_gap:")),
        "translation gaps must be marked explicitly, not papered over"
    );
}
