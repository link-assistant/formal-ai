//! Translation source/target language detection.
//!
//! Neither routine matches a hardcoded natural-language phrase. Each asks the
//! language-independent meaning lexicon (`data/seed/meanings-translation.lino`)
//! "which surface forms evidence a translation *source* / *target* marker?" and
//! resolves the marker's language by walking its `defined_by` edges down to one
//! of the four `language_*` meanings. Adding a spelling variant, a synonym, or a
//! whole new supported language is therefore a pure data edit: drop a
//! `word`/`description` into the relevant marker meaning and this code reasons
//! about it automatically.
//!
//! Surfaces are matched as raw substrings ([`str::contains`]), exactly as the
//! previous hardcoded disjunction did, so detection stays byte-faithful — a
//! Chinese marker like `从中文` has no inter-word spaces, and a Cyrillic marker
//! like `с английского` must match inside a longer sentence. Marker meanings are
//! walked in declaration order (English → Russian → Hindi → Chinese), which
//! preserves the original first-match priority.

use crate::seed::{self, Meaning, ROLE_TRANSLATION_SOURCE_MARKER, ROLE_TRANSLATION_TARGET_MARKER};

/// Detect the language a translation reads *from*, or `None`.
///
/// Walks every meaning carrying [`ROLE_TRANSLATION_SOURCE_MARKER`] and returns
/// the language of the first whose surface appears in `normalized`.
pub fn detect_source_language(normalized: &str) -> Option<&'static str> {
    detect_marker_language(ROLE_TRANSLATION_SOURCE_MARKER, normalized)
}

/// Detect the language a translation renders *into*, or `None`.
///
/// Walks every meaning carrying [`ROLE_TRANSLATION_TARGET_MARKER`] and returns
/// the language of the first whose surface appears in `normalized`.
pub fn detect_target_language(normalized: &str) -> Option<&'static str> {
    detect_marker_language(ROLE_TRANSLATION_TARGET_MARKER, normalized)
}

/// The shared recogniser: the first marker meaning of `role` (in declaration
/// order) whose any surface word is a substring of `normalized` reports its
/// language, read off the `language_*` meaning it is `defined_by`.
fn detect_marker_language(role: &str, normalized: &str) -> Option<&'static str> {
    seed::lexicon()
        .meanings_with_role(role)
        .filter(|meaning| meaning.words().any(|word| normalized.contains(word)))
        .find_map(language_code_of)
}

/// The ISO 639-1 code of the `language_*` meaning a marker is `defined_by`.
fn language_code_of(meaning: &Meaning) -> Option<&'static str> {
    meaning
        .defined_by
        .iter()
        .find_map(|slug| language_code(slug))
}

/// Map a `language_*` meaning slug to its fixed ISO 639-1 code.
///
/// The code is the one identifier that stays in the handler: it is the key the
/// [`crate::translation::TranslationPipeline`] and the Wiktionary client are
/// addressed by, not a surface word. The surface *names* of each language live
/// in the seed; only this slug → code bridge is code.
fn language_code(slug: &str) -> Option<&'static str> {
    match slug {
        "language_english" => Some("en"),
        "language_russian" => Some("ru"),
        "language_hindi" => Some("hi"),
        "language_chinese" => Some("zh"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_source_language() {
        assert_eq!(
            detect_source_language("translate apple from english"),
            Some("en")
        );
        assert_eq!(detect_source_language("apple с русского"), Some("ru"));
        assert_eq!(detect_source_language("apple हिंदी से"), Some("hi"));
        assert_eq!(detect_source_language("从中文翻译 apple"), Some("zh"));
        assert_eq!(detect_source_language("what is apple"), None);
    }

    #[test]
    fn detects_each_target_language() {
        assert_eq!(
            detect_target_language("translate apple to english"),
            Some("en")
        );
        assert_eq!(detect_target_language("apple на русский"), Some("ru"));
        assert_eq!(detect_target_language("apple हिंदी में"), Some("hi"));
        assert_eq!(detect_target_language("apple 成中文"), Some("zh"));
        assert_eq!(detect_target_language("what is apple"), None);
    }

    #[test]
    fn combined_prompt_reads_both_directions() {
        let normalized = "translate apple from english to russian";
        assert_eq!(detect_source_language(normalized), Some("en"));
        assert_eq!(detect_target_language(normalized), Some("ru"));
    }

    #[test]
    fn source_language_names_do_not_leak_into_target() {
        // "english to russian" must not register English as the *target*:
        // "to english" is not a substring here.
        assert_eq!(
            detect_target_language("translate apple from english to russian"),
            Some("ru")
        );
    }
}
