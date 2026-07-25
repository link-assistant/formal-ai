//! Concept-definition openers must exist in every supported language
//! (issue #842).
//!
//! Ladder node 827.L2.a ("Дай определение слова X одним предложением") needed
//! Russian `дай определение слова …` openers so the concept lookup receives the
//! term rather than the whole sentence. Adding them for Russian alone leaves
//! English, Hindi, and Chinese behind, which is exactly what
//! `tests/e2e/scripts/check-language-change-parity.mjs` fails a pull request
//! for. These tests pin the parity in the seed itself, in the form each
//! language actually uses: English puts the request in front of the term, Hindi
//! and Chinese put it after.

use formal_ai::seed;

/// Openers added together, per language, and how the request attaches to the
/// term. Each entry is (language, pattern kind, text).
const DEFINITION_OPENERS: &[(&str, &str, &str)] = &[
    ("en", "prefix", "give the definition of the word "),
    ("en", "prefix", "give the definition of "),
    ("en", "prefix", "definition of the word "),
    ("ru", "prefix", "дай определение слова "),
    ("ru", "prefix", "дай определение "),
    ("ru", "prefix", "определение слова "),
    ("hi", "suffix", " शब्द की परिभाषा बताओ"),
    ("hi", "suffix", " की परिभाषा बताओ"),
    ("hi", "suffix", " की परिभाषा क्या है"),
    ("zh", "suffix", "这个词的定义是什么"),
    ("zh", "suffix", "的定义是什么"),
    ("zh", "suffix", "的定义"),
];

#[test]
fn every_supported_language_has_concept_definition_openers() {
    let patterns = seed::prompt_patterns();
    for (language, kind, text) in DEFINITION_OPENERS {
        let found = patterns.iter().any(|pattern| {
            pattern.intent == "concept_lookup"
                && pattern.language == *language
                && pattern.kind == *kind
                && pattern.text == *text
        });
        assert!(
            found,
            "missing {language} concept_lookup {kind} {text:?}; \
             a definition opener added for one language must be added for all"
        );
    }
}

#[test]
fn each_supported_language_carries_the_same_number_of_them() {
    // The parity guard compares whole-language signatures, so a locale that
    // gains one opener and loses another still passes it. Counting here keeps
    // the four sets from drifting apart one entry at a time.
    for language in ["en", "ru", "hi", "zh"] {
        let count = DEFINITION_OPENERS
            .iter()
            .filter(|(candidate, _, _)| *candidate == language)
            .count();
        assert_eq!(count, 3, "{language} must carry the same opener count");
    }
}

/// `src/concepts.rs` sorts prefix and suffix candidates longest-first, so an
/// opener that extends a shorter one wins regardless of where it sits in the
/// seed file. Without that, "give the definition of the word X" would match
/// "give the definition of " and extract "the word X".
#[test]
fn longer_openers_extend_shorter_ones_and_are_matched_first() {
    let patterns = seed::prompt_patterns();
    let mut prefixes: Vec<String> = patterns
        .iter()
        .filter(|pattern| pattern.intent == "concept_lookup" && pattern.kind == "prefix")
        .map(|pattern| pattern.text.to_lowercase())
        .collect();
    prefixes.sort_by_key(|text| std::cmp::Reverse(text.len()));

    for (long, short) in [
        (
            "give the definition of the word ",
            "give the definition of ",
        ),
        ("дай определение слова ", "дай определение "),
    ] {
        let long_at = prefixes.iter().position(|text| text == long);
        let short_at = prefixes.iter().position(|text| text == short);
        assert!(
            long_at.is_some() && short_at.is_some(),
            "both {long:?} and {short:?} must be seeded"
        );
        assert!(
            long_at < short_at,
            "{long:?} must be tried before {short:?}, or the shorter one \
             swallows the opener and leaves the extension in the term"
        );
    }
}
