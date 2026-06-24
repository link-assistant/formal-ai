//! Traceability for issue #559 (general meta algorithm).
//!
//! These tests keep the REQUIREMENTS.md rows for issue #559 honest: each row
//! names the source and tests that implement it, and the test asserts those
//! artifacts exist and carry the named entry points. Rows are added as each
//! behavior-preserving phase lands; this guard grows with them.

use std::fs;
use std::path::Path;

#[test]
fn issue_559_problem_frame_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R330: the requirements section and row exist and cite the shipped frame.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "## Issue #559 General Meta Algorithm",
            "| R330 ",
            "problem frame",
            "src/meta_frame.rs",
            "record_problem_frame",
            "tests/unit/specification/meta_frame.rs",
        ],
    );

    // The frame module ships the named structures and the loop-event recorder.
    let meta_frame = read(root.join("src/meta_frame.rs"));
    assert_contains_all(
        "src/meta_frame.rs",
        &meta_frame,
        &[
            "pub struct ProblemFrame",
            "pub struct Need",
            "pub enum NeedStatus",
            "fn from_formalization",
            "fn to_links_notation",
            "fn record_problem_frame",
            "problem_frame",
        ],
    );

    // The frame is wired into the meta core, which the solver loop invokes.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::meta_frame::record_problem_frame"),
        "src/meta_core.rs should emit the problem frame in the meta-core pass"
    );
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("crate::meta_core::record_meta_core"),
        "src/solver.rs should invoke the meta core in the main loop"
    );
}

#[test]
fn issue_559_recursive_work_units_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R332: the requirements row exists and cites the shipped recursive core.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R332 ",
            "recursive, bounded work-unit tree",
            "src/meta_frame.rs",
            "record_work_units",
            "max_decomposition_depth",
        ],
    );

    // The frame module ships the recursive work-unit structures and recorder.
    let meta_frame = read(root.join("src/meta_frame.rs"));
    assert_contains_all(
        "src/meta_frame.rs",
        &meta_frame,
        &[
            "pub struct WorkUnit",
            "pub enum AtomicityReason",
            "fn decompose_once",
            "fn record_work_units",
            "work_unit:enter",
            "work_unit:exit",
        ],
    );

    // The recursive pass is wired into the meta core, bounded by the existing
    // depth knob so it stays terminating and behavior-preserving.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::meta_frame::record_work_units"),
        "src/meta_core.rs should emit the work-unit decomposition"
    );
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("self.config.max_decomposition_depth"),
        "the recursive pass must be bounded by max_decomposition_depth"
    );
}

#[test]
fn issue_559_need_ledger_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R333: the requirements row exists and cites the shipped ledger.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R333 ",
            "need-satisfaction ledger",
            "src/meta_frame.rs",
            "record_need_ledger",
        ],
    );

    // The frame module ships the ledger structures and recorder.
    let meta_frame = read(root.join("src/meta_frame.rs"));
    assert_contains_all(
        "src/meta_frame.rs",
        &meta_frame,
        &[
            "pub struct NeedLedger",
            "pub struct LedgerRow",
            "fn resolve",
            "fn every_need_accounted_for",
            "fn record_need_ledger",
            "need:status",
        ],
    );

    // The ledger is wired into the meta core as a trace-only event.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::meta_frame::record_need_ledger"),
        "src/meta_core.rs should emit the need ledger"
    );
}

#[test]
fn issue_559_method_registry_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R331: the requirements row exists and cites the shipped registry.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R331 ",
            "method registry",
            "src/method_registry.rs",
            "from_dispatch",
            "record_method_registry",
        ],
    );

    // The registry module ships the named structures and the loop-event recorder.
    let registry = read(root.join("src/method_registry.rs"));
    assert_contains_all(
        "src/method_registry.rs",
        &registry,
        &[
            "pub struct MethodRegistry",
            "pub struct Method",
            "pub enum MethodSurface",
            "fn from_dispatch",
            "fn to_links_notation",
            "fn record_method_registry",
            "method_registry",
        ],
    );

    // The catalogue is grounded in the live dispatch constants.
    let dispatch = read(root.join("src/solver_dispatch.rs"));
    assert!(
        dispatch.contains("CONTEXTUAL_HANDLER_NAMES"),
        "src/solver_dispatch.rs should expose the contextual handler names the registry reads"
    );

    // The registry is wired into the meta core as a trace-only event.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::method_registry::record_method_registry"),
        "src/meta_core.rs should emit the method registry"
    );
}

