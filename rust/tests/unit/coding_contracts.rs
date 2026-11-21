//! Replay the complete typed composition and search contracts in the focused lane.

mod coding_discovery {
    mod composition;
    mod composition_search;
}

// Preserve the existing planner and transport fixtures in the hosted coding lane.
#[path = "agentic-coding/agentic_general_planner.rs"]
mod agentic_general_planner;
#[path = "../fixtures/observed-plan-tools.rs"]
mod observed_plan_tools;
#[path = "issue_1066_ladder_capability/tool_workspace.rs"]
mod tool_workspace;
