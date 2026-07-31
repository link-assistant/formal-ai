//! Deterministic parallel candidate-solution portfolios (issue #704).
//!
//! The Universal Problem Solving Algorithm spawns *multiple independent drafts*
//! for a work-unit leaf, lets the generated tests of step 6 select among them,
//! and composes the survivors hierarchically. This module is the
//! domain-independent engine for that axis: it knows how to plan `k` draft
//! slots, seed each one reproducibly, evaluate them concurrently, merge the
//! results in draft-index order, rank the passing drafts by least action, and
//! record the comparison that explains the winner.
//!
//! Nothing here knows what a draft *is*. A solver leaf implements
//! [`PortfolioLeaf`] to say which strategies it can actually run, how to draft
//! under one strategy and seed, how to run its generated tests against a draft,
//! and whether a draft composes with the rest of the solution. Arithmetic
//! reachability (`crate::solver_search`) and rule synthesis
//! (`crate::rule_synthesis`) both plug in through that trait, so the portfolio
//! is a property of the meta algorithm rather than of one handler.
//!
//! Determinism (`VISION.md` contract): concurrency is an execution detail and
//! never an answer-changing one. Every seed is derived from the impulse hash,
//! the draft index, and the attempt number; results are sorted back into draft
//! order before selection, so the recorded comparison and the selected answer
//! are identical whether the drafts ran on one thread or on many.
//!
//! The strategy catalog itself is data, not code: the ordered strategy slugs
//! come from `data/seed/draft-strategies.lino`, so a new draft generator is
//! introduced by declaring a row there, not by editing a match arm here
//! (issue #386, R379).

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::seed;

/// Bounded retry budget for one draft slot.
///
/// The methodology's learning-from-failure pattern retries a failed attempt at
/// most three times, recording what was learned each time. The bound keeps the
/// portfolio's cost linear in `draft_count` and its trace finite.
pub const MAX_ATTEMPTS: u32 = 3;

/// One planned draft slot: which strategy generates it, with which seed, on
/// which attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftPlan {
    /// Position in the portfolio. The merge order, and the final tie-break.
    pub index: usize,
    /// Meaning slug of the generating strategy (`reuse`, `search`, …).
    pub strategy: String,
    /// Deterministic seed for this slot and attempt.
    pub seed: u64,
    /// 1-based attempt number, bounded by [`MAX_ATTEMPTS`].
    pub attempt: u32,
}

/// A generated draft plus the time-independent cost metrics selection ranks it
/// by.
///
/// `cost_steps` counts the work the strategy spent (evaluations, expansions);
/// `cost_size` measures the produced artifact (its rendered length). Both are
/// wall-clock independent so ranking cannot depend on machine speed.
pub struct DraftArtifact<A> {
    pub value: A,
    pub cost_steps: u32,
    pub cost_size: usize,
    /// Events the strategy recorded while drafting. Only the winner's trace is
    /// merged into the shared log, so a losing draft never pollutes the
    /// evidence of the answer that was actually given.
    pub trace: EventLog,
}

/// A solver leaf that can be solved by more than one strategy.
pub trait PortfolioLeaf: Sync {
    /// What one draft produces.
    type Artifact: Send + Clone;

    /// Can this leaf actually run `strategy`? Strategies the leaf does not
    /// support are skipped when the portfolio is planned, so a slot is never
    /// spent on a generator that is guaranteed to produce nothing.
    fn supports(&self, strategy: &str) -> bool;

    /// Draft a candidate under `plan`, or `None` when this strategy has nothing
    /// to offer for this problem instance.
    fn draft(&self, plan: &DraftPlan) -> Option<DraftArtifact<Self::Artifact>>;

    /// Run the generated tests (step 6 of the loop) against one draft, in
    /// declaration order. The portfolio treats an all-`true` vector as a pass.
    fn run_tests(&self, artifact: &Self::Artifact) -> Vec<bool>;

    /// How many generated tests exist. Reported for drafts that produced no
    /// artifact at all, so the trace never shows a misleading `0/0`.
    fn test_count(&self) -> usize;

    /// Does this draft compose with the rest of the solution? A draft that
    /// passes its own tests but fails composition is *backtracked*: selection
    /// moves on to the next-best passing draft.
    fn composes(&self, artifact: &Self::Artifact) -> bool;
}

