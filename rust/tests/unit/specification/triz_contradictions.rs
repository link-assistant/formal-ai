//! Issue #1138 B12, plan 12 leaves 15-16: TRIZ contradictions as links with a
//! 0-1 selection value (#901).
//!
//! "Each such contradiction or union of different criteria/metrics of trade offs
//! are links in our theory. Where we can assign value from 0 to 1 … by selecting
//! 50% or 10% or 80%." The value is derived from the requirement's own clauses,
//! never guessed, and it is stored as integer basis points so a content id stays
//! exactly comparable and hashable.
//!
//! Also pins R901-3: contradiction resolution is registry-selected (the seeded
//! `heuristic_triz` row at order 3 with `applies_when contradiction_detected`)
//! and the ranking call sites resolve the ranker through `heuristics_for`.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::method_registry::MethodRegistry;
use formal_ai::selection_heuristics::{
    ActionCost, CandidateRanker, CandidateScore, ContradictionDerivation, HeuristicRole,
    LeastActionRanker, TrizRanker, TrizResolution, contradictions_in,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

/// Two candidates that each win on a different cost dimension: the shorter one
/// takes more steps, the faster one is longer. That is a technical
/// contradiction, not a tie to be broken by index.
fn tied_pair() -> Vec<CandidateScore> {
    vec![
        CandidateScore {
            candidate_id: "short_but_slow".to_owned(),
            checks: (4, 4),
            cost: ActionCost {
                steps: 18,
                code_size: 120,
                resource_units: 0,
                leaf_count: 2,
            },
        },
        CandidateScore {
            candidate_id: "long_but_direct".to_owned(),
            checks: (4, 4),
            cost: ActionCost {
                steps: 4,
                code_size: 480,
                resource_units: 0,
                leaf_count: 2,
            },
        },
    ]
}

/// The requirement states the trade-off explicitly and says which way to lean,
/// so the selection value is countable from its clauses.
const REQUIREMENT: &str = "Give me the shortest answer that still covers every case, \
and if you must choose, favour completeness.";

#[test]
fn a_tie_on_the_least_action_key_with_two_winning_dimensions_is_a_contradiction() {
    let found = contradictions_in(&tied_pair(), REQUIREMENT);
    assert_eq!(
        found.len(),
        1,
        "two candidates that each win on a different cost dimension are one technical \
         contradiction, detected rather than resolved by the arbitrary index tie-break"
    );
    let link = &found[0];
    assert_ne!(
        link.criterion_a, link.criterion_b,
        "a contradiction is between two criteria"
    );
    assert!(
        !link.link_id.is_empty(),
        "the contradiction is a link, so it carries a stable id"
    );
}

#[test]
fn the_selection_value_is_derived_from_requirement_clauses_not_guessed() {
    let found = contradictions_in(&tied_pair(), REQUIREMENT);
    let link = &found[0];
    match &link.derivation {
        ContradictionDerivation::RequirementClauses {
            a_clauses,
            b_clauses,
        } => {
            assert!(
                a_clauses + b_clauses > 0,
                "the value must be counted from clauses that actually demand each criterion"
            );
            assert!(
                *b_clauses > *a_clauses,
                "the requirement says `favour completeness`, so completeness carries the \
                 extra clause and the value leans toward it"
            );
        }
        other => panic!("the requirement states the trade-off explicitly, got {other:?}"),
    }
    assert!(
        link.selection_basis_points > 5_000,
        "`favour completeness` moves the value past the midpoint, got {}",
        link.selection_basis_points
    );
    assert!(
        !link.evidence.is_empty(),
        "the requirement spans the value was derived from are recorded with it"
    );
}

#[test]
fn an_underivable_contradiction_is_named_and_left_unresolved() {
    // No clause in this requirement demands either criterion, so there is
    // nothing to count and nothing to invent. There is no default 50 %.
    let found = contradictions_in(&tied_pair(), "Do the thing.");
    let link = found
        .first()
        .expect("the contradiction is still detected; only its value is underivable");
    assert!(
        matches!(link.derivation, ContradictionDerivation::Underivable { .. }),
        "an unstated trade-off must be reported as underivable, got {:?}",
        link.derivation
    );
    assert!(
        matches!(link.resolution, TrizResolution::Unresolved { .. }),
        "an underivable contradiction is named and left unresolved, never silently \
         defaulted to 50 %, got {:?}",
        link.resolution
    );
}

#[test]
fn the_selection_value_is_integer_basis_points_so_ids_stay_hashable() {
    let source = fs::read_to_string(repo_root().join("rust/src/selection_heuristics.rs"))
        .expect("selection_heuristics.rs readable");
    let block = source
        .split("pub struct ContradictionLink {")
        .nth(1)
        .and_then(|tail| tail.split("\n}").next())
        .expect("ContradictionLink declares its fields");
    assert!(
        block.contains("selection_basis_points: u16"),
        "the selection value is integer basis points (0 = fully A, 10000 = fully B); a \
         float in a content id is not exactly comparable"
    );
    for float in ["f32", "f64"] {
        assert!(
            !block.contains(float),
            "ContradictionLink names `{float}`; no float may enter a content id"
        );
    }

    let found = contradictions_in(&tied_pair(), REQUIREMENT);
    assert!(
        found[0].selection_basis_points <= 10_000,
        "basis points run 0..=10000"
    );
}

#[test]
fn the_forty_principles_are_seed_data_and_survive_forget_and_rediscover() {
    // Delete the seed, rediscover it through the sources registry, replay
    // offline, and the content hash matches. Wave T asserts the seed exists and
    // is complete; plan 12 leaf 16 supplies it and the rediscovery path.
    let path = repo_root().join("data/seed/triz-principles.lino");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "plan 12 leaf 16 owes {}: the forty inventive and four separation principles \
             as link data, each with a principle_id, a canonical name, a `resolves` \
             criterion pair where one is known, and a `source` pointing at the article \
             the sources registry already declares ({error})",
            path.display()
        )
    });
    let principles = text.matches("principle_id ").count();
    assert_eq!(
        principles, 44,
        "forty inventive principles plus four separation principles, got {principles}"
    );
    assert!(
        text.contains("source "),
        "each principle names the source it is rediscoverable from, which is what makes \
         the seed forgettable"
    );
}

