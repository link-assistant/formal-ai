//! Issue #559 (R339): registry-driven method selection, compared to the legacy.
//!
//! Two authorities can name the method that resolves an atomic work-unit leaf: the
//! hardcoded legacy mapping (`specialized_handler_name`) and the data-driven
//! registry resolver (`MethodRegistry::method_for_route`, alias-aware). For the
//! registry to eventually *drive* selection — and to retire the hardcoded
//! authority — it must never contradict a valid legacy selection. These tests pin
//! that invariant (zero contradictions across the canonical prompts), the
//! `registry_rescues` case the route→method aliases exist to fix, and that the
//! `SelectionMode` knob is strictly additive and trace-only (R13): the default
//! `Legacy` records nothing and never changes the answer.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::WorkUnit;
use formal_ai::method_registry::MethodRegistry;
use formal_ai::selection::{SelectionAgreement, SelectionComparison, SelectionMode};
use formal_ai::translation::formalize_prompt;
use formal_ai::{SolverConfig, UniversalSolver};

fn comparison_for(prompt: &str, max_depth: u8) -> (WorkUnit, SelectionComparison) {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let root = WorkUnit::from_formalization(&formalization, max_depth);
    let registry = MethodRegistry::from_dispatch();
    let comparison = SelectionComparison::for_unit(&root, &registry);
    (root, comparison)
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
fn legacy_is_the_default_and_emits_no_artifact() {
    assert_eq!(SelectionMode::default(), SelectionMode::Legacy);
    assert!(!SelectionMode::Legacy.emits_artifact());
    assert!(!SelectionMode::Legacy.records_comparison());
}

#[test]
fn registry_emits_an_artifact_without_the_comparison_and_compare_adds_it() {
    assert!(SelectionMode::Registry.emits_artifact());
    assert!(!SelectionMode::Registry.records_comparison());
    assert!(SelectionMode::Compare.emits_artifact());
    assert!(SelectionMode::Compare.records_comparison());
}

#[test]
fn modes_round_trip_through_their_slugs() {
    for mode in [
        SelectionMode::Legacy,
        SelectionMode::Registry,
        SelectionMode::Compare,
    ] {
        assert_eq!(SelectionMode::from_slug(mode.slug()), Some(mode));
    }
    assert_eq!(
        SelectionMode::from_slug("  COMPARE "),
        Some(SelectionMode::Compare)
    );
    assert_eq!(SelectionMode::from_slug("sideways"), None);
}

// ---------------------------------------------------------------------------
// The comparison classifies each leaf faithfully.
// ---------------------------------------------------------------------------

#[test]
fn a_routed_leaf_has_both_authorities_agree() {
    let (_root, comparison) = comparison_for("translate apple to Russian", 4);
    assert_eq!(comparison.leaf_count(), 1, "a single need is one leaf");
    let leaf = &comparison.leaves[0];
    assert_eq!(leaf.route.as_deref(), Some("translation"));
    assert_eq!(leaf.legacy_method.as_deref(), Some("translation"));
    assert_eq!(leaf.registry_method.as_deref(), Some("translation"));
    assert_eq!(leaf.agreement, SelectionAgreement::Agree);
}

#[test]
fn an_aliased_route_is_a_registry_rescue_not_a_contradiction() {
    // `write_program` has no handler of that name: the legacy authority's
    // catch-all names a non-existent handler, so it resolves nothing real, while
    // the registry resolves `write_script` through the route→method alias (R336).
    let (_root, comparison) = comparison_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let rescued = comparison
        .leaves
        .iter()
        .find(|leaf| leaf.registry_method.as_deref() == Some("write_script"))
        .expect("the program-writing leaf must resolve to write_script");
    assert_eq!(rescued.route.as_deref(), Some("write_program"));
    assert_eq!(
        rescued.legacy_method, None,
        "the legacy authority names no real method for write_program"
    );
    assert_eq!(rescued.agreement, SelectionAgreement::RegistryRescues);
    assert!(
        comparison.rescue_count() >= 1,
        "the conjunction must record at least one registry rescue"
    );
}

#[test]
fn an_unroutable_leaf_is_unresolved_by_both_authorities() {
    let (_root, comparison) = comparison_for("zzqqx unfathomable gibberish token", 4);
    assert!(
        comparison
            .leaves
            .iter()
            .all(|leaf| leaf.agreement == SelectionAgreement::Unresolved),
        "a gibberish prompt resolves to no method under either authority"
    );
    assert_eq!(
        comparison.divergence_count(),
        0,
        "both authorities agreeing a leaf is blocked is not divergence"
    );
}

// ---------------------------------------------------------------------------
// The zero-contradiction invariant: the registry never contradicts the legacy.
// ---------------------------------------------------------------------------

#[test]
fn the_registry_never_contradicts_a_valid_legacy_selection() {
    for prompt in CANONICAL_PROMPTS {
        let (_root, comparison) = comparison_for(prompt, 4);
        assert_eq!(
            comparison.contradiction_count(),
            0,
            "no leaf may have the registry name a different real method than the \
             legacy authority — prompt: {prompt}"
        );
        // Every leaf the legacy resolves to a real method, the registry resolves
        // to the same one (that is what "no contradiction" means leaf by leaf).
        for leaf in &comparison.leaves {
            if let Some(legacy) = &leaf.legacy_method {
                assert_eq!(
                    leaf.registry_method.as_ref(),
                    Some(legacy),
                    "leaf {} diverges: legacy={legacy:?} registry={:?}",
                    leaf.unit_id,
                    leaf.registry_method
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Serialization shows the chosen method; the comparison fields are compare-only.
// ---------------------------------------------------------------------------

#[test]
fn registry_mode_serializes_the_chosen_method_without_the_comparison() {
    let (_root, comparison) = comparison_for("translate apple to Russian", 4);
    let lino = comparison.to_links_notation(SelectionMode::Registry);
    assert!(lino.contains("record_type \"selection\""), "{lino}");
    assert!(lino.contains("mode \"registry\""), "{lino}");
    assert!(lino.contains("registry_method \"translation\""), "{lino}");
    assert!(
        !lino.contains("legacy_method"),
        "registry mode must not surface the legacy authority:\n{lino}"
    );
    assert!(
        !lino.contains("agreement"),
        "registry mode records the choice, not the comparison:\n{lino}"
    );
}

#[test]
fn compare_mode_serializes_both_authorities_and_the_agreement() {
    let (_root, comparison) = comparison_for("translate apple to Russian", 4);
    let lino = comparison.to_links_notation(SelectionMode::Compare);
    assert!(lino.contains("mode \"compare\""), "{lino}");
    assert!(lino.contains("legacy_method \"translation\""), "{lino}");
    assert!(lino.contains("registry_method \"translation\""), "{lino}");
    assert!(lino.contains("agreement \"agree\""), "{lino}");
    assert!(lino.contains("contradiction_count \"0\""), "{lino}");
}

// ---------------------------------------------------------------------------
// The knob gates the emitted trace; the default is behavior-preserving.
// ---------------------------------------------------------------------------

fn has_prefix(links: &[String], prefix: &str) -> bool {
    links.iter().any(|link| link.starts_with(prefix))
}

#[test]
fn default_legacy_mode_emits_no_selection_artifact() {
    let links = solve_with_selection("translate apple to Russian", SelectionMode::Legacy);
    assert!(
        !has_prefix(&links, "selection"),
        "default Legacy mode must not emit a selection artifact: {links:?}"
    );
}

#[test]
fn registry_mode_emits_the_selection_but_no_contradiction_count() {
    let links = solve_with_selection("translate apple to Russian", SelectionMode::Registry);
    assert!(
        has_prefix(&links, "selection"),
        "Registry mode must emit the selection artifact: {links:?}"
    );
    assert!(
        !has_prefix(&links, "selection:contradictions"),
        "the contradiction count is a compare-only summary: {links:?}"
    );
}

#[test]
fn compare_mode_emits_the_selection_and_the_contradiction_count() {
    let links = solve_with_selection(
        "translate apple to Russian and write a hello world program in Python",
        SelectionMode::Compare,
    );
    assert!(has_prefix(&links, "selection"), "{links:?}");
    assert!(
        has_prefix(&links, "selection:contradictions"),
        "Compare mode must surface the contradiction count: {links:?}"
    );
}

#[test]
fn the_knob_changes_only_the_trace_not_the_answer() {
    let prompt = "translate apple to Russian and write a hello world program in Python";
    let legacy = UniversalSolver::new(SolverConfig {
        selection_mode: SelectionMode::Legacy,
        ..SolverConfig::default()
    })
    .solve(prompt);
    let compare = UniversalSolver::new(SolverConfig {
        selection_mode: SelectionMode::Compare,
        ..SolverConfig::default()
    })
    .solve(prompt);
    assert_eq!(
        legacy.answer, compare.answer,
        "the answer must not depend on the selection mode"
    );
    assert_eq!(
        legacy.intent, compare.intent,
        "the intent must not depend on the selection mode"
    );
}
