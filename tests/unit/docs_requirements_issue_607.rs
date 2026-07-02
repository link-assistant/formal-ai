use std::fs;
use std::path::Path;

#[test]
fn issue_607_agent_cli_shell_docs_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "formal-ai serve --agent-mode --host 127.0.0.1 --port 8080",
            "agent --model formal-ai/formal-ai --permission-mode plan",
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
            "agent --model formal-ai/formal-ai --permission-mode plan",
            "`bash` / `shell` / `run_command` `tool_calls`",
            "{\"command\":\"ls\"}",
            "disable shell execution entirely",
        ],
    );
}

#[test]
fn issue_607_shell_planner_and_serve_opt_in_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let planner = read(root.join("src/agentic_coding/planner.rs"));
    assert_contains_all(
        "src/agentic_coding/planner.rs",
        &planner,
        &[
            "shell_command_for_task",
            "plan_shell_step",
            "bash",
            "shell",
            "run_command",
            "\"command\": command",
            "The `{command}` command completed",
        ],
    );

    let server = read(root.join("src/server.rs"));
    assert_contains_all(
        "src/server.rs",
        &server,
        &[
            "enable_http_agent_mode_for_current_process",
            "HTTP_AGENT_MODE_FORCED",
            "config.agent_mode = true",
        ],
    );

    let cli = read(root.join("src/main.rs"));
    assert_contains_all(
        "src/main.rs",
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
