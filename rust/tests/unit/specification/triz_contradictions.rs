//! Issue #1138 B12, plan 12 leaves 15-16: TRIZ contradictions as links with a
//! 0-1 selection value (#901).
//!
//! "Each such contradiction or union of different criteria/metrics of trade offs
//! are links in our theory. Where we can assign value from 0 to 1 … by selecting
//! 50% or 10% or 80%." The value is derived from the requirement's own clauses,
//! never guessed, and it is stored as integer basis points so a content id stays
//! exactly comparable and hashable.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::selection_heuristics::{
    ActionCost, CandidateScore, ContradictionDerivation, TrizResolution, contradictions_in,
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
