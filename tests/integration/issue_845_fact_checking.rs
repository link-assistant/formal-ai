//! Public-boundary end-to-end coverage for issue #845.

use formal_ai::{
    AuditScope, ConversationTurn, Dependency, FactChecker, FormalSystem, GeneralMemoryPermission,
    ProbabilityBasis, RelativeEvidence, SolverConfig, SourceTier, Stance, TruthValue,
    UniversalSolver, WorldModel, WorldStatement,
};

#[test]
fn world_model_audit_runs_proof_probability_jtms_and_permission_boundaries() {
    let mut model = WorldModel::new();
    model
        .current
        .add_statement(WorldStatement::new("general-memory-only statement"));
    model
        .commit_current_to_general(GeneralMemoryPermission::Allowed)
        .unwrap();
    model.current = formal_ai::Context::with_formal_system(
        "current_dialogue",
        FormalSystem::new("integer_arithmetic")
            .with_universe("integers")
            .with_interpretation("standard_arithmetic")
            .with_axiom("peano_arithmetic"),
    );
    let false_premise =
        model
            .current
            .add_statement(
                WorldStatement::new("2 + 2 = 5").with_evidence(RelativeEvidence::new(
                    "repost",
                    SourceTier::Unoriginal,
                    Stance::Supports,
                    1.0,
                )),
            );
    let dependent = model.current.add_statement(
        WorldStatement::new("the false premise supports this unknown")
            .with_dependency(Dependency::supports(false_premise.clone())),
    );
    let checker = FactChecker::from_solver_config(SolverConfig {
        max_decomposition_depth: 2,
        ..SolverConfig::default()
    });
    let current = checker
        .audit_world_model(&mut model, AuditScope::default(), None)
        .unwrap();

    assert_eq!(current.formal_system_name, "integer_arithmetic");
    assert_eq!(current.statements.len(), 2);
    assert_eq!(
        current.statement(&false_premise).unwrap().probability,
        TruthValue::FALSE
    );
    assert_eq!(
        current.statement(&false_premise).unwrap().probability_basis,
        ProbabilityBasis::EvidenceWeighted
    );
    assert!(current
        .recalculation
        .checked_links
        .iter()
        .any(|link| link.statement_id == dependent && link.depends_on == false_premise));
    assert!(current.links_notation().contains("counterexample"));

    assert!(checker
        .audit_world_model(&mut model, AuditScope::GeneralMemory, None)
        .is_err());
    let general = checker
        .audit_world_model(
            &mut model,
            AuditScope::GeneralMemory,
            Some(GeneralMemoryPermission::Allowed),
        )
        .unwrap();
    assert_eq!(general.statements.len(), 1);
}

const FACT_CHECK_QUERIES: &[(&str, &str)] = &[
    ("en", "fact-check this dialogue"),
    ("ru", "проверь факты в диалоге"),
    ("hi", "इस संवाद के तथ्यों की जाँच करें"),
    ("zh", "核查此对话中的事实"),
];

fn arithmetic_history() -> Vec<ConversationTurn> {
    vec![
        ConversationTurn::user("1 + 1 = 2"),
        ConversationTurn::assistant("noted"),
        ConversationTurn::user("1 + 1 = 3"),
        ConversationTurn::assistant("noted"),
    ]
}

#[test]
fn solver_fact_checks_every_current_dialogue_statement_in_every_language() {
    let solver = UniversalSolver::default();
    for (language, query) in FACT_CHECK_QUERIES {
        let answer = solver.solve_with_history(query, &arithmetic_history());

        assert_eq!(
            answer.intent, "fact_check_current_dialogue",
            "[{language}] the request must reach the fact-checking handler: {}",
            answer.answer
        );
        assert!(
            answer.answer.contains("1 + 1 = 2") && answer.answer.contains("1 + 1 = 3"),
            "[{language}] every dialogue statement must be reported: {}",
            answer.answer
        );
        assert!(
            answer.answer.contains("1.000000") && answer.answer.contains("0.000000"),
            "[{language}] proved and refuted statements must expose their calculated probabilities: {}",
            answer.answer
        );
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link.contains("fact_check:audit")),
            "[{language}] the audit must be grounded in an append-only trace: {:?}",
            answer.evidence_links
        );
        assert!(
            answer.evidence_links.iter().all(|link| {
                !link.contains("example.org")
                    && !link.contains("source:http")
                    && !link.contains("cache_hit")
            }),
            "[{language}] an offline audit must not pretend it fetched evidence: {:?}",
            answer.evidence_links
        );
    }
}

#[test]
fn an_empty_dialogue_reports_that_there_are_no_statements_to_check() {
    let answer = UniversalSolver::default().solve("fact-check this dialogue");

    assert_eq!(answer.intent, "fact_check_current_dialogue");
    assert!(
        answer.answer.contains('0'),
        "the empty audit must report its actual statement count: {}",
        answer.answer
    );
}
