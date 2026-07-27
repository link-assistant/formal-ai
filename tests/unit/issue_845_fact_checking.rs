//! Executable acceptance coverage for issue #845.
//!
//! These tests join the previously separate proof engine, relative probability
//! kernel, and world model. They intentionally exercise the public API so the
//! fact-checking operation is useful to every Formal AI surface.

use formal_ai::{
    AuditScope, Context, ContextAccessEventKind, Dependency, FactCheckError, FactChecker,
    FormalSystem, GeneralMemoryPermission, ProbabilityBasis, RefutationOutcome, RefutationStage,
    RelativeEvidence, SolverConfig, SourceTier, Stance, TruthValue, WorldModel, WorldStatement,
    ASSUMED_TRUE_PRIOR,
};

fn checker(depth: u8) -> FactChecker {
    FactChecker::from_solver_config(SolverConfig {
        max_decomposition_depth: depth,
        ..SolverConfig::default()
    })
}

#[test]
fn probabilities_are_relative_to_a_named_formal_system() {
    let statement = "the selected element is maximal";
    let ordered = FormalSystem::new("finite_total_order")
        .with_universe("finite_elements")
        .with_interpretation("standard_order")
        .with_axiom("totality");
    let partial = FormalSystem::new("finite_partial_order")
        .with_universe("finite_elements")
        .with_interpretation("standard_order")
        .with_axiom("antisymmetry");

    let mut ordered_context = Context::with_formal_system("dialogue_ordered", ordered);
    let ordered_id = ordered_context.add_statement(WorldStatement::new(statement).with_evidence(
        RelativeEvidence::new(
            "order_witness",
            SourceTier::OriginalFirstParty,
            Stance::Supports,
            0.9,
        ),
    ));
    let mut partial_context = Context::with_formal_system("dialogue_partial", partial);
    let partial_id = partial_context.add_statement(WorldStatement::new(statement).with_evidence(
        RelativeEvidence::new(
            "counter_order_witness",
            SourceTier::OriginalFirstParty,
            Stance::Contradicts,
            0.9,
        ),
    ));

    let ordered_report = checker(2)
        .verify_statement(&mut ordered_context, &ordered_id)
        .unwrap();
    let partial_report = checker(2)
        .verify_statement(&mut partial_context, &partial_id)
        .unwrap();

    assert_eq!(ordered_report.formal_system_name, "finite_total_order");
    assert_eq!(partial_report.formal_system_name, "finite_partial_order");
    assert_ne!(ordered_report.probability, partial_report.probability);
}

#[test]
fn recursive_verification_is_disproof_first_and_reports_a_counterexample() {
    let mut context = Context::new("arithmetic_dialogue");
    let false_part = context.add_statement(WorldStatement::new("1 + 1 = 3"));
    let whole = context.add_statement(
        WorldStatement::new("an unknown composite claim")
            .with_dependency(Dependency::supports(false_part.clone())),
    );

    let report = checker(2).verify_statement(&mut context, &whole).unwrap();
    let stages = report
        .refutation_trace
        .iter()
        .map(|attempt| attempt.stage)
        .collect::<Vec<_>>();

    assert_eq!(stages[0], RefutationStage::DisproveStatement);
    assert_eq!(stages[1], RefutationStage::DisproveNegation);
    assert_eq!(stages[2], RefutationStage::Decompose);
    let refuted_part = report
        .refutation_trace
        .iter()
        .find(|attempt| attempt.statement_id == false_part && attempt.counterexample.is_some())
        .expect("the recursive child must be refuted with a counterexample");
    assert_eq!(refuted_part.depth, 1);
}

#[test]
fn direct_proof_still_runs_the_negation_refutation_before_support() {
    let mut context = Context::new("arithmetic_dialogue");
    let statement = context.add_statement(WorldStatement::new("1 + 1 = 2"));

    let report = checker(2)
        .verify_statement(&mut context, &statement)
        .unwrap();

    assert_eq!(
        report.refutation_trace[0].stage,
        RefutationStage::DisproveStatement
    );
    assert_eq!(
        report.refutation_trace[0].outcome,
        RefutationOutcome::Unrefuted
    );
    assert_eq!(
        report.refutation_trace[1].stage,
        RefutationStage::DisproveNegation
    );
    assert_eq!(
        report.refutation_trace[1].outcome,
        RefutationOutcome::Inconclusive
    );
    assert!(report
        .evidence
        .iter()
        .any(|item| item.source_label.starts_with("proof:")));
}

#[test]
fn recursion_bound_comes_from_solver_config() {
    let mut context = Context::new("bounded_dialogue");
    let child = context.add_statement(WorldStatement::new("another unknown claim"));
    let root = context.add_statement(
        WorldStatement::new("unknown root").with_dependency(Dependency::supports(child.clone())),
    );

    let report = checker(0).verify_statement(&mut context, &root).unwrap();

    assert!(report
        .refutation_trace
        .iter()
        .any(|attempt| attempt.stage == RefutationStage::DepthBound));
    assert!(report
        .refutation_trace
        .iter()
        .all(|attempt| attempt.statement_id != child));
}

