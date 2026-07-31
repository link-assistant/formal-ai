//! The arithmetic-reachability leaf of the candidate-solution portfolio
//! (issue #704).
//!
//! [`crate::draft_portfolio`] owns everything that is *not* arithmetic: planning
//! `k` slots, seeding them, running them concurrently, merging in draft order,
//! ranking by least action, backtracking on composition failure, and recording
//! the comparison. This module only answers the four questions the engine asks a
//! leaf: which strategies this instance can run, how to draft under one of them,
//! how to test a draft, and whether a draft composes.
//!
//! Every strategy here is a genuinely different generator, not a placeholder:
//! `reuse` replays the composition the impulse itself suggests, `rule_derivation`
//! derives one greedily from the target, `oracle_lookup` tabulates every
//! reachable value when the instance is small enough to tabulate,
//! `search` is the budget-driven random/evolutionary search of issue #662, and
//! `program_synthesis` enumerates the canonical composition space exhaustively.
//! They disagree on real problems, which is what makes the recorded comparison
//! evidence rather than decoration.

use super::{run_search, seed_from_prompt, Candidate, Op, SearchProblem, SearchSolution};
use crate::draft_portfolio::{run_portfolio, DraftArtifact, DraftPlan, PortfolioLeaf};
use crate::event_log::EventLog;
use crate::solver::SolverConfig;
use std::collections::BTreeMap;

/// Largest instance an exhaustive oracle table is allowed to cover. Beyond this
/// the table is not "a lookup" any more, so the strategy honestly declines.
const ORACLE_MAX_OPERANDS: usize = 2;

/// Cap on the compositions bounded program synthesis may enumerate before it
/// gives up, so the most general strategy still terminates in bounded work.
const SYNTHESIS_MAX_COMPOSITIONS: u32 = 200_000;

/// The generated tests of step 6, in declaration order.
const TEST_COUNT: usize = 3;

/// One arithmetic-reachability instance, viewed as a portfolio leaf.
struct ReachabilityLeaf<'a> {
    problem: &'a SearchProblem,
    budget: u32,
}

impl PortfolioLeaf for ReachabilityLeaf<'_> {
    type Artifact = SearchSolution;

    fn supports(&self, strategy: &str) -> bool {
        if self.problem.numbers.is_empty() || self.problem.ops.is_empty() {
            return false;
        }
        match strategy {
            "reuse" | "rule_derivation" | "search" | "program_synthesis" => true,
            // Applicability is a property of the instance, not of the leaf: a
            // table is only a table while it fits.
            "oracle_lookup" => self.problem.numbers.len() <= ORACLE_MAX_OPERANDS,
            _ => false,
        }
    }

    fn draft(&self, plan: &DraftPlan) -> Option<DraftArtifact<Self::Artifact>> {
        match plan.strategy.as_str() {
            "reuse" => self.draft_reuse(),
            "rule_derivation" => self.draft_rule_derivation(),
            "oracle_lookup" => self.draft_oracle_lookup(),
            "search" => self.draft_search(plan.seed),
            "program_synthesis" => self.draft_program_synthesis(),
            _ => None,
        }
    }

    fn run_tests(&self, artifact: &Self::Artifact) -> Vec<bool> {
        run_generated_tests(artifact, self.problem).to_vec()
    }

    fn test_count(&self) -> usize {
        TEST_COUNT
    }

    fn composes(&self, artifact: &Self::Artifact) -> bool {
        run_generated_tests(artifact, self.problem)
            .into_iter()
            .all(|passed| passed)
    }
}

