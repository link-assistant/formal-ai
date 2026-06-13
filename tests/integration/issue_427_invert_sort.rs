//! Issue #427: after a numeric-list sort, "Сделай инверсию сортировки." (make
//! the inversion of the sort) answered `unknown`.
//!
//! The conversation established a JavaScript numeric-list coding context and
//! sorted a concrete list ascending. The bare follow-up names an operation
//! (invert the sort → descending) but no numbers — it refers to the list from
//! the previous turn. The solver must inherit both the language and the list,
//! apply the reverse sort, and emit code plus the descending result instead of
//! falling through to `unknown`.

use formal_ai::{ConversationTurn, UniversalSolver};

/// The active coding context: JavaScript, a concrete list sorted ascending.
fn javascript_sort_context() -> Vec<ConversationTurn> {
    vec![
        ConversationTurn::user(
            "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
        ),
        ConversationTurn::assistant(
            "Вот код на JavaScript, который сортирует числа 3, 5, 6, 7, 8 по возрастанию:\n\n```javascript\nconst numbers = [3, 5, 6, 7, 8];\nconst sorted = [...numbers].sort((a, b) => a - b);\nconsole.log(sorted.join(\", \"));\n```\n\nРезультат: 3, 5, 6, 7, 8",
        ),
        ConversationTurn::user("Отсортируй 3, 1, 2"),
        ConversationTurn::assistant(
            "Вот код на JavaScript, который сортирует числа 3, 1, 2 по возрастанию:\n\n```javascript\nconst numbers = [3, 1, 2];\nconst sorted = [...numbers].sort((a, b) => a - b);\nconsole.log(sorted.join(\", \"));\n```\n\nРезультат: 1, 2, 3",
        ),
    ]
}

#[test]
fn issue_427_invert_sort_followup_is_not_unknown() {
    let solver = UniversalSolver::default();
    let response =
        solver.solve_with_history("Сделай инверсию сортировки.", &javascript_sort_context());

    assert_eq!(
        response.intent, "write_program",
        "the invert-sort follow-up must continue the coding context, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```javascript"),
        "answer must inherit the JavaScript language, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("const numbers = [3, 1, 2];"),
        "code must inherit the previous list, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Результат: 3, 2, 1"),
        "result must be the inherited list sorted descending, got: {}",
        response.answer
    );
}

/// English parity: the same bare invert-sort follow-up over an English Python
/// coding context inherits the language and the list, then renders the
/// descending result.
#[test]
fn issue_427_english_invert_sort_followup_inherits_language_and_list() {
    let solver = UniversalSolver::default();
    let history = vec![
        ConversationTurn::user(
            "I have numbers 4, 1, 3 — sort them in Python, give me the code and the result",
        ),
        ConversationTurn::assistant("Result: 1, 3, 4"),
    ];

    let response = solver.solve_with_history("Invert the sort.", &history);

    assert_eq!(
        response.intent, "write_program",
        "the English invert-sort follow-up must continue the coding context, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```python"),
        "answer must inherit Python, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Result: 4, 3, 1"),
        "result must be the inherited list sorted descending, got: {}",
        response.answer
    );
}

/// Hindi parity: a bare invert-sort follow-up over a Hindi Python coding
/// context inherits the language and the list, then renders the descending
/// result. Exercises the `combo उलट+क्रम` `reverse_sort` vocabulary (issue #427).
#[test]
fn issue_427_hindi_invert_sort_followup_inherits_language_and_list() {
    let solver = UniversalSolver::default();
    let history = vec![
        ConversationTurn::user(
            "मेरे पास संख्याएँ 4, 1, 3 हैं — उन्हें Python में क्रमबद्ध करो, मुझे कोड और परिणाम दो",
        ),
        ConversationTurn::assistant("परिणाम: 1, 3, 4"),
    ];

    let response = solver.solve_with_history("क्रम उलट दो।", &history);

    assert_eq!(
        response.intent, "write_program",
        "the Hindi invert-sort follow-up must continue the coding context, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```python"),
        "answer must inherit Python, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("परिणाम: 4, 3, 1"),
        "result must be the inherited list sorted descending, got: {}",
        response.answer
    );
}

/// Chinese parity: a bare invert-sort follow-up over a Chinese Python coding
/// context inherits the language and the list, then renders the descending
/// result. Exercises the `combo 反转+排序` `reverse_sort` vocabulary (issue #427).
#[test]
fn issue_427_chinese_invert_sort_followup_inherits_language_and_list() {
    let solver = UniversalSolver::default();
    let history = vec![
        ConversationTurn::user("我有数字 4, 1, 3 — 用 Python 排序，给我代码和结果"),
        ConversationTurn::assistant("结果: 1, 3, 4"),
    ];

    let response = solver.solve_with_history("反转排序。", &history);

    assert_eq!(
        response.intent, "write_program",
        "the Chinese invert-sort follow-up must continue the coding context, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```python"),
        "answer must inherit Python, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("结果: 4, 3, 1"),
        "result must be the inherited list sorted descending, got: {}",
        response.answer
    );
}

/// Without any prior numeric-list context, a bare invert-sort prompt names no
/// list and no language, so it must not fabricate a program (it stays out of
/// the `write_program` intent).
#[test]
fn issue_427_invert_sort_without_context_does_not_fabricate() {
    let solver = UniversalSolver::default();
    let response = solver.solve("Сделай инверсию сортировки.");

    assert_ne!(
        response.intent, "write_program",
        "a bare invert-sort with no established list must not invent one, got: {}",
        response.answer
    );
}
