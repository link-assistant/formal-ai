use formal_ai::agentic_coding::planner::tool_capability;
use formal_ai::agentic_coding::{plan_symbolic_command_reroute, AgenticPlan};
use formal_ai::protocol::ChatMessage;
use formal_ai::solver::{SolverConfig, UniversalSolver};

#[test]
fn codex_process_input_is_not_a_workspace_file_writer() {
    assert_eq!(
        tool_capability("write_stdin"),
        None,
        "writing to a running process must not satisfy a file-write recipe"
    );
}

#[test]
fn codex_hello_world_starts_with_apply_patch_instead_of_write_stdin() {
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve("Give me hello world program in Rust");
    let plan = plan_symbolic_command_reroute(
        &[ChatMessage::user("Give me hello world program in Rust")],
        &["exec_command", "write_stdin", "apply_patch"],
        &answer,
    );
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("Codex should receive a workspace creation tool call: {plan:?}");
    };

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "apply_patch");
}
