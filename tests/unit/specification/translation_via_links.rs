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
fn issue_230_russian_compositional_translation_handles_search_phrase() {
    let response = answer("Переведи \"Найти синонимы или примеры согласования\" на ангилйский");
    assert_eq!(
        response.intent, "translate_ru_to_en",
        "reported prompt should remain a Russian to English translation, got {}: {}",
        response.intent, response.answer,
    );
    assert!(
        response
            .answer
            .contains("Find synonyms or examples of agreement"),
        "expected the reported phrase to translate compositionally, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("[en]") && !response.answer.contains("[En]"),
        "reported phrase must not fall back to an English placeholder, got: {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language_from:ru"),
        "translation should record Russian source language, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language_to:en"),
        "translation should record English target language, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn translation_gaps_are_reported_without_language_placeholders() {
    let cases: &[(&str, &str, &str, &str, &[&str])] = &[
        (
            "en",
            "Переведи \"неведомослово\" на английский",
            "translate_ru_to_en",
            "translation_gap:неведомослово",
            &["[en]", "[En]"],
        ),
        (
            "ru",
            "Translate \"zzqxqv\" to Russian",
            "translate_en_to_ru",
            "translation_gap:zzqxqv",
            &["[ru]", "[Ru]"],
        ),
        (
            "hi",
            "Translate \"zzqxqv\" to Hindi",
            "translate_en_to_hi",
            "translation_gap:zzqxqv",
            &["[hi]", "[Hi]"],
        ),
        (
            "zh",
            "Translate \"zzqxqv\" to Chinese",
            "translate_en_to_zh",
            "translation_gap:zzqxqv",
            &["[zh]", "[Zh]"],
        ),
    ];

    for (language, prompt, expected_intent, expected_gap, forbidden_placeholders) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, *expected_intent,
            "translation gap should stay on the translation handler for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        for placeholder in *forbidden_placeholders {
            assert!(
                !response.answer.contains(placeholder),
                "translation gap must not render placeholder {placeholder} for {prompt:?}, got: {}",
                response.answer,
            );
        }
        assert!(
            response
                .answer
                .to_lowercase()
                .contains("could not translate"),
            "translation gap should be explicit to the user for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == expected_gap),
            "translation gap must be traceable as {expected_gap}, got {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("language_to:{language}")),
            "translation gap should record target language {language} for {prompt:?}, got {:?}",
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
fn issue_216_unquoted_apple_covers_every_supported_target_language() {
    let cases: &[(&str, &str, &[&str], &str)] = &[
        (
            "translate apple to english",
            "translate_en_to_en",
            &["apple"],
            "language_to:en",
        ),
        (
            "translate apple to russian",
            "translate_en_to_ru",
            &["яблоко"],
            "language_to:ru",
        ),
        (
            "translate apple to hindi",
            "translate_en_to_hi",
            &["सेब"],
            "language_to:hi",
        ),
        (
            "translate apple to chinese",
            "translate_en_to_zh",
            &["蘋果", "苹果"],
            "language_to:zh",
        ),
    ];

    for (prompt, expected_intent, expected_surfaces, target_evidence) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, *expected_intent,
            "unquoted apple prompt should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            expected_surfaces
                .iter()
                .any(|expected| response.answer.contains(expected)),
            "expected one of {expected_surfaces:?} in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[en]")
                && !response.answer.contains("[ru]")
                && !response.answer.contains("[hi]")
                && !response.answer.contains("[zh]"),
            "supported target language must not fall back to a placeholder for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == target_evidence),
            "expected {target_evidence} in evidence for {prompt:?}, got {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn native_hindi_and_chinese_unquoted_translation_prompts_are_supported() {
    let cases: &[(&str, &str, &[&str], &str)] = &[
        (
            "apple का हिंदी में अनुवाद करो",
            "translate_en_to_hi",
            &["सेब"],
            "language_to:hi",
        ),
        (
            "把 apple 翻译成中文",
            "translate_en_to_zh",
            &["蘋果", "苹果"],
            "language_to:zh",
        ),
    ];

    for (prompt, expected_intent, expected_surfaces, target_evidence) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, *expected_intent,
            "native unquoted prompt should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            expected_surfaces
                .iter()
                .any(|expected| response.answer.contains(expected)),
            "expected one of {expected_surfaces:?} in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "language_from:en"),
            "native prompt with English surface should infer language_from:en for {prompt:?}, got {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == target_evidence),
            "expected {target_evidence} in evidence for {prompt:?}, got {:?}",
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

#[test]
fn issue_221_common_russian_nouns_translate_to_english() {
    // Issue #221: `Переведи "помидор" на английский.` produced the
    // placeholder `[en] помидор` because the offline cache only covered
    // `яблоко`. The fix seeds a broader common-noun set so the pipeline
    // resolves any common noun (tomato, cucumber, ...) to its target
    // surface offline.
    let cases: &[(&str, &str)] = &[
        ("Переведи \"помидор\" на английский.", "tomato"),
        ("Переведи \"огурец\" на английский.", "cucumber"),
        ("переведи \"картофель\" на английский", "potato"),
        ("переведи \"морковь\" на английский", "carrot"),
        ("переведи \"хлеб\" на английский", "bread"),
        ("переведи \"вода\" на английский", "water"),
    ];
    for (prompt, expected) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "translate_ru_to_en",
            "common Russian noun should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.to_lowercase().contains(expected),
            "expected {expected:?} in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[en]"),
            "common Russian noun must not fall back to [en] placeholder for {prompt:?}, got: {}",
            response.answer,
        );
    }
}

