//! Issue #662: budget-driven random and evolutionary search in synthesis (F4).
//!
//! `GOALS.md` (Universal Solver Goals): "When no reusable part exists, combine
//! reasoning, random search, and evolutionary search according to the available
//! compute budget instead of giving up." These tests exercise a fixture problem
//! with no direct rule path: it is solved under a sufficient budget, remains
//! `unknown` (with `search:` evidence) under budget `0`, and produces identical
//! output across runs.

use formal_ai::{SolverConfig, UniversalSolver};

const SEARCH_PROMPT: &str =
    "Using the numbers 3, 5, and 7 with the operations + and *, find an expression that equals 26.";

fn solver_with_budget(compute_budget: u32) -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        compute_budget,
        ..SolverConfig::default()
    })
}

#[test]
fn budget_search_solves_reachability_under_sufficient_budget() {
    let solver = solver_with_budget(256);
    let answer = solver.solve(SEARCH_PROMPT);

    assert!(
        answer.answer.contains("budget-driven search"),
        "sufficient budget should solve via search; got: {}",
        answer.answer,
    );
    assert!(
        answer.answer.contains("26"),
        "the solved expression must evaluate to the target 26; got: {}",
        answer.answer,
    );
    assert_eq!(
        answer.intent, "budget_search_solution",
        "the solved answer should carry the budget-search intent",
    );
    assert!(
        answer.links_notation.contains("search:solution"),
        "the trace must record the `search:solution` derivation marker; got: {}",
        answer.links_notation,
    );
    // The search only combines the provided operators; the solved expression
    // must be one that actually evaluates to the target.
    assert!(
        answer.answer.contains("3 * 7 + 5 = 26") || answer.answer.contains("= 26"),
        "the answer should render a concrete satisfying composition; got: {}",
        answer.answer,
    );
}

#[test]
fn budget_search_success_proposes_a_gated_skill() {
    // A solved composition is a demonstrated capability, so the stage records a
    // proposal-only auto-learning event. It must be human-gated: proposed, never
    // promotable, and nothing about routing or the answer changes.
    let solver = solver_with_budget(256);
    let answer = solver.solve(SEARCH_PROMPT);

    assert!(
        answer.links_notation.contains("search:skill"),
        "a search success should record a proposal-only skill event; got: {}",
        answer.links_notation,
    );
    assert!(
        answer.links_notation.contains("status \"proposed\""),
        "the proposed skill must be recorded as `proposed`, not promoted; got: {}",
        answer.links_notation,
    );
    assert!(
        answer.links_notation.contains("promotable \"false\""),
        "the proposed skill must not be promotable without review; got: {}",
        answer.links_notation,
    );
    assert!(
        answer.links_notation.contains("search:skill:promotable"),
        "the auditable promotable count must be recorded; got: {}",
        answer.links_notation,
    );
}

#[test]
fn budget_search_zero_budget_proposes_no_skill() {
    // Nothing was demonstrated, so no skill is proposed.
    let solver = solver_with_budget(0);
    let answer = solver.solve(SEARCH_PROMPT);

    assert!(
        !answer.links_notation.contains("search:skill"),
        "an unsolved search must not propose a skill; got: {}",
        answer.links_notation,
    );
}

#[test]
fn budget_search_stays_unknown_with_evidence_under_zero_budget() {
    let solver = solver_with_budget(0);
    let answer = solver.solve(SEARCH_PROMPT);

    assert!(
        !answer.answer.contains("budget-driven search"),
        "budget 0 must not solve the problem; got: {}",
        answer.answer,
    );
    assert_ne!(
        answer.intent, "budget_search_solution",
        "budget 0 must fall back to the honest unknown-reasoning reply",
    );
    // The `search:` evidence must stay attached to the trace even when the
    // stage declines, so "why did you answer that?" explains the search path.
    assert!(
        answer.links_notation.contains("search:problem"),
        "budget 0 should still record the recognized search problem; got: {}",
        answer.links_notation,
    );
    assert!(
        answer.links_notation.contains("search:budget"),
        "budget 0 should record the compute budget it was given; got: {}",
        answer.links_notation,
    );
    assert!(
        answer.links_notation.contains("search:exhausted"),
        "budget 0 should record that the search produced no candidate; got: {}",
        answer.links_notation,
    );
    assert!(
        !answer.links_notation.contains("search:solution"),
        "budget 0 must not claim a solved composition; got: {}",
        answer.links_notation,
    );
}

