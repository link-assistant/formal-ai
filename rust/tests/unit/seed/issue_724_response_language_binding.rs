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

    // Plan 01's honest miss for a definition question is the consulted-source
    // record ("what does X mean" owes the record; batch a5abd1ab2 pinned that
    // shape), so the bound conversation renders that family in Russian. The
    // consulted list is environment-dependent, so the assertion pins the seed
    // template up to the consulted field.
    let russian_record = formal_ai::seed::render_response(
        "concept_lookup_unresolved",
        "ru",
        &[("surface", "an isogram"), ("consulted", "\u{1}")],
    )
    .expect("the Russian unresolved-source record is seeded");
    let expected = russian_record.split('\u{1}').next().unwrap_or_default();
    assert!(
        follow_up.answer.contains(expected),
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

    // The negative probe is the same Russian record prefix the binding test
    // pins: only an established language can put it in the answer.
    let russian_record = formal_ai::seed::render_response(
        "concept_lookup_unresolved",
        "ru",
        &[("surface", "an isogram"), ("consulted", "\u{1}")],
    )
    .expect("the Russian unresolved-source record is seeded");
    let russian = russian_record.split('\u{1}').next().unwrap_or_default();
    assert!(
        !follow_up.answer.contains(russian),
        "no language was established, so the answer must not be Russian: {}",
        follow_up.answer
    );
}

/// A directive that names a language only to forbid it ("Never answer in
/// Russian.") is a requirement against Russian, read from the forbidden
/// surfaces of `data/seed/statement-audit-registry.lino`: it is not the routed
/// demonstration, and it establishes no language for later turns. The
/// JavaScript twin is `rust/tests/web/issue-0661-requirement-contradiction.test.mjs`.
#[test]
fn a_forbidden_language_is_neither_demonstrated_nor_established() {
    let alone = offline_solver().solve_with_history("Never answer in Russian.", &[]);
    assert_ne!(alone.intent, "response_language_demonstration");
    assert!(
        !alone
            .evidence_links
            .iter()
            .any(|link| link == "language_to:ru"),
        "a forbidden language must not be answered in: {:?}",
        alone.evidence_links
    );

    let established = vec![
        ConversationTurn::user("Say something to me in Russian."),
        ConversationTurn::assistant("Здравствуйте! Чем могу помочь?"),
    ];
    let bound = offline_solver().solve_with_history("What is an isogram?", &established);
    assert!(
        bound
            .evidence_links
            .iter()
            .any(|link| link == "language:ru")
    );

    let mut revoked = established;
    revoked.push(ConversationTurn::user("Never answer in Russian."));
    revoked.push(ConversationTurn::assistant("Understood."));
    let unbound = offline_solver().solve_with_history("What is an isogram?", &revoked);
    assert!(
        unbound
            .evidence_links
            .iter()
            .any(|link| link == "language:en"),
        "the latest directive forbids Russian, so no language is established: {:?}",
        unbound.evidence_links
    );
}
