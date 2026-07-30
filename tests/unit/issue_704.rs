//! Issue #704: deterministic parallel candidate-solution portfolios.

use formal_ai::draft_portfolio::{DraftArtifact, DraftPlan, PortfolioLeaf};
use formal_ai::{ConversationTurn, EventLog, SolverConfig, UniversalSolver};
use std::time::Instant;

const SEARCH_PROMPT: &str =
    "Using the numbers 3, 5, and 7 with the operations + and *, find an expression that equals 26.";

fn portfolio_solver(draft_count: u8) -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        compute_budget: 256,
        draft_count,
        ..SolverConfig::default()
    })
}

fn event_count(links_notation: &str, kind: &str) -> usize {
    links_notation.matches(&format!(" {kind} ")).count()
}

#[test]
fn three_drafts_are_tested_compared_and_deterministic() {
    let solver = portfolio_solver(3);
    let first = solver.solve(SEARCH_PROMPT);
    let second = solver.solve(SEARCH_PROMPT);

    assert_eq!(first.answer, second.answer);
    assert_eq!(first.links_notation, second.links_notation);
    assert_eq!(
        event_count(&first.links_notation, "draft:result"),
        3,
        "{}",
        first.links_notation
    );
    assert_eq!(event_count(&first.links_notation, "draft_comparison"), 1);
    assert_eq!(event_count(&first.links_notation, "draft_failure"), 2);
    assert!(
        first.links_notation.contains(r#"draft_index "0""#)
            && first.links_notation.contains("status \"failed\""),
        "the first strategy must fail so the portfolio proves rescue: {}",
        first.links_notation
    );
    assert!(
        first.links_notation.contains(r#"winner_index "2""#),
        "the passing search draft should rescue the failed first path: {}",
        first.links_notation
    );
    assert!(
        first.links_notation.contains(r#"passed_tests "3""#)
            && first.links_notation.contains(r#"total_tests "3""#),
        "selection must be grounded in all generated tests: {}",
        first.links_notation
    );
    assert!(first.links_notation.contains(r#"max_attempts "3""#));
    assert!(first
        .links_notation
        .contains(r#"learning_status "available_for_dreaming""#));
    assert_eq!(first.intent, "budget_search_solution");
    assert!(first.answer.contains("= 26"));
}

#[test]
fn default_one_draft_preserves_the_existing_search_path() {
    let answer = portfolio_solver(1).solve(SEARCH_PROMPT);
    let default_answer = UniversalSolver::new(SolverConfig {
        offline: true,
        compute_budget: 256,
        ..SolverConfig::default()
    })
    .solve(SEARCH_PROMPT);

    assert_eq!(SolverConfig::default().draft_count, 1);
    assert_eq!(answer.answer, default_answer.answer);
    assert_eq!(answer.links_notation, default_answer.links_notation);
    assert_eq!(event_count(&answer.links_notation, "draft:result"), 0);
    assert_eq!(event_count(&answer.links_notation, "draft_comparison"), 0);
    assert_eq!(answer.intent, "budget_search_solution");
    assert!(answer.answer.contains("= 26"));
}

#[test]
fn comparison_ledger_answers_why_in_all_supported_languages() {
    struct LanguageCase {
        language: &'static str,
        prompt: &'static str,
        expected: &'static str,
    }

    let answer = portfolio_solver(3).solve(SEARCH_PROMPT);
    let cases = [
        LanguageCase {
            language: "en",
            prompt: "Why did you choose that solution?",
            expected: "passed 3/3",
        },
        LanguageCase {
            language: "ru",
            prompt: "Почему ты выбрал это решение?",
            expected: "3/3",
        },
        LanguageCase {
            language: "hi",
            prompt: "आपने वह समाधान क्यों चुना?",
            expected: "3/3",
        },
        LanguageCase {
            language: "zh",
            prompt: "你为什么选择那个解决方案？",
            expected: "3/3",
        },
    ];

    for case in cases {
        let explanation = portfolio_solver(3).solve_with_history(
            case.prompt,
            &[ConversationTurn::assistant(answer.answer.clone())],
        );
        assert_eq!(explanation.intent, "draft_comparison_explanation");
        assert!(
            explanation.answer.contains(case.expected),
            "{} prompt {:?} should explain the recorded comparison: {}",
            case.language,
            case.prompt,
            explanation.answer
        );
    }
}

#[test]
fn draft_count_can_be_configured_from_the_environment() {
    let previous = std::env::var_os("FORMAL_AI_DRAFT_COUNT");
    std::env::set_var("FORMAL_AI_DRAFT_COUNT", "3");
    let config = SolverConfig::from_env();
    if let Some(value) = previous {
        std::env::set_var("FORMAL_AI_DRAFT_COUNT", value);
    } else {
        std::env::remove_var("FORMAL_AI_DRAFT_COUNT");
    }

    assert_eq!(config.draft_count, 3);
}

#[test]
fn grounded_meta_recipe_covers_the_complete_portfolio_loop() {
    let recipe = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/meta/draft-portfolio-recipe.lino"),
    )
    .expect("draft portfolio recipe");

    for expected in [
        r#"topic "candidate_solution_portfolio""#,
        r#"order "1""#,
        r#"order "8""#,
        r#"passing_gate "passed_tests equals total_tests""#,
        r#"least_action_order "cost_size, cost_steps, draft_index""#,
        r#"composition_backtrack "enabled""#,
        r#"max_attempts "3""#,
        r#"benchmark "data/benchmarks/industry-suite.lino""#,
    ] {
        assert!(
            recipe.contains(expected),
            "missing recipe marker {expected}"
        );
    }
}

/// A synthetic leaf that lets a test dictate, per strategy, whether a draft is
/// produced, whether it passes its tests, whether it composes, and how large it
/// is — so the engine's own behaviour can be asserted without arithmetic.
struct ScriptedLeaf {
    /// `strategy -> (drafts, passes_tests, composes, cost_size)`
    script: Vec<(&'static str, bool, bool, bool, usize)>,
}

impl ScriptedLeaf {
    fn row(&self, strategy: &str) -> Option<&(&'static str, bool, bool, bool, usize)> {
        self.script.iter().find(|row| row.0 == strategy)
    }
}

impl PortfolioLeaf for ScriptedLeaf {
    type Artifact = String;

    fn supports(&self, strategy: &str) -> bool {
        self.row(strategy).is_some()
    }

    fn draft(&self, plan: &DraftPlan) -> Option<DraftArtifact<Self::Artifact>> {
        let row = self.row(&plan.strategy)?;
        row.1.then(|| DraftArtifact {
            value: format!("{}:{}", row.0, plan.seed),
            cost_steps: 1,
            cost_size: row.4,
            trace: EventLog::new(),
        })
    }

    fn run_tests(&self, artifact: &Self::Artifact) -> Vec<bool> {
        let passes = artifact
            .split(':')
            .next()
            .and_then(|strategy| self.row(strategy))
            .is_some_and(|row| row.2);
        vec![passes, passes]
    }

    fn test_count(&self) -> usize {
        2
    }

    fn composes(&self, artifact: &Self::Artifact) -> bool {
        artifact
            .split(':')
            .next()
            .and_then(|strategy| self.row(strategy))
            .is_some_and(|row| row.3)
    }
}

fn payloads(log: &EventLog, kind: &str) -> Vec<String> {
    log.events()
        .iter()
        .filter(|event| event.kind == kind)
        .map(|event| event.payload.clone())
        .collect()
}

#[test]
fn a_passing_draft_that_fails_composition_is_backtracked_past() {
    // The cheapest passing draft (`reuse`, size 4) does not compose, so
    // selection must fall through to the larger `rule_derivation` draft.
    let leaf = ScriptedLeaf {
        script: vec![
            ("reuse", true, true, false, 4),
            ("rule_derivation", true, true, true, 9),
        ],
    };
    let mut log = EventLog::new();
    let selection = formal_ai::draft_portfolio::run_portfolio(&leaf, 7, 2, &mut log);

    assert!(selection
        .winner
        .as_ref()
        .is_some_and(|winner| winner.starts_with("rule_derivation")));
    let results = payloads(&log, "draft:result");
    assert_eq!(results.len(), 2);
    assert!(
        results[0].contains(r#"selection_verdict "backtracked""#),
        "{}",
        results[0]
    );
    assert!(
        results[1].contains(r#"selection_verdict "selected""#),
        "{}",
        results[1]
    );
    let comparison = payloads(&log, "draft_comparison");
    assert_eq!(comparison.len(), 1);
    assert!(
        comparison[0].contains(r#"backtracked_drafts "1""#),
        "{}",
        comparison[0]
    );
}

#[test]
fn concurrent_drafts_are_merged_in_draft_index_order() {
    let leaf = ScriptedLeaf {
        script: vec![
            ("reuse", true, false, false, 4),
            ("rule_derivation", true, false, false, 6),
            ("oracle_lookup", true, true, true, 8),
            ("search", true, true, true, 3),
            ("program_synthesis", true, true, true, 5),
        ],
    };
    let mut first = EventLog::new();
    let selection = formal_ai::draft_portfolio::run_portfolio(&leaf, 11, 5, &mut first);
    let mut second = EventLog::new();
    let repeat = formal_ai::draft_portfolio::run_portfolio(&leaf, 11, 5, &mut second);

    assert_eq!(selection.winner, repeat.winner);
    assert_eq!(
        payloads(&first, "draft:result"),
        payloads(&second, "draft:result")
    );
    let indices = payloads(&first, "draft:result")
        .iter()
        .map(|record| {
            record
                .lines()
                .find_map(|line| line.trim().strip_prefix("draft_index "))
                .map(|value| value.trim_matches('"').to_owned())
                .expect("draft_index field")
        })
        .collect::<Vec<_>>();
    assert_eq!(indices, ["0", "1", "2", "3", "4"]);
    // Least action decides among the passing drafts, not the draft order:
    // `search` (size 3) beats `program_synthesis` (5) and `oracle_lookup` (8).
    assert!(selection
        .winner
        .as_ref()
        .is_some_and(|winner| winner.starts_with("search")));
}

#[test]
fn a_failing_slot_exhausts_its_bounded_retry_budget_and_records_the_failure() {
    let leaf = ScriptedLeaf {
        script: vec![("reuse", true, false, false, 4)],
    };
    let mut log = EventLog::new();
    let selection = formal_ai::draft_portfolio::run_portfolio(&leaf, 3, 1, &mut log);

    assert!(selection.winner.is_none());
    assert!(selection.comparison_artifact.is_none());
    let failures = payloads(&log, "draft_failure");
    assert_eq!(failures.len(), 1);
    assert!(
        failures[0].contains(&format!(
            r#"max_attempts "{}""#,
            formal_ai::draft_portfolio::MAX_ATTEMPTS
        )),
        "{}",
        failures[0]
    );
    assert!(
        payloads(&log, "draft:result")[0].contains(&format!(
            r#"attempts "{}""#,
            formal_ai::draft_portfolio::MAX_ATTEMPTS
        )),
        "a failing slot must spend its whole bounded retry budget"
    );
}

#[test]
fn each_slot_and_attempt_gets_a_distinct_reproducible_seed() {
    let leaf = ScriptedLeaf {
        script: vec![("reuse", true, false, false, 4)],
    };
    let planned = formal_ai::draft_portfolio::plan_drafts(&leaf, 42, 4);
    let mut seeds = planned.iter().map(|plan| plan.seed).collect::<Vec<_>>();
    let unique = {
        seeds.sort_unstable();
        seeds.dedup();
        seeds.len()
    };
    assert_eq!(unique, 4, "sibling drafts must explore different streams");
    assert_eq!(
        formal_ai::draft_portfolio::seed_for_draft(42, 1, 1),
        formal_ai::draft_portfolio::seed_for_draft(42, 1, 1),
        "seeds are a pure function of the impulse, slot, and attempt"
    );
    assert_ne!(
        formal_ai::draft_portfolio::seed_for_draft(42, 1, 1),
        formal_ai::draft_portfolio::seed_for_draft(42, 1, 2),
        "a retry must not repeat the attempt that already failed"
    );
}

#[test]
fn the_strategy_catalog_is_seed_data_not_rust() {
    let catalog = formal_ai::draft_portfolio::strategy_catalog();
    assert_eq!(
        catalog,
        [
            "reuse",
            "rule_derivation",
            "oracle_lookup",
            "search",
            "program_synthesis"
        ],
        "the shipped seed declares the portfolio's preference order"
    );
    let reordered = formal_ai::seed::draft_strategies_from("draft_strategies\n  search\n  reuse\n");
    assert_eq!(
        reordered,
        ["search", "reuse"],
        "reordering the seed reorders the portfolio, with no Rust change"
    );
}

#[test]
fn parallel_wall_clock_stays_within_single_plus_slowest_when_enabled() {
    if std::env::var_os("FORMAL_AI_PARALLEL_TIMING_TEST").is_none() {
        return;
    }

    let _ = portfolio_solver(1).solve(SEARCH_PROMPT);
    let _ = portfolio_solver(8).solve(SEARCH_PROMPT);

    let single_started = Instant::now();
    let _ = portfolio_solver(1).solve(SEARCH_PROMPT);
    let single_baseline = single_started.elapsed();

    let parallel_started = Instant::now();
    let portfolio = portfolio_solver(8).solve(SEARCH_PROMPT);
    let parallel_elapsed = parallel_started.elapsed();

    assert_eq!(event_count(&portfolio.links_notation, "draft:result"), 8);
    assert!(
        parallel_elapsed <= single_baseline.saturating_mul(2),
        "parallel={parallel_elapsed:?}, single={single_baseline:?}"
    );
}
