//! Issue #703 controller boundaries the other #703 suites leave implicit.
//!
//! R703-3: two passing leaves that leave different bytes at the same path
//! cannot both be right, so the controller refuses the composition by naming
//! the path and leaves the granted workspace untouched, instead of letting the
//! later leaf silently overwrite the earlier one.
//!
//! R703-5: a requested custom entrypoint must say where the task goes; an argv
//! without `{task}` is refused before anything runs, even when its program is
//! explicitly granted.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::process::Command;

use formal_ai::orchestration::{AgentCommand, AgentRunPermission, DispatchConfig, dispatch_agents};

use super::issue_703_orchestration::TestWorkspace;

#[test]
fn leaves_writing_different_bytes_to_one_path_are_refused_by_name() {
    let bin = TestWorkspace::new("conflict-bin");
    let workspace = TestWorkspace::new("conflict");
    // Every leaf writes its own task text to the same file: two leaves, two
    // different `after_sha256` values for README.md.
    let script = bin.path().join("leaf-writer");
    fs::write(&script, "#!/bin/sh\nprintf '%s\\n' \"$1\" > README.md\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let command = AgentCommand::new(&script).arg("{task}");

    let mut config = DispatchConfig::new(
        "Create a README badge and add a release note.",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .allowlisted_agent_commands
        .insert(command.program.to_string_lossy().into_owned());
    config
        .command_overrides
        .insert("codex".to_string(), command);

    let Err(error) = dispatch_agents(&config) else {
        panic!("conflicting leaves must not compose");
    };
    assert_eq!(error.to_string(), "composition_conflict:README.md");
    assert!(
        !workspace.path().join("README.md").exists(),
        "a refused composition must not apply either leaf's effect"
    );
}

#[test]
fn a_custom_entrypoint_without_a_task_placeholder_is_refused_before_it_runs() {
    let workspace = TestWorkspace::new("custom-no-task");
    let session_path = workspace.path().join("session.json");
    let command = serde_json::to_string(&["sh", "-c", "printf ran > ran.txt"]).unwrap();

    let refused = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "agent",
            "run",
            "--cli",
            "private-neural-agent",
            "--target",
            "vendor",
            "--task",
            "answer through bash",
            "--workspace",
        ])
        .arg(workspace.path())
        .args([
            "--command",
            &command,
            "--allow-agent-command",
            "sh",
            "--session",
        ])
        .arg(&session_path)
        .output()
        .unwrap();

    assert!(!refused.status.success());
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(stderr.contains("missing_task_placeholder"), "{stderr}");
    assert!(
        !session_path.exists(),
        "a refused entrypoint records no session"
    );
    assert!(
        !workspace.path().join("ran.txt").exists(),
        "a refused entrypoint never runs"
    );
}
