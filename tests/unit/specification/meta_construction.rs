//! Issue #559: the upward construction pass and the `recursion_mode` knob (R338).
//!
//! Decomposition (the downward pass) is only half of a recursive algorithm; the
//! other half is construction — composing the children's results back up into the
//! parent's answer, leaf to root. These tests pin that the construction is a
//! faithful post-order (bottom-up) walk of the work-unit tree, that each step
//! carries a rationale and resolves leaf methods through the same route→method
//! bridge the evidence join uses, and that which directions the meta core emits is
//! governed by [`RecursionMode`] — `Down` (the default) reproduces the pre-knob
//! trace exactly, so the knob is strictly additive and trace-only (R13).

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_construction::{RecursionMode, UpwardConstruction};
use formal_ai::meta_frame::WorkUnit;
use formal_ai::method_registry::MethodRegistry;
use formal_ai::translation::formalize_prompt;
use formal_ai::{SolverConfig, UniversalSolver};

fn construction_for(prompt: &str, max_depth: u8) -> (WorkUnit, UpwardConstruction) {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let root = WorkUnit::from_formalization(&formalization, max_depth);
    let registry = MethodRegistry::from_dispatch();
    let construction = UpwardConstruction::for_unit(&root, &registry);
    (root, construction)
}

fn solve_with_mode(prompt: &str, mode: RecursionMode) -> Vec<String> {
    let config = SolverConfig {
        recursion_mode: mode,
        ..SolverConfig::default()
    };
    UniversalSolver::new(config).solve(prompt).evidence_links
}

// ---------------------------------------------------------------------------
// The RecursionMode knob itself.
// ---------------------------------------------------------------------------

#[test]
fn down_is_the_default_and_emits_only_the_downward_direction() {
    assert_eq!(RecursionMode::default(), RecursionMode::Down);
    assert!(RecursionMode::Down.emits_downward());
    assert!(!RecursionMode::Down.emits_upward());
}

#[test]
fn up_emits_only_the_upward_direction_and_both_emits_both() {
    assert!(!RecursionMode::Up.emits_downward());
    assert!(RecursionMode::Up.emits_upward());
    assert!(RecursionMode::Both.emits_downward());
    assert!(RecursionMode::Both.emits_upward());
}

#[test]
fn modes_round_trip_through_their_slugs() {
    for mode in [RecursionMode::Down, RecursionMode::Up, RecursionMode::Both] {
        assert_eq!(RecursionMode::from_slug(mode.slug()), Some(mode));
    }
    // Tolerant of surrounding whitespace and case; rejects the unknown.
    assert_eq!(
        RecursionMode::from_slug("  BOTH "),
        Some(RecursionMode::Both)
    );
    assert_eq!(RecursionMode::from_slug("sideways"), None);
}

// ---------------------------------------------------------------------------
// The construction is a faithful post-order walk of the work-unit tree.
// ---------------------------------------------------------------------------

#[test]
fn construction_visits_every_unit_once_in_post_order() {
    let (root, construction) = construction_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    // One construction step per work unit.
    assert_eq!(
        construction.step_count(),
        root.unit_count(),
        "there must be exactly one construction step per work unit"
    );
    // Post-order: orders are 1..=N, strictly increasing, and the root is last.
    let orders: Vec<usize> = construction.steps.iter().map(|s| s.order).collect();
    assert_eq!(orders, (1..=construction.step_count()).collect::<Vec<_>>());
    let last = construction
        .steps
        .last()
        .expect("a non-empty tree has a last step");
    assert_eq!(
        last.unit_id, root.unit_id,
        "the root must be constructed last (children before parents)"
    );
    assert_eq!(last.kind, "compose");
}

#[test]
fn leaves_construct_from_a_method_and_parents_compose_their_children() {
    let (_root, construction) = construction_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    for step in &construction.steps {
        assert!(
            !step.rationale.is_empty(),
            "step {} must explain how its answer is constructed",
            step.unit_id
        );
        match step.kind.as_str() {
            "leaf_method" => assert!(step.inputs.is_empty(), "a leaf has no children to compose"),
            "compose" => {
                assert!(
                    step.method.is_none(),
                    "a composing parent constructs from children, not a method"
                );
                assert!(
                    !step.inputs.is_empty(),
                    "a composing parent must name the children it composes"
                );
            }
            other => panic!("unknown construction kind `{other}`"),
        }
    }
    // The two leaves resolve to the same methods the evidence join resolves.
    let methods: Vec<&str> = construction
        .steps
        .iter()
        .filter_map(|s| s.method.as_deref())
        .collect();
    assert!(
        methods.contains(&"write_script"),
        "the program-writing leaf must construct from write_script, got {methods:?}"
    );
    assert!(
        methods.contains(&"translation"),
        "the translation leaf must construct from translation, got {methods:?}"
    );
}

