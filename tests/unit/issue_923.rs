//! Issue #923 — symbolic-kernel coverage growth.

use std::fs;
use std::path::Path;

use formal_ai::external_benchmarks::{BenchmarkCase, Expectation};
use formal_ai::external_benchmarks::{Grading, Ledger, manifest, run_suite};
use formal_ai::proof_engine::{ProofMethod, ProofOutcome, attempt_proof};

#[test]
fn equality_saturation_proves_a_non_numeric_rewrite() {
    let outcome = attempt_proof("Prove (+ x 0) = x", "(+ x 0) = x", "en", false, false);

    let ProofOutcome::Proven { proof } = outcome else {
        panic!("expected equality saturation to prove the rewrite, got {outcome:?}");
    };
    assert_eq!(proof.method, ProofMethod::DecisionProcedure);
    assert!(
        proof.steps.iter().any(|step| step.text.contains("egraph")),
        "the certificate should identify the e-graph discharge"
    );
}

#[test]
fn equality_saturation_does_not_claim_inequality_from_search_failure() {
    let outcome = attempt_proof("Prove (+ x 1) = x", "(+ x 1) = x", "en", false, false);

    assert!(
        matches!(
            outcome,
            ProofOutcome::PartialPlan { .. } | ProofOutcome::Inconclusive { .. }
        ),
        "failure to find an equality proof is not a disproof: {outcome:?}"
    );
}

#[test]
fn equality_dispatch_preserves_infix_linear_reasoning() {
    let claim = "2 * (x + 3) = 2 * x + 6";
    let outcome = attempt_proof(claim, claim, "en", false, false);

    let ProofOutcome::Proven { proof } = outcome else {
        panic!("expected affine normalization to prove the identity, got {outcome:?}");
    };
    assert!(
        proof
            .steps
            .iter()
            .any(|step| step.text.contains("affine normal form")),
        "infix arithmetic must stay with the linear solver: {proof:?}"
    );
}

#[test]
fn datalog_rule_inference_proves_transitive_reachability() {
    let claim = concat!(
        "facts { edge(a,b); edge(b,c) } ",
        "rules { reachable(X,Y) :- edge(X,Y); ",
        "reachable(X,Z) :- reachable(X,Y), edge(Y,Z) } ",
        "query { reachable(a,c) }",
    );
    let outcome = attempt_proof(claim, claim, "en", false, false);

    let ProofOutcome::Proven { proof } = outcome else {
        panic!("expected the least fixed point to contain reachable(a,c), got {outcome:?}");
    };
    assert_eq!(proof.method, ProofMethod::DecisionProcedure);
    assert!(
        proof.steps.iter().any(|step| step.text.contains("datalog")),
        "the certificate should identify the Datalog discharge"
    );
}

#[test]
fn datalog_rejects_non_range_restricted_rules_as_inconclusive() {
    let claim = concat!(
        "facts { edge(a,b) } ",
        "rules { reachable(X,Z) :- edge(X,Y) } ",
        "query { reachable(a,c) }",
    );
    let outcome = attempt_proof(claim, claim, "en", false, false);

    assert!(
        matches!(outcome, ProofOutcome::Inconclusive { .. }),
        "an unsafe head variable is malformed Datalog, not negative evidence: {outcome:?}"
    );
}

#[test]
fn datalog_join_work_limit_is_inconclusive() {
    let facts = (0..160)
        .flat_map(|index| {
            ["left", "middle", "right"].map(|predicate| format!("{predicate}({index})"))
        })
        .collect::<Vec<_>>()
        .join("; ");
    let claim = format!(
        "facts {{ {facts} }} rules {{ answer(?x) :- left(?x), middle(?y), right(?z) }} query {{ answer(0) }}"
    );
    let outcome = attempt_proof(&claim, &claim, "en", false, false);

    assert!(
        matches!(outcome, ProofOutcome::Inconclusive { .. }),
        "a bounded join must report its resource ceiling: {outcome:?}"
    );
}

