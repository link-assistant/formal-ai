//! Multilingual operation vocabulary loaded from
//! `data/seed/operation-vocabulary.lino`.

use std::collections::BTreeMap;

use super::parser::parse_lino;
use super::OPERATION_VOCABULARY_LINO;

/// Localized surface forms for one operation in one supported language.
#[derive(Debug, Clone, Default)]
pub struct OperationLanguageForms {
    pub phrases: Vec<String>,
    pub combos: Vec<Vec<String>>,
}

impl OperationLanguageForms {
    fn matches(&self, normalized: &str) -> bool {
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

/// One canonical operation token plus localized trigger phrases.
#[derive(Debug, Clone, Default)]
pub struct OperationTrigger {
    pub canonical: String,
    pub languages: BTreeMap<String, OperationLanguageForms>,
}

impl OperationTrigger {
    /// Does any phrase or combo for this operation appear in `normalized`?
    #[must_use]
    pub fn matches(&self, normalized: &str) -> bool {
        self.languages
            .values()
            .any(|forms| forms.matches(normalized))
    }
}

/// The full multilingual operation vocabulary.
#[derive(Debug, Clone, Default)]
pub struct OperationVocabulary {
    pub operations: Vec<OperationTrigger>,
}

impl OperationVocabulary {
    /// Returns `true` when the operation with this canonical token is requested
    /// by the normalized prompt in any supported language.
    #[must_use]
    pub fn matches(&self, canonical: &str, normalized: &str) -> bool {
        self.operations
            .iter()
            .any(|op| op.canonical == canonical && op.matches(normalized))
    }

    /// Every canonical operation token whose phrasing appears in the normalized
    /// prompt, in declaration order.
    #[must_use]
    pub fn detect(&self, normalized: &str) -> Vec<String> {
        self.operations
            .iter()
            .filter(|op| op.matches(normalized))
            .map(|op| op.canonical.clone())
            .collect()
    }

    /// Append canonical English operation tokens to a normalized prompt.
    ///
    /// Handlers can keep their canonical matching logic while accepting native
    /// verbs from `operation-vocabulary.lino`.
    #[must_use]
    pub fn canonicalized_prompt(&self, normalized: &str) -> String {
        let detected = self.detect(normalized);
        if detected.is_empty() {
            return normalized.to_owned();
        }

        let mut out = String::from(normalized);
        for canonical in detected {
            out.push(' ');
            out.push_str(&canonical);
            let phrase = canonical.replace('_', " ");
            if phrase != canonical {
                out.push(' ');
                out.push_str(&phrase);
            }
        }
        out
    }
}

#[must_use]
pub fn operation_vocabulary() -> OperationVocabulary {
    let tree = parse_lino(OPERATION_VOCABULARY_LINO);
    let mut vocabulary = OperationVocabulary::default();
    if let Some(root) = tree.children.first() {
        for operation_node in root.children.iter().filter(|c| c.name == "operation") {
            let mut languages = BTreeMap::new();
            for language_node in operation_node
                .children
                .iter()
                .filter(|c| c.name == "language")
            {
                let mut forms = OperationLanguageForms::default();
                for entry in &language_node.children {
                    match entry.name.as_str() {
                        "phrase" => forms.phrases.push(entry.id.clone()),
                        "combo" => forms.combos.push(split_combo(&entry.id)),
                        _ => {}
                    }
                }
                languages.insert(language_node.id.clone(), forms);
            }
            vocabulary.operations.push(OperationTrigger {
                canonical: operation_node.id.clone(),
                languages,
            });
        }
    }
    vocabulary
}

fn split_combo(raw: &str) -> Vec<String> {
    raw.split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::operation_vocabulary;

    fn supported_languages() -> BTreeSet<String> {
        crate::seed::agent_info()
            .get("supported_languages")
            .expect("agent-info must define supported_languages")
            .split('|')
            .map(ToOwned::to_owned)
            .collect()
    }

    #[test]
    fn operation_vocabulary_loads_every_canonical_operation() {
        let vocabulary = operation_vocabulary();
        let canonicals: BTreeSet<String> = vocabulary
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
            "path_argument",
            "reverse_sort",
            "function",
            "implement",
            "write",
            "return",
            "tuple",
            "numbers",
            "vowels",
            "count_vowels",
            "similar_elements",
            "distinct_numbers",
            "differ",
            "threshold",
        ] {
            assert!(
                canonicals.contains(expected),
                "missing operation {expected}"
            );
        }
    }

    #[test]
    fn operation_vocabulary_covers_every_supported_language_per_operation() {
        let supported = supported_languages();
        let vocabulary = operation_vocabulary();
        for operation in vocabulary.operations {
            let languages = operation.languages.keys().cloned().collect::<BTreeSet<_>>();
            assert_eq!(
                languages, supported,
                "{} must define synonyms for every supported language",
                operation.canonical
            );
        }
    }

    #[test]
    fn operation_vocabulary_canonicalizes_native_verbs() {
        let vocabulary = operation_vocabulary();
        let uppercase = vocabulary.canonicalized_prompt("переведи в верхний регистр");
        assert!(uppercase.contains("uppercase"), "{uppercase}");

        let synthesis =
            vocabulary.canonicalized_prompt("реализуй python функцию count_vowels верни гласных");
        for expected in [
            "implement",
            "function",
            "count_vowels",
            "count vowels",
            "return",
        ] {
            assert!(synthesis.contains(expected), "{synthesis}");
        }
    }

    #[test]
    fn operation_vocabulary_canonicalizes_hindi_program_prompt() {
        let vocabulary = operation_vocabulary();
        let synthesis = vocabulary.canonicalized_prompt(
            "python फ़ंक्शन count_vowels(text: str) -> int लागू करें। पाठ में स्वरों की संख्या लौटाएँ।",
        );

        for expected in [
            "function",
            "implement",
            "return",
            "vowels",
            "count_vowels",
            "count vowels",
        ] {
            assert!(synthesis.contains(expected), "{synthesis}");
        }
    }

    #[test]
    fn operation_vocabulary_combos_require_every_token() {
        let vocabulary = operation_vocabulary();
        assert!(vocabulary.matches("extract_email", "please extract the email here"));
        assert!(!vocabulary.matches("extract_email", "extract the phone number"));
    }
}
