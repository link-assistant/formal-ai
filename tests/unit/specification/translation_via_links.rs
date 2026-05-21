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

#[test]
fn russian_translate_how_are_you_prompt_returns_english_surface() {
    // R213/R214: the answer is the deformalized surface form, preserving the
    // source fragment's lowercase casing and trailing question mark. No
    // `meaning: ...` / `surface (...)` template anymore.
    let response = answer("Переведи \"как у тебя дела?\" на английский.");
    assert_eq!(
        response.intent, "translate_ru_to_en",
        "Russian translation prompt should resolve to translation, got {}: {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("how are you?"),
        "translation should preserve the source's lowercase casing, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("How are you?"),
        "source was lowercase, so the target must not be capitalized; got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("surface ("),
        "natural translation must not use the robotic surface (lang): template; got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("meaning:"),
        "natural translation must not embed the meaning id in the body; got: {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language_from:ru"),
        "translation should record the Russian source language, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language_to:en"),
        "translation should record the English target language, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("meaning:")),
        "meaning id must remain available in evidence_links for traceability, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn russian_capitalized_how_are_you_keeps_target_capitalization() {
    // R214: when the source fragment starts capitalized, the target stays
    // capitalized.
    let response = answer("Переведи \"Как у тебя дела?\" на английский.");
    assert_eq!(response.intent, "translate_ru_to_en");
    assert!(
        response.answer.contains("How are you?"),
        "capitalized source should yield capitalized target, got: {}",
        response.answer,
    );
}

#[test]
fn natural_translation_drops_terminal_when_source_has_none() {
    // R214: terminal punctuation is mirrored — when the source has none, the
    // target has none either.
    let response = answer("Переведи \"как дела\" на английский.");
    assert_eq!(response.intent, "translate_ru_to_en");
    assert!(
        response.answer.contains("how are you") && !response.answer.contains("how are you?"),
        "source had no question mark, so target must not invent one; got: {}",
        response.answer,
    );
}

#[test]
fn issue_210_russian_translation_prompts_keep_translation_intent() {
    let cases: &[(&str, &str)] = &[
        ("Переведи \"кто ты такой\" на английский.", "who are you"),
        (
            "Переведи \"что это такое?\" на английский.",
            "what is this?",
        ),
        ("Переведи \"доброе яблоко\" на английский.", "good apple"),
    ];

    for (prompt, expected_surface) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "translate_ru_to_en",
            "translation prompt should not be routed to another handler for {prompt:?}; got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .answer
                .to_lowercase()
                .contains(&expected_surface.to_lowercase()),
            "expected English surface {expected_surface:?} for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[en] "),
            "translation must not fall back to a placeholder for {prompt:?}; got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("formal-ai"),
            "translation prompt must not return assistant identity/capabilities for {prompt:?}; got: {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "language_from:ru"),
            "translation should record Russian source language for {prompt:?}, got {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "language_to:en"),
            "translation should record English target language for {prompt:?}, got {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn translation_meaning_registry_covers_extended_phrases() {
    // R215: the formalize → meaning → deformalize pipeline must cover more
    // than the single hardcoded greeting_how_are_you id.
    let cases: &[(&str, &str, &str)] = &[
        (
            "Переведи \"спасибо\" на английский.",
            "translate_ru_to_en",
            "thank you",
        ),
        (
            "Переведи \"да\" на английский.",
            "translate_ru_to_en",
            "yes",
        ),
        (
            "Переведи \"нет\" на английский.",
            "translate_ru_to_en",
            "no",
        ),
        (
            "Переведи \"привет\" на английский.",
            "translate_ru_to_en",
            "hello",
        ),
        (
            "Translate \"hello\" to Russian",
            "translate_en_to_ru",
            "привет",
        ),
        (
            "Translate \"thank you\" to Russian",
            "translate_en_to_ru",
            "спасибо",
        ),
        ("Translate \"hello\" to Hindi", "translate_en_to_hi", "नमस्ते"),
        (
            "Translate \"hello\" to Chinese",
            "translate_en_to_zh",
            "你好",
        ),
    ];
    for (prompt, expected_intent, expected_substring) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, *expected_intent,
            "intent mismatch for prompt {prompt:?}, answer was: {}",
            response.answer,
        );
        assert!(
            response
                .answer
                .to_lowercase()
                .contains(&expected_substring.to_lowercase()),
            "expected target surface {expected_substring:?} in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[en] ")
                && !response.answer.contains("[ru] ")
                && !response.answer.contains("[hi] ")
                && !response.answer.contains("[zh] "),
            "registry-backed translation must not fall back to a [lang] placeholder; got: {}",
            response.answer,
        );
    }
}

#[test]
fn issue_216_translate_apple_to_russian_without_quotes() {
    // Issue #216: `translate apple to russian` (no quotes around `apple`)
    // used to return the placeholder `[ru]` because the surface
    // extractor only handled quoted fragments. The handler must now
    // recover the surface from the unquoted form and translate it.
    let cases: &[&str] = &[
        "translate apple to russian",
        "Translate apple to Russian",
        "translate apple to russian.",
        "translate Apple to Russian",
    ];
    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "translate_en_to_ru",
            "unquoted English→Russian prompt should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.contains("яблоко") || response.answer.contains("Яблоко"),
            "unquoted apple→russian must produce the Russian surface, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[ru]"),
            "unquoted apple→russian must not fall back to the [ru] placeholder, got: {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "language_to:ru"),
            "expected language_to:ru in evidence for {prompt:?}, got {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn issue_217_single_russian_noun_quoted() {
    // Issue #217: `переведи "яблоко" на английский` used to return the
    // placeholder `[en] яблоко` because the Wiktionary cache was empty
    // for the single noun. The cache is now seeded so the pipeline
    // resolves it to `apple`.
    let cases: &[&str] = &[
        "переведи \"яблоко\" на английский",
        "Переведи \"яблоко\" на английский.",
        "переведи «яблоко» на английский",
        "переведи 'яблоко' на английский",
    ];
    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "translate_ru_to_en",
            "quoted Russian noun should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.to_lowercase().contains("apple"),
            "expected `apple` in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[en]"),
            "single Russian noun must not fall back to [en] placeholder for {prompt:?}, got: {}",
            response.answer,
        );
    }
}

#[test]
fn issue_218_unquoted_russian_translation() {
    // Issue #218 / mirror of #216 in the Russian direction:
    // `переведи яблоко на английский` (no quotes) should also work.
    let response = answer("переведи яблоко на английский");
    assert_eq!(
        response.intent, "translate_ru_to_en",
        "unquoted Russian→English should route to translation, got {}: {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.to_lowercase().contains("apple"),
        "expected `apple` in answer, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("[en]"),
        "unquoted Russian→English must not fall back to [en], got: {}",
        response.answer,
    );
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
