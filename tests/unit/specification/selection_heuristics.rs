//! Issue #1138 B12, plan 12 leaves 1-4: the heuristics the recursive core uses
//! to choose among candidates are registry *data*, not a `sort_by` at each call
//! site (#491).
//!
//! Correctness first, cost second; no ranking key may depend on a wall clock;
//! and reordering the key dimensions is a `.lino` edit.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::selection_heuristics::{
    ActionCost, CandidateRanker, CandidateScore, HeuristicRole, LeastActionRanker, catalog_from,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn catalog_text() -> String {
    let path = repo_root().join("data/meta/selection-heuristics.lino");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("selection-heuristics.lino readable: {error}"))
}

fn parameters_for(slug: &str) -> Vec<(String, String)> {
    catalog_from(&catalog_text())
        .expect("the heuristic catalog parses")
        .into_iter()
        .find(|heuristic| heuristic.name == slug)
        .unwrap_or_else(|| panic!("`{slug}` is declared in data/meta/selection-heuristics.lino"))
        .parameters
}

/// The portfolio fixture: four candidates whose `(cost_size, cost_steps, index)`
/// order is the order `src/draft_portfolio.rs:335` produces today.
fn portfolio_fixture() -> Vec<CandidateScore> {
    vec![
        CandidateScore {
            candidate_id: "draft_0".to_owned(),
            checks: (3, 3),
            cost: ActionCost {
                steps: 9,
                code_size: 240,
                resource_units: 0,
                leaf_count: 1,
            },
        },
        CandidateScore {
            candidate_id: "draft_1".to_owned(),
            checks: (3, 3),
            cost: ActionCost {
                steps: 4,
                code_size: 120,
                resource_units: 0,
                leaf_count: 1,
            },
        },
        CandidateScore {
            candidate_id: "draft_2".to_owned(),
            checks: (3, 3),
            cost: ActionCost {
                steps: 7,
                code_size: 120,
                resource_units: 0,
                leaf_count: 1,
            },
        },
        CandidateScore {
            candidate_id: "draft_3".to_owned(),
            checks: (3, 3),
            cost: ActionCost {
                steps: 2,
                code_size: 480,
                resource_units: 0,
                leaf_count: 1,
            },
        },
    ]
}

#[test]
fn the_seeded_least_action_ranker_reproduces_the_previous_portfolio_order() {
    // The migration identity: with `key_order = "code_size,steps,candidate_index"`
    // the seeded heuristic must produce exactly `(cost_size, cost_steps, index)`.
    // Only after this passes may a second ranker be seeded (plan 12, Gates).
    let scores = portfolio_fixture();
    let mut expected: Vec<usize> = (0..scores.len()).collect();
    expected.sort_by_key(|index| {
        let score = &scores[*index];
        (score.cost.code_size, score.cost.steps, *index)
    });

    let ranked = LeastActionRanker.rank(&scores, &parameters_for("heuristic_least_action"));
    assert_eq!(
        ranked, expected,
        "the seeded least_action heuristic must reproduce today's portfolio order byte \
         for byte before any behaviour changes"
    );
}

#[test]
fn an_unsatisfying_candidate_never_outranks_a_satisfying_one() {
    // R491-C2: shorter reasoning must retain the full required behaviour.
    // Incomplete work is never a cheaper solution.
    let mut scores = portfolio_fixture();
    scores.push(CandidateScore {
        candidate_id: "draft_incomplete".to_owned(),
        checks: (1, 3),
        cost: ActionCost {
            steps: 1,
            code_size: 10,
            resource_units: 0,
            leaf_count: 1,
        },
    });
    let ranked = LeastActionRanker.rank(&scores, &parameters_for("heuristic_least_action"));
    assert!(
        !ranked.contains(&4),
        "the cheapest candidate fails one of its three declared checks and must not be \
         ranked at all; the caller returns it separately as refuted"
    );
    assert!(
        !scores[4].satisfies(),
        "a candidate with 1 of 3 checks satisfied does not satisfy"
    );
}

#[test]
fn a_zero_zero_check_count_is_not_a_pass() {
    let unmeasured = CandidateScore {
        candidate_id: "draft_unmeasured".to_owned(),
        checks: (0, 0),
        cost: ActionCost::default(),
    };
    assert!(
        !unmeasured.satisfies(),
        "`(0, 0)` means nothing was declared and nothing was observed; it is not a pass \
         (plan 00 section 9 R17: the counts come from Evidence rows)"
    );
}

#[test]
fn no_ranking_key_depends_on_wall_clock() {
    // `ActionCost` carries no time field, and the ranker reads only `ActionCost`.
    let source = fs::read_to_string(repo_root().join("src/selection_heuristics.rs"))
        .expect("selection_heuristics.rs readable");
    let cost_block = source
        .split("pub struct ActionCost {")
        .nth(1)
        .and_then(|tail| tail.split('}').next())
        .expect("ActionCost declares its fields");
    for forbidden in [
        "elapsed",
        "wall_time",
        "duration",
        "millis",
        "Instant",
        "SystemTime",
    ] {
        assert!(
            !cost_block.contains(forbidden),
            "ActionCost names `{forbidden}`; a ranking key that depends on machine speed \
             is not reproducible, and R491-C3's elapsed time is reported beside the \
             ranking, never inside it"
        );
    }

    // The declared key order may only name fields of `ActionCost` plus the index.
    for (key, value) in parameters_for("heuristic_least_action") {
        if key != "key_order" {
            continue;
        }
        for dimension in value.split(',') {
            assert!(
                matches!(
                    dimension.trim(),
                    "steps" | "code_size" | "resource_units" | "leaf_count" | "candidate_index"
                ),
                "key_order names `{dimension}`, which is not a deterministic cost dimension"
            );
        }
    }
}

#[test]
fn reordering_the_key_order_is_a_lino_edit_not_a_rust_edit() {
    let scores = portfolio_fixture();
    let by_size = LeastActionRanker.rank(&scores, &parameters_for("heuristic_least_action"));
    let swapped = vec![(
        "key_order".to_owned(),
        "steps,code_size,candidate_index".to_owned(),
    )];
    let by_steps = LeastActionRanker.rank(&scores, &swapped);

    assert_ne!(
        by_size, by_steps,
        "swapping the two key dimensions must change the order; otherwise the key order \
         is not being read from the seed at all"
    );
    assert_eq!(
        by_steps.first(),
        Some(&3),
        "with steps first, the two-step candidate wins even though it is the largest"
    );
}

#[test]
fn an_empty_heuristic_table_falls_back_deterministically_and_says_so() {
    // `heuristics_for(role, situation)` empty is a reportable state: the core
    // uses the deterministic identity ordering and emits `heuristic:none`
    // naming the role and the situation. It never silently falls back.
    let catalog = catalog_from(&catalog_text()).expect("the heuristic catalog parses");
    assert!(
        catalog
            .iter()
            .any(|heuristic| heuristic.role == HeuristicRole::Experiment),
        "the catalog must declare the experiment role, so an empty result is a fact \
         about the situation rather than about the file"
    );

    let scores = portfolio_fixture();
    let identity = LeastActionRanker.rank(&scores, &[]);
    assert_eq!(
        identity,
        (0..scores.len()).collect::<Vec<usize>>(),
        "with no parameters the ranker must fall back to the deterministic identity \
         ordering rather than inventing a key"
    );
}