#[test]
fn issue_221_common_english_nouns_translate_to_russian() {
    let cases: &[(&str, &[&str])] = &[
        ("Translate \"tomato\" to Russian.", &["помидор", "томат"]),
        ("translate \"cucumber\" to russian", &["огурец"]),
        (
            "translate \"potato\" to russian",
            &["картофель", "картошка"],
        ),
        ("translate \"carrot\" to russian", &["морковь"]),
        ("translate \"bread\" to russian", &["хлеб"]),
        ("translate \"water\" to russian", &["вода"]),
    ];
    for (prompt, expected_any) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "translate_en_to_ru",
            "common English noun should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        let lower = response.answer.to_lowercase();
        assert!(
            expected_any.iter().any(|expected| lower.contains(expected)),
            "expected one of {expected_any:?} in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains("[ru]"),
            "common English noun must not fall back to [ru] placeholder for {prompt:?}, got: {}",
            response.answer,
        );
    }
}

#[test]
fn issue_221_unquoted_common_noun_works_in_all_languages() {
    // Unquoted forms must also work for the broader common-noun set,
    // not just the quoted variants. Mirrors issue #216 / #218 for the
    // longer tail of vocabulary.
    let cases: &[(&str, &str, &str, &str)] = &[
        (
            "translate tomato to russian",
            "translate_en_to_ru",
            "помидор",
            "[ru]",
        ),
        (
            "translate cucumber to russian",
            "translate_en_to_ru",
            "огурец",
            "[ru]",
        ),
        (
            "переведи помидор на английский",
            "translate_ru_to_en",
            "tomato",
            "[en]",
        ),
        (
            "переведи огурец на английский",
            "translate_ru_to_en",
            "cucumber",
            "[en]",
        ),
    ];
    for (prompt, expected_intent, expected_surface, placeholder) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, *expected_intent,
            "unquoted common-noun prompt should route to translation for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.to_lowercase().contains(expected_surface),
            "expected {expected_surface:?} in answer for {prompt:?}, got: {}",
            response.answer,
        );
        assert!(
            !response.answer.contains(placeholder),
            "unquoted common-noun translation must not fall back to {placeholder} for {prompt:?}, got: {}",
            response.answer,
        );
    }
}

// ---------------------------------------------------------------------------
// full-scope expectations.
// ---------------------------------------------------------------------------

#[test]
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
fn translation_request_returns_target_surface_form() {
    let response = answer("Translate 'Hello' to Russian");
    assert!(
        response.answer.contains("Привет") || response.answer.contains("Здравствуйте"),
        "Russian translation should return a Russian surface form, got: {}",
        response.answer
    );
}

#[test]
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
fn translation_trace_includes_intermediate_meaning() {
    let response = answer("Translate 'Hello' to Russian");
    assert!(
        response.links_notation.contains("meaning") && response.links_notation.contains("surface"),
        "translation trace must show projection into Links Notation and back to a surface form"
    );
}

#[test]
fn cross_language_code_translation_preserves_semantics() {
    let response = answer("Translate `def add(a, b): return a + b` from Python to Rust");
    assert!(response.intent.starts_with("translate_"));
    assert!(response.answer.contains("fn add"));
    assert!(response.answer.contains("a + b"));
}

#[test]
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

#[test]
fn issue_386_define_in_links_notation_resolves_to_the_links_notation_concept() {
    // Issue #386: the `try_translation` request-gate recognises a
    // "define <phrase> in links notation" command from *meaning* — the
    // `definition_command` verb and the `links_notation_format` markers seeded in
    // data/seed/meanings-translation.lino — rather than the hardcoded literals it
    // used before. The refactor is behaviour-preserving: across the full dispatch
    // pipeline the `concept_lookup` handler answers these prompts first (the phrase
    // "links notation" names a known concept), so the define-gate's routing never
    // changes the observable answer. This test locks that public contract so a
    // future dispatch-order change can't silently alter it; the seed→code wiring of
    // the gate itself is locked by the lib test
    // `define_in_links_roles_expose_the_scanned_surfaces` in src/seed/meanings.rs.
    let cases: &[&str] = &[
        "define `apple` in links notation",
        "define \"apple\" in links notation",
        "define `apple` в links notation",
        "define apple in links notation",
    ];
    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "concept_lookup",
            "define-in-links prompt should resolve to the Links Notation concept for {prompt:?}, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.starts_with("Links Notation (data-format):"),
            "expected the Links Notation concept definition for {prompt:?}, got: {}",
            response.answer,
        );
    }
}
