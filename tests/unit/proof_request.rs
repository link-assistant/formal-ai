//! Issue #185: proof-shaped prompts (`Prove …`, `Докажи …`, `साबित कर …`,
//! `证明 …`) must route to the dedicated `proof_request` handler and now
//! produce a real proof body (`Proven` / `Disproven` / `PartialPlan`) — never
//! the unknown-intent fallback and never a structured refusal.

use formal_ai::FormalAiEngine;

#[test]
fn proof_requests_return_proof_response() {
    // Every proof-shaped prompt must reach the proof_request handler and
    // come back with a real proof body. A few rules apply to every case:
    //   1. The intent is `proof_request`.
    //   2. The answer does not contain the unknown-intent fallback.
    //   3. The answer is non-trivial (long enough to embed a worked proof
    //      or a structured plan).
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
            !response
                .answer
                .contains("cannot answer that from local Links Notation rules"),
            "prompt {prompt:?} should not return the unknown-intent error"
        );
        assert!(
            !response
                .answer
                .to_lowercase()
                .contains("i cannot discharge"),
            "prompt {prompt:?} should not return the legacy structured refusal, got: {}",
            response.answer
        );
        assert!(
            response.answer.len() > 80,
            "prompt {prompt:?} should produce a worked-out proof body, got: {}",
            response.answer
        );
    }
}

#[test]
fn arithmetic_proof_request_contains_evaluated_values() {
    // "Prove that 1 + 1 = 2" must route through the arithmetic engine and
    // include the evaluated values in the discharged proof.
    let response = FormalAiEngine.answer("Prove that 1 + 1 = 2");
    assert_eq!(response.intent, "proof_request");
    assert!(response.answer.contains("∎"));
    assert!(
        response.answer.contains("1 + 1 = 2"),
        "arithmetic proof should restate the claim, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .to_lowercase()
            .contains("direct calculation"),
        "arithmetic proof should label the method, got: {}",
        response.answer
    );
}

#[test]
fn arithmetic_disproof_reports_counterexample() {
    let response = FormalAiEngine.answer("Prove that 2 + 2 = 5");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("counterexample")
            || response.answer.to_lowercase().contains("disproof"),
        "false arithmetic claim should be disproven, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains('4'),
        "disproof should report the actual evaluated value 4, got: {}",
        response.answer
    );
}

#[test]
fn pythagorean_request_contains_textbook_proof() {
    let response = FormalAiEngine.answer("Can you prove the Pythagorean theorem?");
    assert_eq!(response.intent, "proof_request");
    assert!(response.answer.contains("∎"));
    assert!(
        response.answer.to_lowercase().contains("right triangle")
            || response.answer.contains("a² + b² = c²"),
        "Pythagorean proof should mention right triangles or a² + b² = c², got: {}",
        response.answer
    );
}

#[test]
fn sqrt_two_proof_uses_contradiction() {
    let response = FormalAiEngine.answer("Show that the square root of two is irrational");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("contradiction"),
        "√2 proof should be done by contradiction, got: {}",
        response.answer
    );
    assert!(response.answer.contains("∎"));
}

#[test]
fn euclid_primes_proof_is_returned() {
    let response = FormalAiEngine.answer("Demonstrate that there are infinitely many primes");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("contradiction")
            || response.answer.to_lowercase().contains("euclid")
            || response.answer.contains("p₁"),
        "Euclid's proof should be discharged, got: {}",
        response.answer
    );
}

#[test]
fn russian_greeting_primes_prompt_returns_euclid_proof() {
    // Issue #209: the real report included a greeting and the compact Russian
    // wording "простых бесконечно". That must still resolve to the known
    // theorem instead of falling back to a generic axiom-set plan.
    let response = FormalAiEngine.answer("привет. докажи что простых бесконечно");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("Простых чисел бесконечно много")
            || response.answer.contains("p₁"),
        "Russian prime-infinitude request should return Euclid's proof, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("План доказательства"),
        "known theorem should not return the generic proof plan, got: {}",
        response.answer
    );
}

#[test]
fn fermat_little_proof_uses_induction() {
    let response = FormalAiEngine.answer("Give me a proof of Fermat's little theorem");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("induction") || response.answer.contains("aᵖ"),
        "Fermat's little theorem proof should mention induction or aᵖ, got: {}",
        response.answer
    );
}

#[test]
fn russian_pythagoras_returns_russian_proof() {
    let response = FormalAiEngine.answer("Докажите теорему Пифагора");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("прямоугольн") || response.answer.contains("Пифагор"),
        "Russian Pythagorean proof should use Russian terminology, got: {}",
        response.answer
    );
}

#[test]
fn chinese_fermat_little_returns_chinese_proof() {
    let response = FormalAiEngine.answer("证明费马小定理");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("素数")
            || response.answer.contains("归纳")
            || response.answer.contains("费马"),
        "Chinese Fermat proof should use Chinese terminology, got: {}",
        response.answer
    );
}

#[test]
fn godel_determinism_proof_request_mentions_axiom_set() {
    // Issue #185: the original reproduction prompt is philosophically
    // ambiguous because "determinism" is not a formal proposition on its
    // own. The handler must call this out and ask the user to supply an
    // axiom set so the proof can be reduced to a checkable claim. The
    // engine returns a PartialPlan that walks through Laplacian
    // determinism inside Newton's axioms and references Gödel's first
    // incompleteness theorem as the limit.
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
    assert!(
        response.answer.contains("Picard")
            || response.answer.contains("incompleteness")
            || response.answer.contains("Gödel"),
        "Gödel + determinism prompt should reference Picard–Lindelöf or Gödel's incompleteness, \
         got: {}",
        response.answer
    );
}

#[test]
fn unknown_theorem_returns_partial_plan_not_refusal() {
    // The engine should never refuse — when it does not have the proof in
    // its library, it must hand back a structured PartialPlan that names
    // the supported axiom sets and proof techniques.
    let response = FormalAiEngine.answer("Prove the Riemann hypothesis");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("axiom set")
            || response.answer.to_lowercase().contains("proof plan")
            || response.answer.to_lowercase().contains("pipeline"),
        "unknown theorem should return a structured plan, got: {}",
        response.answer
    );
    assert!(
        !response.answer.to_lowercase().contains("i cannot"),
        "engine should never refuse outright, got: {}",
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
