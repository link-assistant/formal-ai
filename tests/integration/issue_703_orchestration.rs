use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::orchestration::{
    dispatch_agents, replay_session, run_agent, write_session, AgentCommand, AgentRunConfig,
    AgentRunError, AgentRunPermission, AgentStatus, ComparisonEntry, ComparisonLedger,
    DispatchConfig, DispatchMode, ReplayError, VerificationCommand,
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

const FIXTURE_ENV: &str = "FORMAL_AI_ISSUE_703_FIXTURE";
const FIXTURE_TEST: &str = "issue_703_orchestration::external_agent_fixture_process";
const AUTHORSHIP_SESSION: &str = "ses_050646852ffetdnQ73vR1yZ8la";
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
    assert!(arguments["query"]
        .as_str()
        .unwrap()
        .contains("run_shell_command"));

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
        assert!(session
            .events
            .iter()
            .any(|event| event.kind == "workspace_effect"));
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
        assert!(session
            .changes
            .iter()
            .any(|change| change.path == "README.md"));
    }
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
                assert!(session
                    .args
                    .windows(2)
                    .any(|args| args == ["-p", "vendor task"]));
                assert!(session
                    .args
                    .iter()
                    .any(|arg| arg == "--no-retry-on-rate-limits"));
            }
            "gemini" | "qwen" => {
                assert!(session
                    .args
                    .windows(2)
                    .any(|args| args == ["-p", "vendor task"]));
            }
            "claude" => {
                assert!(session
                    .args
                    .windows(2)
                    .any(|args| args == ["--print", "vendor task"]));
            }
            "codex" => {
                assert!(session
                    .args
                    .windows(2)
                    .any(|args| args == ["-m", "formal-ai"]));
                assert!(session
                    .args
                    .windows(2)
                    .any(|args| args == ["--sandbox", "workspace-write"]));
                assert!(session.args.iter().any(|arg| arg == "--json"));
                assert!(!session.args.iter().any(|arg| arg == "read-only"));
            }
            "opencode" => {
                assert!(session.args.iter().any(|arg| arg == "--auto"));
                assert!(session
                    .args
                    .windows(2)
                    .any(|args| args == ["--format", "json"]));
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
fn formal_ai_authored_invariant_is_byte_pinned_to_its_session_evidence() {
    let repository_artifact = include_bytes!("../../data/meta/orchestration-safety-invariant.lino");
    let captured_artifact = include_bytes!(
        "../../docs/case-studies/issue-703/self-hosting-authorship/orchestration-safety-invariant.lino"
    );
    let evidence =
        include_str!("../../docs/case-studies/issue-703/self-hosting-authorship/agent-cli.log");

    assert_eq!(repository_artifact, captured_artifact);
    assert!(evidence.contains(AUTHORSHIP_SESSION));
    assert!(evidence.contains("\"providerID\": \"formal-ai\""));
    assert!(evidence.contains("\"tool\": \"write\""));
}

#[test]
fn real_formal_ai_controller_agent_run_replays_from_committed_bytes() {
    let bytes = include_bytes!(
        "../../docs/case-studies/issue-703/controller-agent-run/controller-session.json"
    );
    let artifact = include_bytes!(
        "../../docs/case-studies/issue-703/controller-agent-run/controller-proof.lino"
    );

    let session = replay_session(bytes).expect("committed controller session is canonical");

    assert_eq!(session.cli, "agent");
    assert!(session.passed());
    assert!(session.stdout.contains("ses_05046b1c9ffe59CvG0N3QrrsV4"));
    assert!(session.stdout.contains("--no-retry-on-rate-limits"));
    assert!(session.stdout.contains("\"name\":\"write\""));
    let change = session
        .changes
        .iter()
        .find(|change| change.path == "controller-proof.lino")
        .expect("the real Agent CLI run recorded its file effect");
    assert_eq!(
        change.after_sha256.as_deref(),
        Some("b70e7374ba7654d7d57a908ae0d1b7d40e7b1c1651d6bcc2dd80671d35777637")
    );
    assert_eq!(
        artifact,
        b"controller_proof and the phrase replayable workspace edit."
    );
}

#[test]
fn committed_parallel_comparison_ledger_records_the_selected_winner() {
    let ledger: ComparisonLedger = serde_json::from_slice(include_bytes!(
        "../../docs/case-studies/issue-703/comparison/comparison-ledger.json"
    ))
    .unwrap();

    assert_eq!(ledger.schema, "formal-ai-comparison-ledger-v1");
    assert_eq!(ledger.entries.len(), 2);
    assert!(ledger.entries.iter().all(|entry| entry.passed));
    assert_eq!(ledger.winner.as_deref(), Some("codex"));
    assert_eq!(
        ComparisonLedger::select_winner(&ledger.entries),
        ledger.winner
    );
    for bytes in [
        include_bytes!("../../docs/case-studies/issue-703/comparison/sessions/000-codex.json")
            .as_slice(),
        include_bytes!("../../docs/case-studies/issue-703/comparison/sessions/001-claude.json")
            .as_slice(),
    ] {
        let session = replay_session(bytes).unwrap();
        assert!(session.passed());
        assert_eq!(session.verification.len(), 1);
        assert_eq!(session.verification[0].program, "test");
    }
}

#[test]
fn timeout_is_recorded_without_an_implicit_retry() {
    let workspace = TestWorkspace::new("timeout");
    let mut config = fixture_config("codex", workspace.path(), "timeout");
    config.timeout = Duration::from_millis(20);

    let session = run_agent(&config).expect("timeout is a recorded agent result");

    assert_eq!(session.status, AgentStatus::TimedOut);
    assert_eq!(
        session
            .events
            .iter()
            .filter(|event| event.kind == "process_started")
            .count(),
        1
    );
}

#[cfg(unix)]
#[test]
fn timeout_terminates_descendant_processes() {
    let workspace = TestWorkspace::new("descendant-timeout");
    let mut config = fixture_config("codex", workspace.path(), "descendant_timeout");
    config.timeout = Duration::from_millis(20);

    let session = run_agent(&config).expect("timeout is recorded");
    std::thread::sleep(Duration::from_millis(250));

    assert_eq!(session.status, AgentStatus::TimedOut);
    assert!(!workspace.path().join("descendant-survived").exists());
}

#[test]
fn verification_commands_must_be_explicitly_allowlisted() {
    let workspace = TestWorkspace::new("allowlist");
    let mut config = fixture_config("codex", workspace.path(), "success");
    config.verification.push(VerificationCommand::new(
        "unreviewed-command",
        std::iter::empty::<String>(),
    ));

    let error = run_agent(&config).expect_err("unreviewed command must not execute");

    assert!(matches!(
        error,
        AgentRunError::CommandNotAllowlisted(command) if command == "unreviewed-command"
    ));
}

#[test]
fn verification_timeout_is_recorded_and_fails_the_session() {
    let workspace = TestWorkspace::new("verification-timeout");
    let mut config = fixture_config("codex", workspace.path(), "success");
    let test_binary = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    config.timeout = Duration::from_millis(20);
    config.allowlisted_commands.insert(test_binary.clone());
    config.verification.push(VerificationCommand::new(
        test_binary,
        [
            "--ignored",
            "--exact",
            "issue_703_orchestration::external_slow_verification_fixture",
        ],
    ));

    let session = run_agent(&config).expect("verification timeout is recorded");

    assert!(!session.passed());
    assert_eq!(session.verification.len(), 1);
    assert!(session.verification[0].timed_out);
    assert!(session
        .events
        .iter()
        .any(|event| event.kind == "verification_started"));
    assert!(session
        .events
        .iter()
        .any(|event| event.kind == "verification_timed_out"));
}

#[test]
fn failed_external_process_is_visible_in_the_session() {
    let workspace = TestWorkspace::new("failed");
    let config = fixture_config("agent", workspace.path(), "failed");

    let session = run_agent(&config).expect("non-zero status is evidence, not a controller error");

    assert_eq!(session.status, AgentStatus::Failed);
    assert_eq!(session.exit_code, Some(7));
    assert_eq!(
        session
            .events
            .iter()
            .filter(|event| event.kind == "process_started")
            .count(),
        1
    );
}

#[test]
fn recorded_session_replays_byte_for_byte() {
    let workspace = TestWorkspace::new("replay");
    let session = run_agent(&fixture_config("gemini", workspace.path(), "success")).unwrap();
    let path = workspace.path().join("session.json");
    write_session(&path, &session).unwrap();
    let bytes = fs::read(path).unwrap();

    assert_eq!(replay_session(&bytes).unwrap(), session);
}

#[test]
fn replay_rejects_valid_but_noncanonical_json() {
    let workspace = TestWorkspace::new("noncanonical-replay");
    let session = run_agent(&fixture_config("gemini", workspace.path(), "success")).unwrap();
    let bytes = serde_json::to_vec(&session).unwrap();

    assert!(matches!(
        replay_session(&bytes),
        Err(ReplayError::NonCanonical)
    ));
}

#[test]
fn parallel_comparison_records_a_ledger_and_composes_the_winner() {
    let workspace = TestWorkspace::new("compare");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string(), "claude".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides = fixture_commands(&["codex", "claude"], "success");

    let report = dispatch_agents(&config).expect("parallel comparison");

    assert_eq!(report.sessions.len(), 2);
    assert_eq!(report.ledger.entries.len(), 2);
    assert!(report.ledger.winner.is_some());
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "fixture change\n"
    );
    assert!(config.output_dir.join("comparison-ledger.json").is_file());
}