#[test]
fn compose_inputs_are_the_children_in_source_order() {
    let (root, construction) = construction_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let root_step = construction
        .steps
        .iter()
        .find(|s| s.unit_id == root.unit_id)
        .expect("the root has a construction step");
    let child_ids: Vec<String> = root.children.iter().map(|c| c.unit_id.clone()).collect();
    assert_eq!(
        root_step.inputs, child_ids,
        "the root composes its children in source order"
    );
}

#[test]
fn construction_serializes_to_links_notation_records() {
    let (_root, construction) = construction_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let lino = construction.to_links_notation();
    assert!(
        lino.contains("record_type \"upward_construction\""),
        "the construction must carry a header record:\n{lino}"
    );
    let steps = lino.matches("record_type \"construction_step\"").count();
    assert_eq!(
        steps,
        construction.step_count(),
        "every construction step must serialize to exactly one record"
    );
}

// ---------------------------------------------------------------------------
// The knob gates the emitted trace; the default is behavior-preserving.
// ---------------------------------------------------------------------------

fn has_prefix(links: &[String], prefix: &str) -> bool {
    links.iter().any(|link| link.starts_with(prefix))
}

#[test]
fn default_mode_traces_the_downward_reasoning_but_not_the_construction() {
    let links = solve_with_mode("translate apple to Russian", RecursionMode::Down);
    assert!(
        has_prefix(&links, "work_unit_reasoning"),
        "default Down mode must still emit the downward reasoning: {links:?}"
    );
    assert!(
        !has_prefix(&links, "upward_construction"),
        "default Down mode must not emit the upward construction: {links:?}"
    );
}

#[test]
fn up_mode_traces_the_construction_but_not_the_downward_reasoning() {
    let links = solve_with_mode("translate apple to Russian", RecursionMode::Up);
    assert!(
        has_prefix(&links, "upward_construction"),
        "Up mode must emit the upward construction: {links:?}"
    );
    assert!(
        !has_prefix(&links, "work_unit_reasoning"),
        "Up mode must not emit the downward reasoning: {links:?}"
    );
}

#[test]
fn both_mode_traces_both_directions() {
    let links = solve_with_mode("translate apple to Russian", RecursionMode::Both);
    assert!(has_prefix(&links, "work_unit_reasoning"), "{links:?}");
    assert!(has_prefix(&links, "upward_construction"), "{links:?}");
}

#[test]
fn the_structural_work_unit_events_are_emitted_in_every_mode() {
    // The decomposition events are structural, not directional reasoning: they
    // must stay always-on regardless of the knob, so the recursion is always
    // observable (R332).
    for mode in [RecursionMode::Down, RecursionMode::Up, RecursionMode::Both] {
        let links = solve_with_mode("translate apple to Russian", mode);
        assert!(
            has_prefix(&links, "work_unit:enter"),
            "{mode:?} must still record work-unit entry: {links:?}"
        );
        assert!(
            has_prefix(&links, "work_unit:exit"),
            "{mode:?} must still record work-unit exit: {links:?}"
        );
    }
}

#[test]
fn the_knob_changes_only_the_trace_not_the_answer() {
    // Trace-only contract (R13): the produced answer is identical across modes;
    // only which reasoning direction is recorded differs.
    let prompt = "translate apple to Russian";
    let down = UniversalSolver::new(SolverConfig {
        recursion_mode: RecursionMode::Down,
        ..SolverConfig::default()
    })
    .solve(prompt);
    let both = UniversalSolver::new(SolverConfig {
        recursion_mode: RecursionMode::Both,
        ..SolverConfig::default()
    })
    .solve(prompt);
    assert_eq!(
        down.answer, both.answer,
        "the answer must not depend on the mode"
    );
    assert_eq!(
        down.intent, both.intent,
        "the intent must not depend on the mode"
    );
}
