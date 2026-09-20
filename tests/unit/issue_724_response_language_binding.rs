//! Issue #724, plan 10 leaf 15: a language the conversation has established
//! keeps forcing the response language.
//!
//! The routed demonstration ("Say something to me in Russian.") answers once;
//! the leaf's claim is that the named language then *binds*: later requests in
//! the same conversation are answered in it, the way an explicitly forced
//! `SolverConfig::forced_response_language` would bind, until the user names
//! another language. The marker that establishes the language is the same
//! seed-grounded response-language role the route reads, so nothing here is a
//! phrase table.

use formal_ai::solver::{ConversationTurn, SolverConfig, UniversalSolver};

fn offline_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
}

#[test]
fn a_demonstrated_language_binds_for_the_rest_of_the_conversation() {
    let established = offline_solver();
    let demonstration = established.solve_with_history("Say something to me in Russian.", &[]);
    assert_eq!(
        demonstration.intent, "response_language_demonstration",
        "the first turn is the routed demonstration itself"
    );

    // The conversation carries the request and the demonstration forward; the
    // next request names no language at all.
    let history = vec![
        ConversationTurn::user("Say something to me in Russian."),
        ConversationTurn::assistant(demonstration.answer),
    ];
    let bound = offline_solver();
    let follow_up = bound.solve_with_history("What is an isogram?", &history);

    // Plan 01 reworked the offline miss into the unknown-reasoning gather, so
    // the bound conversation renders that family in Russian (`unknown_reasoning_trace`).
    let expected = formal_ai::seed::render_response(
        "unknown_reasoning_trace",
        "ru",
        &[("focus", "an isogram")],
    )
    .expect("the Russian unknown-reasoning trace response is seeded");
    assert!(
        follow_up.answer.contains(&expected),
        "a conversation that demonstrated Russian must keep answering in Russian: \
         {}",
        follow_up.answer
    );
}

#[test]
fn a_conversation_with_no_established_language_stays_in_the_prompt_language() {
    let history = vec![
        ConversationTurn::user("What is an isogram?"),
        ConversationTurn::assistant("An isogram is a word with no repeating letters."),
    ];
    let unbound = offline_solver();
    let follow_up = unbound.solve_with_history("What is an isogram?", &history);

    let russian = formal_ai::seed::render_response(
        "unknown_reasoning_trace",
        "ru",
        &[("focus", "an isogram")],
    )
    .expect("the Russian unknown-reasoning trace response is seeded");
    assert!(
        !follow_up.answer.contains(&russian),
        "no language was established, so the answer must not be Russian: {}",
        follow_up.answer
    );
}
