//! The principle of least action for task splitting and solution choice
//! (issue #491).
//!
//! Once reasoning steps are real, the next optimization is to spend fewer
//! of them. This module is the scoring half of that: work is planned as a
//! *balanced binary tree of subtasks* — on the highest level of
//! abstraction there is always 1, 2, 4, 8, … tasks, so the abstraction
//! ladder stays regular even when individual tasks near the bottom are
//! already atomic with known solutions. The thing actually optimized for
//! is the **total number of smallest subtasks** (and the steps they
//! cost): when several solution paths are generated, the least-action
//! one — shortest path/steps, then shortest code, then least compute,
//! then least memory — is the one to keep.
//!
//! Two rules keep the scoring honest:
//!
//! 1. A candidate that does not solve the *entire range of inputs* is
//!    not a shorter solution; it is no solution. Failing candidates are
//!    excluded before any cost comparison — "shortest code that still
//!    solves the entire range" only ever ranks solutions.
//! 2. Unhandled paths (subtasks with no known solution) are listed, not
//!    costed: the plan never invents a step count for work it cannot do
//!    yet. That list is the auto-improvement surface the issue asks for.
//!
//! Benchmarks and evaluation run on time, computation resources, and
//! memory ([`ActionCost`]); user satisfaction is maximized *within*
//! least resources, so the ordering is lexicographic: steps first, then
//! code size, then compute time, then memory — a path that saves a
//! millisecond by adding a reasoning step loses.
//!
//! No Cargo changes; std only, and the module is pure (no I/O), so both
//! the engine and the WASM build can consume it.

/// The measured cost of one solution path: the benchmark dimensions of
/// issue #491 — steps, code size, compute time, memory.
///
/// Lexicographic order: fewer steps wins first; ties break on code
/// units, then compute milliseconds, then memory kilobytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionCost {
    /// Reasoning steps / path length — the primary least-action measure.
    pub steps: u32,
    /// Code size in statements (or bytes/10) — "shortest code that
    /// still solves the entire range of inputs".
    pub code_units: u32,
    /// Wall or CPU milliseconds.
    pub compute_ms: u64,
    /// Peak memory in kilobytes.
    pub memory_kb: u64,
}

impl ActionCost {
    /// The zero cost (an already-solved path costs nothing to re-derive).
    pub const ZERO: Self = Self {
        steps: 0,
        code_units: 0,
        compute_ms: 0,
        memory_kb: 0,
    };

    /// Lexicographic least-action comparison: `self < other` when self
    /// acts less — fewer steps, or equal steps and less code, and so on.
    #[must_use]
    pub fn is_less_action_than(&self, other: &Self) -> bool {
        (self.steps, self.code_units, self.compute_ms, self.memory_kb)
            < (
                other.steps,
                other.code_units,
                other.compute_ms,
                other.memory_kb,
            )
    }
}

/// One smallest subtask: atomic (a known solution exists, with its step
/// count) or unhandled (no solution yet — listed, never costed).
#[derive(Clone, Debug)]
pub struct Subtask {
    pub title: String,
    /// Whether a known solution exists for this subtask.
    pub atomic: bool,
    /// The reasoning steps of the known solution (ignored when not
    /// atomic; the plan does not invent costs for open work).
    pub solution_steps: u32,
}

/// A balanced-binary work plan over the smallest subtasks.
///
/// The abstraction ladder is regular — 1, 2, 4, 8, … tasks per level —
/// with the deepest level holding the concrete subtasks (some already
/// atomic, as the issue notes). What the plan optimizes for is
/// [`WorkPlan::smallest_subtasks`] and [`WorkPlan::planned_steps`].
#[derive(Clone, Debug)]
pub struct WorkPlan {
    /// Tasks at each level of abstraction, top first: 1, 2, 4, … with
    /// the final level holding the actual subtask count.
    pub level_counts: Vec<usize>,
    /// The total number of smallest subtasks — the optimization target.
    pub smallest_subtasks: usize,
    /// Subtasks that already have known solutions.
    pub solved: usize,
    /// Subtasks with no known solution yet: the unhandled paths.
    pub unhandled: Vec<String>,
    /// The steps of the known solutions that the plan would take
    /// (unhandled work contributes nothing — it has no invented cost).
    pub planned_steps: u32,
    /// The deepest level of abstraction (0 = the root task alone).
    pub depth: usize,
}

