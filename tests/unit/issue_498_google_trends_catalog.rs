use formal_ai::agentic_coding::{
    google_trends_catalog as recipe, is_google_trends_catalog_task, plan_chat_step,
    run_agentic_task, AgenticPlan, PlannedToolCall, DRIVER_TOOLS, GOOGLE_TRENDS_CATALOG_PATH,
    GOOGLE_TRENDS_CATALOG_TASK,
};
use formal_ai::{ChatMessage, ToolCall};
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
    let committed = include_str!("../../data/meta/google-trends-catalog.lino");
    assert_eq!(
        committed,
        recipe::render_document(),
        "the committed Trends catalog is stale; regenerate it from the issue-498 recipe",
    );
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
fn planner_walks_the_google_trends_catalog_recipe() {
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
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_google_trends_run() {
    let committed =
        include_str!("../../docs/case-studies/issue-498/agent-cli-session-google-trends.json");
    let fresh = run_agentic_task(GOOGLE_TRENDS_CATALOG_TASK).expect("workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).unwrap()
    );
    assert_eq!(
        committed, rendered,
        "the committed Google Trends Agent CLI session is stale",
    );
}

#[test]
fn driver_drives_the_google_trends_recipe_to_a_write() {
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
}
