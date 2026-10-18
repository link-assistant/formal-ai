//! Issue #607: The agent CLI shell documentation, shell planner and serve opt-in are traceable.

use std::fs;
use std::path::Path;

#[test]
fn issue_607_agent_cli_shell_docs_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "formal-ai serve --agent-mode --host 127.0.0.1 --port 8080",
            "agent --model formalai/formal-ai --permission-mode plan",
            "run ls to list files here",
            "`bash` / `shell` / `run_command` `tool_calls`",
            "{\"command\":\"ls\"}",
            "hard `--read-only`",
        ],
    );

    let server_api = read(root.join("docs/desktop/server-api.md"));
    assert_contains_all(
        "docs/desktop/server-api.md",
        &server_api,
        &[
            "formal-ai serve --agent-mode --host 127.0.0.1 --port 8080",
            "FORMAL_AI_AGENT_MODE=1",
            "agent --model formalai/formal-ai --permission-mode plan",
            "`bash` / `shell` / `run_command` `tool_calls`",
            "{\"command\":\"ls\"}",
            "disable shell execution entirely",
        ],
    );
}

#[test]
fn issue_607_shell_planner_and_serve_opt_in_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");

    let planner = read(root.join("rust/src/agentic_coding/planner.rs"));
    let recovery = read(root.join("rust/src/agentic_coding/prerequisite_recovery.rs"));
    assert_contains_all(
        "planner delegates its shell transaction to the canonical recovery helper",
        &planner,
        &["super::prerequisite_recovery::plan_shell_step(messages, tool_names, command, result)"],
    );
    let shell_source = format!("{planner}\n{recovery}");
    assert_contains_all(
        "rust/src/agentic_coding/planner.rs",
        &shell_source,
        &[
            "shell_command_for_task",
            "plan_shell_step",
            "\"command\": command",
            "tool_result::render",
        ],
    );

    // Tool names belong to the shared capability seed, reached by the shell helper.
    assert_contains_all(
        "canonical shell helper",
        &recovery,
        &["tool_for(tool_names, Capability::Run)"],
    );
    let router = read(root.join("rust/src/agentic_coding/capability_router.rs"));
    assert_contains_all(
        "canonical capability owner",
        &router,
        &["seed::agentic_tool_capabilities()", ".aliases"],
    );
    let loader = read(root.join("rust/src/seed/agentic_tool_capabilities.rs"));
    assert_contains_all(
        "canonical capability loader",
        &loader,
        &["parse_lino(AGENTIC_TOOL_CAPABILITIES_LINO)"],
    );
    let registry = read(root.join("rust/src/seed/embedded_registry.rs"));
    assert_contains_all(
        "canonical capability embedding",
        &registry,
        &["../../embedded/data/seed/agentic-tool-capabilities.lino"],
    );
    for tool in ["bash", "shell", "run_command"] {
        use formal_ai::agentic_coding::{AgenticPlan, Capability, plan_chat_step, tool_capability};
        assert_eq!(tool_capability(tool), Some(Capability::Run), "{tool}");
        let messages = [formal_ai::ChatMessage::user("run ls to list files here")];
        match plan_chat_step(&messages, &[tool]) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].tool, tool);
                let arguments: serde_json::Value =
                    serde_json::from_str(&calls[0].arguments).unwrap();
                assert_eq!(arguments["command"], "ls");
            }
            other => panic!("the advertised {tool} owns the documented command: {other:?}"),
        }
    }

    let responses = read(root.join("data/seed/multilingual-responses.lino"));
    assert_contains_all(
        "data/seed/multilingual-responses.lino",
        &responses,
        &["The `{tool}` command completed"],
    );

    let server = read(root.join("rust/src/server.rs"));
    assert_contains_all(
        "rust/src/server.rs",
        &server,
        &[
            "enable_http_agent_mode_for_current_process",
            "HTTP_AGENT_MODE_FORCED",
            "config.agent_mode = true",
        ],
    );

    let cli = read(root.join("rust/src/main.rs"));
    assert_contains_all(
        "rust/src/main.rs",
        &cli,
        &[
            "agent_mode",
            "FORMAL_AI_AGENT_MODE",
            "enable_http_agent_mode_for_current_process",
        ],
    );
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
// Kept as a nested documentation-requirements suite by issue #1138 plan 11.