/// What the portfolio decided, and the record that explains it.
pub struct PortfolioSelection<A> {
    /// The selected draft, if any draft passed its tests and composed.
    pub winner: Option<A>,
    /// The `draft_comparison` record, re-tagged as a recoverable artifact so a
    /// later "why did you pick that solution?" turn can read it back.
    pub comparison_artifact: Option<String>,
}

/// One evaluated draft slot after its bounded retries.
struct DraftEvaluation<A> {
    index: usize,
    strategy: String,
    seed: u64,
    attempts: u32,
    passed_tests: usize,
    total_tests: usize,
    cost_steps: u32,
    cost_size: usize,
    composition_passed: bool,
    artifact: Option<A>,
    trace: EventLog,
}

impl<A> DraftEvaluation<A> {
    const fn passed(&self) -> bool {
        self.total_tests > 0 && self.passed_tests == self.total_tests
    }
}

/// The ordered strategy catalog, read from `data/seed/draft-strategies.lino`
/// via [`seed::draft_strategies`].
///
/// Declaration order in the seed is the portfolio's preference order, so the
/// cheapest and most reusable generators occupy the low draft indices and the
/// expensive ones only run when the budget allows more slots.
#[must_use]
pub fn strategy_catalog() -> Vec<String> {
    seed::draft_strategies()
}

/// Plan up to `draft_count` slots for `leaf`: the seed-declared strategies the
/// leaf supports, in catalog order, each seeded from `impulse_seed` and its
/// index.
///
/// When the leaf supports fewer strategies than the requested count, the
/// remaining slots repeat the last supported strategy with different seeds —
/// independent restarts of the same generator are themselves a portfolio, and
/// repeating the *most general* strategy is the deterministic way to spend a
/// larger budget.
#[must_use]
pub fn plan_drafts<L: PortfolioLeaf>(
    leaf: &L,
    impulse_seed: u64,
    draft_count: usize,
) -> Vec<DraftPlan> {
    let supported = strategy_catalog()
        .into_iter()
        .filter(|strategy| leaf.supports(strategy))
        .collect::<Vec<_>>();
    if supported.is_empty() {
        return Vec::new();
    }
    (0..draft_count)
        .map(|index| DraftPlan {
            index,
            strategy: supported[index.min(supported.len() - 1)].clone(),
            seed: seed_for_draft(impulse_seed, index, 1),
            attempt: 1,
        })
        .collect()
}

/// Derive the seed for one draft slot and attempt.
///
/// Mixing the golden-ratio constant into the index (and the attempt) decorrelates
/// the streams of sibling drafts, so two slots running the same strategy explore
/// genuinely different paths while both stay reproducible from the impulse alone.
#[must_use]
pub const fn seed_for_draft(impulse_seed: u64, index: usize, attempt: u32) -> u64 {
    let index_mix = (index as u64)
        .wrapping_add(1)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let attempt_mix = (attempt as u64)
        .wrapping_add(1)
        .wrapping_mul(0xbf58_476d_1ce4_e5b9);
    impulse_seed ^ index_mix ^ attempt_mix
}

/// Run the portfolio: plan, evaluate concurrently, merge in draft order, select
/// the least-action passing draft that composes, and record the comparison.
///
/// The shared `log` receives one `draft:result` per slot, one `draft_failure`
/// per failing slot, exactly one `draft_comparison`, and finally the winner's
/// own trace — in that order, on every run.
pub fn run_portfolio<L: PortfolioLeaf>(
    leaf: &L,
    impulse_seed: u64,
    draft_count: usize,
    log: &mut EventLog,
) -> PortfolioSelection<L::Artifact> {
    let plans = plan_drafts(leaf, impulse_seed, draft_count);
    let mut drafts = evaluate_in_parallel(&plans, |plan| evaluate_slot(leaf, plan, impulse_seed));
    drafts.sort_by_key(|draft| draft.index);

    let winner_index = select_composable_draft(&drafts);
    record_draft_results(log, &drafts, winner_index);

    let comparison = comparison_record(&drafts, winner_index);
    log.append("draft_comparison", comparison.clone());

    let winner = winner_index.map(|index| {
        for event in drafts[index].trace.events() {
            log.append(event.kind, event.payload.clone());
        }
        drafts[index]
            .artifact
            .clone()
            .expect("a selected passing draft always carries its artifact")
    });
    let comparison_artifact = winner_index
        .map(|_| comparison.replacen("draft_comparison\n", "draft_comparison_artifact\n", 1));
    PortfolioSelection {
        winner,
        comparison_artifact,
    }
}

