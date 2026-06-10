use std::collections::BTreeSet;

use super::operation_vocabulary;

fn supported_languages() -> BTreeSet<String> {
    crate::seed::supported_languages().into_iter().collect()
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
        "sort",
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
            .any(|(canonical, base)| canonical == "cancel_reverse_sort" && base == "reverse_sort"),
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
