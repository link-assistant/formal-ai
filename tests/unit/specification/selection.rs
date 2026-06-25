//! Issue #559 (R339): registry-driven method selection trace.
//!
//! A single data-driven authority names the method that resolves an atomic
//! work-unit leaf: the registry resolver (`MethodRegistry::method_for_route`,
//! alias-aware). The legacy hardcoded mapper was removed once the registry became
//! the sole dispatch authority, so the selection artifact records *what the
//! registry selects* rather than a comparison. These tests pin that the registry
//! resolves a routed leaf to its method, rescues an aliased route (e.g.
//! `write_program` → `write_script`), records an unroutable leaf as `unresolved`,
//! and that the `SelectionMode` knob only controls trace verbosity: the default
//! `Off` records nothing.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::WorkUnit;
use formal_ai::method_registry::MethodRegistry;
use formal_ai::selection::{MethodSelection, SelectionMode};
use formal_ai::translation::formalize_prompt;
use formal_ai::{SolverConfig, UniversalSolver};

fn selection_for(prompt: &str, max_depth: u8) -> (WorkUnit, MethodSelection) {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let root = WorkUnit::from_formalization(&formalization, max_depth);
    let registry = MethodRegistry::from_dispatch();
    let selection = MethodSelection::for_unit(&root, &registry);
    (root, selection)
}

fn solve_with_selection(prompt: &str, mode: SelectionMode) -> Vec<String> {
    let config = SolverConfig {
        selection_mode: mode,
        ..SolverConfig::default()
    };
    UniversalSolver::new(config).solve(prompt).evidence_links
}

/// The canonical prompts the case study exercises: a single routed need, a
/// conjunction of two needs (one of them an aliased route), and an unroutable one.
const CANONICAL_PROMPTS: &[&str] = &[
    "translate apple to Russian",
    "translate apple to Russian and write a hello world program in Python",
    "zzqqx unfathomable gibberish token",
];

// ---------------------------------------------------------------------------
// The SelectionMode knob itself.
// ---------------------------------------------------------------------------

#[test]
fn off_is_the_default_and_emits_no_artifact() {
    assert_eq!(SelectionMode::default(), SelectionMode::Off);
    assert!(!SelectionMode::Off.emits_artifact());
}

#[test]
fn record_emits_an_artifact() {
    assert!(SelectionMode::Record.emits_artifact());
}

#[test]
fn modes_round_trip_through_their_slugs() {
    for mode in [SelectionMode::Off, SelectionMode::Record] {
        assert_eq!(SelectionMode::from_slug(mode.slug()), Some(mode));
    }
    assert_eq!(
        SelectionMode::from_slug("  RECORD "),
        Some(SelectionMode::Record)
    );
    assert_eq!(SelectionMode::from_slug("sideways"), None);
}

// ---------------------------------------------------------------------------
// The registry resolves each leaf faithfully.
// ---------------------------------------------------------------------------

#[test]
fn a_routed_leaf_resolves_to_its_method() {
    let (_root, selection) = selection_for("translate apple to Russian", 4);
    assert_eq!(selection.leaf_count(), 1, "a single need is one leaf");
    let leaf = &selection.leaves[0];
    assert_eq!(leaf.route.as_deref(), Some("translation"));
    assert_eq!(leaf.method.as_deref(), Some("translation"));
    assert!(leaf.is_resolved());
    assert_eq!(selection.resolved_count(), 1);
    assert_eq!(selection.unresolved_count(), 0);
}

#[test]
fn an_aliased_route_resolves_through_the_alias() {
    // `write_program` has no handler of that name; the registry resolves
    // `write_script` through the route→method alias (R336).
    let (_root, selection) = selection_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let rescued = selection
        .leaves
        .iter()
        .find(|leaf| leaf.method.as_deref() == Some("write_script"))
        .expect("the program-writing leaf must resolve to write_script");
    assert_eq!(rescued.route.as_deref(), Some("write_program"));
    assert!(rescued.is_resolved());
}

#[test]
fn an_unroutable_leaf_is_unresolved() {
    let (_root, selection) = selection_for("zzqqx unfathomable gibberish token", 4);
    assert!(
        selection.leaves.iter().all(|leaf| !leaf.is_resolved()),
        "a gibberish prompt resolves to no method"
    );
    assert_eq!(selection.resolved_count(), 0);
    assert_eq!(selection.unresolved_count(), selection.leaf_count());
}

// ---------------------------------------------------------------------------
// Every routed leaf resolves to a registered method.
// ---------------------------------------------------------------------------

#[test]
fn resolved_and_unresolved_counts_partition_the_leaves() {
    for prompt in CANONICAL_PROMPTS {
        let (_root, selection) = selection_for(prompt, 4);
        assert_eq!(
            selection.resolved_count() + selection.unresolved_count(),
            selection.leaf_count(),
            "resolved and unresolved leaves must partition the tree — prompt: {prompt}"
        );
        let registry = MethodRegistry::from_dispatch();
        for leaf in &selection.leaves {
            if let Some(method) = &leaf.method {
                assert!(
                    registry.methods.iter().any(|m| &m.name == method),
                    "leaf {} resolved to an unregistered method {method:?}",
                    leaf.unit_id
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Serialization shows the resolved method (or `unresolved`).
// ---------------------------------------------------------------------------

#[test]
fn serialization_shows_the_resolved_method() {
    let (_root, selection) = selection_for("translate apple to Russian", 4);
    let lino = selection.to_links_notation();
    assert!(lino.contains("record_type \"selection\""), "{lino}");
    assert!(lino.contains("method \"translation\""), "{lino}");
    assert!(lino.contains("resolved_count \"1\""), "{lino}");
    assert!(lino.contains("unresolved_count \"0\""), "{lino}");
    assert!(
        !lino.contains("legacy_method"),
        "there is no legacy authority to surface:\n{lino}"
    );
}

#[test]
fn serialization_marks_an_unresolved_leaf() {
    let (_root, selection) = selection_for("zzqqx unfathomable gibberish token", 4);
    let lino = selection.to_links_notation();
    assert!(lino.contains("method \"unresolved\""), "{lino}");
}

// ---------------------------------------------------------------------------
// The knob gates the emitted trace; live method dispatch is registry-backed
// either way.
// ---------------------------------------------------------------------------

fn has_prefix(links: &[String], prefix: &str) -> bool {
    links.iter().any(|link| link.starts_with(prefix))
}

#[test]
fn default_off_mode_emits_no_selection_artifact() {
    let links = solve_with_selection("translate apple to Russian", SelectionMode::Off);
    assert!(
        !has_prefix(&links, "selection"),
        "default Off mode must not emit a selection artifact: {links:?}"
    );
}

#[test]
fn record_mode_emits_the_selection_artifact() {
    let links = solve_with_selection("translate apple to Russian", SelectionMode::Record);
    assert!(
        has_prefix(&links, "selection"),
        "Record mode must emit the selection artifact: {links:?}"
    );
}

#[test]
fn the_knob_changes_only_the_trace_not_the_answer() {
    let prompt = "translate apple to Russian and write a hello world program in Python";
    let off = UniversalSolver::new(SolverConfig {
        selection_mode: SelectionMode::Off,
        ..SolverConfig::default()
    })
    .solve(prompt);
    let record = UniversalSolver::new(SolverConfig {
        selection_mode: SelectionMode::Record,
        ..SolverConfig::default()
    })
    .solve(prompt);
    assert_eq!(
        off.answer, record.answer,
        "the answer must not depend on the selection mode"
    );
    assert_eq!(
        off.intent, record.intent,
        "the intent must not depend on the selection mode"
    );
}