#[test]
fn issue_559_recursive_core_recipe_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R335: the requirements row exists and cites the shipped self-description.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R335 ",
            "describe itself as grounded link data",
            "data/meta/recursive-core-recipe.lino",
        ],
    );

    // The recipe exists and declares the meta_recipe header plus its steps.
    let recipe = read(root.join("data/meta/recursive-core-recipe.lino"));
    assert_contains_all(
        "data/meta/recursive-core-recipe.lino",
        &recipe,
        &[
            "record_type \"meta_recipe\"",
            "topic \"recursive_core\"",
            "record_type \"meta_step\"",
            "record_type \"meta_function\"",
        ],
    );
}

#[test]
fn issue_559_solution_evidence_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R334: the requirements row exists and cites the shipped evidence pipeline.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R334 ",
            "end-to-end evidence chain",
            "src/solution_evidence.rs",
            "record_solution_evidence",
        ],
    );

    // The evidence module ships the named structures and the loop-event recorder.
    let evidence = read(root.join("src/solution_evidence.rs"));
    assert_contains_all(
        "src/solution_evidence.rs",
        &evidence,
        &[
            "pub struct SolutionEvidence",
            "pub struct EvidenceTrail",
            "fn assemble",
            "fn accounted_for",
            "fn fully_resolved",
            "fn record_solution_evidence",
            "solution_evidence",
        ],
    );

    // The evidence join is wired into the meta core as a trace-only event.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::solution_evidence::record_solution_evidence"),
        "src/meta_core.rs should emit the solution evidence"
    );
}

#[test]
fn issue_559_route_method_alias_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R336: the requirements row exists and cites the shipped alias bridge.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R336 ",
            "route vocabulary",
            "data/meta/route-method-aliases.lino",
            "src/route_method_alias.rs",
            "method_for_route",
        ],
    );

    // The alias data declares the catalogue and at least the write_program entry.
    let aliases = read(root.join("data/meta/route-method-aliases.lino"));
    assert_contains_all(
        "data/meta/route-method-aliases.lino",
        &aliases,
        &[
            "record_type \"route_method_alias\"",
            "route \"write_program\"",
            "method \"write_script\"",
        ],
    );

    // The loader module ships the named structures and the resolver helper.
    let module = read(root.join("src/route_method_alias.rs"));
    assert_contains_all(
        "src/route_method_alias.rs",
        &module,
        &[
            "pub struct RouteMethodAlias",
            "pub fn aliases",
            "pub fn method_for_alias",
        ],
    );

    // The registry consumes the alias data through the route→method resolver, and
    // the evidence join surfaces alias provenance.
    let registry = read(root.join("src/method_registry.rs"));
    assert!(
        registry.contains("fn method_for_route"),
        "src/method_registry.rs should expose method_for_route"
    );
    assert!(
        registry.contains("crate::route_method_alias::method_for_alias"),
        "method_for_route should consult the route→method alias data"
    );
    let evidence = read(root.join("src/solution_evidence.rs"));
    assert!(
        evidence.contains("method_via_alias"),
        "src/solution_evidence.rs should record alias provenance on each trail"
    );
}

#[test]
fn issue_559_work_unit_reasoning_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R337: the requirements row exists and cites the shipped white-box reasoning.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R337 ",
            "white-box recursive reasoning",
            "src/meta_reasoning.rs",
            "record_work_unit_reasoning",
        ],
    );

    // The reasoning module ships the named structure and the loop-event recorder.
    let module = read(root.join("src/meta_reasoning.rs"));
    assert_contains_all(
        "src/meta_reasoning.rs",
        &module,
        &[
            "pub struct WorkUnitReasoning",
            "fn for_unit",
            "fn to_links_notation",
            "fn record_work_unit_reasoning",
            "work_unit_reasoning",
            "downward_rationale",
            "upward_rationale",
        ],
    );

    // The reasoning is wired into the meta core as a trace-only event.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::meta_reasoning::record_work_unit_reasoning"),
        "src/meta_core.rs should emit the white-box work-unit reasoning"
    );
}

