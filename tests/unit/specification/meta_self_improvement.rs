//! Issue #559 (R340): the meta algorithm reasoning about improving *itself*.
//!
//! The headline self-improvement requirement of #559 is meta-circular: the
//! algorithm takes *itself* — the recursive-core recipe, the algorithm encoded as
//! Links Notation — together with what it is required to do (the stages the live
//! `record_meta_core` pipeline runs), both meta-language encoded, and emits the
//! *updated* algorithm as link-encoded output. These tests pin three things: the
//! loop is gated and proposal-only (the default `Off` proposes nothing, R13/C3);
//! it genuinely detects drift on synthetic input; and, on the checked-in recipe
//! and pipeline, it is already self-consistent — proving the recipe describes every
//! stage the pipeline actually runs, which is the regression the loop exists to
//! guard.

use formal_ai::meta_self_improvement::{
    propose_recipe_update, MetaSelfImprovement, SelfImprovementMode,
};

// ---------------------------------------------------------------------------
// The SelfImprovementMode gate.
// ---------------------------------------------------------------------------

#[test]
fn off_is_the_default_and_proposes_nothing() {
    assert_eq!(SelfImprovementMode::default(), SelfImprovementMode::Off);
    assert!(!SelfImprovementMode::Off.proposes());
    assert!(SelfImprovementMode::Propose.proposes());
}

#[test]
fn the_gate_returns_none_when_off_and_some_when_proposing() {
    // Synthetic drift: the pipeline runs a stage the recipe does not describe.
    let recipe = "fn_x\n  record_type \"meta_function\"\n  function \"record_a\"\n  source_file \"src/a.rs\"\n";
    let pipeline = "fn run() { crate::a::record_a(log); crate::b::record_b(log); }\n";
    assert!(
        propose_recipe_update(recipe, pipeline, SelfImprovementMode::Off).is_none(),
        "the default Off mode must keep the loop dormant"
    );
    let proposal = propose_recipe_update(recipe, pipeline, SelfImprovementMode::Propose)
        .expect("Propose mode must produce a proposal");
    assert!(!proposal.is_self_consistent());
}

#[test]
fn modes_round_trip_through_their_slugs() {
    for mode in [SelfImprovementMode::Off, SelfImprovementMode::Propose] {
        assert_eq!(SelfImprovementMode::from_slug(mode.slug()), Some(mode));
    }
    assert_eq!(
        SelfImprovementMode::from_slug("  PROPOSE "),
        Some(SelfImprovementMode::Propose)
    );
    assert_eq!(SelfImprovementMode::from_slug("rewrite-everything"), None);
}

// ---------------------------------------------------------------------------
// The loop detects real drift on synthetic input.
// ---------------------------------------------------------------------------

#[test]
fn an_undescribed_pipeline_stage_is_proposed_as_an_addition() {
    let recipe = "fn_a\n  record_type \"meta_function\"\n  function \"record_a\"\n  source_file \"src/a.rs\"\n";
    let pipeline = "fn run() { crate::a::record_a(log); crate::evidence::record_evidence(log); }\n";
    let proposal = MetaSelfImprovement::from_sources(recipe, pipeline).propose();
    assert!(!proposal.is_self_consistent());
    assert_eq!(proposal.change_count(), 1);
    assert!(proposal.stale_citations.is_empty());
    let added = &proposal.undescribed_stages;
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].function, "record_evidence");
    assert_eq!(added[0].source_file(), "src/evidence.rs");
    // The proposed updated algorithm is itself link-encoded output.
    let lino = proposal.to_links_notation(SelfImprovementMode::Propose);
    assert!(
        lino.contains("record_type \"meta_recipe_proposal\""),
        "{lino}"
    );
    assert!(lino.contains("self_consistent \"false\""), "{lino}");
    assert!(
        lino.contains("record_type \"proposed_meta_function\""),
        "{lino}"
    );
    assert!(lino.contains("function \"record_evidence\""), "{lino}");
    assert!(lino.contains("source_file \"src/evidence.rs\""), "{lino}");
}

#[test]
fn a_stale_recipe_citation_is_proposed_for_removal() {
    // The recipe still cites a stage the pipeline no longer runs.
    let recipe = "fn_a\n  record_type \"meta_function\"\n  function \"record_a\"\n  source_file \"src/a.rs\"\nfn_gone\n  record_type \"meta_function\"\n  function \"record_gone\"\n  source_file \"src/gone.rs\"\n";
    let pipeline = "fn run() { crate::a::record_a(log); }\n";
    let proposal = MetaSelfImprovement::from_sources(recipe, pipeline).propose();
    assert!(!proposal.is_self_consistent());
    assert!(proposal.undescribed_stages.is_empty());
    assert_eq!(proposal.stale_citations, vec!["record_gone".to_owned()]);
    let lino = proposal.to_links_notation(SelfImprovementMode::Propose);
    assert!(
        lino.contains("record_type \"stale_meta_function\""),
        "{lino}"
    );
    assert!(lino.contains("function \"record_gone\""), "{lino}");
}

#[test]
fn non_record_functions_are_ignored_so_helpers_are_not_flagged() {
    // The recipe cites helpers like `decompose_once`; only `record_*` stages are
    // compared, so a non-record citation is never treated as a stale stage.
    let recipe = "fn_helper\n  record_type \"meta_function\"\n  function \"decompose_once\"\n  source_file \"src/meta_frame.rs\"\n";
    let pipeline = "fn run() { crate::a::record_a(log); }\n";
    let proposal = MetaSelfImprovement::from_sources(recipe, pipeline).propose();
    assert!(
        proposal.stale_citations.is_empty(),
        "a non-record helper citation must not be reported as stale"
    );
    // The pipeline's record_a is still undescribed, of course.
    assert_eq!(proposal.undescribed_stages.len(), 1);
    assert_eq!(proposal.undescribed_stages[0].function, "record_a");
}

// ---------------------------------------------------------------------------
// On the checked-in recipe and pipeline, the algorithm already describes itself.
// ---------------------------------------------------------------------------

#[test]
fn the_live_recipe_already_describes_every_pipeline_stage() {
    let loop_ = MetaSelfImprovement::from_repo();
    let proposal = loop_.propose();
    assert!(
        proposal.is_self_consistent(),
        "the recipe drifted from the pipeline; the loop would propose: {}",
        proposal.summary()
    );
    assert_eq!(proposal.change_count(), 0);
    // The serialized "updated algorithm" for a self-consistent run is a no-op.
    let lino = proposal.to_links_notation(SelfImprovementMode::Propose);
    assert!(lino.contains("self_consistent \"true\""), "{lino}");
    assert!(lino.contains("change_count \"0\""), "{lino}");
    assert!(
        !lino.contains("proposed_meta_function"),
        "a self-consistent run proposes no additions:\n{lino}"
    );
}

#[test]
fn the_loop_sees_every_record_stage_the_pipeline_runs() {
    // The live pipeline runs these nine record_* stages; the loop must parse all
    // of them out of the meta_core source, in source order.
    let loop_ = MetaSelfImprovement::from_repo();
    let functions: Vec<&str> = loop_
        .pipeline_stages()
        .iter()
        .map(|stage| stage.function.as_str())
        .collect();
    for expected in [
        "record_problem_frame",
        "record_work_units",
        "record_need_ledger",
        "record_method_registry",
        "record_work_unit_reasoning",
        "record_upward_construction",
        "record_solution_evidence",
        "record_selection",
        "record_skill_ledger",
    ] {
        assert!(
            functions.contains(&expected),
            "the loop must see the live pipeline stage {expected}: {functions:?}"
        );
    }
}
