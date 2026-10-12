//! Observe auxiliary persistence before selecting the original target tool call.
use super::observed_plan_tools::{ToolWorkspace, observe_append};
use formal_ai::agentic_coding::general_planner::{GeneralPlanMode, compose_general_change_plan};
use formal_ai::agentic_coding::planner::{Capability, tool_capability};
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use serde_json::Value;

pub fn first_workspace_call(prompt: &str, tools: &[&str]) -> (String, Value) {
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
