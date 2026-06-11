//! Issue #412: a bare numeric-list follow-up answered `unknown`.
//!
//! After a first turn established a coding context — "…отсортируй их в
//! JavaScript, дай мне код и результат" — the user typed only
//! `Отсортируй 4, 3, 1, 17, 8, 9, 15`. That turn names no programming language
//! and does not ask for code, so the numeric-list handler declined it and the
//! request fell through to `unknown`. The solver must now recover the target
//! language (and the code request) from the conversation and continue the
//! coding context: emit idiomatic code in the established language and the
//! deterministically-computed result.

use formal_ai::{ConversationTurn, UniversalSolver};

/// The active coding context from the previous turn (JavaScript, code+result).
fn javascript_sort_context() -> Vec<ConversationTurn> {
    vec![
        ConversationTurn::user(
            "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
        ),
        ConversationTurn::assistant(
            "Вот код на JavaScript, который сортирует числа 3, 5, 6, 7, 8 по возрастанию:\n\n```javascript\nconst numbers = [3, 5, 6, 7, 8];\nconst sorted = [...numbers].sort((a, b) => a - b);\nconsole.log(sorted.join(\", \"));\n```\n\nРезультат: 3, 5, 6, 7, 8",
        ),
    ]
}

/// The exact reported follow-up must no longer be `unknown`: it inherits the
/// JavaScript target and produces runnable code plus the sorted result.
#[test]
fn issue_412_bare_followup_inherits_language_and_is_not_unknown() {
    let solver = UniversalSolver::default();
    let response = solver.solve_with_history(
        "Отсортируй 4, 3, 1, 17, 8, 9, 15",
        &javascript_sort_context(),
    );

    assert_eq!(
        response.intent, "write_program",
        "the follow-up must continue the coding context, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```javascript"),
        "answer must inherit the JavaScript language, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("const numbers = [4, 3, 1, 17, 8, 9, 15];"),
        "code must keep the user's new given order, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains(".sort((a, b) => a - b)"),
        "answer must contain the ascending JS comparator, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Результат: 1, 3, 4, 8, 9, 15, 17"),
        "result must be the new list sorted ascending, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("numeric_list_coreference inherited_language=javascript"),
        "trace must record the inherited language, got: {}",
        response.links_notation
    );
}

/// Without any prior coding context, the same bare prompt stays `unknown`: the
/// fix recovers context, it does not steal plain prose. (No language is known,
/// so there is nothing to continue.)
#[test]
fn issue_412_bare_prompt_without_context_does_not_fabricate_language() {
    let solver = UniversalSolver::default();
    let response = solver.solve("Отсортируй 4, 3, 1, 17, 8, 9, 15");

    assert_ne!(
        response.intent, "write_program",
        "a bare sort with no established language must not invent one, got: {}",
        response.answer
    );
}

/// A reduction follow-up ("now sum them") inherits both the language and the
/// code request, so it produces code plus the computed scalar even though the
/// follow-up itself does not say "code".
#[test]
fn issue_412_reduction_followup_inherits_code_request() {
    let solver = UniversalSolver::default();
    let mut history = javascript_sort_context();
    history.push(ConversationTurn::user("Отсортируй 4, 3, 1, 17, 8, 9, 15"));
    history.push(ConversationTurn::assistant(
        "Результат: 1, 3, 4, 8, 9, 15, 17",
    ));

    let response = solver.solve_with_history("Теперь просуммируй 2, 4, 6", &history);

    assert_eq!(
        response.intent, "write_program",
        "a reduction follow-up must continue the coding context, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```javascript"),
        "reduction follow-up must inherit JavaScript, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Результат: 12"),
        "result must be the computed sum, got: {}",
        response.answer
    );
}

/// English parity: an English coding context followed by a bare English sort
/// follow-up inherits the language and renders an English result label.
#[test]
fn issue_412_english_followup_inherits_language() {
    let solver = UniversalSolver::default();
    let history = vec![
        ConversationTurn::user(
            "I have numbers 3, 5, 6, 7, 8 — sort them in Python, give me the code and the result",
        ),
        ConversationTurn::assistant("Result: 3, 5, 6, 7, 8"),
    ];

    let response = solver.solve_with_history("Sort 9, 2, 7, 1", &history);

    assert_eq!(response.intent, "write_program", "got: {}", response.answer);
    assert!(
        response.answer.contains("```python"),
        "answer must inherit Python, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Result: 1, 2, 7, 9"),
        "result must be sorted ascending, got: {}",
        response.answer
    );
}
