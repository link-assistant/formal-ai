//! Public-boundary end-to-end coverage for issue #845.

use formal_ai::{
    AuditScope, Dependency, FactChecker, FormalSystem, GeneralContextPermission, ProbabilityBasis,
    RelativeEvidence, SolverConfig, SourceTier, Stance, TruthValue, WorldModel, WorldStatement,
};

#[test]
fn world_model_audit_runs_proof_probability_jtms_and_permission_boundaries() {
    let mut model = WorldModel::new();
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
    model
        .general
        .add_statement(WorldStatement::new("general-memory-only statement"));

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
    let permission: GeneralContextPermission = model
        .record_general_context_permission(true, "integration whole-memory audit")
        .unwrap();
    let general = checker
        .audit_world_model(&mut model, AuditScope::GeneralMemory, Some(&permission))
        .unwrap();
    assert_eq!(general.statements.len(), 1);
}
