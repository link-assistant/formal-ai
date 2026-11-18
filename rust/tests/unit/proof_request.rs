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
                .contains("cannot answer that from local links rules"),
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
fn decision_procedure_proves_tautology_outside_theorem_registry() {
    let response = FormalAiEngine.answer("Prove that p or not p");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("decision procedure") || response.answer.contains("truth table"),
        "tautology should be discharged by the delegated decision procedure, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("p = false") && response.answer.contains("p = true"),
        "truth-table trace should enumerate the boolean cases, got: {}",
        response.answer
    );
    assert!(response.answer.contains("∎"));
}

#[test]
fn decision_procedure_proves_wide_tautology_via_sat() {
    // Nine variables exceed the eight-variable truth-table limit, so the
    // propositional decision procedure must delegate to the DPLL satisfiability
    // backend instead of enumerating 2^9 rows. `a or not a …` is a tautology.
    let response =
        FormalAiEngine.answer("Prove that a or not a or b or c or d or e or f or g or h or i");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("decision procedure"),
        "wide tautology should be discharged by the delegated decision procedure, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("DPLL") && response.answer.contains("Tseitin"),
        "wide tautology should be proved by the SAT backend, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("tautology"),
        "conclusion should declare the formula a tautology, got: {}",
        response.answer
    );
    assert!(response.answer.contains("∎"));
}

#[test]
fn decision_procedure_disproves_wide_non_tautology_via_sat() {
    // A nine-way conjunction is not a tautology; the SAT backend must return a
    // countermodel rather than a proof.
    let response =
        FormalAiEngine.answer("Prove that a and b and c and d and e and f and g and h and i");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("counterexample")
            || response.answer.to_lowercase().contains("disproof")
            || response.answer.to_lowercase().contains("countermodel"),
        "wide non-tautology should be disproven, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("DPLL"),
        "disproof should cite the SAT backend, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("false"),
        "countermodel should report a falsifying assignment, got: {}",
        response.answer
    );
}

#[test]
fn decision_procedure_proves_symbolic_linear_identity() {
    let response = FormalAiEngine.answer("Prove that 2 * (x + 3) = 2 * x + 6");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("linear real arithmetic")
            || response.answer.contains("affine normal form"),
        "symbolic linear identity should be proved by normalization, got: {}",
        response.answer
    );
    assert!(response.answer.contains('0'));
    assert!(response.answer.contains("∎"));
}

#[test]
fn decision_procedure_proves_linear_constraint_entailment() {
    let response = FormalAiEngine.answer("Prove that if x > 3 and x < 5 then x > 2");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("linear real arithmetic")
            || response.answer.contains("SMT")
            || response.answer.contains("decision procedure"),
        "linear constraint entailment should be delegated to the decision procedure, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("x > 2"),
        "proof should restate the entailed constraint, got: {}",
        response.answer
    );
    assert!(response.answer.contains("∎"));
}

#[test]
fn decision_procedure_disproves_linear_constraint_entailment() {
    let response = FormalAiEngine.answer("Prove that if x > 3 and x < 5 then x > 4");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.to_lowercase().contains("counterexample")
            || response.answer.to_lowercase().contains("disproof"),
        "non-entailed linear constraint should return a counterexample, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("x = 4"),
        "counterexample should include x = 4, got: {}",
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
fn prime_infinitude_proofs_are_supported_for_every_language() {
    // Issue #209 follow-up: the supported-language matrix must not only cover
    // the reported Russian word order. Each supported language should recognize
    // the theorem's natural localized statement and return Euclid's proof.
    let cases = [
        (
            "en",
            "Hello. Prove that there are infinitely many prime numbers",
            "There are infinitely many prime numbers",
            "Proof plan",
        ),
        (
            "ru",
            "привет. докажи что простых бесконечно",
            "Простых чисел бесконечно много",
            "План доказательства",
        ),
        (
            "hi",
            "नमस्ते. साबित करो कि अभाज्य संख्याएँ अनंत हैं",
            "अभाज्य संख्याएँ अनंत हैं",
            "प्रमाण योजना",
        ),
        (
            "zh",
            "你好。证明素数有无穷多个",
            "素数有无穷多个",
            "证明计划",
        ),
    ];

    for (language, prompt, expected_statement, plan_heading) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "proof_request",
            "{language} prompt should resolve to proof_request, got {}",
            response.intent
        );
        assert!(
            response.answer.contains(expected_statement),
            "{language} prime-infinitude request should return localized Euclid proof, got: {}",
            response.answer
        );
        assert!(
            response.answer.contains("relative-meta-logic")
                || response.answer.contains("Peano")
                || response.answer.contains("Пеано"),
            "{language} proof should name the formal context, got: {}",
            response.answer
        );
        assert!(
            !response.answer.contains(plan_heading),
            "{language} known theorem should not return a generic proof plan, got: {}",
            response.answer
        );
    }
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