#[test]
fn comparison_rejects_duplicate_cli_identities_before_creating_candidates() {
    let workspace = TestWorkspace::new("duplicate-cli");
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string(), "codex".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("success"));

    let error = dispatch_agents(&config).expect_err("duplicate identities are ambiguous");

    assert_eq!(error.to_string(), "duplicate_cli:codex");
    assert!(!config.output_dir.exists());
}

#[test]
fn comparison_refuses_to_overwrite_workspace_drift_after_candidates_fork() {
    let workspace = TestWorkspace::new("workspace-drift");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("delayed_success"));
    let original = workspace.path().join("README.md");
    let concurrent_update = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        fs::write(original, "external update\n").unwrap();
    });

    let error = dispatch_agents(&config).expect_err("composition must detect workspace drift");
    concurrent_update.join().unwrap();

    assert!(error.to_string().contains("workspace_drift:README.md"));
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "external update\n"
    );
}

#[test]
fn custom_output_directory_inside_workspace_is_excluded_from_candidate_copies() {
    let workspace = TestWorkspace::new("nested-output");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.output_dir = workspace.path().join("agent-artifacts");
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides = fixture_commands(&["codex"], "success");

    let report = dispatch_agents(&config).expect("nested output stays out of candidate snapshots");

    assert_eq!(report.sessions.len(), 1);
    assert!(config.output_dir.join("comparison-ledger.json").is_file());
    assert!(!config
        .output_dir
        .join("candidates/000-codex/agent-artifacts")
        .exists());
}

