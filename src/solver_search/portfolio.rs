use super::{run_search, seed_from_prompt, SearchProblem, SearchSolution};
use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::solver::SolverConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DraftStrategy {
    Reuse,
    RuleDerivation,
    Search,
    OracleLookup,
    ProgramSynthesis,
}

impl DraftStrategy {
    const fn slug(self) -> &'static str {
        match self {
            Self::Reuse => "reuse",
            Self::RuleDerivation => "rule_derivation",
            Self::Search => "search",
            Self::OracleLookup => "oracle_lookup",
            Self::ProgramSynthesis => "program_synthesis",
        }
    }
}

struct DraftEvaluation {
    index: usize,
    strategy: DraftStrategy,
    seed: u64,
    passed_tests: u8,
    total_tests: u8,
    steps: u32,
    size: usize,
    composition_passed: bool,
    solution: Option<SearchSolution>,
    trace: EventLog,
}

impl DraftEvaluation {
    const fn passed(&self) -> bool {
        self.passed_tests == self.total_tests
    }
}

pub(super) fn run_draft_portfolio(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    config: SolverConfig,
) -> (Option<SearchSolution>, Option<String>) {
    let strategies = strategies_for_count(usize::from(config.draft_count));
    let mut drafts = ordered_parallel_map(strategies, |index, strategy| {
        evaluate_draft(prompt, problem, config.compute_budget, index, strategy)
    });
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
            .solution
            .clone()
            .expect("a selected passing draft must have a solution")
    });
    let artifact = winner_index
        .map(|_| comparison.replacen("draft_comparison\n", "draft_comparison_artifact\n", 1));
    (winner, artifact)
}

fn record_draft_results(
    log: &mut EventLog,
    drafts: &[DraftEvaluation],
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
                    ("strategy", draft.strategy.slug().to_owned()),
                    ("seed", draft.seed.to_string()),
                    (
                        "status",
                        if draft.passed() {
                            "passed".to_owned()
                        } else {
                            "failed".to_owned()
                        },
                    ),
                    ("passed_tests", draft.passed_tests.to_string()),
                    ("total_tests", draft.total_tests.to_string()),
                    ("cost_steps", draft.steps.to_string()),
                    ("cost_size", draft.size.to_string()),
                    ("selection_verdict", verdict.to_owned()),
                ],
            ),
        );
        if !draft.passed() {
            log.append(
                "draft_failure",
                format_lino_record(
                    &format!("draft_failure_{}", draft.index),
                    &[
                        ("draft_index", draft.index.to_string()),
                        ("strategy", draft.strategy.slug().to_owned()),
                        ("attempt", "1".to_owned()),
                        ("max_attempts", "3".to_owned()),
                        ("learning_status", "available_for_dreaming".to_owned()),
                    ],
                ),
            );
        }
    }
}

fn ordered_parallel_map<T, R, F>(items: Vec<T>, evaluate: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(usize, T) -> R + Sync,
{
    std::thread::scope(|scope| {
        let evaluate = &evaluate;
        let handles = items
            .into_iter()
            .enumerate()
            .map(|(index, item)| scope.spawn(move || evaluate(index, item)))
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("draft evaluation panicked"))
            .collect()
    })
}

fn strategies_for_count(count: usize) -> Vec<DraftStrategy> {
    match count {
        0 => Vec::new(),
        1 => vec![DraftStrategy::Search],
        2 => vec![DraftStrategy::Reuse, DraftStrategy::Search],
        _ => {
            let mut strategies = vec![
                DraftStrategy::Reuse,
                DraftStrategy::RuleDerivation,
                DraftStrategy::Search,
            ];
            if count >= 4 {
                strategies.push(DraftStrategy::OracleLookup);
            }
            if count >= 5 {
                strategies.push(DraftStrategy::ProgramSynthesis);
            }
            while strategies.len() < count {
                strategies.push(DraftStrategy::Search);
            }
            strategies
        }
    }
}

