//! Regression coverage for GitHub issue #870.

use formal_ai::FormalAiEngine;

const REPORTED_PROMPT: &str = "Проверь какие процессы запущены на моём компьютере";

#[test]
fn reported_russian_process_request_reaches_agent_permission_flow() {
    let answer = FormalAiEngine.answer(REPORTED_PROMPT);
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert!(answer.answer.contains("ps"), "answer should name ps: {answer:?}");
    assert!(
        answer.links_notation.contains("terminal:command")
            && answer.links_notation.contains("command ps"),
        "answer should carry executable terminal evidence: {answer:?}"
    );
}