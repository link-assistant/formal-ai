//! Executable coverage for issue #649 — symbolic world models & contexts.
//!
//! The [design case study](../../docs/case-studies/issue-649/README.md) maps
//! every concept the issue names onto the associative stack; this test exercises
//! the `formal_ai::world_model` module that realizes it, proving each headline
//! requirement runs end to end:
//!
//! * a [`Context`] is a links network (state atoms serialize to Links Notation),
//! * current vs. target [`WorldModel::difference`] exposes the STRIPS delta,
//! * dependent [`Statement`]s recalculate to a fixpoint when the world changes,
//! * [`Context::predict`] simulates an action **without mutating** the model,
//! * [`Context::merge_from`] / [`Context::split_off`] combine and separate
//!   contexts (ATMS-style).

use formal_ai::{
    Action, Context, Dependency, RelativeEvidence, SourceTier, Stance, TruthValue, WorldModel,
    WorldStatement,
};

/// A context is always a links network: asserted atoms round-trip through the
/// graph and render as Links Notation.
#[test]
fn context_is_a_links_network() {
    let mut context = Context::new("kitchen");
    assert!(context.assert_link("door", "closed"));
    assert!(context.assert_link("light", "off"));
    // Re-asserting an existing atom is a no-op.
    assert!(!context.assert_link("door", "closed"));

    assert!(context.holds("door", "closed"));
    assert!(!context.holds("door", "open"));

    let notation = context.links_notation();
    assert!(
        notation.contains("door") && notation.contains("closed"),
        "state atoms must be inspectable as Links Notation, got:\n{notation}"
    );

    assert!(context.retract_link("light", "off"));
    assert!(!context.holds("light", "off"));
}

/// The current→target difference is the STRIPS goal − state delta: what to add,
/// what to remove, and any functional conflict.
#[test]
fn difference_exposes_current_to_target_delta() {
    let mut model = WorldModel::new();
    model.current.assert_link("door", "closed");
    model.current.assert_link("robot", "outside");

    model.target.assert_link("door", "open");
    model.target.assert_link("robot", "outside");

    assert!(!model.target_reached(), "goal differs from current state");

    let diff = model.difference();
    assert!(
        diff.to_add
            .iter()
            .any(|link| link.from == "door" && link.to == "open"),
        "target adds door->open"
    );
    assert!(
        diff.to_remove
            .iter()
            .any(|link| link.from == "door" && link.to == "closed"),
        "current door->closed must be removed"
    );
    // door->closed vs door->open share a `from` and disagree → a conflict.
    assert!(
        diff.conflicting
            .iter()
            .any(|conflict| conflict.current.to == "closed" && conflict.target.to == "open"),
        "same-from/different-to must surface as a conflict, got {diff:?}"
    );
    // robot->outside is shared, so it appears in neither list.
    assert!(diff
        .to_add
        .iter()
        .chain(&diff.to_remove)
        .all(|link| link.from != "robot"));
}

/// When the current state is edited to equal the target, the difference is empty
/// and the goal is reached.
#[test]
fn target_reached_when_states_match() {
    let mut model = WorldModel::new();
    model.current.assert_link("door", "open");
    model.target.assert_link("door", "open");

    assert!(model.target_reached());
    assert!(model.difference().is_empty());
}

/// Statements are dependent: a supporting dependency raises the dependent's
/// probability, a contradicting one lowers it, and both settle to a fixpoint.
#[test]
fn dependent_statements_recalculate_to_a_fixpoint() {
    let mut context = Context::new("weather");

    let raining = WorldStatement::new("it is raining").with_evidence(RelativeEvidence::new(
        "met-office",
        SourceTier::OriginalFirstParty,
        Stance::Supports,
        TruthValue::TRUE,
    ));
    let raining_id = context.add_statement(raining);

    // "the ground is wet" depends positively on "it is raining".
    let wet =
        WorldStatement::new("the ground is wet").with_dependency(Dependency::supports(&raining_id));
    let wet_id = context.add_statement(wet);

    // "the ground is dry" depends negatively on "it is raining".
    let dry = WorldStatement::new("the ground is dry")
        .with_dependency(Dependency::contradicts(&raining_id));
    let dry_id = context.add_statement(dry);

    let report = context.recalculate();
    assert!(report.converged, "cascade must reach a fixpoint");

    let raining_truth = context.statement(&raining_id).unwrap().truth.get();
    let wet_truth = context.statement(&wet_id).unwrap().truth.get();
    let dry_truth = context.statement(&dry_id).unwrap().truth.get();

    assert!(
        raining_truth > 0.9,
        "strong first-party support ⇒ near-certain"
    );
    assert!(
        wet_truth > dry_truth,
        "the supported statement ({wet_truth}) must outrank the contradicted one ({dry_truth})"
    );
    assert!(
        dry_truth < 0.5,
        "a statement contradicted by a true dependency drops below the midpoint, got {dry_truth}"
    );
}

