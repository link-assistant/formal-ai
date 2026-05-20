//! Issue #185: proof-shaped prompts (`Prove …`, `Докажи …`, `साबित कर …`,
//! `证明 …`) must route to the dedicated `proof_request` handler instead of
//! falling through to the unknown-intent fallback.

use formal_ai::FormalAiEngine;

#[test]
fn proof_requests_return_proof_response() {
    // Issue #185: "Prove determinism the way logic can handle paradoxes like
    // Godel's math incompleteness" previously fell through to the generic
    // unknown-intent error. Proof-shaped prompts must now route to a
    // dedicated `proof_request` handler that names the formalization
    // pipeline (relative-meta-logic) instead of the dead-end fallback.
    let cases = [
        "Prove determinism the way logic can handle paradoxes like Godel's math incompleteness",
        "Prove that 1 + 1 = 2",
        "Can you prove the Pythagorean theorem?",
        "Show that the square root of two is irrational",
        "Demonstrate that there are infinitely many primes",
        "Give me a proof of Fermat's little theorem",
        // Russian
        "Докажи, что 2 + 2 = 4",
        "Докажите теорему Пифагора",
        // Hindi
        "साबित करो कि 1 + 1 = 2",
        // Chinese
        "证明费马小定理",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "proof_request",
            "prompt {prompt:?} should resolve to proof_request intent (got {})",
            response.intent
        );
        assert!(
            response.answer.contains("relative-meta-logic"),
            "response for {prompt:?} should mention the relative-meta-logic pipeline, got: {}",
            response.answer
        );
        assert!(
            !response
                .answer
                .contains("cannot answer that from local Links Notation rules"),
            "prompt {prompt:?} should not return the unknown-intent error"
        );
    }
}

#[test]
fn godel_determinism_proof_request_mentions_axiom_set() {
    // Issue #185: the original reproduction prompt is philosophically
    // ambiguous because "determinism" is not a formal proposition on its
    // own. The handler must call this out and ask the user to supply an
    // axiom set so the proof can be reduced to a checkable claim.
    let response = FormalAiEngine.answer(
        "Prove determinism the way logic can handle paradoxes like Godel's math incompleteness",
    );

    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("axiom set"),
        "Gödel + determinism prompt should ask the user to supply an axiom set, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Laplacian") || response.answer.contains("Laplace"),
        "Gödel + determinism prompt should reference Laplacian determinism as a concrete \
         reduction example, got: {}",
        response.answer
    );
}

#[test]
fn proof_request_handler_does_not_swallow_opinion_questions() {
    // The proof handler must not be greedy: opinion questions that do not
    // ask for a proof should still resolve to `opinion_question`.
    let response = FormalAiEngine.answer("Do you think space is continuous or discrete");
    assert_eq!(
        response.intent, "opinion_question",
        "non-proof opinion question should still route to opinion_question"
    );
}

#[test]
fn proof_request_handler_does_not_swallow_concept_lookups() {
    // "What is a proof?" is a concept lookup, not a proof request.
    let response = FormalAiEngine.answer("What is a proof?");
    assert_ne!(
        response.intent, "proof_request",
        "concept lookups should not be hijacked by the proof_request handler"
    );
}
