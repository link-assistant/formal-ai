//! Regression coverage for proactive issue-report invitations after detected failures.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::{SolverConfig, UniversalSolver};

const REPORT_COMMAND: &str = "`Report issue`";

fn failed_tool_turn(prompt: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "call_864",
            "exec_command",
            r#"{"cmd":"false"}"#,
        )]),
        ChatMessage::tool_result(
            "call_864",
            "exec_command",
            r#"{"error":"exit status 1","exit_code":1}"#,
        ),
    ]
}

fn failed_tool_answer(prompt: &str) -> String {
    match plan_chat_step(&failed_tool_turn(prompt), &["exec_command"]) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected a final failure answer, got {other:?}"),
    }
}

fn plain_text_failed_tool_answer(prompt: &str) -> String {
    let mut messages = failed_tool_turn(prompt);
    messages.pop();
    messages.push(ChatMessage::tool_result(
        "call_864",
        "exec_command",
        "/bin/sh: 1: command: not found",
    ));
    match plan_chat_step(&messages, &["exec_command"]) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected a final plain-text failure answer, got {other:?}"),
    }
}

#[test]
fn detected_tool_failures_proactively_ask_to_report_on_agentic_harnesses() {
    let cases = [
        (
            "Run false",
            "Would you like me to prepare an issue report with the diagnostic context?",
        ),
        (
            "Выполни false",
            "Хотите, чтобы я подготовил отчёт о проблеме с диагностическим контекстом?",
        ),
        (
            "false चलाएँ",
            "क्या आप चाहेंगे कि मैं diagnostic context के साथ issue report तैयार करूँ?",
        ),
        ("执行 false", "需要我用诊断上下文准备问题报告吗？"),
        (
            "Por favor, ejecuta false",
            "¿Quieres que prepare un informe del problema con el contexto de diagnóstico?",
        ),
    ];

    for (prompt, invitation) in cases {
        let answer = failed_tool_answer(prompt);
        assert!(answer.contains(invitation), "prompt={prompt}: {answer}");
        assert!(answer.contains(REPORT_COMMAND), "prompt={prompt}: {answer}");
    }
}

#[test]
fn plain_text_agent_cli_failures_use_the_same_proactive_invitation() {
    let answer = plain_text_failed_tool_answer("Run command");
    assert!(answer.contains("The command failed:"), "{answer}");
    assert!(
        answer.contains("Would you like me to prepare an issue report"),
        "{answer}"
    );
    assert!(answer.contains(REPORT_COMMAND), "{answer}");
}

#[test]
fn unresolved_reasoning_proactively_asks_to_report_on_every_rust_surface_language() {
    let cases = [
        (
            "en",
            "Would you like me to prepare an issue report with the diagnostic context?",
        ),
        (
            "ru",
            "Хотите, чтобы я подготовил отчёт о проблеме с диагностическим контекстом?",
        ),
        (
            "hi",
            "क्या आप चाहेंगे कि मैं diagnostic context के साथ issue report तैयार करूँ?",
        ),
        ("zh", "需要我用诊断上下文准备问题报告吗？"),
    ];

    for (language, invitation) in cases {
        let solver = UniversalSolver::new(SolverConfig {
            forced_response_language: Some(language),
            offline: true,
            compute_budget: 0,
            ..SolverConfig::default()
        });
        let answer = solver.solve("Explain the issue-864 frobnicator ritual");
        assert_eq!(answer.intent, "unknown", "language={language}");
        assert!(
            answer.answer.contains(invitation),
            "language={language}: {}",
            answer.answer
        );
        assert!(
            answer.answer.contains(REPORT_COMMAND),
            "language={language}: {}",
            answer.answer
        );
    }

    let language = "es";
    let solver = UniversalSolver::new(SolverConfig {
        forced_response_language: Some(language),
        offline: true,
        compute_budget: 0,
        ..SolverConfig::default()
    });
    let answer = solver.solve("Explain the issue-864 frobnicator ritual");
    assert_eq!(answer.intent, "unknown");
    assert!(answer
        .answer
        .contains("Detecté un fallo mientras trabajaba en esta solicitud"));
    assert!(answer.answer.contains(REPORT_COMMAND));
}

#[test]
fn expected_tool_refusals_do_not_claim_formal_ai_detected_a_failure() {
    for result in [
        r#"{"ok":false,"executed":false,"status":"refused"}"#,
        r#"{"success":false,"state":"DENIED"}"#,
        r#"{"ok":false,"outcome":"cancelled"}"#,
    ] {
        let mut messages = failed_tool_turn("Run false");
        messages.pop();
        messages.push(ChatMessage::tool_result("call_864", "exec_command", result));
        let answer = match plan_chat_step(&messages, &["exec_command"]) {
            Some(AgenticPlan::Final(answer)) => answer,
            other => panic!("expected a final expected-stop answer, got {other:?}"),
        };
        assert!(
            !answer.contains("Would you like me to prepare an issue report"),
            "result={result}: {answer}"
        );
        assert!(
            !answer.contains(REPORT_COMMAND),
            "result={result}: {answer}"
        );
    }
}

#[test]
fn explicit_unsuccessful_results_invite_reports_but_pending_results_do_not() {
    for (result, should_invite) in [
        (r#"{"ok":false}"#, true),
        (r#"{"success":false}"#, true),
        (r#"{"ok":false,"status":"awaiting_approval"}"#, false),
    ] {
        let mut messages = failed_tool_turn("Run command");
        messages.pop();
        messages.push(ChatMessage::tool_result("call_864", "exec_command", result));
        let answer = match plan_chat_step(&messages, &["exec_command"]) {
            Some(AgenticPlan::Final(answer)) => answer,
            other => panic!("expected a final tool answer, got {other:?}"),
        };
        assert_eq!(
            answer.contains(REPORT_COMMAND),
            should_invite,
            "result={result}: {answer}"
        );
    }
}