fn seed_for_draft(prompt: &str, draft_index: usize) -> u64 {
    seed_from_prompt(prompt)
        ^ (draft_index as u64)
            .wrapping_add(1)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

fn evaluate_draft(
    prompt: &str,
    problem: &SearchProblem,
    budget: u32,
    index: usize,
    strategy: DraftStrategy,
) -> DraftEvaluation {
    let seed = seed_for_draft(prompt, index);
    let mut trace = EventLog::new();
    let solution = if strategy == DraftStrategy::Search {
        run_search(seed, &mut trace, problem, budget)
    } else {
        None
    };
    let passed_tests = u8::try_from(
        run_generated_tests(solution.as_ref(), problem)
            .into_iter()
            .filter(|passed| *passed)
            .count(),
    )
    .unwrap_or(0);
    let steps = solution.as_ref().map_or(0, |value| value.evaluations);
    let size = solution
        .as_ref()
        .map_or(0, |value| value.expression.chars().count());
    DraftEvaluation {
        index,
        strategy,
        seed,
        passed_tests,
        total_tests: 3,
        steps,
        size,
        composition_passed: solution
            .as_ref()
            .is_some_and(|value| composition_is_valid(value, problem)),
        solution,
        trace,
    }
}

fn run_generated_tests(solution: Option<&SearchSolution>, problem: &SearchProblem) -> [bool; 3] {
    let Some(solution) = solution else {
        return [false; 3];
    };
    let mut used = solution.candidate.order.clone();
    used.sort_unstable();
    let uses_every_operand_once = used == (0..problem.numbers.len()).collect::<Vec<_>>();
    let uses_only_allowed_operators = solution
        .candidate
        .ops
        .iter()
        .all(|candidate_op| problem.ops.contains(candidate_op));
    let reaches_target = solution.candidate.evaluate(&problem.numbers) == Some(problem.target);
    [
        uses_every_operand_once,
        uses_only_allowed_operators,
        reaches_target,
    ]
}

fn composition_is_valid(solution: &SearchSolution, problem: &SearchProblem) -> bool {
    run_generated_tests(Some(solution), problem)
        .into_iter()
        .all(|passed| passed)
}

fn rank_passing_drafts(drafts: &[DraftEvaluation]) -> Vec<usize> {
    let mut ranked = drafts
        .iter()
        .filter(|draft| draft.passed())
        .map(|draft| draft.index)
        .collect::<Vec<_>>();
    ranked.sort_by_key(|index| {
        let draft = &drafts[*index];
        (draft.size, draft.steps, draft.index)
    });
    ranked
}

fn select_composable_draft(drafts: &[DraftEvaluation]) -> Option<usize> {
    rank_passing_drafts(drafts)
        .into_iter()
        .find(|index| drafts[*index].composition_passed)
}

fn comparison_record(drafts: &[DraftEvaluation], winner_index: Option<usize>) -> String {
    let winner = winner_index.map(|index| &drafts[index]);
    let runner_up_size = winner.and_then(|selected| {
        drafts
            .iter()
            .filter(|draft| draft.index != selected.index && draft.passed())
            .map(|draft| draft.size)
            .min()
    });
    let smaller_percent = winner.zip(runner_up_size).map_or(0, |(selected, other)| {
        other
            .saturating_sub(selected.size)
            .saturating_mul(100)
            .checked_div(other.max(1))
            .unwrap_or(0)
    });
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
                winner
                    .map_or("none", |draft| draft.strategy.slug())
                    .to_owned(),
            ),
            (
                "passed_tests",
                winner.map_or(0, |draft| draft.passed_tests).to_string(),
            ),
            (
                "total_tests",
                winner.map_or(3, |draft| draft.total_tests).to_string(),
            ),
            ("smaller_percent", smaller_percent.to_string()),
            ("tie_break", "least_action".to_owned()),
            ("merge_order", "draft_index".to_owned()),
        ],
    )
}
