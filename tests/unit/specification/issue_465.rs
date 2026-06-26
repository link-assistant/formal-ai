//! Regression coverage for issue #465: pronoun follow-ups should preserve the
//! prior topic when asking a creator question.

use formal_ai::{ConversationTurn, UniversalSolver};

#[test]
fn pronoun_followup_resolves_prior_rust_topic_for_creator_question() {
    let solver = UniversalSolver::default();
    let first = solver.solve("What is Rust?");
    assert!(
        first.intent.starts_with("concept_lookup"),
        "first turn should establish the Rust topic, got: {}",
        first.intent
    );

    let history = [
        ConversationTurn::user("What is Rust?"),
        ConversationTurn::assistant(first.answer),
    ];
    let response = solver.solve_with_history("Who created it?", &history);

    assert_eq!(response.intent, "fact_lookup");
    assert!(
        response.answer.contains("Graydon Hoare"),
        "creator answer should name Graydon Hoare, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Mozilla"),
        "creator answer should include the Mozilla context, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q575650"),
        "Rust follow-up should keep the Rust Wikidata anchor, got: {:?}",
        response.evidence_links
    );
    assert!(
        response.links_notation.contains("coreference:resolved")
            && response.links_notation.contains("coreference:rewrite"),
        "follow-up trace should record the coreference resolution and rewrite: {}",
        response.links_notation
    );
}
