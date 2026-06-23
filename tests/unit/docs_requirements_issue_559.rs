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

    // The frame is wired into the solver loop as a trace-only event.
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("crate::meta_frame::record_problem_frame"),
        "src/solver.rs should emit the problem frame in the main loop"
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

    // The recursive pass is wired into the solver loop, bounded by the existing
    // depth knob so it stays terminating and behavior-preserving.
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("crate::meta_frame::record_work_units"),
        "src/solver.rs should emit the work-unit decomposition in the main loop"
    );
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

    // The ledger is wired into the solver loop as a trace-only event.
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("crate::meta_frame::record_need_ledger"),
        "src/solver.rs should emit the need ledger in the main loop"
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

    // The registry is wired into the solver loop as a trace-only event.
    let solver = read(root.join("src/solver.rs"));
    assert!(
        solver.contains("crate::method_registry::record_method_registry"),
        "src/solver.rs should emit the method registry in the main loop"
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
