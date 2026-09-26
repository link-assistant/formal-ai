//! Plan 08 capability/debt ratchets.

use std::fs;

use super::repo_root;

/// The three legacy authored word-problem shapes remain fallback candidates;
/// new capability must enter through the derived verifiable-task route.
#[test]
fn authored_word_problem_shapes_do_not_grow_and_are_demoted() {
    let source = fs::read_to_string(repo_root().join("rust/src/calculation_word_problem.rs"))
        .expect("word-problem normalizer source is readable");
    let shapes = [
        "normalize_box_total_problem(trimmed)",
        "normalize_train_meeting_problem(trimmed)",
        "resolve_fibonacci_references(&arithmetic.join",
    ];
    assert_eq!(
        shapes
            .iter()
            .filter(|shape| source.contains(**shape))
            .count(),
        3,
        "the reviewed authored-shape ceiling is three"
    );

    let precedence = formal_ai::seed::handler_precedence();
    let derived = precedence
        .iter()
        .position(|handler| handler == "verifiable_task")
        .expect("the generic derived route is registered");
    let legacy = precedence
        .iter()
        .position(|handler| handler == "arithmetic")
        .expect("the legacy arithmetic fallback is registered");
    assert!(
        derived < legacy,
        "derived candidates must run before authored word-problem fallbacks"
    );
}

/// Pattern inference donated its registry slot to the generic interpreter, so
/// Plan 08 does not raise the handler-count debt ceiling.
#[test]
fn the_generic_interpreter_reuses_the_retired_pattern_slot() {
    let precedence = formal_ai::seed::handler_precedence();
    assert!(precedence.iter().any(|name| name == "verifiable_task"));
    assert!(!precedence.iter().any(|name| name == "pattern_inference"));
}
