//! Observe complete real plan-event execution before inspecting the requested target call.
use formal_ai::agentic_coding::PlannedToolCall;
use formal_ai::agentic_coding::general_planner::PLAN_PATH;
use formal_ai::protocol::ChatMessage;
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
