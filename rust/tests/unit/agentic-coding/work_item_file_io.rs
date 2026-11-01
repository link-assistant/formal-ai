//! CI-only registered IO transport qualification. No native product is built or
//! executed by the local development workflow; these tests run on main CI.
use super::*;
use crate::protocol::ToolCall;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn directory() -> Directory {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "formal-ai-file-io-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    Directory(fs::canonicalize(path).unwrap())
}
fn observe(messages: &mut Vec<ChatMessage>, tool: &str, args: Value, output: String, status: i32) {
    let id = format!("native-file-io-{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        &id,
        tool,
        args.to_string(),
    )]));
    let frame = format!(
        "Chunk ID: fixture\nWall time: 0.01 seconds\nProcess exited with code {status}\nOutput:\n{output}"
    );
    let mut receipt = ChatMessage::tool_result(&id, tool, frame);
    receipt.is_error = status != 0;
    messages.push(receipt);
}
#[test]
fn native_registered_program_observes_real_source_replaces_and_reads_back() {
    let directory = directory();
    fs::create_dir(directory.0.join("src")).unwrap();
    let root = directory.0.to_str().unwrap();
    let before = "export const enabled = false;\n";
    let after = "export const enabled = true;\n";
    fs::write(directory.0.join("src/owner.mjs"), before).unwrap();
    let mut messages = vec![
        ChatMessage::new(
            "system",
            format!("<env>\nWorking directory: {root}\n</env>"),
        ),
        ChatMessage::user("Update the requested repository file."),
    ];
    for (operation, preimage, candidate) in [
        ("read", None, None),
        ("replace", Some(before), Some(after)),
        ("read", None, None),
    ] {
        let command = file_io_command(
            operation,
            Some(root),
            Some("src/owner.mjs"),
            preimage,
            candidate,
        )
        .unwrap();
        let output = Command::new("/bin/sh")
            .args(["-c", &command])
            .current_dir(&directory.0)
            .output()
            .unwrap();
        let status = output.status.code().unwrap();
        assert_eq!(status, 0, "{}", String::from_utf8_lossy(&output.stderr));
        observe(
            &mut messages,
            "exec_command",
            json!({"cmd":command}),
            String::from_utf8(output.stdout).unwrap(),
            status,
        );
    }
    let progress = WorkItemFileProgress::scan(&messages);
    assert!(progress.gap.is_none());
    assert_eq!(progress.research_attempts().len(), 3);
    assert_eq!(
        progress
            .source_read_for("src/owner.mjs")
            .unwrap()
            .source
            .as_deref(),
        Some(after)
    );
    let edit = &progress.research_attempts()[1];
    let args: Value = serde_json::from_str(edit.arguments.as_deref().unwrap()).unwrap();
    assert_eq!(args["oldString"], before);
    assert_eq!(edit.transport["kind"], "registered-run-file-io");
    assert_eq!(edit.transport["exitCode"], 0);
    assert_eq!(
        fs::read_to_string(directory.0.join("src/owner.mjs")).unwrap(),
        after
    );
}
#[test]
fn native_absolute_alias_normalization_requires_exact_caller_root() {
    assert_eq!(
        canonical_file_io_path("/checkout/src/owner.mjs", Some("/checkout")).as_deref(),
        Some("src/owner.mjs")
    );
    assert_eq!(
        canonical_file_io_path("/checkout-foreign/src/owner.mjs", Some("/checkout")),
        None
    );
    assert_eq!(
        canonical_file_io_path("/elsewhere/src/owner.mjs", Some("/checkout")),
        None
    );
    assert_eq!(
        canonical_file_io_path("/checkout/../elsewhere/src/owner.mjs", Some("/checkout")),
        None
    );
}
