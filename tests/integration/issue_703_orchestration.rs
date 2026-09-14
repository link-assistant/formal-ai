use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::orchestration::{
    AgentCommand, AgentRunConfig, AgentRunError, AgentRunPermission, AgentStatus, DispatchConfig,
    replay_session, run_agent,
};
use formal_ai::protocol::{ChatMessage, ToolCall};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const FIXTURE_ENV: &str = "FORMAL_AI_ISSUE_703_FIXTURE";
pub const FIXTURE_RELEASE_ENV: &str = "FORMAL_AI_ISSUE_703_RELEASE";
pub const FIXTURE_STARTED_ENV: &str = "FORMAL_AI_ISSUE_703_STARTED";
const FIXTURE_TEST: &str = "issue_703_orchestration::external_agent_fixture_process";
const REQUIRED_CLIS: [&str; 6] = ["agent", "claude", "codex", "gemini", "qwen", "opencode"];

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);

#[test]
fn exact_readme_badge_task_routes_to_the_real_codex_run_surface() {
    let messages = vec![ChatMessage::user("add a README badge")];
    let AgenticPlan::ToolCalls(calls) = plan_chat_step(&messages, &["exec_command", "apply_patch"])
        .expect("the issue acceptance task must produce a Codex tool call")
    else {
        panic!("the issue acceptance task must execute instead of returning prose");
    };

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "exec_command");
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("valid Codex tool arguments");
    let command = arguments["command"]
        .as_str()
        .expect("Codex command argument");
    assert!(command.contains("README.md"));
    assert!(command.contains("img.shields.io"));
}

#[test]
fn exact_readme_badge_task_adapts_to_geminis_file_tools() {
    let tools = ["read_file", "write_file", "replace"];
    let mut messages = vec![ChatMessage::user("add a README badge")];
    let read = match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1);
            calls[0].clone()
        }
        other => panic!("expected Gemini read_file call, got {other:?}"),
    };
    assert_eq!(read.tool, "read_file");

    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read_badge",
        &read.tool,
        &read.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "read_badge",
        "read_file",
        "# Existing title\n\nExisting introduction.\n",
    ));
    let write = match plan_chat_step(&messages, &tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1);
            calls[0].clone()
        }
        other => panic!("expected Gemini write_file call, got {other:?}"),
    };
    assert_eq!(write.tool, "write_file");
    let arguments: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    let content = arguments["content"].as_str().unwrap();
    assert!(content.starts_with("# Existing title\n\nExisting introduction.\n"));
    assert!(content.contains("img.shields.io"));
}

#[test]
fn exact_readme_badge_task_discovers_qwens_deferred_shell_tool() {
    let initial_tools = ["tool_search", "read_file", "list_directory"];
    let messages = vec![ChatMessage::user("add a README badge")];
    let discovery = match plan_chat_step(&messages, &initial_tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1);
            calls[0].clone()
        }
        other => panic!("expected Qwen tool_search call, got {other:?}"),
    };
    assert_eq!(discovery.tool, "tool_search");
    let arguments: serde_json::Value = serde_json::from_str(&discovery.arguments).unwrap();
    assert!(
        arguments["query"]
            .as_str()
            .unwrap()
            .contains("run_shell_command")
    );

    let messages = vec![
        ChatMessage::user("add a README badge"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "discover_shell",
            &discovery.tool,
            &discovery.arguments,
        )]),
        ChatMessage::tool_result(
            "discover_shell",
            "tool_search",
            "<functions>run_shell_command</functions>",
        ),
    ];
    let shell = match plan_chat_step(
        &messages,
        &["tool_search", "read_file", "run_shell_command"],
    ) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1);
            calls[0].clone()
        }
        other => panic!("expected discovered Qwen shell call, got {other:?}"),
    };
    assert_eq!(shell.tool, "run_shell_command");
    assert!(shell.arguments.contains("img.shields.io"));
}

#[test]
fn qwen_orchestration_uses_noninteractive_auto_edit_config() {
    let qwen = formal_ai::seed::client_integrations()
        .into_iter()
        .find(|client| client.id == "qwen")
        .expect("qwen client integration");

    assert_eq!(qwen.invocation.temp_home_config_path, ".qwen/settings.json");
    assert_eq!(
        qwen.invocation
            .temp_home_json_settings
            .iter()
            .find(|(key, _)| key == "tools.approvalMode")
            .map(|(_, value)| value.as_str()),
        Some("auto-edit")
    );
}

