use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("formal-ai-issue-757-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn make_executable(path: &Path) {
    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("chmod fake cli");
}

fn write_fake_codex_with_session(bin_dir: &Path) {
    let path = bin_dir.join("codex");
    std::fs::write(
        &path,
        r#"#!/bin/sh
session_id="019f70d1-1111-7222-8333-444455556666"
session="$HOME/.codex/sessions/2026/07/18/rollout-test-$session_id.jsonl"
mkdir -p "$(dirname "$session")"
printf '%s\n' "{\"type\":\"session_meta\",\"payload\":{\"id\":\"$session_id\"}}" > "$session"
printf 'Hi, how may I help you?\n'
"#,
    )
    .expect("write fake codex session cli");
    make_executable(&path);
}

fn write_fake_cli_with_tool_session(bin_dir: &Path, name: &str) {
    let path = bin_dir.join(name);
    std::fs::write(
        &path,
        r#"#!/bin/sh
tool=${0##*/}
if [ "$tool" = opencode ] && [ "${1:-}" = session ]; then
  printf '%s\n' '[{"id":"ses_opencode_test"}]'
  exit 0
fi
session_id="ses_${tool}_test"
case "$tool" in
  gemini) dir="$GEMINI_CLI_HOME/.gemini/tmp/project/chats"; file="$dir/session-test.jsonl" ;;
  qwen) dir="$HOME/.qwen/projects/project/chats"; file="$dir/$session_id.jsonl" ;;
  opencode) dir="$HOME/.local/share/opencode"; file="$dir/opencode.db" ;;
  agent) dir="$HOME/.local/share/link-assistant-agent/storage/session/project"; file="$dir/$session_id.json" ;;
  claude) dir="$CLAUDE_CONFIG_DIR/projects/project"; file="$dir/$session_id.jsonl" ;;
  grok) dir="$HOME/.grok"; file="$dir/$session_id.jsonl" ;;
esac
mkdir -p "$dir"
if [ "$tool" = opencode ]; then
  printf 'SQLite format 3\000' > "$file"
else
  printf '%s\n' "{\"sessionId\":\"$session_id\"}" > "$file"
fi
printf 'Hi, how may I help you?\n'
"#,
    )
    .expect("write fake tool session cli");
    make_executable(&path);
}

fn path_with_fake_clis(bin_dir: &Path) -> String {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    format!("{}:{}", bin_dir.display(), existing.to_string_lossy())
}

#[test]
fn prints_persistent_session_file_after_one_shot_and_interactive_runs() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_codex_with_session(&bin_dir);

    for mode in ["--non-interactive", "--interactive"] {
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--no-start-server", mode, "codex", "hi"])
            .env("HOME", &home)
            .env("PATH", path_with_fake_clis(&bin_dir))
            .output()
            .expect("run wrapper with fake transcript");
        assert!(output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("formal-ai: session files for debugging:"),
            "stderr:\n{stderr}"
        );
        assert!(stderr.contains("codex:"), "stderr:\n{stderr}");
        assert!(stderr.contains("rollout-test-019f70d1-1111-7222-8333-444455556666.jsonl"));
        assert!(stderr.contains("resume: codex resume 019f70d1-1111-7222-8333-444455556666"));
        let reported = stderr
            .lines()
            .find_map(|line| line.trim().strip_prefix("codex: "))
            .and_then(|line| line.split("   (resume:").next())
            .expect("reported codex session path");
        assert!(
            Path::new(reported).is_file(),
            "reported path must survive wrapper exit"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reports_every_requested_tool_session_and_proxy_log() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    let proxy_log = dir.join("proxy.jsonl");
    std::fs::write(&proxy_log, "{}\n").expect("proxy log");

    for (tool, expected_file, expected_resume) in [
        (
            "gemini",
            "session-test.jsonl",
            Some("gemini --resume ses_gemini_test"),
        ),
        (
            "qwen",
            "ses_qwen_test.jsonl",
            Some("qwen --resume ses_qwen_test"),
        ),
        (
            "opencode",
            "opencode.db",
            Some("opencode --session ses_opencode_test"),
        ),
        (
            "agent",
            "ses_agent_test.json",
            Some("agent --resume ses_agent_test"),
        ),
        (
            "claude",
            "ses_claude_test.jsonl",
            Some("claude --resume ses_claude_test"),
        ),
        ("grok", "ses_grok_test.jsonl", None),
    ] {
        write_fake_cli_with_tool_session(&bin_dir, tool);
        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .args(["with", "--no-start-server", "--non-interactive", tool, "hi"])
            .env("HOME", &home)
            .env("PATH", path_with_fake_clis(&bin_dir))
            .env("FORMAL_AI_PROXY_LOG", &proxy_log)
            .output()
            .expect("run wrapper with fake tool transcript");
        assert!(output.status.success(), "{tool} failed");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("  {tool}: ")), "stderr:\n{stderr}");
        assert!(stderr.contains(expected_file), "stderr:\n{stderr}");
        assert!(stderr.contains(&format!("  server log: {}", proxy_log.display())));
        if let Some(resume) = expected_resume {
            assert!(stderr.contains(resume), "stderr:\n{stderr}");
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}