#[test]
fn support_fallback_uses_source_tiers_and_marks_prior_only_unknowns() {
    let mut context = Context::new("evidence_dialogue");
    let supported = context.add_statement(
        WorldStatement::new("an observational claim")
            .with_evidence(RelativeEvidence::new(
                "original_report",
                SourceTier::OriginalJournalism,
                Stance::Supports,
                0.9,
            ))
            .with_evidence(RelativeEvidence::new(
                "repost",
                SourceTier::Unoriginal,
                Stance::Supports,
                1.0,
            )),
    );
    let unknown = context.add_statement(WorldStatement::new("an unevidenced claim"));

    let audit = checker(1).audit_context(&mut context);
    let supported_report = audit.statement(&supported).unwrap();
    let unknown_report = audit.statement(&unknown).unwrap();

    assert!(supported_report.probability.get() > ASSUMED_TRUE_PRIOR);
    assert_eq!(
        supported_report.probability_basis,
        ProbabilityBasis::EvidenceWeighted
    );
    assert!(supported_report.evidence.iter().any(|item| {
        item.source_label == "original_report"
            && item.tier == SourceTier::OriginalJournalism
            && !item.ignored
    }));
    assert!(supported_report
        .evidence
        .iter()
        .any(|item| item.source_label == "repost" && item.ignored));
    assert_eq!(
        unknown_report.probability,
        TruthValue::new(ASSUMED_TRUE_PRIOR)
    );
    assert_eq!(
        unknown_report.probability_basis,
        ProbabilityBasis::PriorOnly
    );
}

#[test]
fn recalculation_trace_names_every_dependency_link() {
    let mut context = Context::new("dependent_dialogue");
    let premise = context.add_statement(WorldStatement::new("2 + 2 = 5"));
    let dependent = context.add_statement(
        WorldStatement::new("dependent unknown")
            .with_dependency(Dependency::supports(premise.clone())),
    );

    let audit = checker(2).audit_context(&mut context);

    assert!(audit
        .recalculation
        .checked_links
        .iter()
        .any(|link| link.statement_id == dependent && link.depends_on == premise));
    assert_eq!(
        context.statement(&premise).unwrap().truth,
        TruthValue::FALSE
    );
}

#[test]
fn general_memory_requires_recorded_permission_and_commit_uses_the_same_gate() {
    let mut model = WorldModel::new();
    model
        .current
        .add_statement(WorldStatement::new("general memory fact"));
    model
        .commit_current_to_general(GeneralMemoryPermission::Allowed)
        .unwrap();
    model.current = Context::new("current");
    model
        .current
        .add_statement(WorldStatement::new("current dialogue fact"));
    let fact_checker = checker(1);

    let denied = fact_checker.audit_world_model(&mut model, AuditScope::GeneralMemory, None);
    assert!(matches!(denied, Err(FactCheckError::PermissionRequired)));

    let audit = fact_checker
        .audit_world_model(
            &mut model,
            AuditScope::GeneralMemory,
            Some(GeneralMemoryPermission::Allowed),
        )
        .unwrap();
    model
        .commit_current_to_general(GeneralMemoryPermission::Allowed)
        .unwrap();

    assert_eq!(audit.scope, AuditScope::GeneralMemory);
    assert!(model
        .context_access_events()
        .iter()
        .any(|event| event.kind == ContextAccessEventKind::PermissionGranted));
    assert!(model
        .context_access_events()
        .iter()
        .any(|event| event.kind == ContextAccessEventKind::PermissionDenied));
    assert!(model
        .context_access_events()
        .iter()
        .any(|event| event.kind == ContextAccessEventKind::GeneralContextRead));
    assert_eq!(model.general().statements().len(), 2);
}

#[test]
fn current_dialogue_is_the_default_scope_and_audit_enumerates_every_statement() {
    assert_eq!(AuditScope::default(), AuditScope::CurrentDialogue);

    let mut model = WorldModel::new();
    model
        .current
        .add_statement(WorldStatement::new("not in the current dialogue"));
    model
        .commit_current_to_general(GeneralMemoryPermission::Allowed)
        .unwrap();
    model.current = Context::new("current");
    let first = model
        .current
        .add_statement(WorldStatement::new("1 + 1 = 2"));
    let second = model
        .current
        .add_statement(WorldStatement::new("1 + 1 = 3"));

    let audit = checker(2)
        .audit_world_model(&mut model, AuditScope::default(), None)
        .unwrap();

    assert_eq!(audit.statements.len(), 2);
    assert!(audit.statement(&first).is_some());
    assert!(audit.statement(&second).is_some());
    assert!(audit
        .statements
        .iter()
        .any(|statement| statement.counterexample.is_some()));
}

#[test]
fn fabricated_source_links_are_excluded_from_the_probability() {
    let mut context = Context::new("honest_evidence_dialogue");
    let statement = context.add_statement(
        WorldStatement::new("unsupported network claim").with_evidence(RelativeEvidence::new(
            "source:http://example.org/fabricated",
            SourceTier::OriginalFirstParty,
            Stance::Supports,
            1.0,
        )),
    );

    let audit = checker(1).audit_context(&mut context);
    let report = audit.statement(&statement).unwrap();

    assert_eq!(report.probability, TruthValue::new(ASSUMED_TRUE_PRIOR));
    assert_eq!(report.probability_basis, ProbabilityBasis::PriorOnly);
    assert!(report
        .evidence
        .iter()
        .any(|evidence| evidence.rejected_as_fabricated));
}

#[test]
fn identical_context_and_evidence_replay_byte_identically() {
    fn run() -> String {
        let mut context = Context::new("deterministic_dialogue");
        let premise = context.add_statement(WorldStatement::new("3 * 3 = 9").with_evidence(
            RelativeEvidence::new(
                "calculator_fixture",
                SourceTier::OriginalFirstParty,
                Stance::Supports,
                1.0,
            ),
        ));
        context.add_statement(
            WorldStatement::new("derived statement").with_dependency(Dependency::supports(premise)),
        );
        checker(3).audit_context(&mut context).links_notation()
    }

    assert_eq!(run().as_bytes(), run().as_bytes());
}
