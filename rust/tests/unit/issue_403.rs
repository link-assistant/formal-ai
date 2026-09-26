//! Issue #403: a Russian hidden-number interval prompt is a reasoning task.
//! The solver should formalize the bounds and use the proof machinery instead
//! of falling through to the unknown-intent response.

use formal_ai::FormalAiEngine;

#[test]
fn russian_interval_number_riddle_returns_formal_reasoning() {
    let response = FormalAiEngine.answer("Я загадал число больше 1 но меньше 3. что это за число?");

    assert_eq!(response.intent, "number_constraint_reasoning");
    assert!(
        !response.answer.contains("не удаётся сопоставить"),
        "interval riddle should not return the Russian unknown fallback, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains('2'),
        "answer should identify 2 as the integer solution, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("x > 1") && response.answer.contains("x < 3"),
        "answer should expose the formalized linear constraints, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("цел")
            && (response.answer.contains("веществен") || response.answer.contains("реальн")),
        "answer should distinguish integer and real-number readings, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("relative-meta-logic")
            || response.answer.contains("linear real arithmetic")
            || response.answer.contains("SMT"),
        "answer should cite the formal proof/decision procedure, got: {}",
        response.answer
    );
}