#[test]
fn budget_search_is_deterministic_across_runs() {
    let solver = solver_with_budget(256);
    let first = solver.solve(SEARCH_PROMPT);
    let second = solver.solve(SEARCH_PROMPT);

    assert_eq!(
        first.answer, second.answer,
        "the seeded search must produce identical answers across runs",
    );
    assert_eq!(
        first.links_notation, second.links_notation,
        "the seeded search must produce an identical trace across runs",
    );

    // A separate solver instance built from the same config must also agree.
    let third = solver_with_budget(256).solve(SEARCH_PROMPT);
    assert_eq!(
        first.answer, third.answer,
        "determinism must not depend on solver instance identity",
    );
}

#[test]
fn budget_search_respects_the_allowed_operator_set() {
    // Only addition is allowed, so the search may not reach a product-only
    // target; it should decline and leave the honest unknown reply, with the
    // recognized operator set recorded in the trace.
    let prompt =
        "Using the numbers 2, 3, and 4 with the operation + only, find an expression that equals 24.";
    let solver = solver_with_budget(512);
    let answer = solver.solve(prompt);

    assert!(
        !answer.answer.contains("budget-driven search"),
        "an unreachable target under the allowed operators must not be claimed as solved; got: {}",
        answer.answer,
    );
    assert!(
        answer.links_notation.contains("search:problem"),
        "the unreachable attempt should still record its recognized problem; got: {}",
        answer.links_notation,
    );
}

#[test]
fn budget_search_recognizes_reachability_across_supported_languages() {
    // The same reachability puzzle (reach 26 from 3, 5, and 7 with + and *)
    // phrased in every supported language. Digits and the +/* symbols are
    // language-neutral; only the "numbers", search-verb, and target framings are
    // localized, so the search stage must recognize and solve each variant.
    let cases = [
        // english
        (
            "english",
            "Using the numbers 3, 5, and 7 with the operations + and *, find an expression that equals 26.",
        ),
        // russian / русский
        (
            "russian",
            "Используя числа 3, 5 и 7 с операциями + и *, найдите выражение, которое равно 26.",
        ),
        // hindi / हिंदी
        (
            "hindi",
            "संख्याओं 3, 5 और 7 का + और * संक्रियाओं के साथ उपयोग करके, 26 के बराबर एक व्यंजक खोजें।",
        ),
        // chinese / 中文
        (
            "chinese",
            "使用数字 3、5 和 7 以及运算 + 和 *，找出一个等于 26 的表达式。",
        ),
    ];

    for (language, prompt) in cases {
        let solver = solver_with_budget(256);
        let answer = solver.solve(prompt);
        assert!(
            answer.links_notation.contains("search:problem"),
            "[{language}] the search stage should recognize the localized reachability problem; got: {}",
            answer.links_notation,
        );
        assert!(
            answer.answer.contains("budget-driven search") && answer.answer.contains("26"),
            "[{language}] a sufficient budget should solve the localized puzzle; got: {}",
            answer.answer,
        );
        assert_eq!(
            answer.intent, "budget_search_solution",
            "[{language}] the solved answer should carry the budget-search intent",
        );
    }
}

#[test]
fn budget_search_stays_inert_for_ordinary_prompts() {
    // A plain calculation must not be captured by the search stage.
    let solver = solver_with_budget(512);
    let answer = solver.solve("What is 2 + 2?");

    assert!(
        answer.answer.contains('4'),
        "ordinary arithmetic should be answered directly; got: {}",
        answer.answer,
    );
    assert!(
        !answer.links_notation.contains("search:problem"),
        "the search stage must stay inert for ordinary prompts; got: {}",
        answer.links_notation,
    );
    assert_ne!(answer.intent, "budget_search_solution");
}
