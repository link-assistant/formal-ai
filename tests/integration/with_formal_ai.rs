use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> PathBuf {
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

fn write_fake_cli(bin_dir: &Path, name: &str) {
    let path = bin_dir.join(name);
    std::fs::write(
        &path,
        r#"#!/bin/sh
{
  echo "tool=$0"
  i=0
  for arg in "$@"; do
    echo "arg[$i]=$arg"
    i=$((i + 1))
  done
  echo "FORMAL_AI_API_KEY=$FORMAL_AI_API_KEY"
  echo "LINK_ASSISTANT_AGENT_CONFIG_CONTENT=$LINK_ASSISTANT_AGENT_CONFIG_CONTENT"
  echo "OPENCODE_CONFIG=$OPENCODE_CONFIG"
  echo "OPENCODE_CONFIG_DIR=$OPENCODE_CONFIG_DIR"
  echo "OPENCODE_ENABLE_EXA=$OPENCODE_ENABLE_EXA"
  echo "GEMINI_API_KEY=$GEMINI_API_KEY"
  echo "GEMINI_DEFAULT_AUTH_TYPE=$GEMINI_DEFAULT_AUTH_TYPE"
  echo "GEMINI_CLI_TRUST_WORKSPACE=$GEMINI_CLI_TRUST_WORKSPACE"
  echo "GEMINI_CLI_HOME=$GEMINI_CLI_HOME"
  echo "GOOGLE_GEMINI_BASE_URL=$GOOGLE_GEMINI_BASE_URL"
  echo "GOOGLE_VERTEX_BASE_URL=$GOOGLE_VERTEX_BASE_URL"
  echo "ANTHROPIC_AUTH_TOKEN=$ANTHROPIC_AUTH_TOKEN"
  echo "ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY"
  echo "ANTHROPIC_BASE_URL=$ANTHROPIC_BASE_URL"
  echo "OPENAI_API_KEY=$OPENAI_API_KEY"
  echo "OPENAI_BASE_URL=$OPENAI_BASE_URL"
  echo "OPENAI_API_BASE=$OPENAI_API_BASE"
  echo "OPENAI_MODEL=$OPENAI_MODEL"
  echo "XAI_API_KEY=$XAI_API_KEY"
  echo "XAI_BASE_URL=$XAI_BASE_URL"
  if [ -n "$OPENCODE_CONFIG" ] && [ -f "$OPENCODE_CONFIG" ]; then
    echo "---OPENCODE_CONFIG---"
    cat "$OPENCODE_CONFIG"
  fi
  if [ -n "$GEMINI_CLI_HOME" ] && [ -f "$GEMINI_CLI_HOME/.gemini/settings.json" ]; then
    echo "---GEMINI_CLI_SETTINGS---"
    cat "$GEMINI_CLI_HOME/.gemini/settings.json"
  fi
  if [ -f "$HOME/.gemini/settings.json" ]; then
    echo "---HOME_GEMINI_SETTINGS---"
    cat "$HOME/.gemini/settings.json"
  fi
} > "$FORMAL_AI_CAPTURE"
printf 'Hi, how may I help you?\n'
"#,
    )
    .expect("write fake cli");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod fake cli");
}

fn path_with_fake_clis(bin_dir: &Path) -> String {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    format!("{}:{}", bin_dir.display(), existing.to_string_lossy())
}

fn run_with_capture(
    home: &Path,
    bin_dir: &Path,
    capture: &Path,
    args: &[&str],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_formal-ai"));
    command.arg(args[0]);
    if args.first() == Some(&"with") && !args.contains(&"--global") && !args.contains(&"--undo") {
        command.arg("--no-start-server");
    }
    command
        .args(&args[1..])
        .env("HOME", home)
        .env("PATH", path_with_fake_clis(bin_dir))
        .env("FORMAL_AI_CAPTURE", capture)
        .env_remove("FORMAL_AI_API_KEY")
        .env_remove("LINK_ASSISTANT_AGENT_CONFIG_CONTENT")
        .env_remove("OPENCODE_CONFIG")
        .env_remove("OPENCODE_CONFIG_DIR")
        .env_remove("OPENCODE_ENABLE_EXA")
        .env_remove("GEMINI_API_KEY")
        .env_remove("GEMINI_DEFAULT_AUTH_TYPE")
        .env_remove("GEMINI_CLI_TRUST_WORKSPACE")
        .env_remove("GEMINI_CLI_HOME")
        .env_remove("GOOGLE_GEMINI_BASE_URL")
        .env_remove("GOOGLE_VERTEX_BASE_URL")
        .output()
        .expect("run formal-ai with")
}

#[test]
fn with_formal_ai_codex_ephemeral_uses_seeded_responses_provider_config() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "codex");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--base-url",
            "http://127.0.0.1:18080",
            "codex",
            "hi",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Hi, how may I help you?"
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("arg[0]=exec"), "capture:\n{captured}");
    assert!(
        captured.contains("arg[1]=--skip-git-repo-check"),
        "capture:\n{captured}"
    );
    assert!(
        captured.contains("arg[2]=--sandbox"),
        "capture:\n{captured}"
    );
    assert!(
        captured.contains("arg[3]=read-only"),
        "capture:\n{captured}"
    );
    assert!(captured.contains("model_provider=\"formalai\""));
    assert!(captured.contains("model=\"formal-ai\""));
    assert!(captured.contains("wire_api=\"responses\""));
    assert!(captured.contains("base_url=\"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(captured.contains("FORMAL_AI_API_KEY=formal-ai"));

    let _ = std::fs::remove_dir_all(&dir);
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
        String::from_utf8_lossy(&output.stderr)
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
        String::from_utf8_lossy(&output.stderr)
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
        ("codex", "exec"),
        ("opencode", "run"),
        ("agent", "-p"),
        ("gemini", "-p"),
        ("claude", "--print"),
        ("qwen", "-p"),
        ("grok", "--prompt"),
        ("aider", "--message"),
    ];
    for (tool, one_shot_arg) in cases {
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
        assert!(
            !captured.contains(&format!("={one_shot_arg}\n")),
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
        assert!(
            captured.contains(&format!("={one_shot_arg}\n")),
            "{tool} non-interactive capture:\n{captured}"
        );
        assert!(
            captured.contains("=hi\n"),
            "{tool} non-interactive capture:\n{captured}"
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
        String::from_utf8_lossy(&output.stderr)
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
            String::from_utf8_lossy(&output.stderr)
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
                assert!(captured.contains("ANTHROPIC_BASE_URL=http://127.0.0.1:8080/api/anthropic"));
                assert!(captured.contains("arg[0]=--model"));
                assert!(captured.contains("arg[1]=formal-ai"));
            }
            "qwen" => {
                assert!(captured.contains("OPENAI_API_KEY=formal-ai"));
                assert!(captured.contains("OPENAI_BASE_URL=http://127.0.0.1:8080/api/openai/v1"));
                assert!(captured.contains("OPENAI_MODEL=formal-ai"));
                assert!(captured.contains("arg[0]=--model"));
                assert!(captured.contains("arg[1]=formal-ai"));
            }
            "grok" => {
                assert!(captured.contains("XAI_API_KEY=formal-ai"));
                assert!(captured.contains("XAI_BASE_URL=http://127.0.0.1:8080/api/openai/v1"));
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
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("temporary server in agent mode"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("started a temporary server"));
    assert!(capture.exists(), "wrapped CLI was not invoked");
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_reuses_an_existing_loopback_listener() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "agent");
    let capture = dir.join("capture.txt");
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("address").port();
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--port", &port.to_string(), "agent", "-p", "hi"])
        .env("HOME", &home)
        .env("PATH", path_with_fake_clis(&bin_dir))
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run formal-ai with existing server");

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("started a temporary server"));
    assert!(
        listener.local_addr().is_ok(),
        "existing listener was replaced"
    );
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
        String::from_utf8_lossy(&output.stderr)
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
    assert!(
        captured.contains("OPENCODE_ENABLE_EXA=1"),
        "OpenCode must advertise its websearch tool in agentic mode:\n{captured}"
    );

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
        String::from_utf8_lossy(&output.stderr)
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
        String::from_utf8_lossy(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("GOOGLE_GEMINI_BASE_URL="));
    assert!(captured.contains("GOOGLE_VERTEX_BASE_URL=http://127.0.0.1:18080/api/vertex"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_global_configures_idempotently_and_undo_restores_backups() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(home.join(".codex")).expect("codex dir");
    std::fs::write(
        home.join(".codex/config.toml"),
        "approval_policy = \"never\"\n",
    )
    .expect("seed codex config");

    let first = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--base-url",
            "http://127.0.0.1:18080",
            "--all",
        ])
        .env("HOME", &home)
        .output()
        .expect("global configure");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let codex_config = std::fs::read_to_string(home.join(".codex/config.toml")).expect("codex");
    assert!(codex_config.contains("approval_policy = \"never\""));
    assert!(codex_config.contains("model_provider = \"formalai\""));
    assert!(codex_config.contains("model = \"formal-ai\""));
    assert!(codex_config.contains("[model_providers.formalai]"));
    assert!(codex_config.contains("base_url = \"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(home.join(".codex/config.toml.formal-ai.bak").exists());

    let opencode_config =
        std::fs::read_to_string(home.join(".config/opencode/opencode.json")).expect("opencode");
    assert!(opencode_config.contains("\"formalai\""));
    assert!(opencode_config.contains("\"model\": \"formalai/formal-ai\""));
    assert!(home
        .join(".config/opencode/opencode.json.formal-ai.bak")
        .exists());

    let agent_config =
        std::fs::read_to_string(home.join(".config/link-assistant-agent/opencode.json"))
            .expect("agent");
    assert!(agent_config.contains("\"formalai\""));
    assert!(agent_config.contains("\"model\": \"formalai/formal-ai\""));
    assert!(home
        .join(".config/link-assistant-agent/opencode.json.formal-ai.bak")
        .exists());

    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    assert_eq!(profile.matches("formal-ai gemini").count(), 2);
    assert!(profile.contains("GOOGLE_GEMINI_BASE_URL=\"http://127.0.0.1:18080/api/gemini\""));
    assert!(home.join(".profile.formal-ai.bak").exists());

    let second = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "-g",
            "--base-url",
            "http://127.0.0.1:18080",
            "--all",
        ])
        .env("HOME", &home)
        .output()
        .expect("global configure again");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let profile_again = std::fs::read_to_string(home.join(".profile")).expect("profile again");
    assert_eq!(profile_again.matches("formal-ai gemini").count(), 2);

    let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--global", "--undo", "--all"])
        .env("HOME", &home)
        .output()
        .expect("global undo");
    assert!(
        undo.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&undo.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(home.join(".codex/config.toml")).expect("restored codex"),
        "approval_policy = \"never\"\n"
    );
    assert!(!home.join(".config/opencode/opencode.json").exists());
    assert!(!home
        .join(".config/link-assistant-agent/opencode.json")
        .exists());
    assert!(!home.join(".profile").exists());

    let _ = std::fs::remove_dir_all(&dir);
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

    let output = Command::new(env!("CARGO_BIN_EXE_with-formal-ai"))
        .args(["--base-url", "http://127.0.0.1:18080", "gemini", "-p", "hi"])
        .env("HOME", &home)
        .env("PATH", path_with_fake_clis(&bin_dir))
        .env("FORMAL_AI_CAPTURE", &capture)
        .output()
        .expect("run standalone wrapper");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("GOOGLE_GEMINI_BASE_URL=http://127.0.0.1:18080/api/gemini"));
    assert!(captured.contains("arg[1]=formal-ai"));

    let _ = std::fs::remove_dir_all(&dir);
}