#[test]
fn seed_registry_pins_each_clients_exact_native_resume_contract() {
    let expected = [
        (
            "agent",
            "agent --resume session-703 --no-fork",
            &["--resume", "{session_id}", "--no-fork"][..],
        ),
        (
            "claude",
            "claude --resume session-703",
            &["--resume", "{session_id}"][..],
        ),
        (
            "codex",
            "codex exec resume session-703",
            &["resume", "{session_id}"][..],
        ),
        (
            "gemini",
            "gemini --resume session-703",
            &["--resume", "{session_id}"][..],
        ),
        (
            "opencode",
            "opencode --session session-703",
            &["--session", "{session_id}"][..],
        ),
        (
            "qwen",
            "qwen --resume session-703",
            &["--resume", "{session_id}"][..],
        ),
    ];
    let integrations = formal_ai::seed::client_integrations();

    for (cli, display, argv) in expected {
        let integration = integrations
            .iter()
            .find(|integration| integration.id == cli)
            .unwrap_or_else(|| panic!("missing {cli} adapter"));
        assert_eq!(
            integration
                .invocation
                .resume_command
                .replace("{session_id}", "session-703"),
            display
        );
        assert_eq!(
            integration
                .invocation
                .resume_args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            argv
        );
    }
}

#[test]
fn external_agent_exports_the_canonical_workspace_as_pwd() {
    let workspace = TestWorkspace::new("canonical-pwd");
    let session = run_agent(&fixture_config("opencode", workspace.path(), "assert_pwd"))
        .expect("fixture process");

    assert_eq!(session.status, AgentStatus::Succeeded, "{}", session.stderr);
}

#[test]
fn external_agent_run_requires_an_explicit_permission_grant() {
    let workspace = TestWorkspace::new("denied");
    let config = AgentRunConfig::new("codex", "add a README badge", workspace.path());

    let error = run_agent(&config).expect_err("an ungranted external process must not start");

    assert!(matches!(error, AgentRunError::PermissionDenied));
}

#[test]
fn arbitrary_agent_program_requires_a_separate_explicit_grant() {
    let workspace = TestWorkspace::new("custom-agent-grant");
    let command = fixture_command("success");
    let program = command.program.to_string_lossy().into_owned();
    let mut config = AgentRunConfig::new("private-neural-tui", "answer the task", workspace.path())
        .with_permission(AgentRunPermission::grant_for(workspace.path()))
        .with_command(command);

    let denied = run_agent(&config).expect_err("workspace permission is not a program grant");
    assert!(matches!(
        denied,
        AgentRunError::AgentCommandNotAllowlisted(command) if command == program
    ));

    config.allowlisted_agent_commands.insert(program);
    let session = run_agent(&config).expect("the exact custom argv is now explicitly granted");
    assert_eq!(session.cli, "private-neural-tui");
    assert!(session.passed());
    assert!(
        session
            .events
            .iter()
            .any(|event| event.kind == "custom_adapter_granted")
    );
}