#[test]
fn issue_559_upward_construction_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R338: the requirements row exists and cites the upward construction pass.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R338 ",
            "upward construction pass",
            "src/meta_construction.rs",
            "record_upward_construction",
            "recursion_mode",
        ],
    );

    // The construction module ships the mode knob, the structures, and recorder.
    let module = read(root.join("src/meta_construction.rs"));
    assert_contains_all(
        "src/meta_construction.rs",
        &module,
        &[
            "pub enum RecursionMode",
            "pub struct UpwardConstruction",
            "pub struct ConstructionStep",
            "fn emits_downward",
            "fn emits_upward",
            "fn for_unit",
            "fn to_links_notation",
            "fn record_upward_construction",
            "upward_construction",
        ],
    );

    // The pass is wired into the meta core, gated by the solver config knob.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::meta_construction::record_upward_construction"),
        "src/meta_core.rs should emit the upward construction pass"
    );
    assert!(
        meta_core.contains("mode.emits_downward()"),
        "src/meta_core.rs should gate the downward reasoning on the recursion mode"
    );
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("recursion_mode"),
        "src/solver.rs should expose the recursion_mode config knob"
    );
    assert!(
        solver.contains("self.config.recursion_mode"),
        "src/solver.rs should pass the recursion mode into the meta core"
    );
}

#[test]
fn issue_559_selection_comparison_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R339: the requirements row exists and cites the selection comparison.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R339 ",
            "method-selection comparison",
            "src/selection.rs",
            "record_selection",
            "tests/unit/specification/selection.rs",
        ],
    );

    // The selection module ships the mode knob, the structures, and recorder.
    let module = read(root.join("src/selection.rs"));
    assert_contains_all(
        "src/selection.rs",
        &module,
        &[
            "pub enum SelectionMode",
            "pub enum SelectionAgreement",
            "pub struct LeafSelection",
            "pub struct SelectionComparison",
            "fn emits_artifact",
            "fn records_comparison",
            "fn for_unit",
            "fn to_links_notation",
            "fn record_selection",
            "selection",
        ],
    );

    // The two compared authorities both exist: the hardcoded legacy mapping and
    // the data-driven registry resolver.
    let intent = read(root.join("src/intent_formalization.rs"));
    assert!(
        intent.contains("fn specialized_handler_name"),
        "src/intent_formalization.rs should expose the legacy selection authority"
    );
    let registry = read(root.join("src/method_registry.rs"));
    assert!(
        registry.contains("fn method_for_route"),
        "src/method_registry.rs should expose the registry selection authority"
    );

    // The comparison is wired into the meta core, gated by the solver config knob.
    let meta_core = read(root.join("src/meta_core.rs"));
    assert!(
        meta_core.contains("crate::selection::record_selection"),
        "src/meta_core.rs should emit the method-selection comparison"
    );
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("selection_mode"),
        "src/solver.rs should expose the selection_mode config knob"
    );
    assert!(
        solver.contains("self.config.selection_mode"),
        "src/solver.rs should pass the selection mode into the meta core"
    );
}

#[test]
fn issue_559_meta_self_improvement_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R340: the requirements row exists and cites the self-improvement loop.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R340 ",
            "reason about improving itself",
            "src/meta_self_improvement.rs",
            "propose_recipe_update",
            "tests/unit/specification/meta_self_improvement.rs",
        ],
    );

    // The module ships the gate, the proposal, and the meta-circular loop.
    let module = read(root.join("src/meta_self_improvement.rs"));
    assert_contains_all(
        "src/meta_self_improvement.rs",
        &module,
        &[
            "pub enum SelfImprovementMode",
            "pub struct PipelineStage",
            "pub struct MetaRecipeProposal",
            "pub struct MetaSelfImprovement",
            "fn proposes",
            "fn is_self_consistent",
            "fn from_repo",
            "fn propose",
            "fn to_links_notation",
            "fn propose_recipe_update",
        ],
    );

    // The loop reads itself: the recipe (algorithm-as-data) and the live pipeline
    // (algorithm-as-code) are both embedded at compile time.
    assert!(
        module.contains("data/meta/recursive-core-recipe.lino"),
        "the loop must read the recipe — the algorithm encoded as link data"
    );
    assert!(
        module.contains("include_str!(\"meta_core.rs\")"),
        "the loop must read the live pipeline source as its ground truth"
    );

    // The recipe now describes the solution-evidence stage the pipeline runs, so
    // the live loop is self-consistent (the finding it surfaced, now adopted).
    let recipe = read(root.join("data/meta/recursive-core-recipe.lino"));
    assert!(
        recipe.contains("record_solution_evidence"),
        "the recipe must cite every record_* stage the pipeline runs"
    );
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
