//! Observe complete real plan-event execution before inspecting the requested target call.
use formal_ai::agentic_coding::general_planner::{
    GeneralPlanMode, PLAN_PATH, compose_general_change_plan,
};
use formal_ai::agentic_coding::planner::{Capability, tool_capability};
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use serde_json::Value;
#[path = "../unit/issue_1066_ladder_capability/tool_workspace.rs"]
mod tool_workspace;
pub(crate) use tool_workspace::ToolWorkspace;

pub(crate) fn observe_append(
    workspace: &mut ToolWorkspace,
    id: &str,
    call: &PlannedToolCall,
    goal: &str,
) -> ChatMessage {
    let observation = workspace.execute(id, call);
    assert!(!observation.is_error, "actual append provider must succeed");
    let receipt: Value =
        serde_json::from_str(&observation.content.plain_text()).expect("actual process receipt");
    assert_eq!(receipt["exit_code"], 0);
    assert_eq!(receipt["complete"], true);
    assert_eq!(receipt["timed_out"], false);
    let stored = workspace.read(PLAN_PATH).expect("physical appended event");
    let stdout = receipt["stdout"].as_str().expect("actual observed bytes");
    assert_eq!(
        stdout,
        stored.strip_prefix('\n').expect("append separator"),
        "whole appended record readback"
    );
    let parsed = formal_ai::seed::parse_lino(stdout);
    let event = parsed.children.first().expect("actual plan event");
    assert_eq!(event.name, "general_change_plan");
    assert_eq!(event.find_child_value("goal"), goal);
    observation
}

pub(crate) fn first_workspace_call(prompt: &str, tools: &[&str]) -> (String, Value) {
    let general = compose_general_change_plan(prompt)
        .filter(|plan| plan.mode == GeneralPlanMode::LiteralFile);
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut workspace = None;
    for turn in 0..12 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, tools) else {
            panic!("expected actual workspace call for {prompt:?}");
        };
        assert_eq!(calls.len(), 1);
        let call = &calls[0];
        let arguments: Value = serde_json::from_str(&call.arguments).expect("tool arguments");
        if tool_capability(&call.tool) != Some(Capability::Run) || general.is_none() {
            return (call.tool.clone(), arguments);
        }
        let id = format!("plan-event-{turn}");
        let workspace = workspace.get_or_insert_with(|| ToolWorkspace::new(prompt));
        let observation = observe_append(workspace, &id, call, prompt);
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(observation);
    }
    panic!("no target call after bounded actual plan persistence")
}