/// Evaluate one slot with bounded, learning retries.
///
/// A slot that fails its generated tests retries with a fresh seed up to
/// [`MAX_ATTEMPTS`] times; the best attempt (most tests passed, then least
/// action) is what the slot reports, so a retry can never make a slot worse.
fn evaluate_slot<L: PortfolioLeaf>(
    leaf: &L,
    plan: &DraftPlan,
    impulse_seed: u64,
) -> DraftEvaluation<L::Artifact> {
    let mut best: Option<DraftEvaluation<L::Artifact>> = None;
    let mut spent = 0;
    for attempt in 1..=MAX_ATTEMPTS {
        spent = attempt;
        let attempt_plan = DraftPlan {
            index: plan.index,
            strategy: plan.strategy.clone(),
            seed: seed_for_draft(impulse_seed, plan.index, attempt),
            attempt,
        };
        let evaluation = evaluate_attempt(leaf, &attempt_plan);
        let passed = evaluation.passed();
        if best
            .as_ref()
            .is_none_or(|current| is_better(&evaluation, current))
        {
            best = Some(evaluation);
        }
        if passed {
            break;
        }
    }
    let mut selected = best.expect("MAX_ATTEMPTS is non-zero so a slot always reports an attempt");
    // Report the retry budget actually spent, not the index of the attempt that
    // happened to win: the dreaming loop needs to know a slot exhausted its
    // budget, which "attempt 1 was best" would hide.
    selected.attempts = spent.max(1);
    selected
}

/// Is `candidate` a better report for its slot than `current`? More tests
/// passed wins; ties break by least action (smaller artifact, then fewer steps).
fn is_better<A>(candidate: &DraftEvaluation<A>, current: &DraftEvaluation<A>) -> bool {
    (
        std::cmp::Reverse(candidate.passed_tests),
        candidate.cost_size,
        candidate.cost_steps,
    ) < (
        std::cmp::Reverse(current.passed_tests),
        current.cost_size,
        current.cost_steps,
    )
}

fn evaluate_attempt<L: PortfolioLeaf>(leaf: &L, plan: &DraftPlan) -> DraftEvaluation<L::Artifact> {
    let drafted = leaf.draft(plan);
    let total_tests = leaf.test_count();
    let (passed_tests, composition_passed) = drafted.as_ref().map_or((0, false), |draft| {
        let results = leaf.run_tests(&draft.value);
        (
            results.iter().filter(|passed| **passed).count(),
            leaf.composes(&draft.value),
        )
    });
    DraftEvaluation {
        index: plan.index,
        strategy: plan.strategy.clone(),
        seed: plan.seed,
        attempts: plan.attempt,
        passed_tests,
        total_tests,
        cost_steps: drafted.as_ref().map_or(0, |draft| draft.cost_steps),
        cost_size: drafted.as_ref().map_or(0, |draft| draft.cost_size),
        composition_passed,
        artifact: drafted.as_ref().map(|draft| draft.value.clone()),
        trace: drafted.map_or_else(EventLog::new, |draft| draft.trace),
    }
}

/// Evaluate every plan concurrently, returning the results in completion-order.
///
/// Callers sort by draft index afterwards, which is what makes the concurrency
/// invisible in the answer. Scoped threads keep the borrow of `leaf` alive
/// without requiring `'static`, so no cloning of the problem is needed.
// The intermediate `collect` is the parallelism: every draft must be spawned
// before the first `join`, or the slots would run one after another.
#[allow(clippy::needless_collect)]
fn evaluate_in_parallel<R, F>(plans: &[DraftPlan], evaluate: F) -> Vec<R>
where
    R: Send,
    F: Fn(&DraftPlan) -> R + Sync,
{
    std::thread::scope(|scope| {
        let evaluate = &evaluate;
        let handles = plans
            .iter()
            .map(|plan| scope.spawn(move || evaluate(plan)))
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("draft evaluation panicked"))
            .collect()
    })
}

/// Rank the passing drafts by least action: smallest artifact first, then fewest
/// steps, then the lowest draft index as the final deterministic tie-break
/// (issue #491).
fn rank_passing_drafts<A>(drafts: &[DraftEvaluation<A>]) -> Vec<usize> {
    let mut ranked = drafts
        .iter()
        .filter(|draft| draft.passed())
        .map(|draft| draft.index)
        .collect::<Vec<_>>();
    ranked.sort_by_key(|index| {
        let draft = &drafts[*index];
        (draft.cost_size, draft.cost_steps, draft.index)
    });
    ranked
}

