use formal_ai::solve;

#[test]
fn issue_531_pattern_vocabulary_resolves() {
    for term in [
        "sequence", "pattern", "repetition", "compression", "deduplication",
        "symmetry", "rotation", "reflection", "translation", "analogy",
        "invariant", "transformation",
    ] {
        let answer = solve(&format!("what is a {term}?"));
        assert_eq!(
            answer.intent, "concept_lookup",
            "'{term}' should route to concept_lookup, got intent {} / answer {}",
            answer.intent, answer.answer
        );
        assert!(
            answer.answer.to_lowercase().contains("link")
                || answer.answer.to_lowercase().contains(term),
            "answer for '{term}' should be grounded/on-topic: {}",
            answer.answer
        );
    }
}
