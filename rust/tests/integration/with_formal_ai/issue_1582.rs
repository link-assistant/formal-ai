//! Actual native wrapper regressions for #1582; no vendor CLI installation.
use super::*;
use std::io::Read as _;

#[test]
fn every_configurable_client_uses_the_remote_servers_limits_and_preserves_denies() {
    let dir = tmpdir();
    let home = dir.join("home");
    std::fs::create_dir_all(home.join(".claude")).expect("home");
    std::fs::write(home.join(".claude/settings.json"), r#"{"permissions":{"allow":["Read(custom)"],"deny":["WebFetch(domain:private.example)"]},"theme":"dark"}"#).expect("existing permissions");
    let listener = TcpListener::bind("127.0.0.1:0").expect("metadata server");
    listener.set_nonblocking(true).expect("nonblocking");
    let address = listener.local_addr().expect("address");
    let server = std::thread::spawn(move || {
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(10) {
            if let Ok((mut stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .expect("read bound");
                let mut request = [0; 8192];
                let count = stream.read(&mut request).expect("metadata request");
                assert!(String::from_utf8_lossy(&request[..count]).starts_with("GET /v1/models "));
                let body = r#"{"models":[{"id":"formal-ai","context_window_tokens":1234567,"max_output_tokens":32768}]}"#;
                write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).expect("response");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the wrapper did not discover the remote context window");
    });
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--global",
            "--all",
            "--base-url",
            &format!("http://{address}"),
        ])
        .env("HOME", &home)
        .env_remove("FORMAL_AI_LOG_STDERR")
        .output()
        .expect("configure all clients");
    server.join().expect("metadata worker");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "routine diagnostics must stay out of the harness"
    );
    let profile = std::fs::read_to_string(home.join(".profile")).expect("profile");
    assert!(profile.contains("CLAUDE_CODE_MAX_CONTEXT_TOKENS=\"1234567\""));
    assert!(profile.contains("CLAUDE_CODE_MAX_OUTPUT_TOKENS=\"32768\""));
    for (file, pointer) in [
        (
            ".qwen/settings.json",
            "/model/generationConfig/contextWindowSize",
        ),
        (
            ".aider.model.metadata.json",
            "/openai~1formal-ai/max_input_tokens",
        ),
        (
            ".config/link-assistant-agent/opencode.json",
            "/provider/formalai/models/formal-ai/limit/context",
        ),
    ] {
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(home.join(file)).expect(file))
                .expect("JSON");
        assert_eq!(
            value.pointer(pointer),
            Some(&serde_json::json!(1234567)),
            "{file}: {value}"
        );
    }
    let catalog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".codex/formal-ai-model-catalog.json"))
            .expect("Codex catalog"),
    )
    .expect("JSON");
    assert_eq!(
        catalog["models"][0]["context_window"],
        serde_json::json!(1234567)
    );
    let settings: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".claude/settings.json")).expect("Claude settings"),
    )
    .expect("JSON");
    assert_eq!(
        settings["permissions"]["deny"],
        serde_json::json!(["WebFetch(domain:private.example)"])
    );
    for rule in [
        "Read(custom)",
        "Read",
        "Glob",
        "Grep",
        "WebFetch",
        "WebSearch",
    ] {
        assert!(
            settings["permissions"]["allow"]
                .as_array()
                .expect("allow array")
                .contains(&serde_json::json!(rule))
        );
    }
    assert_eq!(settings["theme"], "dark");
    let grok = std::fs::read_to_string(home.join(".grok/config.toml"))
        .expect("Grok config")
        .parse::<toml_edit::DocumentMut>()
        .expect("TOML");
    assert_eq!(
        grok["model"]["formal-ai"]["context_window"].as_integer(),
        Some(1234567)
    );
    let undo = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["with", "--global", "--undo", "--all"])
        .env("HOME", &home)
        .output()
        .expect("undo");
    assert!(undo.status.success());
    let restored: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".claude/settings.json")).expect("restore"),
    )
    .expect("JSON");
    assert_eq!(
        restored["permissions"]["allow"],
        serde_json::json!(["Read(custom)"])
    );
    assert!(!home.join(".grok/config.toml").exists());
    std::fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn claude_ephemeral_receives_a_real_context_override_and_read_only_permissions() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin).expect("bin");
    write_fake_cli(&bin, "claude");
    let capture = dir.join("capture");
    let output = run_with_capture(&home, &bin, &capture, &["with", "claude", "hi"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let capture = std::fs::read_to_string(capture).expect("capture");
    let window = capture
        .lines()
        .find_map(|line| line.strip_prefix("CLAUDE_CODE_MAX_CONTEXT_TOKENS="))
        .expect("context override")
        .parse::<u64>()
        .expect("numeric context");
    assert!(
        window > 200_000,
        "a resource-derived limit replaces Claude's unknown-model guess"
    );
    assert!(capture.contains("Read,Glob,Grep,WebFetch,WebSearch"));
    assert!(!capture.contains("bypassPermissions"));
    std::fs::remove_dir_all(dir).expect("cleanup");
}
