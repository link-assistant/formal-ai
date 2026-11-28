use crate::issue_498_google_trends::{captured_feed, with_trends_replay};
use formal_ai::agentic_coding::{
    AgenticPlan, DRIVER_TOOLS, GOOGLE_TRENDS_CATALOG_PATH, GOOGLE_TRENDS_CATALOG_TASK,
    GOOGLE_TRENDS_LEARNING_PATH, PlannedToolCall, google_trends_catalog as recipe,
    is_google_trends_catalog_task, is_google_trends_learning_task, plan_chat_step,
    run_agentic_task,
};
use formal_ai::{ChatMessage, ToolCall, answer_google_trends_catalog};
use lino_objects_codec::format::parse_indented;

fn expect_single_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert_eq!(calls.len(), 1, "the planner emits one call per step");
            calls.remove(0)
        }
        other => panic!("expected a single tool call, got {other:?}"),
    }
}

fn answer_tool_call(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

#[test]
fn committed_google_trends_catalog_is_generated_by_the_recipe() {
    let committed =
        include_str!("../../../../docs/case-studies/issue-498/replay/google-trends-catalog.lino");
    let generated = recipe::render_catalog(&answer_google_trends_catalog(captured_feed()));
    parse_indented(&generated).expect("the derived catalog should parse");
    assert!(generated.contains("source_sha256"));
    for field in ["topic_count", "prompt_count", "answered_count"] {
        let from_history = committed
            .lines()
            .find(|line| line.trim_start().starts_with(field));
        let from_replay = generated
            .lines()
            .find(|line| line.trim_start().starts_with(field));
        assert_eq!(from_replay, from_history, "{field}");
    }
    parse_indented(committed).expect("the catalog should parse as Links Notation");
    assert!(committed.contains("google_trends_catalog"));
    assert!(committed.contains("topic_count \"10\""));
    assert!(committed.contains("prompt_language \"ru\""));
    assert!(committed.contains("prompt_language \"hi\""));
    assert!(committed.contains("prompt_language \"zh\""));
}

#[test]
fn recognises_the_google_trends_catalog_task_without_colliding_with_siblings() {
    assert!(is_google_trends_catalog_task(GOOGLE_TRENDS_CATALOG_TASK));
    assert!(is_google_trends_catalog_task(
        "Build a Google Trends catalog from the top searches and answer each request."
    ));
    assert!(is_google_trends_catalog_task(
        "Convert trending searches into multilingual Formal AI test prompts."
    ));

    assert!(!is_google_trends_catalog_task("what is formal ai?"));
    assert!(!is_google_trends_catalog_task(
        formal_ai::agentic_coding::QUESTION_CATALOG_TASK
    ));
    assert!(!formal_ai::agentic_coding::is_question_catalog_task(
        GOOGLE_TRENDS_CATALOG_TASK
    ));
}

#[test]
fn learning_frontier_takes_precedence_when_the_request_also_names_the_catalog() {
    let request = "Map the Google Trends learning frontier for unresolved catalog prompts.";
    assert!(is_google_trends_learning_task(request));
    assert!(is_google_trends_catalog_task(request));

    with_trends_replay(|| {
        let call = expect_single_call(&[ChatMessage::user(request)], &["write_file"]);
        assert_eq!(call.tool, "write_file");
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(arguments["path"], GOOGLE_TRENDS_LEARNING_PATH);
    });
}

#[test]
fn planner_walks_the_google_trends_catalog_recipe() {
    with_trends_replay(|| {
        let tools = ["web_search", "web_fetch", "write_file", "run_command"];
        let mut messages = vec![ChatMessage::user(GOOGLE_TRENDS_CATALOG_TASK)];

        let call = expect_single_call(&messages, &tools);
        assert_eq!(call.tool, "write_file");
        assert!(call.arguments.contains(GOOGLE_TRENDS_CATALOG_PATH));
        let written: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(written["content"], recipe::render_document());
        answer_tool_call(&mut messages, &call, "wrote google-trends-catalog.lino");

        let call = expect_single_call(&messages, &tools);
        assert_eq!(call.tool, "run_command");
        assert!(call.arguments.contains(GOOGLE_TRENDS_CATALOG_PATH));
        answer_tool_call(&mut messages, &call, &recipe::render_document());

        match plan_chat_step(&messages, &tools) {
            Some(AgenticPlan::Final(answer)) => {
                assert!(answer.contains(GOOGLE_TRENDS_CATALOG_PATH));
                assert!(answer.contains("Google Trends"));
            }
            other => panic!("expected a final answer, got {other:?}"),
        }
    });
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_google_trends_run() {
    let committed = include_str!(
        "../../../../docs/case-studies/issue-498/agent-cli-session-google-trends.json"
    );
    let captured: serde_json::Value =
        serde_json::from_str(committed).expect("captured session JSON");
    let tools: Vec<&str> = captured["tools_advertised"]
        .as_array()
        .expect("captured tool schema")
        .iter()
        .map(|tool| tool.as_str().expect("captured tool name"))
        .collect();
    assert_eq!(captured["task"].as_str(), Some(GOOGLE_TRENDS_CATALOG_TASK));
    let steps = captured["steps"].as_array().expect("complete native steps");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["tool"].as_str(), Some("write_file"));
    let old = steps[0]["arguments"]["content"]
        .as_str()
        .expect("historic report");
    parse_indented(old).expect("historic Links Notation");
    assert_eq!(steps[1]["tool"].as_str(), Some("run_command"));
    assert_eq!(
        steps[1]["arguments"]["command"].as_str(),
        Some(recipe::verification_command().as_str())
    );
    let receipt: serde_json::Value =
        serde_json::from_str(steps[1]["result"].as_str().expect("typed command receipt"))
            .expect("valid native command receipt JSON");
    assert_eq!(
        receipt["schema"].as_str(),
        Some("command-execution-receipt/v1")
    );
    assert_eq!(receipt["exit_code"].as_i64(), Some(0));

    let fresh = with_trends_replay(|| {
        formal_ai::agentic_coding::run_agentic_task_with_tools(GOOGLE_TRENDS_CATALOG_TASK, &tools)
            .expect("workspace under the captured tool schema")
    });
    assert_eq!(fresh.steps.len(), steps.len());
    assert!(fresh.steps[0].arguments.contains("source_sha256"));
}

#[test]
fn driver_drives_the_google_trends_recipe_to_a_write() {
    with_trends_replay(|| {
        assert!(DRIVER_TOOLS.contains(&"write_file"));
        let outcome = run_agentic_task(GOOGLE_TRENDS_CATALOG_TASK).expect("workspace");
        assert!(!outcome.hit_turn_cap, "the loop must finish");

        let write = outcome
            .steps
            .iter()
            .find(|step| step.tool == "write_file")
            .expect("a write step");
        let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
        assert_eq!(written["content"], recipe::render_document());
        assert_eq!(written["path"], GOOGLE_TRENDS_CATALOG_PATH);
    });
}