// ── R901-3: registry-selected ranking ────────────────────────────────────────

/// The candidate pair used to test registry selection: index 0 wins on steps,
/// index 1 wins on `code_size`, so each wins on a different dimension.
fn contradicted_pair_for_registry() -> Vec<CandidateScore> {
    vec![
        CandidateScore {
            candidate_id: "fast_but_large".to_owned(),
            checks: (4, 4),
            cost: ActionCost {
                steps: 5,
                code_size: 200,
                resource_units: 0,
                leaf_count: 1,
            },
        },
        CandidateScore {
            candidate_id: "slow_but_small".to_owned(),
            checks: (4, 4),
            cost: ActionCost {
                steps: 10,
                code_size: 100,
                resource_units: 0,
                leaf_count: 1,
            },
        },
    ]
}

#[test]
fn the_seeded_catalog_declares_a_triz_rank_row_at_order_3() {
    // R901-3: `data/meta/selection-heuristics.lino` must declare a `rank` row
    // with slug `triz`, `order 3`, and `applies_when contradiction_detected`.
    let path = repo_root().join("data/meta/selection-heuristics.lino");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("selection-heuristics.lino readable: {error}"));

    let catalog =
        formal_ai::selection_heuristics::catalog_from(&text).expect("the heuristic catalog parses");

    let triz = catalog
        .iter()
        .find(|h| h.parameter("slug") == Some("triz"))
        .unwrap_or_else(|| panic!("no `triz` slug in data/meta/selection-heuristics.lino"));

    assert_eq!(
        triz.role,
        HeuristicRole::Rank,
        "the triz heuristic must have the `rank` role"
    );
    assert_eq!(triz.order, 3, "the triz heuristic must be at order 3");
    assert!(
        triz.applies_when
            .iter()
            .any(|s| s == "contradiction_detected"),
        "the triz heuristic must declare `applies_when contradiction_detected`; \
         got {:?}",
        triz.applies_when
    );
}

#[test]
fn registry_selects_triz_when_contradiction_detected() {
    // R901-3: `heuristics_for(HeuristicRole::Rank, "contradiction_detected")`
    // must include `heuristic_triz` and that must be the most-specific one
    // (highest order) returned.
    let registry = MethodRegistry::shared();
    let heuristics = registry.heuristics_for(HeuristicRole::Rank, "contradiction_detected");
    let last = heuristics
        .last()
        .expect("at least one rank heuristic applies in contradiction_detected");

    assert_eq!(
        last.parameter("slug"),
        Some("triz"),
        "the most-specific rank heuristic for `contradiction_detected` must be \
         `triz`; got {:?}",
        last.parameter("slug")
    );
}

#[test]
fn registry_keeps_least_action_when_no_contradiction() {
    // R901-3: without a contradiction the situation is empty, and the registry
    // must still select the default `least_action` ranker, so every other
    // situation keeps today's behaviour exactly.
    let registry = MethodRegistry::shared();
    let heuristics = registry.heuristics_for(HeuristicRole::Rank, "");
    let last = heuristics
        .last()
        .expect("at least one rank heuristic applies with empty situation");

    assert_eq!(
        last.parameter("slug"),
        Some("least_action"),
        "with no contradiction the most-specific rank heuristic must be \
         `least_action`; got {:?}",
        last.parameter("slug")
    );
}

#[test]
fn triz_ranker_and_least_action_ranker_disagree_on_the_contradicted_pair() {
    // R901-3: TrizRanker must produce a different ranking than LeastActionRanker
    // for the contradicted pair (fast_but_large vs slow_but_small), demonstrating
    // that selecting TrizRanker through the registry changes the outcome.
    //
    // fast_but_large (index 0): code_size=200, steps=5
    // slow_but_small (index 1): code_size=100, steps=10
    //
    // LeastActionRanker (key_order code_size,...): slow_but_small (100) first.
    // TrizRanker (no key_order in heuristic_triz params): identity order → fast_but_large first.
    let scores = contradicted_pair_for_registry();

    // LeastActionRanker uses heuristic_least_action's parameters.
    let registry = MethodRegistry::shared();
    let least_action_params = registry
        .heuristics_for(HeuristicRole::Rank, "")
        .last()
        .map(|h| h.parameters.clone())
        .unwrap_or_default();
    let la_ranked = LeastActionRanker.rank(&scores, &least_action_params);

    // TrizRanker uses heuristic_triz's parameters.
    let triz_params = registry
        .heuristics_for(HeuristicRole::Rank, "contradiction_detected")
        .last()
        .map(|h| h.parameters.clone())
        .unwrap_or_default();
    let triz_ranked = TrizRanker.rank(&scores, &triz_params);

    assert_ne!(
        la_ranked, triz_ranked,
        "TrizRanker and LeastActionRanker must disagree on the contradicted pair, \
         proving the registry selection changes the outcome"
    );
    assert_eq!(
        la_ranked.first().copied(),
        Some(1),
        "LeastActionRanker puts slow_but_small (smaller code_size) first"
    );
    assert_eq!(
        triz_ranked.first().copied(),
        Some(0),
        "TrizRanker (identity, no key_order) puts fast_but_large (index 0) first"
    );
}
