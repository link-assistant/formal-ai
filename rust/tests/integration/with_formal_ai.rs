use crate::structured_stderr;
use std::io::Write as _;

use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

mod fixture_env;
#[path = "with_formal_ai/global_configuration.rs"]
mod global_configuration;
mod issue_1582;
mod model_metadata;
mod verbosity;

pub fn tmpdir() -> PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let thread_id = format!("{:?}", std::thread::current().id())
        .replace(|c: char| !c.is_ascii_alphanumeric(), "");
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-with-{}-{}-{seq}",
        std::process::id(),
        thread_id,
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn unused_loopback_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local address")
        .port()
}

pub use fixture_env::write_fake_cli;

fn write_fake_tui_cli(bin_dir: &Path, name: &str) {
    let path = bin_dir.join(name);
    std::fs::write(
        &path,
        r#"#!/bin/sh
[ -t 0 ] && [ -t 1 ] || { echo "interactive launch did not inherit a PTY" >&2; exit 90; }
for arg in "$@"; do
  if [ "$arg" = "--skip-git-repo-check" ]; then
    echo "error: unexpected argument '--skip-git-repo-check' found" >&2
    exit 2
  fi
  if [ "$arg" = "--prompt-interactive" ]; then
    echo "Not enough arguments following: prompt-interactive" >&2
    exit 1
  fi
done
printf 'TUI_READY\n'
IFS= read -r input
[ "$input" = "hi" ] || { echo "interactive input was not forwarded" >&2; exit 91; }
printf 'Hi, how may I help you?\n'
"#,
    )
    .expect("write fake TUI cli");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod fake TUI cli");
}

pub fn path_with_fake_clis(bin_dir: &Path) -> String {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    format!("{}:{}", bin_dir.display(), existing.to_string_lossy())
}

pub fn run_with_capture(
    home: &Path,
    bin_dir: &Path,
    capture: &Path,
    args: &[&str],
) -> std::process::Output {
    run_with_capture_stdin(home, bin_dir, capture, args, None)
}

