use formal_ai::solve;

#[test]
fn issue_531_pattern_vocabulary_resolves() {
    for term in [
        "sequence",
        "pattern",
        "repetition",
        "compression",
        "deduplication",
        "symmetry",
        "rotation",
        "reflection",
        "translation",
        "analogy",
        "invariant",
        "transformation",
    ] {
        let answer = solve(&format!("what is a {term}?"));
        if term == "sequence" {
            assert_eq!(
                answer.answer,
                "sequence (knowledge-representation): A sequence is an ordered collection of elements. In Links meta-theory a sequence is not a primitive: it is built from links, folded into a balanced tree of doublets where each doublet pairs two sub-sequences and every leaf is a point (a self-referential link modelling an atomic symbol). formal-ai reimplements the Data.Doublets.Sequences converters in Rust, so a text, an event stream, or a row of an image all become the same structure — a root link whose expansion reproduces the original elements. Sequences are treated as more basic than sets because order is retained; a set is the order-forgetting projection of a sequence.\n\nSource: docs/case-studies/issue-531/README.md (project-docs)."
            );
        }
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
