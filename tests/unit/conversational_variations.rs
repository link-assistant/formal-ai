//! The conversational wording-variation corpus, executed (issue #933).
//!
//! Issue #123 asked for "at least 5 variations per 4 languages" on every
//! conversational test case, and for CI to enforce it. The floor itself is
//! enforced by `tests/e2e/scripts/check-conversational-variation-floor.mjs`,
//! which only counts records. This test is the other half of the contract: it
//! answers every recorded prompt with the deterministic engine, so a case can
//! never reach the floor on wordings the router does not actually accept.
//!
//! Corpus layout mirrors the issue #819 benchmark: one manifest
//! (`data/benchmarks/conversational-variations-suite.lino`) listing the cases,
//! the languages and the per-language partition files.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use formal_ai::FormalAiEngine;
use unicode_general_category::get_general_category;
use unicode_normalization::UnicodeNormalization;

fn lino_records(text: &str) -> Vec<Vec<&str>> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current);
            current = Vec::new();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn lino_fields<'a>(record: &[&'a str], wanted: &str) -> Vec<&'a str> {
    record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .filter(|(name, _)| *name == wanted)
        .map(|(_, raw)| raw.trim().trim_matches('"'))
        .collect()
}

fn lino_field<'a>(record: &[&'a str], wanted: &str) -> &'a str {
    lino_fields(record, wanted)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing {wanted:?} in {record:?}"))
}

/// Two wordings count as one unless they differ in more than punctuation,
/// spacing or letter case -- the same rule the CI floor check applies, so the
/// two halves of the gate can never disagree about how many wordings a case
/// holds.
fn normalize_variation(prompt: &str) -> String {
    prompt
        .chars()
        .flat_map(char::to_lowercase)
        .nfkc()
        .filter(|character| {
            let family = get_general_category(*character).abbreviation().as_bytes()[0];
            !matches!(family, b'P' | b'S' | b'Z') && !character.is_whitespace()
        })
        .collect()
}

#[test]
fn conversational_variation_normalization_matches_the_unicode_ci_convention() {
    // The JavaScript gate applies NFKC and removes punctuation, symbols and
    // separators while retaining combining marks. The engine-side check must
    // make the same decisions, especially for full-width Chinese/Latin input
    // and Devanagari vowel marks.
    assert_eq!(normalize_variation("Ａ"), normalize_variation("A"));
    assert_eq!(normalize_variation("１"), normalize_variation("1"));
    assert_ne!(normalize_variation("क"), normalize_variation("का"));
}

#[test]
fn conversational_variation_benchmark_routes_every_case() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_text =
        fs::read_to_string(root.join("data/benchmarks/conversational-variations-suite.lino"))
            .expect("conversational variation suite manifest");
    let manifest = lino_records(&manifest_text);
    let suite = &manifest[0];
    assert_eq!(
        lino_field(suite, "record_type"),
        "conversational_variation_suite"
    );

    let minimum: usize = lino_field(suite, "minimum_variations_per_language")
        .parse()
        .expect("numeric minimum_variations_per_language");
    assert!(
        minimum >= 5,
        "issue #123 set the floor at five wordings per language, got {minimum}"
    );
    let expected_total: usize = lino_field(suite, "minimum_pass_count")
        .parse()
        .expect("numeric minimum_pass_count");
    let cases: Vec<&str> = lino_fields(suite, "case");
    let languages: Vec<&str> = lino_field(suite, "languages").split('|').collect();
    assert_eq!(languages, ["en", "ru", "hi", "zh"]);

    let mut wordings: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut answered = 0usize;

    for language in &languages {
        let partition = fs::read_to_string(root.join(format!(
            "data/benchmarks/conversational-variations/{language}.lino"
        )))
        .unwrap_or_else(|error| panic!("missing {language} variation partition: {error}"));

        for record in lino_records(&partition) {
            assert_eq!(
                lino_field(&record, "record_type"),
                "conversational_variation_case"
            );
            assert_eq!(
                lino_field(&record, "source"),
                "self_authored_multilingual_variation"
            );
            assert_eq!(lino_field(&record, "language"), *language);

            let id = lino_field(&record, "id");
            let case = lino_field(&record, "case");
            let prompt = lino_field(&record, "prompt");
            let expected_intent = lino_field(&record, "expected_intent");
            let expected_evidence = lino_field(&record, "expected_evidence");
            assert!(cases.contains(&case), "{id}: unlisted case {case:?}");

            let response = FormalAiEngine.answer(prompt);
            assert_eq!(
                response.intent, expected_intent,
                "{id}: prompt {prompt:?} should route to {expected_intent:?}, got {:?}",
                response.intent
            );
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link == expected_evidence),
                "{id}: prompt {prompt:?} should cite {expected_evidence:?}, got {:?}",
                response.evidence_links
            );
            // The corpus records the text each wording produces, so a reader
            // sees the answer without running anything (R234-2). Most records
            // pin it exactly; the capability answer is a multi-paragraph
            // listing, so those records pin its opening line instead of
            // inlining the whole page twenty-one times.
            let exact = lino_fields(&record, "expected_answer");
            let contained = lino_fields(&record, "expected_answer_contains");
            assert!(
                !exact.is_empty() || !contained.is_empty(),
                "{id}: prompt {prompt:?} records no answer; it answers {:?}",
                response.answer
            );
            for expected_answer in exact {
                assert_eq!(
                    response.answer, expected_answer,
                    "{id}: prompt {prompt:?} should answer exactly {expected_answer:?}"
                );
            }
            for expected_answer in contained {
                assert!(
                    response.answer.contains(expected_answer),
                    "{id}: answer {:?} should contain {expected_answer:?}",
                    response.answer
                );
            }

            let inserted = wordings
                .entry((case.to_string(), language.to_string()))
                .or_default()
                .insert(normalize_variation(prompt));
            assert!(
                inserted,
                "{id}: prompt {prompt:?} repeats an earlier wording"
            );
            answered += 1;
        }
    }

    for case in &cases {
        for language in &languages {
            let count = wordings
                .get(&(case.to_string(), language.to_string()))
                .map_or(0, BTreeSet::len);
            assert!(
                count >= minimum,
                "case {case} holds {count} {language} wording(s); the floor is {minimum}"
            );
        }
    }

    assert_eq!(answered, expected_total);
    assert_eq!(wordings.len(), cases.len() * languages.len());
}
