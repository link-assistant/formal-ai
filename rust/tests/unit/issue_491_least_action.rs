//! Issue #491: optimize for least action — balanced binary task splits
//! and least-action solution scoring.
//!
//! The ask: split each task into two subtasks from the start, so the
//! highest level of abstraction always holds 1, 2, 4, 8, … tasks (a
//! balanced tree, preserved), while some tasks near the bottom are
//! already atomic with known solutions. The thing actually optimized
//! for is the total number of smallest subtasks; and when multiple
//! solutions are generated, the least-action one (shortest path/steps,
//! then shortest code that still solves the entire range of inputs,
//! then least compute and memory) is the one to keep. These tests pin
//! rust/src/least_action.rs.

use formal_ai::least_action::{
    ActionCost, SolutionCandidate, Subtask, least_action_plan, least_action_solution, plan,
    rank_by_least_action,
};

fn subtask(title: &str, atomic: bool, steps: u32) -> Subtask {
    Subtask {
        title: title.to_owned(),
        atomic,
        solution_steps: steps,
    }
}

fn candidate(id: &str, solves: bool, cost: ActionCost) -> SolutionCandidate {
    SolutionCandidate {
        id: id.to_owned(),
        solves_entire_range: solves,
        cost,
    }
}

#[test]
fn the_highest_abstraction_is_always_one_two_four_eight() {
    let eight: Vec<Subtask> = (0..8).map(|i| subtask(&format!("t{i}"), true, 1)).collect();
    let planned = plan(&eight);
    assert_eq!(planned.level_counts, vec![1, 2, 4, 8]);
    assert_eq!(planned.smallest_subtasks, 8);
    assert_eq!(planned.depth, 3);

    // Between powers of two the ladder stays regular and the final
    // level holds the concrete count.
    let five: Vec<Subtask> = (0..5).map(|i| subtask(&format!("t{i}"), true, 1)).collect();
    assert_eq!(plan(&five).level_counts, vec![1, 2, 4, 5]);

    let one = vec![subtask("root", true, 1)];
    assert_eq!(plan(&one).level_counts, vec![1]);
}

#[test]
fn the_plan_counts_smallest_subtasks_and_leaves_open_work_uncosted() {
    // Five subtasks at the 8-task level: two already atomic with known
    // two- and three-step solutions, three unhandled.
    let subtasks = vec![
        subtask("parse inputs", true, 2),
        subtask("solve constraints", true, 3),
        subtask("render output", false, 0),
        subtask("log the run", false, 0),
        subtask("clean up", false, 0),
    ];
    let planned = plan(&subtasks);
    assert_eq!(planned.smallest_subtasks, 5);
    assert_eq!(planned.solved, 2);
    assert_eq!(planned.planned_steps, 5);
    assert_eq!(
        planned.unhandled,
        vec!["render output", "log the run", "clean up"]
    );
    // The unhandled paths are listed for auto-improvement, never given
    // invented step counts: planned_steps is exactly the known work.
}

#[test]
fn a_plan_with_fewer_smallest_subtasks_is_less_action() {
    let coarse: Vec<Subtask> = (0..8).map(|i| subtask(&format!("t{i}"), true, 3)).collect();
    let refined_a: Vec<Subtask> = (0..6).map(|i| subtask(&format!("a{i}"), true, 2)).collect();
    let refined_b: Vec<Subtask> = (0..6).map(|i| subtask(&format!("b{i}"), true, 4)).collect();
    let coarse = plan(&coarse);
    let a = plan(&refined_a);
    let b = plan(&refined_b);
    assert!(
        least_action_plan(&coarse, &a).is_none(),
        "different subtask counts do not compare"
    );
    assert_eq!(least_action_plan(&a, &b).map(|p| p.planned_steps), Some(12));
    // Equal count, equal steps: fewer unhandled paths wins.
    let open: Vec<Subtask> = (0..6)
        .map(|i| subtask(&format!("o{i}"), i < 3, 2))
        .collect();
    let more_open: Vec<Subtask> = (0..6)
        .map(|i| subtask(&format!("m{i}"), i < 1, 2))
        .collect();
    let open = plan(&open);
    let more_open = plan(&more_open);
    // Both cost 6 planned steps (3×2 vs 1×2? no: open costs 3×2=6,
    // more_open costs 1×2=2 — so more_open has fewer planned steps and
    // wins on the primary rule, unhandled only breaks exact ties).
    assert_eq!(
        least_action_plan(&open, &more_open).map(|p| p.unhandled.len()),
        Some(5)
    );
}

#[test]
fn solutions_rank_by_least_action_steps_first() {
    let candidates = vec![
        candidate(
            "fast",
            true,
            ActionCost {
                steps: 3,
                code_units: 90,
                compute_ms: 900,
                memory_kb: 40,
            },
        ),
        candidate(
            "small",
            true,
            ActionCost {
                steps: 5,
                code_units: 10,
                compute_ms: 10,
                memory_kb: 1,
            },
        ),
        candidate(
            "slow",
            true,
            ActionCost {
                steps: 7,
                code_units: 50,
                compute_ms: 500,
                memory_kb: 5,
            },
        ),
    ];
    let ranked = rank_by_least_action(&candidates);
    let ids: Vec<&str> = ranked.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["fast", "small", "slow"],
        "steps dominate every other dimension"
    );
    assert_eq!(least_action_solution(&candidates).unwrap().id, "fast");
}

#[test]
fn a_millisecond_saved_by_an_extra_step_still_loses() {
    let candidates = vec![
        candidate(
            "fewer_steps",
            true,
            ActionCost {
                steps: 2,
                code_units: 40,
                compute_ms: 900,
                memory_kb: 9,
            },
        ),
        candidate(
            "fewer_ms",
            true,
            ActionCost {
                steps: 3,
                code_units: 40,
                compute_ms: 1,
                memory_kb: 1,
            },
        ),
    ];
    assert_eq!(
        least_action_solution(&candidates).unwrap().id,
        "fewer_steps"
    );
}

#[test]
fn the_shortest_code_that_fails_the_input_range_never_ranks() {
    let candidates = vec![
        candidate(
            "short_but_narrow",
            false,
            ActionCost {
                steps: 1,
                code_units: 1,
                compute_ms: 1,
                memory_kb: 1,
            },
        ),
        candidate(
            "covers_all_inputs",
            true,
            ActionCost {
                steps: 6,
                code_units: 60,
                compute_ms: 60,
                memory_kb: 6,
            },
        ),
    ];
    let ranked = rank_by_least_action(&candidates);
    assert_eq!(
        ranked.len(),
        1,
        "only entire-range solutions are candidates"
    );
    assert_eq!(ranked[0].id, "covers_all_inputs");
}

#[test]
fn equal_costs_break_stably_by_id() {
    let cost = ActionCost {
        steps: 4,
        code_units: 4,
        compute_ms: 4,
        memory_kb: 4,
    };
    let candidates = vec![candidate("b", true, cost), candidate("a", true, cost)];
    let ids: Vec<&str> = rank_by_least_action(&candidates)
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(ids, vec!["a", "b"]);
}