/// The first ranked draft that also composes. Passing drafts that fail
/// composition are skipped — that skip is the bounded, deterministic
/// backtracking of requirement 4.
fn select_composable_draft<A>(drafts: &[DraftEvaluation<A>]) -> Option<usize> {
    rank_passing_drafts(drafts)
        .into_iter()
        .find(|index| drafts[*index].composition_passed)
}

fn record_draft_results<A>(
    log: &mut EventLog,
    drafts: &[DraftEvaluation<A>],
    winner_index: Option<usize>,
) {
    for draft in drafts {
        let verdict = if Some(draft.index) == winner_index {
            "selected"
        } else if draft.passed() && !draft.composition_passed {
            "backtracked"
        } else {
            "rejected"
        };
        log.append(
            "draft:result",
            format_lino_record(
                &format!("draft_{}", draft.index),
                &[
                    ("draft_index", draft.index.to_string()),
                    ("strategy", draft.strategy.clone()),
                    ("seed", draft.seed.to_string()),
                    (
                        "status",
                        if draft.passed() { "passed" } else { "failed" }.to_owned(),
                    ),
                    ("passed_tests", draft.passed_tests.to_string()),
                    ("total_tests", draft.total_tests.to_string()),
                    ("cost_steps", draft.cost_steps.to_string()),
                    ("cost_size", draft.cost_size.to_string()),
                    ("attempts", draft.attempts.to_string()),
                    ("selection_verdict", verdict.to_owned()),
                ],
            ),
        );
        if !draft.passed() {
            log.append("draft_failure", failure_record(draft));
        }
    }
}

/// A durable, structured record of one failed draft.
///
/// The dreaming/learning loop mines these (see
/// [`crate::dreaming::draft_failure_candidate_tasks`]) to propose what to learn
/// next, which is why the record carries the strategy, how far the draft got,
/// and how much of the retry budget it consumed rather than only "it failed".
fn failure_record<A>(draft: &DraftEvaluation<A>) -> String {
    format_lino_record(
        &format!("draft_failure_{}", draft.index),
        &[
            ("record_type", "draft_failure".to_owned()),
            ("draft_index", draft.index.to_string()),
            ("strategy", draft.strategy.clone()),
            ("passed_tests", draft.passed_tests.to_string()),
            ("total_tests", draft.total_tests.to_string()),
            ("attempt", draft.attempts.to_string()),
            ("max_attempts", MAX_ATTEMPTS.to_string()),
            ("learning_status", "available_for_dreaming".to_owned()),
        ],
    )
}

fn comparison_record<A>(drafts: &[DraftEvaluation<A>], winner_index: Option<usize>) -> String {
    let winner = winner_index.map(|index| &drafts[index]);
    let runner_up_size = winner.and_then(|selected| {
        drafts
            .iter()
            .filter(|draft| draft.index != selected.index && draft.passed())
            .map(|draft| draft.cost_size)
            .min()
    });
    let smaller_percent = winner.zip(runner_up_size).map_or(0, |(selected, other)| {
        other
            .saturating_sub(selected.cost_size)
            .saturating_mul(100)
            .checked_div(other.max(1))
            .unwrap_or(0)
    });
    let total_tests = winner.map_or_else(
        || drafts.first().map_or(0, |draft| draft.total_tests),
        |draft| draft.total_tests,
    );
    format_lino_record(
        "draft_comparison",
        &[
            ("draft_count", drafts.len().to_string()),
            (
                "winner_index",
                winner_index.map_or_else(|| "none".to_owned(), |index| index.to_string()),
            ),
            (
                "winner_strategy",
                winner.map_or_else(|| "none".to_owned(), |draft| draft.strategy.clone()),
            ),
            (
                "passed_tests",
                winner.map_or(0, |draft| draft.passed_tests).to_string(),
            ),
            ("total_tests", total_tests.to_string()),
            (
                "rejected_drafts",
                drafts
                    .iter()
                    .filter(|draft| !draft.passed())
                    .count()
                    .to_string(),
            ),
            (
                "backtracked_drafts",
                drafts
                    .iter()
                    .filter(|draft| draft.passed() && !draft.composition_passed)
                    .count()
                    .to_string(),
            ),
            ("smaller_percent", smaller_percent.to_string()),
            ("tie_break", "least_action".to_owned()),
            ("merge_order", "draft_index".to_owned()),
        ],
    )
}
