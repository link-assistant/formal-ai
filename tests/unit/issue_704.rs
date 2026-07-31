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
        r#"order "9""#,
        r#"engine "src/draft_portfolio.rs""#,
        r#"strategy_catalog "data/seed/draft-strategies.lino""#,
        r#"leaf "src/solver_search/portfolio.rs""#,
        r#"leaf "src/rule_synthesis_portfolio.rs""#,
        r#"learning_consumer "src/dreaming/draft_failures.rs""#,
        r#"miner "draft_failure_lessons""#,
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
    script: Vec<ScriptedStrategy>,
}

struct ScriptedStrategy {
    strategy: &'static str,
    /// Does this strategy produce a draft at all?
    drafts: bool,
    /// How many of the two generated tests the draft passes.
    passed_tests: usize,
    composes: bool,
    cost_size: usize,
}

impl ScriptedLeaf {
    fn row(&self, strategy: &str) -> Option<&ScriptedStrategy> {
        self.script.iter().find(|row| row.strategy == strategy)
    }
}

impl PortfolioLeaf for ScriptedLeaf {
    type Artifact = String;

    fn supports(&self, strategy: &str) -> bool {
        self.row(strategy).is_some()
    }

    fn draft(&self, plan: &DraftPlan) -> Option<DraftArtifact<Self::Artifact>> {
        let row = self.row(&plan.strategy)?;
        row.drafts.then(|| DraftArtifact {
            value: format!("{}:{}", row.strategy, plan.seed),
            cost_steps: 1,
            cost_size: row.cost_size,
            trace: EventLog::new(),
        })
    }

    fn run_tests(&self, artifact: &Self::Artifact) -> Vec<bool> {
        let passed = artifact
            .split(':')
            .next()
            .and_then(|strategy| self.row(strategy))
            .map_or(0, |row| row.passed_tests);
        (0..self.test_count()).map(|index| index < passed).collect()
    }

    fn test_count(&self) -> usize {
        2
    }

