//! Issue #889 (parent #710): the thinking vocabulary lives in seed data and is
//! translated into every registered language.
//!
//! Before this change the sentences a thinking trace is made of were English
//! string literals inside `src/thinking.rs`, so the browser (which has its own
//! i18n catalog) narrated a Russian answer in Russian while the CLI, the
//! OpenAI/Anthropic APIs and Telegram narrated the very same answer in English.
//! These tests pin the data side of the fix: every intent the naturalizer can
//! render exists for every language in `crate::language::registered_languages`,
//! carries the same template fields, and stays distinct per language so a
//! missing translation cannot masquerade as a translated one.

use std::collections::{HashMap, HashSet};

use formal_ai::language::registered_languages;
use formal_ai::seed::multilingual_responses;
use formal_ai::thinking::thinking_prose_intents;

/// `(intent, language) -> text` for the thinking vocabulary only.
fn thinking_records() -> HashMap<(String, String), String> {
    multilingual_responses()
        .into_iter()
        .filter(|record| record.intent.starts_with("thinking_"))
        .map(|record| ((record.intent, record.language), record.text))
        .collect()
}

/// The named `{placeholder}` fields a template interpolates.
fn placeholders(text: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else { break };
        found.insert(after[..end].to_owned());
        rest = &after[end + 1..];
    }
    found
}

/// Every sentence the naturalizer can reach is translated into every registered
/// language — the acceptance criterion of the issue, checked as data rather than
/// per-surface prose so adding a language fails loudly here first.
#[test]
fn every_registered_language_translates_the_whole_thinking_vocabulary() {
    let records = thinking_records();
    let mut missing = Vec::new();
    for intent in thinking_prose_intents() {
        for language in registered_languages() {
            let key = (intent.clone(), language.slug().to_owned());
            match records.get(&key) {
                Some(text) if !text.trim().is_empty() => {}
                _ => missing.push(format!("{intent}/{}", language.slug())),
            }
        }
    }
    assert!(
        missing.is_empty(),
        "untranslated thinking prose: {missing:?}"
    );
}

/// The heading a surface labels a trace with is part of that vocabulary.
#[test]
fn the_trace_heading_is_translated_everywhere() {
    let records = thinking_records();
    for language in registered_languages() {
        let key = (
            String::from("thinking_trace_heading"),
            language.slug().to_owned(),
        );
        assert!(
            records
                .get(&key)
                .is_some_and(|text| !text.trim().is_empty()),
            "missing trace heading for {}",
            language.slug()
        );
    }
}

/// A translation that silently drops `{prompt}` or `{answer}` would render a
/// sentence with a hole in it, so every language's template must interpolate the
/// same fields as the English one.
#[test]
fn translations_keep_the_template_fields_of_the_english_record() {
    let records = thinking_records();
    let mut mismatched = Vec::new();
    for intent in thinking_prose_intents() {
        let Some(english) = records.get(&(intent.clone(), String::from("en"))) else {
            continue;
        };
        // `{article}` is an English grammatical device (a/an); other languages
        // are free to omit it.
        let mut expected = placeholders(english);
        expected.remove("article");
        for language in registered_languages() {
            if language.slug() == "en" {
                continue;
            }
            let Some(text) = records.get(&(intent.clone(), language.slug().to_owned())) else {
                continue;
            };
            let actual = placeholders(text);
            if !expected.is_subset(&actual) {
                mismatched.push(format!(
                    "{intent}/{}: expected {expected:?}, got {actual:?}",
                    language.slug()
                ));
            }
        }
    }
    assert!(
        mismatched.is_empty(),
        "thinking templates lost fields in translation: {mismatched:?}"
    );
}

/// A copy-pasted English record would pass the coverage test above while leaving
/// the surface untranslated, so the step sentences must actually differ from
/// English in every non-English language.
#[test]
fn translations_are_not_copies_of_the_english_record() {
    let records = thinking_records();
    for intent in thinking_prose_intents() {
        // Language *names* legitimately coincide across languages (Spanish
        // "chino" vs Italian would differ, but "espanol"/"español" style
        // overlaps exist), so only the sentences are compared.
        if !intent.starts_with("thinking_step_") && !intent.starts_with("thinking_narrative_") {
            continue;
        }
        let Some(english) = records.get(&(intent.clone(), String::from("en"))) else {
            continue;
        };
        for language in registered_languages() {
            if language.slug() == "en" {
                continue;
            }
            let Some(text) = records.get(&(intent.clone(), language.slug().to_owned())) else {
                continue;
            };
            assert_ne!(
                text,
                english,
                "{intent} is still English in {}",
                language.slug()
            );
        }
    }
}