#[test]
fn a_registered_cli_label_does_not_bypass_the_custom_program_grant() {
    let workspace = TestWorkspace::new("registered-label-custom-grant");
    let command =
        fixture_command("success").env("FORMAL_AI_ISSUE_703_OUTPUT", r#"{"session_id":"forged"}"#);
    let program = command.program.to_string_lossy().into_owned();
    let mut config = AgentRunConfig::new("codex", "answer the task", workspace.path())
        .with_permission(AgentRunPermission::grant_for(workspace.path()))
        .with_command(command);

    let error = run_agent(&config).expect_err("the executable grant applies to every override");

    assert!(matches!(
        error,
        AgentRunError::AgentCommandNotAllowlisted(command) if command == program
    ));

    config.allowlisted_agent_commands.insert(program.clone());
    let session = run_agent(&config).expect("the explicitly granted override runs");
    assert!(session.native_session.is_none());
    assert!(
        session
            .events
            .iter()
            .any(|event| { event.kind == "custom_adapter_granted" && event.detail == program })
    );
    assert!(
        session
            .events
            .iter()
            .any(|event| event.kind == "process_started" && event.detail == program)
    );
}

#[test]
fn external_agent_permission_is_scoped_to_one_workspace() {
    let granted = TestWorkspace::new("granted");
    let other = TestWorkspace::new("not-granted");
    let config = AgentRunConfig::new("codex", "add a README badge", other.path())
        .with_permission(AgentRunPermission::grant_for(granted.path()));

    let error = run_agent(&config).expect_err("a grant for another workspace must not apply");

    assert!(matches!(error, AgentRunError::PermissionDenied));
}

#[test]
fn all_required_seed_adapters_capture_process_and_workspace_events() {
    for cli in REQUIRED_CLIS {
        let workspace = TestWorkspace::new(cli);
        fs::write(workspace.path().join("README.md"), "before\n").unwrap();
        let config = fixture_config(cli, workspace.path(), "success");

        let session = run_agent(&config).expect("registered fake-backed adapter");

        assert_eq!(session.cli, cli);
        assert_eq!(session.status, AgentStatus::Succeeded);
        assert!(session.stdout.contains("fixture_stdout"));
        assert!(session.stderr.contains("fixture_stderr"));
        assert_eq!(session.changes.len(), 1);
        assert_eq!(session.changes[0].path, "README.md");
        assert!(
            session
                .events
                .iter()
                .any(|event| event.kind == "workspace_effect")
        );
    }
}

#[cfg(unix)]
#[test]
fn all_six_cli_entrypoints_run_a_scripted_repo_task_through_the_real_wrapper() {
    use std::os::unix::fs::PermissionsExt as _;

    let fake_bin = TestWorkspace::new("cli-bin");
    for cli in REQUIRED_CLIS {
        let path = fake_bin.path().join(cli);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf 'scripted {cli} %s\\n' \"$*\"\nprintf '# scripted badge\\n' > README.md\n"
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = std::env::join_paths(std::iter::once(fake_bin.path().to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();

    for cli in REQUIRED_CLIS {
        let workspace = TestWorkspace::new(&format!("cli-{cli}"));
        fs::write(workspace.path().join("README.md"), "before\n").unwrap();
        let session_path = workspace.path().join("session.json");
        let (base_url, server) = one_request_health_server();
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "agent",
                "run",
                "--cli",
                cli,
                "--task",
                "add a README badge",
                "--workspace",
            ])
            .arg(workspace.path())
            .args(["--base-url", &base_url, "--session"])
            .arg(&session_path)
            .env("PATH", &path)
            .output()
            .unwrap();
        server.join().unwrap();

        assert!(
            output.status.success(),
            "{cli}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let bytes = fs::read(&session_path).unwrap();
        let session = replay_session(&bytes).unwrap();
        assert_eq!(session.cli, cli);
        assert!(session.passed());
        assert!(session.stdout.contains(&format!("scripted {cli}")));
        match cli {
            "agent" | "gemini" | "qwen" => {
                assert!(session.stdout.contains("-p add a README badge"));
                if cli == "agent" {
                    assert!(session.stdout.contains("--no-retry-on-rate-limits"));
                }
            }
            "claude" => {
                assert!(session.stdout.contains("--print add a README badge"));
            }
            "codex" => {
                assert!(session.stdout.contains("--sandbox workspace-write"));
                assert!(!session.stdout.contains("--sandbox read-only"));
            }
            "opencode" => {
                assert!(session.stdout.contains("run -m formalai/formal-ai"));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            fs::read_to_string(workspace.path().join("README.md")).unwrap(),
            "# scripted badge\n"
        );
        assert!(
            session
                .changes
                .iter()
                .any(|change| change.path == "README.md")
        );
    }
}

#[cfg(unix)]
#[test]
fn public_cli_runs_an_explicitly_granted_agent_through_bash() {
    let workspace = TestWorkspace::new("custom-bash-cli");
    let session_path = workspace.path().join("bash-session.json");
    let command = serde_json::to_string(&[
        "sh",
        "-c",
        "printf 'bash-agent:%s\\n' \"$1\"",
        "formal-ai-custom-agent",
        "{task}",
    ])
    .unwrap();
    let base_args = [
        "agent",
        "run",
        "--cli",
        "private-neural-agent",
        "--target",
        "vendor",
        "--task",
        "answer through bash",
        "--workspace",
    ];

    let denied = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(base_args)
        .arg(workspace.path())
        .args(["--command", &command])
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("AgentCommandNotAllowlisted(\"sh\")"));

    let allowed = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(base_args)
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
    assert!(
        allowed.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&allowed.stdout),
        String::from_utf8_lossy(&allowed.stderr)
    );
    let session = replay_session(&fs::read(session_path).unwrap()).unwrap();
    assert_eq!(session.cli, "private-neural-agent");
    assert_eq!(session.stdout.trim(), "bash-agent:answer through bash");
    assert!(
        session
            .events
            .iter()
            .any(|event| event.kind == "custom_adapter_granted")
    );
}

#[cfg(unix)]
#[test]
fn public_cli_compares_multiple_explicitly_granted_bash_agents() {
    let workspace = TestWorkspace::new("custom-bash-dispatch");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let argv = serde_json::to_string(&[
        "sh",
        "-c",
        "printf '# %s\\n' \"$1\" > README.md",
        "formal-ai-custom-agent",
        "{task}",
    ])
    .unwrap();
    let first = format!("neural-a={argv}");
    let second = format!("neural-b={argv}");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "agent",
            "dispatch",
            "--cli",
            "neural-a,neural-b",
            "--compare",
            "--target",
            "vendor",
            "--task",
            "custom neural result",
            "--workspace",
        ])
        .arg(workspace.path())
        .args([
            "--command",
            &first,
            "--command",
            &second,
            "--allow-agent-command",
            "sh",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["sessions"].as_array().map(Vec::len), Some(2));
    assert!(
        report["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|session| {
                session["events"].as_array().unwrap().iter().any(|event| {
                    event["kind"] == "custom_adapter_granted" && event["detail"] == "sh"
                })
            })
    );
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "# custom neural result\n"
    );
}

