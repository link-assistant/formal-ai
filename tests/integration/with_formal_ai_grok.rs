use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("formal-ai-with-grok-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

#[test]
fn with_formal_ai_grok_reaches_configured_openai_endpoint() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    let grok = bin_dir.join("grok");
    std::fs::write(
        &grok,
        r#"#!/bin/sh
[ "$GROK_API_KEY" = "formal-ai-local" ] || exit 92
case "$GROK_BASE_URL" in */api/openai/v1) ;; *) exit 93 ;; esac
curl --silent --show-error --fail --max-time 5 \
  -H "Authorization: Bearer $GROK_API_KEY" \
  -H "Content-Type: application/json" \
  --data "{\"model\":\"formal-ai\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
  "$GROK_BASE_URL/chat/completions"
"#,
    )
    .expect("write fake grok");
    let mut permissions = std::fs::metadata(&grok).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&grok, permissions).expect("chmod fake grok");

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind recording endpoint");
    let port = listener.local_addr().expect("recording address").port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept grok request");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("request timeout");
        let mut bytes = [0_u8; 8192];
        let count = stream.read(&mut bytes).expect("read grok request");
        let request = String::from_utf8_lossy(&bytes[..count]).into_owned();
        let body = r#"{"choices":[{"message":{"content":"Hi from Formal AI"}}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .expect("respond to grok");
        request
    });

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--no-start-server",
            "--non-interactive",
            "--base-url",
            &format!("http://127.0.0.1:{port}"),
            "grok",
            "hi",
        ])
        .env("HOME", &home)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env_remove("FORMAL_AI_API_KEY")
        .env_remove("GROK_API_KEY")
        .env_remove("GROK_BASE_URL")
        .output()
        .expect("run formal-ai with grok");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Hi from Formal AI"));
    let request = server.join().expect("recording endpoint");
    assert!(request.starts_with("POST /api/openai/v1/chat/completions "));
    assert!(request.contains("Authorization: Bearer formal-ai-local"));
    assert!(request.contains("\"model\":\"formal-ai\""));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_formal_ai_grok_global_settings_and_undo_use_native_json() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");

    let configure = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--base-url",
            "http://127.0.0.1:18080",
            "grok",
        ])
        .env("HOME", &home)
        .output()
        .expect("configure grok globally");
    assert!(
        configure.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let settings_path = home.join(".grok/user-settings.json");
    let settings = std::fs::read_to_string(&settings_path).expect("grok settings");
    assert!(settings.contains("\"apiKey\": \"formal-ai-local\""));
    assert!(settings.contains("\"baseURL\": \"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(home.join(".grok/user-settings.json.formal-ai.bak").exists());

    let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--global", "--undo", "grok"])
        .env("HOME", &home)
        .output()
        .expect("undo grok global settings");
    assert!(
        undo.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&undo.stderr)
    );
    assert!(!settings_path.exists());
    assert!(!home.join(".grok/user-settings.json.formal-ai.bak").exists());
    let _ = std::fs::remove_dir_all(&dir);
}
