use formal_ai::agentic_coding::planner::tool_capability;
use formal_ai::agentic_coding::{plan_symbolic_command_reroute, AgenticPlan};
use formal_ai::protocol::ChatMessage;
use formal_ai::solver::{SolverConfig, UniversalSolver};
use serde_json::Value;

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

#[test]
fn changelog_bytes_match_the_agent_cli_write_call() {
    const SESSION_ID: &str = "ses_040c62ea9ffe21iqJOIzaJL4Xe";
    const TARGET: &str = "/changelog.d/20260801_212200_codex_tool_routing.md";
    let log =
        include_str!("../../docs/case-studies/issue-859/raw-data/agent-cli-changelog-session.log");
    let rows = log
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect::<Vec<_>>();
    assert!(
        rows.iter()
            .any(|row| row.get("session_id").and_then(Value::as_str) == Some(SESSION_ID)),
        "the evidence must name the authored session"
    );
    let authored = rows
        .iter()
        .find(|row| {
            row.get("type").and_then(Value::as_str) == Some("tool_use")
                && row.get("name").and_then(Value::as_str) == Some("write")
                && row
                    .pointer("/input/filePath")
                    .and_then(Value::as_str)
                    .is_some_and(|path| path.ends_with(TARGET))
        })
        .and_then(|row| row.pointer("/input/content"))
        .and_then(Value::as_str)
        .expect("Agent write call for the changelog");

    assert_eq!(
        include_str!("../../changelog.d/20260801_212200_codex_tool_routing.md"),
        authored
    );
}