#[cfg(unix)]
#[test]
fn public_cli_resumes_the_recorded_vendor_session_with_correction_evidence() {
    use std::os::unix::fs::PermissionsExt as _;

    let fake_bin = TestWorkspace::new("native-resume-cli-bin");
    let agent = fake_bin.path().join("agent");
    fs::write(
        &agent,
        concat!(
            "#!/bin/sh\n",
            "printf '%s\\n' '{\"type\":\"result\",\"session_id\":",
            "\"ses_vendor_703\",\"result\":\"vendor answer\"}'\n",
            "printf 'argv:%s\\n' \"$*\"\n",
        ),
    )
    .unwrap();
    fs::set_permissions(&agent, fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::join_paths(std::iter::once(fake_bin.path().to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();
    let workspace = TestWorkspace::new("native-resume-cli");
    let parent_path = workspace.path().join("parent.json");
    let corrected_path = workspace.path().join("corrected.json");

    let initial = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "agent",
            "run",
            "--cli",
            "agent",
            "--target",
            "vendor",
            "--task",
            "state the release date",
            "--workspace",
        ])
        .arg(workspace.path())
        .args(["--session"])
        .arg(&parent_path)
        .env("PATH", &path)
        .output()
        .unwrap();
    assert!(
        initial.status.success(),
        "{}",
        String::from_utf8_lossy(&initial.stderr)
    );
    let parent = replay_session(&fs::read(&parent_path).unwrap()).unwrap();
    assert_eq!(
        parent
            .native_session
            .as_ref()
            .map(|session| session.id.as_str()),
        Some("ses_vendor_703")
    );

    let resumed = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["agent", "resume", "--parent"])
        .arg(&parent_path)
        .args(["--task", "correct the release date", "--workspace"])
        .arg(workspace.path())
        .args([
            "--disproved-claim",
            "The release date is 2027.",
            "--evidence",
            "The signed release record says 2026.",
            "--session",
        ])
        .arg(&corrected_path)
        .env("PATH", &path)
        .output()
        .unwrap();
    assert!(
        resumed.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&resumed.stdout),
        String::from_utf8_lossy(&resumed.stderr)
    );
    let corrected = replay_session(&fs::read(corrected_path).unwrap()).unwrap();
    assert!(
        corrected
            .args
            .windows(2)
            .any(|args| args == ["--resume", "ses_vendor_703"])
    );
    assert!(corrected.args.iter().any(|arg| arg == "--no-fork"));
    assert!(corrected.task.contains("The release date is 2027."));
    assert!(
        corrected
            .task
            .contains("The signed release record says 2026.")
    );
    assert_eq!(
        corrected
            .continuation
            .as_ref()
            .map(|continuation| continuation.native_session_id.as_str()),
        Some("ses_vendor_703")
    );
}