#[test]
fn dispatch_output_cannot_escape_the_granted_workspace() {
    let workspace = TestWorkspace::new("output-boundary");
    let outside = TestWorkspace::new("outside-output");
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string()],
    );
    config.output_dir = outside.path().join("agent-artifacts");
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("success"));

    let error = dispatch_agents(&config).unwrap_err();

    assert_eq!(error.to_string(), "output_outside_workspace");
    assert!(!config.output_dir.exists());
}

#[test]
fn dispatch_joins_every_worker_before_returning_an_error() {
    let workspace = TestWorkspace::new("join-on-error");
    let mut config = DispatchConfig::new(
        "add a README badge",
        workspace.path(),
        vec!["codex".to_string(), "claude".to_string()],
    );
    config.mode = DispatchMode::Compare;
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides.insert(
        "codex".to_string(),
        AgentCommand::new(workspace.path().join("missing-program")),
    );
    config
        .command_overrides
        .insert("claude".to_string(), fixture_command("delayed_success"));
    let started = Instant::now();

    let error = dispatch_agents(&config).expect_err("one worker cannot be started");

    assert!(error.to_string().contains("process:"));
    assert!(started.elapsed() >= Duration::from_millis(100));
    assert_eq!(
        fs::read_to_string(config.output_dir.join("candidates/001-claude/README.md")).unwrap(),
        "fixture change\n"
    );
}