impl ReachabilityLeaf<'_> {
    /// Replay the composition the impulse already names: the operands in the
    /// order they were given, joined by the first allowed operator. The cheapest
    /// possible draft, and on a genuinely reusable instance the correct one.
    fn draft_reuse(&self) -> Option<DraftArtifact<SearchSolution>> {
        let first = *self.problem.ops.first()?;
        let candidate = Candidate {
            order: (0..self.problem.numbers.len()).collect(),
            ops: vec![first; self.problem.numbers.len().saturating_sub(1)],
        };
        Some(self.artifact(candidate, 1))
    }

    /// Derive the composition from a rule instead of guessing: keep the operands
    /// in their given order and, at each step, take the operator whose result is
    /// closest to the target. Deterministic, linear, and right whenever the rule
    /// "move towards the target" happens to hold.
    fn draft_rule_derivation(&self) -> Option<DraftArtifact<SearchSolution>> {
        let numbers = &self.problem.numbers;
        let mut acc = *numbers.first()?;
        let mut ops = Vec::with_capacity(numbers.len().saturating_sub(1));
        let mut steps = 0;
        for operand in numbers.iter().skip(1) {
            let mut best: Option<(Op, i64, i64)> = None;
            for op in &self.problem.ops {
                steps += 1;
                let Some(value) = op.apply(acc, *operand) else {
                    continue;
                };
                let distance = value.saturating_sub(self.problem.target).abs();
                if best.is_none_or(|(_, _, current)| distance < current) {
                    best = Some((*op, value, distance));
                }
            }
            let (op, value, _) = best?;
            ops.push(op);
            acc = value;
        }
        let candidate = Candidate {
            order: (0..numbers.len()).collect(),
            ops,
        };
        Some(self.artifact(candidate, steps))
    }

    /// Tabulate every value the instance can reach, then look the target up.
    /// A lookup, not a scan: the table is built once and keyed by value, which
    /// is only affordable while the instance stays small.
    fn draft_oracle_lookup(&self) -> Option<DraftArtifact<SearchSolution>> {
        let mut table: BTreeMap<i64, Candidate> = BTreeMap::new();
        let mut steps = 0;
        for_each_composition(self.problem, SYNTHESIS_MAX_COMPOSITIONS, |candidate| {
            steps += 1;
            if let Some(value) = candidate.evaluate(&self.problem.numbers) {
                table.entry(value).or_insert_with(|| candidate.clone());
            }
            true
        });
        let candidate = table.get(&self.problem.target)?.clone();
        Some(self.artifact(candidate, steps))
    }

    /// The budget-driven random and evolutionary search of issue #662, seeded
    /// from this slot's seed so sibling slots explore different paths.
    fn draft_search(&self, seed: u64) -> Option<DraftArtifact<SearchSolution>> {
        let mut trace = EventLog::new();
        let solution = run_search(seed, &mut trace, self.problem, self.budget)?;
        let cost_size = solution.expression.chars().count();
        Some(DraftArtifact {
            cost_steps: solution.evaluations,
            cost_size,
            value: solution,
            trace,
        })
    }

    /// Bounded exhaustive enumeration of the canonical composition space: every
    /// operand ordering crossed with every operator assignment, in a fixed order,
    /// stopping at the first exact hit. The most general strategy and the most
    /// expensive one, which is why the seed catalog places it last.
    fn draft_program_synthesis(&self) -> Option<DraftArtifact<SearchSolution>> {
        let mut found: Option<Candidate> = None;
        let mut steps = 0;
        for_each_composition(self.problem, SYNTHESIS_MAX_COMPOSITIONS, |candidate| {
            steps += 1;
            if candidate.evaluate(&self.problem.numbers) == Some(self.problem.target) {
                found = Some(candidate.clone());
                return false;
            }
            true
        });
        Some(self.artifact(found?, steps))
    }

    fn artifact(&self, candidate: Candidate, cost_steps: u32) -> DraftArtifact<SearchSolution> {
        let expression = candidate.render(&self.problem.numbers);
        let cost_size = expression.chars().count();
        DraftArtifact {
            value: SearchSolution {
                expression,
                evaluations: cost_steps,
                candidate,
            },
            cost_steps,
            cost_size,
            trace: EventLog::new(),
        }
    }
}

/// Visit the canonical composition space in a fixed order — operand orderings in
/// lexicographic order, operator assignments as an odometer over the allowed
/// operators — until `visit` returns `false` or `cap` compositions are visited.
///
/// The order is what makes exhaustive strategies reproducible: the same instance
/// always yields the same first hit, on any machine and any thread.
fn for_each_composition<F>(problem: &SearchProblem, cap: u32, mut visit: F)
where
    F: FnMut(&Candidate) -> bool,
{
    let slots = problem.numbers.len().saturating_sub(1);
    let mut order: Vec<usize> = (0..problem.numbers.len()).collect();
    let mut visited: u32 = 0;
    loop {
        let mut assignment = vec![0_usize; slots];
        loop {
            let candidate = Candidate {
                order: order.clone(),
                ops: assignment.iter().map(|slot| problem.ops[*slot]).collect(),
            };
            visited += 1;
            if !visit(&candidate) || visited >= cap {
                return;
            }
            if !advance_assignment(&mut assignment, problem.ops.len()) {
                break;
            }
        }
        if !next_permutation(&mut order) {
            return;
        }
    }
}

/// Odometer step over operator assignments; `false` once every assignment has
/// been produced.
fn advance_assignment(assignment: &mut [usize], radix: usize) -> bool {
    for slot in assignment.iter_mut().rev() {
        *slot += 1;
        if *slot < radix {
            return true;
        }
        *slot = 0;
    }
    false
}

/// Next lexicographic permutation; `false` once the ordering is descending.
fn next_permutation(order: &mut [usize]) -> bool {
    let Some(pivot) = order.windows(2).rposition(|pair| pair[0] < pair[1]) else {
        return false;
    };
    let successor = order
        .iter()
        .rposition(|value| *value > order[pivot])
        .unwrap_or(pivot);
    order.swap(pivot, successor);
    order[pivot + 1..].reverse();
    true
}

/// The generated tests of step 6: the draft must use every operand exactly once,
/// use only the allowed operators, and actually reach the target.
fn run_generated_tests(solution: &SearchSolution, problem: &SearchProblem) -> [bool; TEST_COUNT] {
    let mut used = solution.candidate.order.clone();
    used.sort_unstable();
    [
        used == (0..problem.numbers.len()).collect::<Vec<_>>(),
        solution
            .candidate
            .ops
            .iter()
            .all(|candidate_op| problem.ops.contains(candidate_op)),
        solution.candidate.evaluate(&problem.numbers) == Some(problem.target),
    ]
}

/// Run the candidate-solution portfolio for one recognized reachability problem.
pub(super) fn run_draft_portfolio(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    config: SolverConfig,
) -> (Option<SearchSolution>, Option<String>) {
    let leaf = ReachabilityLeaf {
        problem,
        budget: config.compute_budget,
    };
    let selection = run_portfolio(
        &leaf,
        seed_from_prompt(prompt),
        usize::from(config.draft_count),
        log,
    );
    (selection.winner, selection.comparison_artifact)
}
