//! Cross-surface and self-authorship coverage for issue #870.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, FormalAiEngine};

fn planned_command(prompt: &str) -> String {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["exec_command"])
        .expect("semantic process request should produce a plan");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("semantic process request should produce a tool call");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    arguments["command"]
        .as_str()
        .expect("shell command argument")
        .to_owned()
}

#[test]
fn reported_russian_process_request_reaches_agent_permission_flow() {
    let answer = FormalAiEngine.answer("Проверь какие процессы запущены на моём компьютере");
    assert_eq!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert!(
        answer.answer.contains("ps"),
        "answer should name ps: {answer:?}"
    );
    assert!(
        answer.links_notation.contains("terminal:command")
            && answer.links_notation.contains("command ps"),
        "answer should carry executable terminal evidence: {answer:?}"
    );
}

#[test]
fn process_requests_share_one_multilingual_semantic_route() {
    for (language, prompt) in [
        (
            "English (en)",
            "Check which processes are running on my computer",
        ),
        (
            "Russian (ru)",
            "Проверь какие процессы запущены на моём компьютере",
        ),
        ("Hindi (hi)", "जाँचें कौन सी प्रक्रियाएं चल रही हैं"),
        ("Chinese (zh)", "检查哪些进程正在运行"),
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_eq!(
            answer.intent, "agent_suggestion",
            "{language}: {prompt}: {answer:?}"
        );
        assert_eq!(planned_command(prompt), "ps", "{language}: {prompt}");
    }
}

#[test]
fn agent_cli_authored_regression_is_preserved_byte_for_byte() {
    const EXPECTED: &str = concat!(
        "//! Regression coverage for GitHub issue #870.\n",
        "\n",
        "use formal_ai::FormalAiEngine;\n",
        "\n",
        "const REPORTED_PROMPT: &str = \"Проверь какие процессы запущены на моём компьютере\";\n",
        "\n",
        "#[test]\n",
        "fn reported_russian_process_request_reaches_agent_permission_flow() {\n",
        "    let answer = FormalAiEngine.answer(REPORTED_PROMPT);\n",
        "    assert_eq!(answer.intent, \"agent_suggestion\", \"answer: {answer:?}\");\n",
        "    assert!(answer.answer.contains(\"ps\"), \"answer should name ps: {answer:?}\");\n",
        "    assert!(\n",
        "        answer.links_notation.contains(\"terminal:command\")\n",
        "            && answer.links_notation.contains(\"command ps\"),\n",
        "        \"answer should carry executable terminal evidence: {answer:?}\"\n",
        "    );\n",
        "}",
    );
    let actual =
        std::fs::read_to_string("docs/case-studies/issue-870/agent-cli-evidence/issue_870.rs")
            .expect("Agent CLI-authored regression should be committed");
    assert_eq!(actual, EXPECTED);
}