#[cfg(unix)]
#[test]
fn direct_vendor_entrypoints_keep_editing_and_prompt_arguments() {
    use std::os::unix::fs::PermissionsExt as _;

    let fake_bin = TestWorkspace::new("vendor-cli-bin");
    for cli in REQUIRED_CLIS {
        let path = fake_bin.path().join(cli);
        fs::write(&path, "#!/bin/sh\nprintf '%s\\n' \"$*\"\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = std::env::join_paths(std::iter::once(fake_bin.path().to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();

    for cli in REQUIRED_CLIS {
        let workspace = TestWorkspace::new(&format!("vendor-{cli}"));
        let session_path = workspace.path().join("session.json");
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args([
                "agent",
                "run",
                "--cli",
                cli,
                "--target",
                "vendor",
                "--task",
                "vendor task",
                "--workspace",
            ])
            .arg(workspace.path())
            .args(["--session"])
            .arg(&session_path)
            .env("PATH", &path)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{cli}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let session = replay_session(&fs::read(session_path).unwrap()).unwrap();
        assert!(session.passed());
        assert_eq!(session.args.last().map(String::as_str), Some("vendor task"));
        match cli {
            "agent" => {
                assert!(
                    session
                        .args
                        .windows(2)
                        .any(|args| args == ["-p", "vendor task"])
                );
                assert!(
                    session
                        .args
                        .iter()
                        .any(|arg| arg == "--no-retry-on-rate-limits")
                );
            }
            "gemini" | "qwen" => {
                assert!(
                    session
                        .args
                        .windows(2)
                        .any(|args| args == ["-p", "vendor task"])
                );
            }
            "claude" => {
                assert!(
                    session
                        .args
                        .windows(2)
                        .any(|args| args == ["--print", "vendor task"])
                );
            }
            "codex" => {
                assert!(
                    session
                        .args
                        .windows(2)
                        .any(|args| args == ["-m", "formal-ai"])
                );
                assert!(
                    session
                        .args
                        .windows(2)
                        .any(|args| args == ["--sandbox", "workspace-write"])
                );
                assert!(session.args.iter().any(|arg| arg == "--json"));
                assert!(!session.args.iter().any(|arg| arg == "read-only"));
            }
            "opencode" => {
                assert!(session.args.iter().any(|arg| arg == "--auto"));
                assert!(
                    session
                        .args
                        .windows(2)
                        .any(|args| args == ["--format", "json"])
                );
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(unix)]
#[test]
fn comparison_cli_succeeds_when_one_verified_winner_passes() {
    use std::os::unix::fs::PermissionsExt as _;

    let fake_bin = TestWorkspace::new("comparison-cli-bin");
    for (cli, script) in [
        (
            "codex",
            "#!/bin/sh\nprintf 'passing winner\\n' > README.md\n",
        ),
        (
            "claude",
            "#!/bin/sh\nprintf 'failed candidate\\n' > README.md\nexit 7\n",
        ),
    ] {
        let path = fake_bin.path().join(cli);
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = std::env::join_paths(std::iter::once(fake_bin.path().to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();
    let workspace = TestWorkspace::new("comparison-cli-winner");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "agent",
            "dispatch",
            "--cli",
            "codex,claude",
            "--compare",
            "--target",
            "vendor",
            "--task",
            "replace README",
            "--workspace",
        ])
        .arg(workspace.path())
        .env("PATH", &path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "passing winner\n"
    );
}

#[test]
#[ignore = "spawned explicitly by the verification-timeout test"]
fn external_slow_verification_fixture() {
    std::thread::sleep(Duration::from_millis(250));
}

#[test]
fn external_agent_fixture_process() {
    let Ok(mode) = std::env::var(FIXTURE_ENV) else {
        return;
    };
    match mode.as_str() {
        "success" => {
            fs::write("README.md", "fixture change\n").unwrap();
            println!(
                "{}",
                std::env::var("FORMAL_AI_ISSUE_703_OUTPUT")
                    .unwrap_or_else(|_| "fixture_stdout".to_string())
            );
            eprintln!("fixture_stderr");
        }
        "native_session" => {
            println!(
                "{}",
                std::env::var("FORMAL_AI_ISSUE_703_OUTPUT")
                    .unwrap_or_else(|_| "fixture_stdout".to_string())
            );
            eprintln!(
                "formal-ai: orchestration-session-json:{{\"id\":\"ses_issue_703\",\"resume_command\":\"agent --resume ses_issue_703\"}}"
            );
        }
        "mismatched_native_session" => {
            eprintln!(
                "formal-ai: orchestration-session-json:{{\"id\":\"ses_fresh_703\",\"resume_command\":\"agent --resume ses_fresh_703\"}}"
            );
        }
        "delayed_success" => {
            std::thread::sleep(Duration::from_millis(150));
            fs::write("README.md", "fixture change\n").unwrap();
        }
        "coordinated_success" => {
            let started = PathBuf::from(std::env::var_os(FIXTURE_STARTED_ENV).unwrap());
            let release = PathBuf::from(std::env::var_os(FIXTURE_RELEASE_ENV).unwrap());
            fs::write(started, "started\n").unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !release.exists() {
                assert!(
                    Instant::now() < deadline,
                    "fixture release was not signalled"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            fs::write("README.md", "fixture change\n").unwrap();
        }
        "timeout" => std::thread::sleep(Duration::from_millis(250)),
        #[cfg(unix)]
        "descendant_timeout" => {
            // The descendant proves it is alive by writing a file, but only
            // after a delay far longer than any plausible lateness in the
            // timeout that is supposed to kill it, and far longer than the
            // window the test waits for it to disappear. Issue #1021: the
            // previous 150 ms delay made that assertion a race with the
            // runner's scheduler, and a loaded macOS runner won it (CI/CD
            // Pipeline run 32272689475, job 96137354605).
            let mut descendant = Command::new("sh")
                .args(["-c", "sleep 20; printf escaped > descendant-survived"])
                .spawn()
                .unwrap();
            // Record the pid before waiting: it lets the test ask the kernel
            // whether the descendant outlived the group kill instead of
            // inferring termination from a file that may simply not have been
            // written yet.
            fs::write("descendant-pid", descendant.id().to_string()).unwrap();
            std::thread::sleep(Duration::from_secs(30));
            // Unreachable while the behaviour under test holds -- the timeout
            // kills this fixture long before the sleep above returns. It is
            // here so the child is reaped on the paths where the timeout does
            // not arrive, which is what `clippy::zombie_processes` asks for.
            descendant.wait().unwrap();
        }
        "failed" => std::process::exit(7),
        "failed_change" => {
            fs::write("README.md", "failed agent change\n").unwrap();
            std::process::exit(7);
        }
        "assert_pwd" => {
            assert_eq!(
                std::env::var_os("PWD").map(PathBuf::from),
                Some(std::env::current_dir().unwrap())
            );
        }
        _ => panic!("unknown fixture mode"),
    }
}

pub fn fixture_config(cli: &str, workspace: &Path, mode: &str) -> AgentRunConfig {
    let command = fixture_command(mode);
    let program = command.program.to_string_lossy().into_owned();
    let mut config = AgentRunConfig::new(cli, "fixture task", workspace)
        .with_permission(AgentRunPermission::grant_for(workspace))
        .with_command(command);
    config.allowlisted_agent_commands.insert(program);
    config
}

pub fn fixture_config_with_output(cli: &str, workspace: &Path, output: &str) -> AgentRunConfig {
    let mut config = fixture_config(cli, workspace, "success");
    config.command_override = Some(
        config
            .command_override
            .take()
            .unwrap()
            .env("FORMAL_AI_ISSUE_703_OUTPUT", output),
    );
    config
}

pub fn fixture_commands(clis: &[&str], mode: &str) -> BTreeMap<String, AgentCommand> {
    clis.iter()
        .map(|cli| ((*cli).to_string(), fixture_command(mode)))
        .collect()
}

pub fn grant_fixture_agent_command(config: &mut DispatchConfig) {
    config
        .allowlisted_agent_commands
        .insert(std::env::current_exe().unwrap().display().to_string());
}

pub fn fixture_command(mode: &str) -> AgentCommand {
    AgentCommand::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg(FIXTURE_TEST)
        .arg("--nocapture")
        .env(FIXTURE_ENV, mode)
}

fn one_request_health_server() -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).unwrap();
        let body = format!(r#"{{"version":"{}"}}"#, env!("CARGO_PKG_VERSION"));
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });
    (format!("http://{address}"), server)
}

pub struct TestWorkspace(PathBuf);

impl TestWorkspace {
    pub(super) fn new(label: &str) -> Self {
        let sequence = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "formal-ai-issue-703-{}-{label}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub(super) fn path(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TestWorkspace {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
