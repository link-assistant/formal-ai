// Issue #513: terminal-command requests must resolve to an `agent_suggestion`
// intent in the Rust engine (parity with the JS worker), instead of falling
// through to the `unknown` fallback.

use formal_ai::FormalAiEngine;

#[test]
fn russian_terminal_request_is_not_unknown() {
    let answer = FormalAiEngine.answer("Выполни `ls ~` в терминале");
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert!(
        answer.answer.contains("ls ~"),
        "answer should name the command: {}",
        answer.answer
    );
}

#[test]
fn english_terminal_request_is_not_unknown() {
    let answer = FormalAiEngine.answer("run `ls ~` in terminal");
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert!(
        answer.answer.to_lowercase().contains("agent mode"),
        "answer should explain agent mode: {}",
        answer.answer
    );
}

#[test]
fn hindi_terminal_request_is_not_unknown() {
    // language: hi
    let answer = FormalAiEngine.answer("टर्मिनल में `ls ~` चलाओ");
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert!(
        answer.answer.contains("ls ~"),
        "answer should name the command: {}",
        answer.answer
    );
}

#[test]
fn chinese_terminal_request_is_not_unknown() {
    // language: zh
    let answer = FormalAiEngine.answer("在终端中运行 `ls ~`");
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert!(
        answer.answer.contains("ls ~"),
        "answer should name the command: {}",
        answer.answer
    );
}

#[test]
fn leading_shell_token_is_recognized() {
    let answer = FormalAiEngine.answer("git status");
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
}

#[test]
fn plain_prose_is_not_misclassified() {
    let answer = FormalAiEngine.answer("run a marathon next year");
    assert_ne!(answer.intent, "agent_suggestion", "answer: {answer:?}");
}