#[test]
fn symbolic_kernel_responses_cover_every_registered_language() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let symbolic_seed = root.join("data/seed/multilingual-responses-symbolic.lino");
    assert!(
        symbolic_seed.is_file(),
        "symbolic responses must stay in their bounded seed shard"
    );
    assert!(
        fs::read_to_string(root.join("data/meta/seed-registry.lino"))
            .expect("seed registry")
            .contains("seed multilingual-responses-symbolic\n"),
        "the symbolic response shard must be declared in the seed registry"
    );

    let claim = concat!(
        "facts { edge(a,b); edge(b,c) } ",
        "rules { reachable(X,Y) :- edge(X,Y); ",
        "reachable(X,Z) :- reachable(X,Y), edge(Y,Z) } ",
        "query { reachable(a,c) }",
    );
    let intents = [
        "external_benchmark_proof_expected_event",
        "external_benchmark_proof_status_detail",
        "external_benchmark_rust_adapter_missing",
        "external_benchmark_egg_law_prompt",
        "external_benchmark_egg_too_few_laws",
        "external_benchmark_ascent_missing_rule",
        "external_benchmark_ascent_prompt_prefix",
        "external_benchmark_proof_query",
        "external_benchmark_upstream_missing_start",
        "external_benchmark_upstream_missing_end",
        "external_benchmark_invalid_rust_string",
        "proof_equality_conclusion",
        "proof_datalog_conclusion",
        "proof_datalog_counterexample",
    ];

    for language in formal_ai::language::registered_languages() {
        for intent in intents {
            let response = formal_ai::seed::localized_response(intent, language.slug())
                .unwrap_or_else(|| panic!("{} has no `{intent}` response", language.slug()));
            assert!(
                !response.trim().is_empty(),
                "{} `{intent}` response must not be empty",
                language.slug()
            );
        }

        let outcome = attempt_proof(claim, claim, language.slug(), false, false);
        let ProofOutcome::Proven { proof } = outcome else {
            panic!(
                "expected localized Datalog proof for {}, got {outcome:?}",
                language.slug()
            );
        };
        let expected = formal_ai::seed::render_response(
            "proof_datalog_conclusion",
            language.slug(),
            &[("query", "reachable(a,c)")],
        )
        .unwrap_or_else(|| panic!("{} conclusion did not render", language.slug()));
        assert_eq!(proof.conclusion, expected, "{} conclusion", language.slug());
    }
}

#[test]
fn new_dependency_is_optional_and_both_external_scores_are_registered() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml");
    assert!(cargo.contains("equality-saturation = [\"dep:egg\"]"));
    assert!(cargo.contains("egg = { version = \"0.11.0\""));
    assert!(cargo.contains("optional = true"));

    for (id, family) in [
        ("egg_math", "equality_saturation"),
        ("ascent_transitive_closure", "rule_inference"),
    ] {
        let suite = manifest::suite(id).unwrap_or_else(|| panic!("missing suite {id}"));
        assert_eq!(suite.task_family, family);
        assert_eq!(suite.grading, Grading::ProofStatus);
        assert!(suite.is_runnable());
    }

    let ledger_text = fs::read_to_string(root.join("data/benchmarks/external-results.lino"))
        .expect("external benchmark ledger");
    let ledger = Ledger::parse(&ledger_text).expect("valid external benchmark ledger");
    for (id, slice, passed) in [("egg_math", 20, 20), ("ascent_transitive_closure", 5, 5)] {
        let suites = ledger.suites();
        let suite = suites.get(id).expect("registered suite row");
        assert_eq!(suite.ratchet_slice, slice);
        assert_eq!(suite.minimum_pass_count, passed);
        assert!(ledger.results().iter().any(|result| {
            result.suite == id
                && result.date == "2026-08-14"
                && result.slice == slice
                && result.passed == passed
                && result.total == slice
        }));
    }
}

#[test]
fn external_proof_grading_requires_the_structured_solver_trace() {
    let case = BenchmarkCase {
        id: String::from("proof/status"),
        prompt: String::from("Prove a symbolic claim"),
        expectation: Expectation::Value {
            expected: String::from("proven"),
        },
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/issue-923-grading");

    let forged_answer = formal_ai::external_benchmarks::grade::grade_case_with_trace(
        &case,
        Grading::ProofStatus,
        "proof_outcome proven",
        "proof_outcome inconclusive",
        &workspace,
    );
    assert!(
        !forged_answer.passed,
        "answer prose is not benchmark evidence"
    );

    let structured_trace = formal_ai::external_benchmarks::grade::grade_case_with_trace(
        &case,
        Grading::ProofStatus,
        "any presentation",
        "answer_test\n  steps \"step_0 proof_outcome proven; step_1 proof_method decision_procedure\"",
        &workspace,
    );
    assert!(structured_trace.passed);

    let prefixed_trace = formal_ai::external_benchmarks::grade::grade_case_with_trace(
        &case,
        Grading::ProofStatus,
        "any presentation",
        "answer_test\n  steps \"step_0 not_a_proof_outcome proven\"",
        &workspace,
    );
    assert!(
        !prefixed_trace.passed,
        "only an exact structured trace event may satisfy proof grading"
    );

    let suffixed_payload = formal_ai::external_benchmarks::grade::grade_case_with_trace(
        &case,
        Grading::ProofStatus,
        "any presentation",
        "answer_test\n  steps \"step_0 proof_outcome proven_by_answer_prose\"",
        &workspace,
    );
    assert!(
        !suffixed_payload.passed,
        "only the exact proof status may satisfy proof grading"
    );
}

#[test]
#[ignore = "downloads and executes the pinned upstream Rust suites"]
fn pinned_upstream_symbolic_suites_pass_end_to_end() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (id, slice) in [("egg_math", 20), ("ascent_transitive_closure", 5)] {
        let suite = manifest::suite(id).expect("registered suite");
        let run = run_suite(suite, slice, root).expect("benchmark harness run");
        assert_eq!(run.unavailable, None, "{id} should be executable");
        assert_eq!((run.passed, run.failed, run.total), (slice, 0, slice));
    }
}
