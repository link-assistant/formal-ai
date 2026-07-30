//! Issue #704: deterministic parallel candidate-solution portfolios.

use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};
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
    let answer = portfolio_solver(3).solve(SEARCH_PROMPT);
    let cases = [
        ("Why did you choose that solution?", "passed 3/3"),
        ("Почему ты выбрал это решение?", "3/3"),
        ("आपने वह समाधान क्यों चुना?", "3/3"),
        ("你为什么选择那个解决方案？", "3/3"),
    ];

    for (prompt, expected) in cases {
        let explanation = portfolio_solver(3).solve_with_history(
            prompt,
            &[ConversationTurn::assistant(answer.answer.clone())],
        );
        assert_eq!(explanation.intent, "draft_comparison_explanation");
        assert!(
            explanation.answer.contains(expected),
            "{prompt:?} should explain the recorded comparison: {}",
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

#[test]
fn composition_backtracking_and_ordered_parallelism_are_grounded_in_source() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/solver_search/portfolio.rs"),
    )
    .expect("portfolio implementation");

    for expected in [
        "std::thread::scope",
        "drafts.sort_by_key(|draft| draft.index)",
        ".find(|index| drafts[*index].composition_passed)",
        r#""backtracked""#,
    ] {
        assert!(
            source.contains(expected),
            "missing implementation marker {expected}"
        );
    }
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
