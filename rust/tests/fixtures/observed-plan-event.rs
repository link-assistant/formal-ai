//! Observe only the named auxiliary stream before the requested effect.
use formal_ai::agentic_coding::general_planner::{PLAN_PATH, compose_general_change_plan};
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct EventDirectory(PathBuf);
impl EventDirectory {
    fn new() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "formal-ai-observed-event-{}-{timestamp}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("selected event workspace");
        Self(path)
    }
}
impl Drop for EventDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("observed_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}
pub fn next_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) if calls.len() == 1 => calls.remove(0),
        other => panic!("expected one tool call, got {other:?}"),
    }
}
/// Return the first requested effect unconsumed after real auxiliary I/O.
/// Any source-target read is returned intact, never swallowed by setup.
pub fn before_target(prompt: &str, tools: &[&str]) -> (Vec<ChatMessage>, PlannedToolCall) {
    let directory = EventDirectory::new();
    let path = directory.0.join(PLAN_PATH);
    let mut messages = vec![ChatMessage::user(prompt)];
    for _ in 0..4 {
        let call = next_call(&messages, tools);
        let args: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments");
        let target = ["path", "filePath", "file_path"]
            .iter()
            .find_map(|key| args[*key].as_str());
        if target != Some(PLAN_PATH) {
            if let Some(command) = args["command"]
                .as_str()
                .filter(|command| command.starts_with("mkdir -p -- .formal-ai && (lock="))
            {
                let shell = if cfg!(windows) {
                    ["ProgramFiles", "ProgramFiles(x86)"]
                        .iter()
                        .filter_map(std::env::var_os)
                        .map(|directory| PathBuf::from(directory).join("Git/bin/bash.exe"))
                        .find(|candidate| candidate.is_file())
                        .expect("Git Bash required for the advertised shell fixture")
                } else {
                    PathBuf::from("/bin/sh")
                };
                let output = std::process::Command::new(shell)
                    .args(["-c", command])
                    .current_dir(&directory.0)
                    .output()
                    .expect("execute exact append command");
                assert!(
                    output.status.success(),
                    "actual auxiliary append failed: {output:?}"
                );
                let stdout = String::from_utf8(output.stdout).expect("UTF-8 observed stdout");
                let plan = compose_general_change_plan(prompt).expect("request-derived event");
                assert!(
                    fs::read_to_string(&path)
                        .expect("actual event bytes")
                        .ends_with(&plan.links_notation()),
                    "append must persist the exact event"
                );
                assert_eq!(
                    stdout.trim(),
                    plan.links_notation().trim(),
                    "actual event readback"
                );
                record(
                    &mut messages,
                    &call,
                    &format!("Output: {stdout}\nExit Code: 0"),
                );
                continue;
            }
            return (messages, call);
        }
        let mut read_succeeded = false;
        let mut read_failed = false;
        let result = match call.tool.as_str() {
            "read" | "read_file" => match fs::read_to_string(&path) {
                Ok(bytes) => {
                    read_succeeded = true;
                    bytes
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    read_failed = true;
                    serde_json::json!({"is_error":true,"error":format!("File not found: {PLAN_PATH}")}).to_string()
                }
                Err(error) => panic!("event read failed: {error}"),
            },
            "write" | "write_file" => {
                let prior = match fs::read_to_string(&path) {
                    Ok(bytes) => bytes,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                    Err(error) => panic!("prior event read failed: {error}"),
                };
                let plan = compose_general_change_plan(prompt).expect("request-derived event");
                let separator = if !prior.is_empty() && !prior.ends_with('\n') {
                    "\n"
                } else {
                    ""
                };
                let expected = format!("{prior}{separator}{}", plan.links_notation());
                assert_eq!(
                    args["content"].as_str(),
                    Some(expected.as_str()),
                    "preserve exact event bytes"
                );
                fs::create_dir_all(path.parent().expect("event parent")).expect("event directory");
                fs::write(&path, expected).expect("actual event write");
                String::new()
            }
            other => panic!("unexpected auxiliary tool {other}: {args}"),
        };
        record(&mut messages, &call, &result);
        let observed = messages.last_mut().expect("actual provider result");
        if read_succeeded {
            observed.source_read = Some(serde_json::json!({
                "path": PLAN_PATH, "success": true, "complete": true, "format": "raw"
            }));
        } else if read_failed {
            observed.is_error = true;
        }
    }
    panic!("auxiliary setup exceeded its three-transition bound");
}
