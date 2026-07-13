//! Issue #676 — the assistant advertises "you can name me as you like", but a
//! follow-up such as "Now your name is Ineffa" fell through to the unknown opener.
//! These tests pin the dialog-local set/recall loop for the assistant's name.

use formal_ai::{ConversationTurn, UniversalSolver};

#[test]
fn setting_the_assistant_name_is_acknowledged() {
    let solver = UniversalSolver::default();
    let answer = solver.solve("Now your name is Ineffa");

    assert_eq!(
        answer.intent, "set_assistant_name",
        "naming the assistant should route to set_assistant_name, got {} -> {}",
        answer.intent, answer.answer,
    );
    assert_ne!(
        answer.intent, "unknown",
        "naming the assistant must not fall through to unknown: {}",
        answer.answer,
    );
    assert!(
        answer.answer.contains("Ineffa"),
        "acknowledgement should echo the chosen name, got: {}",
        answer.answer,
    );
}

#[test]
fn assistant_name_is_recalled_after_being_set() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("Now your name is Ineffa"),
        ConversationTurn::assistant("Nice to meet you! I'll go by Ineffa from now on."),
    ];
    let answer = solver.solve_with_history("What is your name?", &history);

    assert_eq!(
        answer.intent, "assistant_name",
        "asking after a rename should recall it, got {} -> {}",
        answer.intent, answer.answer,
    );
    assert!(
        answer.answer.contains("Ineffa"),
        "recall should return the previously set name, got: {}",
        answer.answer,
    );
}

#[test]
fn latest_rename_wins() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("Now your name is Ineffa"),
        ConversationTurn::assistant("Nice to meet you! I'll go by Ineffa from now on."),
        ConversationTurn::user("Actually, I'll call you Ada"),
        ConversationTurn::assistant("Nice to meet you! I'll go by Ada from now on."),
    ];
    let answer = solver.solve_with_history("what's your name?", &history);

    assert!(
        answer.answer.contains("Ada"),
        "the most recent rename should win, got: {}",
        answer.answer,
    );
    assert!(
        !answer.answer.contains("Ineffa"),
        "the superseded name should not be returned, got: {}",
        answer.answer,
    );
}

#[test]
fn name_question_without_a_set_name_keeps_the_static_answer() {
    let solver = UniversalSolver::default();
    let answer = solver.solve("What is your name?");

    assert_eq!(
        answer.intent, "assistant_name",
        "with no prior rename the static assistant_name answer applies, got {}",
        answer.intent,
    );
    assert!(
        answer.answer.contains("name me as you like"),
        "the default answer should still invite naming, got: {}",
        answer.answer,
    );
}

#[test]
fn i_will_call_you_variant_sets_the_name() {
    let solver = UniversalSolver::default();
    let answer = solver.solve("I'll call you Ada");

    assert_eq!(
        answer.intent, "set_assistant_name",
        "\"I'll call you X\" should set the assistant name, got {} -> {}",
        answer.intent, answer.answer,
    );
    assert!(answer.answer.contains("Ada"), "got: {}", answer.answer);
}
