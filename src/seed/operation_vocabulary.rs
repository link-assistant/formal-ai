//! Multilingual operation vocabulary loaded from
//! `data/seed/operation-vocabulary.lino`.
//!
//! A single shared, data-driven table so every reasoning handler recognises an
//! operation request equally in any supported language (`en|ru|hi|zh`) instead
//! of matching hardcoded English literals. Adding a new surface form — or a new
//! language — is a data edit, never a code change, which is what keeps "all
//! languages supported equally" a property of the seed rather than of scattered
//! `if prompt.contains("…")` branches.

use super::parser::parse_lino;
use super::OPERATION_VOCABULARY_LINO;

/// One canonical operation token plus the multilingual phrasing that
/// triggers it, parsed from `data/seed/operation-vocabulary.lino`.
///
/// Match semantics (mirrored wherever this table is consumed):
/// - `phrases`: the value is a substring of the normalized prompt.
/// - `combos`: every token (originally joined by `+` in the seed) is,
///   independently, a substring of the normalized prompt — order-free
///   "verb + object" triggers such as `extract + email`.
///
/// Prompts are expected to already be passed through
/// [`crate::engine::normalize_prompt`] (lower-cased, punctuation collapsed to
/// spaces, Unicode letters preserved) so Cyrillic, Devanagari, and CJK
/// synonyms match by substring exactly like ASCII ones.
#[derive(Debug, Clone, Default)]
pub struct OperationTrigger {
    pub canonical: String,
    pub phrases: Vec<String>,
    pub combos: Vec<Vec<String>>,
}

impl OperationTrigger {
    /// Does any phrase or combo for this operation appear in `normalized`?
    #[must_use]
    pub fn matches(&self, normalized: &str) -> bool {
        self.phrases
            .iter()
            .any(|phrase| normalized.contains(phrase.as_str()))
            || self.combos.iter().any(|combo| {
                !combo.is_empty()
                    && combo
                        .iter()
                        .all(|token| normalized.contains(token.as_str()))
            })
    }
}

/// The full multilingual operation vocabulary.
///
/// A single shared, data-driven table so every reasoning handler recognises
/// an operation request equally in any supported language (`en|ru|hi|zh`)
/// instead of matching hardcoded English literals. See
/// `data/seed/operation-vocabulary.lino`.
#[derive(Debug, Clone, Default)]
pub struct OperationVocabulary {
    pub operations: Vec<OperationTrigger>,
}

impl OperationVocabulary {
    /// Returns `true` when the operation with this canonical token is
    /// requested by the normalized prompt in any supported language.
    #[must_use]
    pub fn matches(&self, canonical: &str, normalized: &str) -> bool {
        self.operations
            .iter()
            .any(|op| op.canonical == canonical && op.matches(normalized))
    }

    /// Every canonical operation token whose phrasing appears in the
    /// normalized prompt, in declaration order.
    #[must_use]
    pub fn detect(&self, normalized: &str) -> Vec<String> {
        self.operations
            .iter()
            .filter(|op| op.matches(normalized))
            .map(|op| op.canonical.clone())
            .collect()
    }
}

#[must_use]
pub fn operation_vocabulary() -> OperationVocabulary {
    let tree = parse_lino(OPERATION_VOCABULARY_LINO);
    let mut vocabulary = OperationVocabulary::default();
    if let Some(root) = tree.children.first() {
        for child in root.children.iter().filter(|c| c.name == "operation") {
            let mut phrases = Vec::new();
            let mut combos = Vec::new();
            for entry in &child.children {
                match entry.name.as_str() {
                    "phrase" => phrases.push(entry.id.clone()),
                    "combo" => combos.push(
                        entry
                            .id
                            .split('+')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(ToOwned::to_owned)
                            .collect(),
                    ),
                    _ => {}
                }
            }
            vocabulary.operations.push(OperationTrigger {
                canonical: child.id.clone(),
                phrases,
                combos,
            });
        }
    }
    vocabulary
}

#[cfg(test)]
mod tests {
    use super::operation_vocabulary;

    #[test]
    fn operation_vocabulary_loads_every_canonical_operation() {
        let vocabulary = operation_vocabulary();
        let canonicals: std::collections::BTreeSet<String> = vocabulary
            .operations
            .iter()
            .map(|op| op.canonical.clone())
            .collect();
        for expected in [
            "uppercase",
            "lowercase",
            "replace",
            "reverse_words",
            "extract_email",
            "count_occurrences",
            "count_unique_words",
            "deduplicate_lines",
            "sort_lines",
        ] {
            assert!(
                canonicals.contains(expected),
                "missing operation {expected}"
            );
        }
    }

    #[test]
    fn operation_vocabulary_matches_each_supported_language() {
        let vocabulary = operation_vocabulary();
        // English literal, Russian, Hindi, and Chinese must all canonicalise
        // to the same `uppercase` operation so every supported language is
        // recognised equally (see agent-info.lino `supported_languages`).
        for normalized in [
            "uppercase this text",
            "переведи в верхний регистр",
            "इस पाठ को बड़े अक्षर में",
            "把文本转为大写",
        ] {
            assert!(
                vocabulary.matches("uppercase", normalized),
                "uppercase should match {normalized:?}",
            );
        }
    }

    #[test]
    fn operation_vocabulary_combos_require_every_token() {
        let vocabulary = operation_vocabulary();
        // "extract + email" is order-free but needs both tokens present.
        assert!(vocabulary.matches("extract_email", "please extract the email here"));
        assert!(!vocabulary.matches("extract_email", "extract the phone number"));
    }
}
