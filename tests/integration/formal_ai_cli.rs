use std::process::Command;

#[test]
fn cli_chat_command_prints_text_response() {
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["chat", "--prompt", "Hi"])
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "Hi, how may I help you?");
}

#[test]
fn cli_chat_command_can_emit_chat_completion_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "chat",
            "--prompt",
            "Write me hello world program in Rust",
            "--format",
            "chat",
        ])
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json["object"], "chat.completion");
    assert!(json["choices"][0]["message"]["content"]
        .as_str()
        .expect("assistant content should be a string")
        .contains("```rust"));
}