#[test]
fn universal_decomposition_dispatches_independent_leaves_in_parallel() {
    let workspace = TestWorkspace::new("decompose");
    let mut config = DispatchConfig::new(
        "Create a README badge and add a release note.",
        workspace.path(),
        vec!["codex".to_string(), "opencode".to_string()],
    );
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config.command_overrides = fixture_commands(&["codex", "opencode"], "success");

    let report = dispatch_agents(&config).expect("decomposed dispatch");

    assert!(report.tasks.len() >= 2, "{:?}", report.tasks);
    assert_eq!(report.tasks.len(), report.sessions.len());
    assert!(!report.composed_changes.is_empty());
}

#[test]
fn decomposition_never_composes_changes_from_a_failed_agent() {
    let workspace = TestWorkspace::new("failed-decomposition");
    fs::write(workspace.path().join("README.md"), "before\n").unwrap();
    let mut config = DispatchConfig::new(
        "Create a README badge and add a release note.",
        workspace.path(),
        vec!["codex".to_string(), "opencode".to_string()],
    );
    config.permission = AgentRunPermission::grant_for(workspace.path());
    config
        .command_overrides
        .insert("codex".to_string(), fixture_command("success"));
    config
        .command_overrides
        .insert("opencode".to_string(), fixture_command("failed_change"));

    let report = dispatch_agents(&config).expect("failed candidate stays isolated");

    assert!(report.sessions.iter().any(|session| !session.passed()));
    assert_eq!(
        fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "fixture change\n"
    );
}

#[test]
fn winner_selection_is_deterministic_for_a_recorded_ledger() {
    let entries = vec![
        ComparisonEntry {
            cli: "codex".to_string(),
            task: "task".to_string(),
            passed: true,
            diff_size: 20,
            wall_time_ms: 10,
            session_file: "codex.json".to_string(),
        },
        ComparisonEntry {
            cli: "claude".to_string(),
            task: "task".to_string(),
            passed: true,
            diff_size: 10,
            wall_time_ms: 50,
            session_file: "claude.json".to_string(),
        },
        ComparisonEntry {
            cli: "agent".to_string(),
            task: "task".to_string(),
            passed: false,
            diff_size: 1,
            wall_time_ms: 1,
            session_file: "agent.json".to_string(),
        },
    ];

    assert_eq!(
        ComparisonLedger::select_winner(&entries).as_deref(),
        Some("claude")
    );
    assert_eq!(ComparisonLedger::select_winner(&entries[2..]), None);
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
            println!("fixture_stdout");
            eprintln!("fixture_stderr");
        }
        "delayed_success" => {
            std::thread::sleep(Duration::from_millis(150));
            fs::write("README.md", "fixture change\n").unwrap();
        }
        "timeout" => std::thread::sleep(Duration::from_millis(250)),
        #[cfg(unix)]
        "descendant_timeout" => {
            let mut descendant = Command::new("sh")
                .args(["-c", "sleep 0.15; printf escaped > descendant-survived"])
                .spawn()
                .unwrap();
            std::thread::sleep(Duration::from_secs(1));
            let _ = descendant.wait();
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

fn fixture_config(cli: &str, workspace: &Path, mode: &str) -> AgentRunConfig {
    AgentRunConfig::new(cli, "fixture task", workspace)
        .with_permission(AgentRunPermission::grant_for(workspace))
        .with_command(fixture_command(mode))
}

fn fixture_commands(clis: &[&str], mode: &str) -> BTreeMap<String, AgentCommand> {
    clis.iter()
        .map(|cli| ((*cli).to_string(), fixture_command(mode)))
        .collect()
}

fn fixture_command(mode: &str) -> AgentCommand {
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

struct TestWorkspace(PathBuf);

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let sequence = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "formal-ai-issue-703-{}-{label}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
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