/// The presentation phrases `rust/src/proof_engine` reads from the seed.
const PROOF_LIBRARY_PHRASES: &[&str] = &[
    "interpretation_label",
    "interpretation_proven",
    "interpretation_disproven",
    "interpretation_partial_plan",
    "interpretation_inconclusive",
    "follow_up_intro",
    "follow_up_label",
    "proof_heading",
    "statement_label",
    "method_intro",
    "disproof_heading",
    "counterexample_label",
    "plan_heading",
    "missing_label",
    "inconclusive_heading",
];

#[test]
fn proof_library_seed_speaks_every_surface_language() {
    // R379: the proof engine's theorems, plans and wording are seed data
    // (data/seed/proof-library.lino), not Rust literals. Every record must be
    // spelled in each surface language so no reader gets an English fallback.
    use formal_ai::proof_engine::{ProofMethod, StepKind};
    let library = formal_ai::seed::proof_library();
    let theorem_ids = library
        .theorems
        .iter()
        .map(|theorem| theorem.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        theorem_ids,
        [
            "pythagorean_theorem",
            "euclid_infinitude_of_primes",
            "sqrt_two_irrational",
            "fermat_little_theorem",
            "godel_first_incompleteness",
            "laplacian_determinism",
        ]
    );
    for plan in [
        "deep_reasoning",
        "godel_determinism_mixed",
        "godel_determinism",
        "generic",
    ] {
        assert!(library.plan(plan).is_some(), "plan {plan} is seeded");
    }
    for entry in library.theorems.iter().chain(&library.plans) {
        assert!(
            ProofMethod::from_slug(&entry.method).is_some(),
            "{} names a known method",
            entry.id
        );
        for language in ["en", "ru", "hi", "zh"] {
            for step in &entry.steps {
                assert!(StepKind::from_slug(&step.kind).is_some(), "{}", entry.id);
                assert!(
                    step.text.surfaces.iter().any(|(slug, _)| slug == language),
                    "{} has a {language} step",
                    entry.id
                );
            }
            for input in &entry.missing_inputs {
                assert!(input.surfaces.iter().any(|(slug, _)| slug == language));
            }
        }
    }
    for language in ["en", "ru", "hi", "zh"] {
        for id in PROOF_LIBRARY_PHRASES {
            assert_ne!(library.text(id, language, &[]), *id, "{id} is seeded");
        }
        for method in ProofMethod::ALL {
            assert!(!method.label(language).starts_with("method_"));
        }
        for kind in StepKind::ALL {
            assert!(!kind.label(language).starts_with("step_"));
        }
        assert_eq!(library.items("disproven_follow_ups", language).len(), 2);
        assert_eq!(library.items("inconclusive_follow_ups", language).len(), 3);
    }
}