    fn composes(&self, artifact: &Self::Artifact) -> bool {
        artifact
            .split(':')
            .next()
            .and_then(|strategy| self.row(strategy))
            .is_some_and(|row| row.composes)
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
            ScriptedStrategy {
                strategy: "reuse",
                drafts: true,
                passed_tests: 2,
                composes: false,
                cost_size: 4,
            },
            ScriptedStrategy {
                strategy: "rule_derivation",
                drafts: true,
                passed_tests: 2,
                composes: true,
                cost_size: 9,
            },
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
            ScriptedStrategy {
                strategy: "reuse",
                drafts: true,
                passed_tests: 0,
                composes: false,
                cost_size: 4,
            },
            ScriptedStrategy {
                strategy: "rule_derivation",
                drafts: true,
                passed_tests: 0,
                composes: false,
                cost_size: 6,
            },
            ScriptedStrategy {
                strategy: "oracle_lookup",
                drafts: true,
                passed_tests: 2,
                composes: true,
                cost_size: 8,
            },
            ScriptedStrategy {
                strategy: "search",
                drafts: true,
                passed_tests: 2,
                composes: true,
                cost_size: 3,
            },
            ScriptedStrategy {
                strategy: "program_synthesis",
                drafts: true,
                passed_tests: 2,
                composes: true,
                cost_size: 5,
            },
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
        script: vec![ScriptedStrategy {
            strategy: "reuse",
            drafts: true,
            passed_tests: 0,
            composes: false,
            cost_size: 4,
        }],
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
        script: vec![ScriptedStrategy {
            strategy: "reuse",
            drafts: true,
            passed_tests: 0,
            composes: false,
            cost_size: 4,
        }],
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

#[test]
fn losing_drafts_become_durable_lessons_the_dreaming_loop_mines() {
    use formal_ai::dreaming::{plan_memory_dreaming, render_dreaming_plan, DreamingConfig};
    use formal_ai::memory::MemoryStore;

    // Two slots fail (`reuse` exhausts its retry budget passing nothing,
    // `rule_derivation` gets halfway), one succeeds.
    let leaf = ScriptedLeaf {
        script: vec![
            ScriptedStrategy {
                strategy: "reuse",
                drafts: false,
                passed_tests: 0,
                composes: false,
                cost_size: 0,
            },
            ScriptedStrategy {
                strategy: "rule_derivation",
                drafts: true,
                passed_tests: 1,
                composes: false,
                cost_size: 6,
            },
            ScriptedStrategy {
                strategy: "oracle_lookup",
                drafts: true,
                passed_tests: 2,
                composes: true,
                cost_size: 8,
            },
        ],
    };
    let mut log = EventLog::new();
    let selection = formal_ai::draft_portfolio::run_portfolio(&leaf, 5, 3, &mut log);
    assert!(selection.winner.is_some());

    // The portfolio's records are persisted through the ordinary projection.
    let mut memory = MemoryStore::new();
    log.append_to_link_store(&mut memory)
        .expect("portfolio evidence should project into the durable store");

    let lessons = formal_ai::dreaming::draft_failures::draft_failure_lessons(memory.events());
    let strategies = lessons
        .iter()
        .map(|lesson| lesson.strategy.as_str())
        .collect::<Vec<_>>();
    assert!(
        strategies.contains(&"reuse") && strategies.contains(&"rule_derivation"),
        "both failing strategies must survive as retained learning: {strategies:?}"
    );
    let reuse = lessons
        .iter()
        .find(|lesson| lesson.strategy == "reuse")
        .expect("reuse lesson");
    assert!(reuse.exhausted_retry_budget);
    assert_eq!(reuse.lesson, "deprioritize_strategy");
    assert_eq!(reuse.attempts, formal_ai::draft_portfolio::MAX_ATTEMPTS);
    let derivation = lessons
        .iter()
        .find(|lesson| lesson.strategy == "rule_derivation")
        .expect("rule_derivation lesson");
    assert_eq!(
        derivation.lesson, "extend_strategy",
        "a strategy that passed some tests is close, not useless"
    );

    // And the lessons reach the dreaming plan itself, not just the miner.
    let plan = plan_memory_dreaming(
        memory.events(),
        &DreamingConfig {
            daydreaming_enabled: true,
            ..DreamingConfig::default()
        },
    );
    assert_eq!(plan.draft_failures, lessons);
    let rendered = render_dreaming_plan(&plan);
    assert!(
        rendered.contains("draft_failure_lesson strategy=reuse"),
        "{rendered}"
    );
}

/// Issue #704, requirement 1: the portfolio must be a property of the meta
/// algorithm, not of one handler. This exercises a *second*, unrelated leaf —
/// rule synthesis for a bare program-modification follow-up — through the same
/// engine, with a differently worded request than the arithmetic case above.
///
/// Both applicable strategies (`reuse` from the learning ledger, `rule_derivation`
/// from the operation vocabulary) draft independently, are tested against the
/// same verification fixture, and the comparison is recorded. The answer must be
/// the same one the sequential fallback chain produces — a portfolio changes how
/// the rule is chosen, never what a correct answer is.
#[test]
fn rule_synthesis_is_a_second_portfolio_leaf_with_the_same_engine() {
    const FIRST: &str = "Write me a Rust program that lists the files in the current directory";
    const FOLLOW_UP: &str = "Sort the results in reverse order";

    let sequential = UniversalSolver::default();
    let first = sequential.solve(FIRST);
    assert_eq!(first.intent, "write_program", "{}", first.answer);
    let history = [
        ConversationTurn::user(FIRST),
        ConversationTurn::assistant(first.answer),
    ];
    let baseline = sequential.solve_with_history(FOLLOW_UP, &history);
    assert_eq!(baseline.intent, "write_program", "{}", baseline.answer);

    let portfolio = UniversalSolver::new(SolverConfig {
        draft_count: 3,
        diagnostic_mode: true,
        ..SolverConfig::default()
    });
    let drafted = portfolio.solve_with_history(FOLLOW_UP, &history);

    let answer = drafted
        .answer
        .split("\n\n[diagnostic]")
        .next()
        .expect("answer before the diagnostic block");
    assert_eq!(
        answer, baseline.answer,
        "the portfolio selects among drafts; it must not change what a correct answer is"
    );
    let repeated = portfolio.solve_with_history(FOLLOW_UP, &history);
    assert_eq!(
        repeated.answer, drafted.answer,
        "the same impulse must seed the same drafts and reach the same answer"
    );

    // Issue #704 acceptance: a case where the first strategy fails outright and
    // a parallel draft rescues the turn. `reuse` has no approved ledger lesson
    // for this wording, so it drafts nothing and burns its bounded retry budget;
    // `rule_derivation` derives the rule from the operation vocabulary and wins.
    for marker in [
        "strategy \"reuse\"",
        "status \"failed\"",
        "record_type \"draft_failure\"",
        "strategy \"rule_derivation\"",
        "status \"passed\"",
        "winner_strategy \"rule_derivation\"",
        "tie_break \"least_action\"",
        "merge_order \"draft_index\"",
    ] {
        assert!(
            drafted.answer.contains(marker),
            "the second leaf must record its comparison, missing {marker}"
        );
    }
}