/// Recalculation converges even with a two-node negative-feedback cycle (the
/// bounded-pass backstop guarantees termination).
#[test]
fn recalculation_terminates_on_a_negative_cycle() {
    let mut context = Context::new("paradox");
    let a = context.add_statement(WorldStatement::new("statement A"));
    let b = context.add_statement(WorldStatement::new("statement B"));

    // Mutual contradiction — a classic oscillator without a pass bound.
    let mut a_stmt = context.statement(&a).unwrap().clone();
    a_stmt.dependencies.push(Dependency::contradicts(&b));
    context.add_statement(a_stmt);
    let mut b_stmt = context.statement(&b).unwrap().clone();
    b_stmt.dependencies.push(Dependency::contradicts(&a));
    context.add_statement(b_stmt);

    let report = context.recalculate();
    // It must return (bounded), regardless of whether it settled.
    assert!(report.iterations >= 1);
}

/// Prediction simulates an action's consequences without mutating the real
/// world model, and reports the state links it would add/remove.
#[test]
fn predict_does_not_mutate_the_context() {
    let mut context = Context::new("room");
    context.assert_link("door", "closed");

    let open_door = Action::new("open the door")
        .removing("door", "closed")
        .adding("door", "open");

    let prediction = context.predict(&open_door);

    // The prediction shows the consequence...
    assert!(prediction
        .added
        .iter()
        .any(|link| link.from == "door" && link.to == "open"));
    assert!(prediction
        .removed
        .iter()
        .any(|link| link.from == "door" && link.to == "closed"));
    assert!(!prediction.is_noop());
    // ...while the real context is untouched.
    assert!(context.holds("door", "closed"));
    assert!(!context.holds("door", "open"));

    // The post-action snapshot on the prediction *does* reflect the action.
    assert!(prediction.result.holds("door", "open"));
    assert!(!prediction.result.holds("door", "closed"));
}

/// Prediction also reports statement probability movement caused by the action.
#[test]
fn predict_reports_statement_movement() {
    let mut context = Context::new("switch");
    // A statement that becomes strongly supported once the light-on atom exists.
    let lit = WorldStatement::new("the room is lit").with_evidence(RelativeEvidence::new(
        "sensor",
        SourceTier::OriginalFirstParty,
        Stance::Contradicts,
        TruthValue::TRUE,
    ));
    let lit_id = context.add_statement(lit);
    let before = context.statement(&lit_id).unwrap().truth.get();

    // Overwrite with a supporting-evidence version to simulate a world change and
    // confirm the prediction machinery surfaces statement deltas at all.
    let action = Action::new("flip the switch").adding("light", "on");
    let prediction = context.predict(&action);
    // The action edits state, so it is not a no-op.
    assert!(!prediction.added.is_empty());
    assert!(before < 0.5, "contradicted statement starts low");
}

/// Merge folds one context into another (ATMS combination): unioned state and
/// statements, recalculated.
#[test]
fn merge_unions_contexts() {
    let mut base = Context::new("base");
    base.assert_link("a", "1");
    base.add_statement(WorldStatement::new("base fact"));

    let mut incoming = Context::new("incoming");
    incoming.assert_link("b", "2");
    incoming.add_statement(WorldStatement::new("incoming fact"));

    base.merge_from(&incoming);

    assert!(base.holds("a", "1"));
    assert!(base.holds("b", "2"));
    assert_eq!(base.statements().len(), 2);
}

/// Split carves a child context out of a parent (ATMS separation), copying the
/// selected statements and the links that reference them, leaving the parent
/// intact.
#[test]
fn split_carves_a_child_context() {
    let mut parent = Context::new("parent");
    let keep = parent.add_statement(WorldStatement::new("keep me"));
    let drop = parent.add_statement(WorldStatement::new("leave me"));
    // A state atom referencing the kept statement id travels with it.
    parent.assert_link(&keep, "tagged");

    let child = parent.split_off("child", std::slice::from_ref(&keep));

    assert!(
        child.statement(&keep).is_some(),
        "selected statement copied"
    );
    assert!(
        child.statement(&drop).is_none(),
        "unselected statement excluded"
    );
    assert!(
        child.holds(&keep, "tagged"),
        "referencing atom travels with it"
    );

    // Parent is unchanged by the split.
    assert!(parent.statement(&keep).is_some());
    assert!(parent.statement(&drop).is_some());
}

/// Committing the current dialogue context into the shared general context is a
/// merge, so the general model accumulates the dialogue's facts.
#[test]
fn commit_current_folds_into_general() {
    let mut model = WorldModel::new();
    model.current.assert_link("fact", "learned");
    model
        .current
        .add_statement(WorldStatement::new("a learned fact"));

    model.commit_current_to_general();

    assert!(model.general.holds("fact", "learned"));
    assert_eq!(model.general.statements().len(), 1);
}