/// Like [`run_with_capture`], with fixed stdin: `None` is empty/nonterminal;
/// `Some(text)` pipes a prompt, avoiding terminal-dependent `isatty(stdin)`.
pub fn run_with_capture_stdin(
    home: &Path,
    bin_dir: &Path,
    capture: &Path,
    args: &[&str],
    stdin: Option<&str>,
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_formal-ai"));
    fixture_env::clear_client_environment(&mut command);
    command.arg(args[0]);
    if args.first() == Some(&"with") && !args.contains(&"--global") && !args.contains(&"--undo") {
        command.arg("--no-start-server");
    }
    command
        .args(&args[1..])
        .env("HOME", home)
        .env("PATH", path_with_fake_clis(bin_dir))
        .env("FORMAL_AI_CAPTURE", capture)
        .stdin(match stdin {
            Some(_) => Stdio::piped(),
            None => Stdio::null(),
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("run formal-ai with");
    if let Some(text) = stdin {
        child
            .stdin
            .take()
            .expect("wrapper stdin")
            .write_all(text.as_bytes())
            .expect("pipe prompt");
    }
    child.wait_with_output().expect("wait for formal-ai with")
}

pub fn captured_args(capture: &str) -> Vec<&str> {
    capture
        .lines()
        .filter_map(|line| {
            line.strip_prefix("arg[")?
                .split_once("]=")
                .map(|(_, arg)| arg)
        })
        .collect()
}

pub fn captured_args_without_model_catalog(capture: &str) -> Vec<&str> {
    let mut args = captured_args(capture);
    if let Some(index) = args
        .iter()
        .position(|arg| arg.starts_with("model_catalog_json="))
    {
        args.drain(index - 1..=index);
    }
    args
}

#[test]
fn with_formal_ai_agent_ephemeral_injects_inline_config_and_model_flag() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "agent");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--base-url",
            "http://127.0.0.1:18080",
            "agent",
            "-p",
            "hi",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(
        captured.contains("arg[0]=--no-summarize-session"),
        "capture:\n{captured}"
    );
    assert!(
        captured.contains("--no-summarize-session"),
        "capture:\n{captured}"
    );
    assert!(
        captured.contains("arg[4]=formalai/formal-ai"),
        "capture:\n{captured}"
    );
    assert!(captured.contains("arg[5]=-p"), "capture:\n{captured}");
    assert!(captured.contains("arg[6]=hi"), "capture:\n{captured}");
    assert!(captured.contains("FORMAL_AI_API_KEY=formal-ai"));
    assert!(
        captured.contains("LINK_ASSISTANT_AGENT_CONFIG_CONTENT={"),
        "capture:\n{captured}"
    );
    assert!(
        captured.contains("\"baseURL\": \"http://127.0.0.1:18080/api/openai/v1\""),
        "capture:\n{captured}"
    );
    let expected_api_key = ["\"apiKey\": \"", "{", "env:FORMAL_AI_API_KEY", "}", "\""].concat();
    assert!(captured.contains(&expected_api_key), "capture:\n{captured}");
    assert!(captured.contains("\"model\": \"formalai/formal-ai\""));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_summarize_restores_agent_default() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "agent");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &["with", "--summarize", "agent", "-p", "hi"],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(
        !captured.contains("--no-summarize-session"),
        "capture:\n{captured}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_selects_uniform_interactive_and_non_interactive_modes() {
    let cases = [
        (
            "codex",
            vec!["--sandbox", "read-only"],
            vec![
                "exec",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "hi",
            ],
        ),
        (
            "opencode",
            vec!["-m", "formalai/formal-ai"],
            vec!["run", "-m", "formalai/formal-ai", "hi"],
        ),
        (
            "agent",
            vec![
                "--no-summarize-session",
                "--compaction-models",
                "(same)",
                "--model",
                "formalai/formal-ai",
                "--interactive",
            ],
            vec![
                "--no-summarize-session",
                "--compaction-models",
                "(same)",
                "--model",
                "formalai/formal-ai",
                "-p",
                "hi",
            ],
        ),
        (
            "gemini",
            vec!["-m", "formal-ai"],
            vec!["-m", "formal-ai", "-p", "hi"],
        ),
        (
            "claude",
            vec![
                "--allowedTools",
                "Read,Glob,Grep,WebFetch,WebSearch",
                "--model",
                "formal-ai",
                "--permission-mode",
                "auto",
            ],
            vec![
                "--allowedTools",
                "Read,Glob,Grep,WebFetch,WebSearch",
                "--model",
                "formal-ai",
                "--permission-mode",
                "auto",
                "--print",
                "hi",
            ],
        ),
        (
            "qwen",
            vec!["--model", "formal-ai"],
            vec!["--model", "formal-ai", "-p", "hi"],
        ),
        (
            "grok",
            vec!["--model", "formal-ai"],
            vec!["--model", "formal-ai", "-p", "hi"],
        ),
        (
            "aider",
            vec!["--no-auto-commits", "--model", "openai/formal-ai"],
            vec![
                "--no-auto-commits",
                "--model",
                "openai/formal-ai",
                "--message",
                "hi",
            ],
        ),
    ];
    for (tool, expected_interactive, expected_non_interactive) in cases {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        write_fake_cli(&bin_dir, tool);

        let interactive_capture = dir.join("interactive.txt");
        let interactive = run_with_capture(
            &home,
            &bin_dir,
            &interactive_capture,
            &["with", "--interactive", tool],
        );
        assert!(
            interactive.status.success(),
            "{tool}: {}",
            String::from_utf8_lossy(&interactive.stderr)
        );
        let captured = std::fs::read_to_string(&interactive_capture).expect("interactive capture");
        assert_eq!(
            captured_args_without_model_catalog(&captured),
            expected_interactive,
            "{tool} interactive capture:\n{captured}"
        );

        let print_capture = dir.join("print.txt");
        let print = run_with_capture(
            &home,
            &bin_dir,
            &print_capture,
            &["with", "--non-interactive", tool, "hi"],
        );
        assert!(
            print.status.success(),
            "{tool}: {}",
            String::from_utf8_lossy(&print.stderr)
        );
        let captured = std::fs::read_to_string(&print_capture).expect("print capture");
        assert_eq!(
            captured_args_without_model_catalog(&captured),
            expected_non_interactive,
            "{tool} non-interactive capture:\n{captured}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn with_formal_ai_value_taking_interactive_flags_have_prompt_values() {
    for (tool, model_arg) in [("gemini", "-m"), ("qwen", "--model")] {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        write_fake_cli(&bin_dir, tool);
        let capture = dir.join("interactive-with-prompt.txt");

        let output = run_with_capture(
            &home,
            &bin_dir,
            &capture,
            &["with", "--interactive", tool, "hi"],
        );

        assert!(
            output.status.success(),
            "{tool}: {}",
            structured_stderr::messages(&output.stderr)
        );
        let captured = std::fs::read_to_string(&capture).expect("interactive capture");
        assert_eq!(
            captured_args(&captured),
            [model_arg, "formal-ai", "--prompt-interactive", "hi"],
            "{tool} interactive capture:\n{captured}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn with_formal_ai_default_interactive_mode_launches_every_tool_in_a_pty() {
    for tool in [
        "codex", "opencode", "agent", "gemini", "claude", "qwen", "grok", "aider",
    ] {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        write_fake_tui_cli(&bin_dir, tool);

        let wrapper = env!("CARGO_BIN_EXE_formal-ai");
        let mut command = super::pty::command(wrapper, &["with", "--no-start-server", tool]);
        command
            .env("HOME", &home)
            .env("PATH", path_with_fake_clis(&bin_dir))
            .env_remove("FORMAL_AI_API_KEY");
        let output = super::pty::interact_after_ready(command, b"TUI_READY", b"hi\n")
            .expect("interact with wrapper in PTY");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = structured_stderr::messages(&output.stderr);

        assert!(
            output.status.success(),
            "{tool} failed to launch in a PTY; stdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(stdout.contains("TUI_READY"), "{tool} stdout:\n{stdout}");
        assert!(
            stdout.contains("Hi, how may I help you?"),
            "{tool} stdout:\n{stdout}"
        );
        assert!(
            !stdout.contains("unexpected argument"),
            "{tool} stdout:\n{stdout}"
        );
        assert!(
            !stdout.contains("Not enough arguments"),
            "{tool} stdout:\n{stdout}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn with_formal_ai_globally_alias_is_accepted() {
    let home = tmpdir();
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--globally", "codex"])
        .env("HOME", &home)
        .output()
        .expect("run --globally");
    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn with_formal_ai_all_seeded_tools_leave_persistent_configs_unchanged() {
    for tool in [
        "codex", "opencode", "agent", "gemini", "claude", "qwen", "grok", "aider",
    ] {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        for parent in [
            ".codex",
            ".config/opencode",
            ".config/link-assistant-agent",
            ".gemini",
        ] {
            std::fs::create_dir_all(home.join(parent)).expect("config parent");
        }
        std::fs::create_dir_all(&bin_dir).expect("bin");
        let candidates = [
            ".codex/config.toml",
            ".config/opencode/opencode.json",
            ".config/link-assistant-agent/opencode.json",
            ".gemini/settings.json",
            ".profile",
        ];
        for candidate in candidates {
            std::fs::write(home.join(candidate), format!("unchanged:{candidate}\n"))
                .expect("seed persistent config");
        }
        write_fake_cli(&bin_dir, tool);
        let capture = dir.join("capture.txt");
        let output = run_with_capture(&home, &bin_dir, &capture, &["with", tool, "hi"]);
        assert!(
            output.status.success(),
            "{tool} stderr: {}",
            structured_stderr::messages(&output.stderr)
        );
        for candidate in candidates {
            assert_eq!(
                std::fs::read_to_string(home.join(candidate)).expect("persistent config"),
                format!("unchanged:{candidate}\n"),
                "{tool} modified {candidate}"
            );
        }
        let captured = std::fs::read_to_string(&capture).expect("capture");
        match tool {
            "claude" => {
                assert!(captured.contains("ANTHROPIC_AUTH_TOKEN=formal-ai"));
                assert!(captured.contains("ANTHROPIC_API_KEY="));
                assert!(
                    captured.contains("ANTHROPIC_BASE_URL=http://127.0.0.1:8080/api/anthropic")
                );
                assert!(captured.contains("---CLAUDE_CONFIG---"));
                assert!(captured.contains(r#""hasCompletedOnboarding": true"#));
                let config_capture = captured
                    .split_once("---CLAUDE_CONFIG---\n")
                    .expect("captured Claude config marker")
                    .1
                    .split_once("\n---HOME_GEMINI_SETTINGS---")
                    .expect("captured config terminator")
                    .0;
                let config: serde_json::Value =
                    serde_json::from_str(config_capture).expect("parse captured Claude config");
                let working_directory = std::env::current_dir().expect("working directory");
                assert_eq!(
                    config["projects"][working_directory.to_string_lossy().as_ref()]["hasTrustDialogAccepted"],
                    true,
                    "an ephemeral interactive run must not stop at Claude's folder-trust dialog"
                );
                assert!(captured.contains("--allowedTools"));
                assert!(captured.contains("Read,Glob,Grep,WebFetch,WebSearch"));
                assert!(
                    captured_args(&captured)
                        .windows(2)
                        .any(|args| args == ["--model", "formal-ai"])
                );
            }
            "qwen" => {
                assert!(captured.contains("OPENAI_API_KEY=formal-ai"));
                assert!(captured.contains("OPENAI_BASE_URL=http://127.0.0.1:8080/api/openai/v1"));
                assert!(captured.contains("OPENAI_MODEL=formal-ai"));
                assert!(captured.contains("arg[0]=--model"));
                assert!(captured.contains("arg[1]=formal-ai"));
            }
            "grok" => {
                assert!(captured.contains("GROK_API_KEY=formal-ai-local"));
                assert!(captured.contains("GROK_BASE_URL=http://127.0.0.1:8080/api/openai/v1"));
                assert!(captured.contains("arg[0]=--model"));
                assert!(captured.contains("arg[1]=formal-ai"));
            }
            "aider" => {
                assert!(captured.contains("OPENAI_API_KEY=formal-ai"));
                assert!(captured.contains("OPENAI_API_BASE=http://127.0.0.1:8080/api/openai/v1"));
                assert!(captured.contains("arg[0]=--no-auto-commits"));
                assert!(captured.contains("arg[1]=--model"));
                assert!(captured.contains("arg[2]=openai/formal-ai"));
            }
            _ => {}
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn with_formal_ai_auto_starts_agent_mode_server_and_tears_it_down() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "agent");
    let capture = dir.join("capture.txt");
    let port = unused_loopback_port();
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--port", &port.to_string(), "agent", "-p", "hi"])
        .env("HOME", &home)
        .env("PATH", path_with_fake_clis(&bin_dir))
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run formal-ai with auto server");

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "routine startup stays out of the harness; actual stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let logs = std::fs::read_dir(home.join(".formal-ai/logs"))
        .expect("default diagnostic directory")
        .flat_map(|entry| std::fs::read(entry.expect("log entry").path()).expect("log"))
        .collect::<Vec<_>>();
    assert!(structured_stderr::messages(&logs).contains("temporary server in agent mode"));
    assert!(
        TcpStream::connect(("127.0.0.1", port)).is_err(),
        "temporary server still listening"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_no_start_server_runs_without_a_listener() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "agent");
    let capture = dir.join("capture.txt");
    let port = unused_loopback_port();
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--no-start-server",
            "--port",
            &port.to_string(),
            "agent",
            "-p",
            "hi",
        ])
        .env("HOME", &home)
        .env("PATH", path_with_fake_clis(&bin_dir))
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run formal-ai with no-start-server");

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    assert!(!structured_stderr::messages(&output.stderr).contains("started a temporary server"));
    assert!(capture.exists(), "wrapped CLI was not invoked");
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_opencode_ephemeral_writes_temp_config_and_model_flag() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "opencode");
    let capture = dir.join("capture.txt");
    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--base-url",
            "http://127.0.0.1:18080",
            "opencode",
            "run",
            "hi",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("arg[0]=run"), "capture:\n{captured}");
    assert!(captured.contains("arg[1]=-m"), "capture:\n{captured}");
    assert!(
        captured.contains("arg[2]=formalai/formal-ai"),
        "capture:\n{captured}"
    );
    assert!(captured.contains("arg[3]=hi"), "capture:\n{captured}");
    assert!(
        captured.contains("OPENCODE_CONFIG="),
        "capture:\n{captured}"
    );
    assert!(captured.contains("\"provider\""), "capture:\n{captured}");
    assert!(captured.contains("\"formalai\""), "capture:\n{captured}");
    assert!(captured.contains("\"baseURL\": \"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(captured.contains("\"model\": \"formalai/formal-ai\""));
    let has_exa = captured.contains("OPENCODE_ENABLE_EXA=1");
    assert!(has_exa, "capture:\n{captured}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_gemini_ephemeral_sets_native_protocol_environment() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(home.join(".gemini")).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    std::fs::write(
        home.join(".gemini/settings.json"),
        r#"{"security":{"auth":{"selectedType":"oauth-personal"}}}"#,
    )
    .expect("seed gemini oauth settings");
    std::fs::write(home.join(".gemini/oauth_creds.json"), "{}\n").expect("seed gemini oauth creds");
    write_fake_cli(&bin_dir, "gemini");
    let capture = dir.join("capture.txt");
    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--base-url",
            "http://127.0.0.1:18080",
            "gemini",
            "-p",
            "hi",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("arg[0]=-m"), "capture:\n{captured}");
    assert!(
        captured.contains("arg[1]=formal-ai"),
        "capture:\n{captured}"
    );
    assert!(captured.contains("arg[2]=-p"), "capture:\n{captured}");
    assert!(captured.contains("GEMINI_API_KEY=formal-ai"));
    assert!(captured.contains("GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key"));
    assert!(captured.contains("GEMINI_CLI_TRUST_WORKSPACE=true"));
    assert!(captured.contains("GEMINI_CLI_HOME="));
    assert!(captured.contains("---GEMINI_CLI_SETTINGS---"));
    assert!(captured.contains("\"selectedType\": \"gemini-api-key\""));
    assert!(captured.contains("---HOME_GEMINI_SETTINGS---"));
    assert!(captured.contains("oauth-personal"));
    assert!(captured.contains("GOOGLE_GEMINI_BASE_URL=http://127.0.0.1:18080/api/gemini"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_gemini_protocol_vertex_sets_vertex_base_url() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "gemini");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--protocol",
            "vertex",
            "--base-url",
            "http://127.0.0.1:18080",
            "gemini",
            "-p",
            "hi",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("GOOGLE_GEMINI_BASE_URL="));
    assert!(captured.contains("GOOGLE_VERTEX_BASE_URL=http://127.0.0.1:18080/api/vertex"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_global_configures_idempotently_and_undo_restores_backups() {
    global_configuration::with_formal_ai_global_configures_idempotently_and_undo_restores_backups();
}

#[test]
fn standalone_with_formal_ai_binary_uses_same_wrapper() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "gemini");
    let capture = dir.join("capture.txt");

    let mut command = Command::new(env!("CARGO_BIN_EXE_with-formal-ai"));
    fixture_env::clear_client_environment(&mut command);
    let output = command
        .args([
            "--no-start-server",
            "--base-url",
            "http://127.0.0.1:18080",
            "gemini",
            "-p",
            "hi",
        ])
        .env("HOME", &home)
        .env("PATH", path_with_fake_clis(&bin_dir))
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run standalone wrapper");

    assert!(
        output.status.success(),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("GOOGLE_GEMINI_BASE_URL=http://127.0.0.1:18080/api/gemini"));
    assert!(captured.contains("arg[1]=formal-ai"));

    let _ = std::fs::remove_dir_all(&dir);
}