#[test]
fn seeded_euclid_proof_renders_with_seeded_labels() {
    // The rendered proof is assembled from the seed: heading, method label,
    // step-kind labels, statement, steps and conclusion.
    use formal_ai::proof_engine::{attempt_proof, render_outcome};
    let outcome = attempt_proof(
        "Prove that there are infinitely many primes",
        "there are infinitely many primes",
        "en",
        false,
        false,
    );
    let body = render_outcome(&outcome, "en");
    assert!(
        body.starts_with(
            "How I interpreted the request: treating the request as the formal claim \
             \"There are infinitely many prime numbers.\" and discharging it by proof by \
             contradiction inside relative-meta-logic.\n\nProof (method: proof by \
             contradiction).\n\nStatement: There are infinitely many prime numbers.\n\n\
             1. Definition: Work in elementary number theory"
        ),
        "{body}"
    );
    assert!(
        body.ends_with(
            "5. Inference: Hence q is a prime not in the list p₁, …, pₙ, contradicting the \
             assumption that the list was complete.\nThe assumption fails, so there are \
             infinitely many primes. ∎"
        ),
        "{body}"
    );
}

#[test]
fn decision_proofs_are_worded_in_the_readers_language() {
    // R379: the arithmetic and decision-procedure proofs are seeded phrases,
    // so a Russian reader gets a Russian proof and an English reader the
    // unchanged English one.
    use formal_ai::proof_engine::{ProofOutcome, attempt_proof};
    let ProofOutcome::Proven { proof } =
        attempt_proof("2 + 2 = 4", "2 + 2 = 4", "ru", false, false)
    else {
        panic!("2 + 2 = 4 should be proven");
    };
    assert_eq!(proof.steps[1].text, "Вычислим левую часть: 2 + 2 = 4.");
    assert_eq!(
        proof.conclusion,
        "Следовательно, 2 + 2 равно 4, и утверждение верно. ∎"
    );
    let ProofOutcome::Proven { proof } =
        attempt_proof("2 + 2 = 4", "2 + 2 = 4", "en", false, false)
    else {
        panic!("2 + 2 = 4 should be proven");
    };
    assert_eq!(
        proof.steps[1].text,
        "Evaluate the left-hand side: 2 + 2 = 4."
    );
    assert_eq!(
        proof.conclusion,
        "Therefore 2 + 2 equals 4, so the claim holds. ∎"
    );
    let ProofOutcome::Disproven { counterexample, .. } =
        attempt_proof("1 + 1 = 3", "1 + 1 = 3", "en", false, false)
    else {
        panic!("1 + 1 = 3 should be disproven");
    };
    assert_eq!(
        counterexample,
        "Evaluated values: 1 + 1 = 2, 3 = 3. The relation = does not hold."
    );
    let ProofOutcome::Proven { proof } =
        attempt_proof("p or not p", "p or not p", "zh", false, false)
    else {
        panic!("p or not p should be proven");
    };
    assert!(
        proof.conclusion.ends_with("所以该公式是重言式。∎"),
        "{}",
        proof.conclusion
    );
}

/// Issue #1487: a theorem keyword only retrieves the theorem; the claim around
/// it decides whether the request asks for the theorem, its negation, or
/// something whose negation scope is not read.
#[test]
fn proof_claims_bind_to_the_exact_theorem_proposition() {
    use formal_ai::proof_engine::ClaimBinding;
    use formal_ai::proof_engine::claim_binding::claim_binding;
    let cases = [
        ("there are infinitely many primes", ClaimBinding::Affirmed),
        (
            "there are not infinitely many primes",
            ClaimBinding::Negated,
        ),
        ("不存在无穷多素数", ClaimBinding::Negated),
        ("素数有无穷多个", ClaimBinding::Affirmed),
        (
            "it is not true that there are not infinitely many primes",
            ClaimBinding::Unsupported,
        ),
        (
            "there are infinitely many primes that are not even",
            ClaimBinding::Unsupported,
        ),
    ];
    for (claim, expected) in cases {
        assert_eq!(claim_binding(claim), Some(expected), "claim {claim:?}");
    }
    assert_eq!(claim_binding("1 + 1 = 2"), None);
}