/// Plan the balanced binary split for one root task and its smallest
/// subtasks.
///
/// ```text
/// plan("ship", [a(atomic, 2), b(atomic, 3), c(open), d(open), e(open)])
///   .level_counts == [1, 2, 4, 5]
///   .smallest_subtasks == 5
///   .solved == 2, .unhandled == [c, d, e], .planned_steps == 5
/// ```
///
/// A subtask list of exactly 2^k entries reproduces the issue's 1, 2, 4,
/// 8 ladder; any other count keeps the ladder regular down to 2^k and
/// holds the remainder at the final level, so the highest abstraction is
/// always the powers of two.
#[must_use]
pub fn plan(subtasks: &[Subtask]) -> WorkPlan {
    let mut level_counts = Vec::new();
    let mut level = 1usize;
    while level < subtasks.len() {
        level_counts.push(level);
        level *= 2;
    }
    // The final level holds the concrete subtasks (1, 2, 4, 8, or a
    // count between powers of two).
    level_counts.push(subtasks.len().max(1));
    let solved = subtasks.iter().filter(|task| task.atomic).count();
    WorkPlan {
        depth: level_counts.len().saturating_sub(1),
        level_counts,
        smallest_subtasks: subtasks.len(),
        solved,
        unhandled: subtasks
            .iter()
            .filter(|task| !task.atomic)
            .map(|task| task.title.clone())
            .collect(),
        planned_steps: subtasks
            .iter()
            .filter(|task| task.atomic)
            .map(|task| task.solution_steps)
            .sum(),
    }
}

/// Compare two plans of the same root task: the better plan needs fewer
/// smallest subtasks, or equal subtasks and fewer planned steps.
///
/// `None` when the plans cover different subtask counts (comparing a
/// plan against a coarser plan is not a least-action question).
#[must_use]
pub fn least_action_plan<'a>(a: &'a WorkPlan, b: &'a WorkPlan) -> Option<&'a WorkPlan> {
    if a.smallest_subtasks != b.smallest_subtasks {
        return None;
    }
    // Fewer planned steps wins; on a tie, fewer unhandled paths wins
    // (more of the work is known).
    let a_cost = (a.planned_steps, a.unhandled.len());
    let b_cost = (b.planned_steps, b.unhandled.len());
    match a_cost.cmp(&b_cost) {
        core::cmp::Ordering::Greater => Some(b),
        core::cmp::Ordering::Less | core::cmp::Ordering::Equal => Some(a),
    }
}

/// One generated solution for a task, with its measured cost.
#[derive(Clone, Debug)]
pub struct SolutionCandidate {
    pub id: String,
    /// Whether it solves the entire range of inputs. `false` candidates
    /// are never ranked, however short they are.
    pub solves_entire_range: bool,
    pub cost: ActionCost,
}

/// Rank generated solutions by least action: only candidates that solve
/// the entire input range, best (least action) first. Lexicographic on
/// (steps, code units, compute, memory), stable for equal costs.
#[must_use]
pub fn rank_by_least_action(candidates: &[SolutionCandidate]) -> Vec<&SolutionCandidate> {
    let mut viable: Vec<&SolutionCandidate> = candidates
        .iter()
        .filter(|candidate| candidate.solves_entire_range)
        .collect();
    viable.sort_by(|a, b| {
        let ac = (
            a.cost.steps,
            a.cost.code_units,
            a.cost.compute_ms,
            a.cost.memory_kb,
        );
        let bc = (
            b.cost.steps,
            b.cost.code_units,
            b.cost.compute_ms,
            b.cost.memory_kb,
        );
        ac.cmp(&bc).then_with(|| a.id.cmp(&b.id))
    });
    viable
}

/// The least-action solution, if any candidate solves the entire range.
#[must_use]
pub fn least_action_solution(candidates: &[SolutionCandidate]) -> Option<&SolutionCandidate> {
    rank_by_least_action(candidates).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subtask(title: &str, atomic: bool, steps: u32) -> Subtask {
        Subtask {
            title: title.to_owned(),
            atomic,
            solution_steps: steps,
        }
    }

    #[test]
    fn the_ladder_is_always_one_two_four_eight() {
        let eight: Vec<Subtask> = (0..8).map(|i| subtask(&format!("t{i}"), true, 1)).collect();
        assert_eq!(plan(&eight).level_counts, vec![1, 2, 4, 8]);
        let one = vec![subtask("root", true, 1)];
        assert_eq!(plan(&one).level_counts, vec![1]);
        let five: Vec<Subtask> = (0..5).map(|i| subtask(&format!("t{i}"), true, 1)).collect();
        assert_eq!(plan(&five).level_counts, vec![1, 2, 4, 5]);
    }

    #[test]
    fn a_shorter_solution_that_fails_inputs_is_not_least_action() {
        let candidates = vec![
            SolutionCandidate {
                id: "short_but_wrong".into(),
                solves_entire_range: false,
                cost: ActionCost {
                    steps: 1,
                    ..ActionCost::ZERO
                },
            },
            SolutionCandidate {
                id: "correct".into(),
                solves_entire_range: true,
                cost: ActionCost {
                    steps: 6,
                    ..ActionCost::ZERO
                },
            },
        ];
        let best = least_action_solution(&candidates).unwrap();
        assert_eq!(best.id, "correct");
        assert_eq!(rank_by_least_action(&candidates).len(), 1);
    }
}
