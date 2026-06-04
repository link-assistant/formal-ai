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
    /// The canonical operation this one undoes, when declared via an `inverse`
    /// child in `operation-vocabulary.lino`. Subtractive program-plan rules are
    /// *derived* from this declaration (issue #386), so adding a "cancel X"
    /// operation stays pure seed data rather than new control flow.
    pub inverse_of: Option<String>,
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

    /// Every declared `(canonical, base)` inverse relationship, where `canonical`
    /// is the operation that undoes `base` (e.g. `("cancel_reverse_sort",
    /// "reverse_sort")`).
    ///
    /// The program-plan engine derives subtractive substitution rules from these
    /// pairs (issue #386), so a new "cancel X" stays pure seed data instead of
    /// requiring new branching logic.
    #[must_use]
    pub fn inverse_pairs(&self) -> Vec<(String, String)> {
        self.operations
            .iter()
            .filter_map(|op| {
                op.inverse_of
                    .as_ref()
                    .map(|base| (op.canonical.clone(), base.clone()))
            })
            .collect()
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
            let inverse_of = match operation_node.find_child_value("inverse") {
                "" => None,
                base => Some(base.to_owned()),
            };
            vocabulary.operations.push(OperationTrigger {
                canonical: operation_node.id.clone(),
                languages,
                inverse_of,
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
            "cancel_reverse_sort",
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
            "reverse",
            "sum",
            "product",
            "minimum",
            "maximum",
            "code_request",
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

    #[test]
    fn operation_vocabulary_exposes_declared_inverse_pairs() {
        let vocabulary = operation_vocabulary();
        let pairs = vocabulary.inverse_pairs();
        assert!(
            pairs
                .iter()
                .any(|(canonical, base)| canonical == "cancel_reverse_sort"
                    && base == "reverse_sort"),
            "cancel_reverse_sort must declare reverse_sort as its inverse: {pairs:?}"
        );

        // Every base named by an inverse pair must itself be a real operation, so
        // derived subtractive rules can never reference a phantom modifier.
        let canonicals: BTreeSet<String> = vocabulary
            .operations
            .iter()
            .map(|op| op.canonical.clone())
            .collect();
        for (canonical, base) in &pairs {
            assert!(
                canonicals.contains(base),
                "{canonical} declares inverse {base}, which is not a known operation"
            );
        }
    }

    #[test]
    fn operation_vocabulary_cancel_reverse_sort_matches_native_phrasings() {
        let vocabulary = operation_vocabulary();
        for prompt in [
            "please cancel the sorting",
            "отмени сортировку",
            "убери сортировку",
            "सॉर्ट हटाओ",
            "取消排序",
        ] {
            assert!(
                vocabulary.matches("cancel_reverse_sort", prompt),
                "cancel_reverse_sort must match {prompt:?}"
            );
        }
    }
}