/// Issue #1487: "Prove there are not infinitely many primes." must never get
/// Euclid's positive success; the library marks the theorem `negation contradiction`,
/// so the request is refuted by Euclid's proof.
#[test]
fn negated_theorem_request_is_refuted_never_proven() {
    use formal_ai::proof_engine::{ProofOutcome, attempt_proof};
    for (language, claim) in [
        ("en", "there are not infinitely many primes"),
        ("zh", "不存在无穷多素数"),
    ] {
        let outcome = attempt_proof(claim, claim, language, false, false);
        let ProofOutcome::Disproven {
            counterexample,
            partial_proof,
            ..
        } = &outcome
        else {
            panic!("negated theorem {claim:?} must be refuted, got {outcome:?}");
        };
        assert!(
            partial_proof.is_some(),
            "the refutation carries Euclid's proof"
        );
        assert!(!counterexample.contains("{statement}"), "{counterexample}");
    }
    let response = FormalAiEngine.answer("Prove there are not infinitely many primes.");
    assert_eq!(response.intent, "proof_request");
    // Euclid's proof is the refutation: `render_disproven` lays out the
    // counterexample, the seeded steps and the conclusion, and
    // `enforce_questions` keeps the first clarifying question and the bare
    // number of the second (the JavaScript worker renders the same answer,
    // rust/tests/web/issue-1487-proposition-identity.test.mjs).
    assert_eq!(
        response.answer,
        "How I interpreted the request: treating the request as a claim to be refuted; applying proof by contradiction and producing a counterexample.\n\nDisproof (method: proof by contradiction).\n\nCounterexample: The claim denies a proven theorem: There are infinitely many prime numbers. The proof below refutes the claim.\n\n1. Definition: Work in elementary number theory, formalizable in Peano arithmetic (PA): a prime is an integer greater than 1 whose only positive divisors are 1 and itself. The proof uses the PA theorem that every integer greater than 1 has a prime divisor; this is the formal context for the relative-meta-logic contradiction tactic.\n2. Hypothesis: Assume for contradiction that only finitely many primes exist; call them p₁, p₂, …, pₙ.\n3. Inference: Form the number N = p₁·p₂·…·pₙ + 1. Then N is an integer greater than 1.\n4. Inference: By the fundamental theorem of arithmetic, N has a prime divisor q. If q = pᵢ for some i, then pᵢ divides both p₁·p₂·…·pₙ and N, so pᵢ divides their difference, which is 1 — impossible.\n5. Inference: Hence q is a prime not in the list p₁, …, pₙ, contradicting the assumption that the list was complete.\nThe assumption fails, so there are infinitely many primes. ∎\n\nClarifying questions:\n1. do you want to weaken the claim into a checkable form (e.g. replace equality with an inequality, or restrict the domain)?\n2."
    );
    assert!(
        response.answer.contains("Disproof") && response.answer.contains("denies a proven theorem"),
        "negated request should be refuted, got: {}",
        response.answer
    );
    assert!(
        !response.answer.starts_with("Proof (") && !response.answer.contains("\nProof ("),
        "negated request must not be presented as a proof of itself, got: {}",
        response.answer
    );
}

/// Issue #1487: a claim whose negation scope around the theorem keyword is not
/// read (double negation, a negated qualifier) gets the honest partial plan,
/// neither the theorem's proof nor its refutation.
#[test]
fn unread_negation_scope_returns_partial_plan() {
    use formal_ai::proof_engine::{ProofOutcome, attempt_proof};
    for claim in [
        "it is not true that there are not infinitely many primes",
        "there are infinitely many primes that are not even",
    ] {
        let outcome = attempt_proof(claim, claim, "en", false, false);
        assert!(
            matches!(outcome, ProofOutcome::PartialPlan { .. }),
            "claim {claim:?} should stay unresolved, got {outcome:?}"
        );
    }
    let affirmed = attempt_proof(
        "there are infinitely many primes",
        "there are infinitely many primes",
        "en",
        false,
        false,
    );
    assert!(
        matches!(affirmed, ProofOutcome::Proven { .. }),
        "{affirmed:?}"
    );
}
