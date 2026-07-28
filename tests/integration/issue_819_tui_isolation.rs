use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmpdir() -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("formal-ai-issue-819-tui-{unique}"));
    std::fs::create_dir_all(&path).expect("temp directory");
    path
}

fn unused_loopback_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind unused port")
        .local_addr()
        .expect("local address")
        .port()
}

fn write_fake_requesting_tui_cli(bin_dir: &Path) {
    let path = bin_dir.join("opencode");
    std::fs::write(
        &path,
        r#"#!/bin/sh
[ -t 0 ] && [ -t 1 ] || { echo "interactive launch did not inherit a PTY" >&2; exit 90; }
curl -fsS -X POST "http://127.0.0.1:$FORMAL_AI_TEST_PORT/api/openai/v1/chat/completions" \
  -H "Authorization: Bearer formal-ai" \
  -H "Content-Type: application/json" \
  --data '{"model":"formal-ai","messages":[{"role":"user","content":"ISSUE819_RAW_REQUEST_MUST_NOT_REACH_TUI"}]}' \
  >/dev/null
printf 'TUI_REMAINED_INTACT\n'
"#,
    )
    .expect("write requesting fake TUI cli");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod requesting fake TUI cli");
}

#[test]
fn temporary_server_diagnostics_do_not_leak_into_the_wrapped_tui() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    let logs = dir.join("dialog-logs");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_requesting_tui_cli(&bin_dir);
    let port = unused_loopback_port();
    let existing_path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin_dir.display(), existing_path.to_string_lossy());

    let wrapper = env!("CARGO_BIN_EXE_formal-ai");
    let command = format!("{wrapper} with --port {port} --start-server opencode");
    let output = Command::new("script")
        .args(["-qfec", &command, "/dev/null"])
        .env("HOME", &home)
        .env("PATH", path)
        .env("FORMAL_AI_DIALOG_LOG_DIR", &logs)
        .env("FORMAL_AI_TEST_PORT", port.to_string())
        .output()
        .expect("launch wrapper with temporary server in PTY");
    let terminal = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success(), "{terminal}");
    assert!(terminal.contains("TUI_REMAINED_INTACT"), "{terminal}");
    assert!(!terminal.contains("ISSUE819_RAW_REQUEST_MUST_NOT_REACH_TUI"));
    let server_log = std::fs::read_dir(&logs)
        .expect("temporary server log directory")
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("temporary-server-")
        })
        .expect("temporary server diagnostic log");
    let diagnostics = std::fs::read_to_string(server_log.path()).expect("read diagnostics");
    assert!(diagnostics.contains("ISSUE819_RAW_REQUEST_MUST_NOT_REACH_TUI"));
    let _ = std::fs::remove_dir_all(&dir);
}
