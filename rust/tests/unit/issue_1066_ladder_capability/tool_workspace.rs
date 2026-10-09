//! Actual bounded filesystem/process receipts for ladder planning fixtures.
use formal_ai::agent::{AgentWorkspace, AgentWorkspaceConfig};
use formal_ai::agentic_coding::PlannedToolCall;
use formal_ai::protocol::ChatMessage;
use serde_json::{Value, json};

pub(super) struct ToolWorkspace(AgentWorkspace);

impl ToolWorkspace {
    pub(super) fn new(prompt: &str) -> Self {
        let config = AgentWorkspaceConfig {
            time_budget: std::time::Duration::from_secs(2),
            ..AgentWorkspaceConfig::default()
        };
        Self(AgentWorkspace::for_prompt(prompt, &config).expect("fixture workspace"))
    }

    pub(super) fn execute(&mut self, id: &str, call: &PlannedToolCall) -> ChatMessage {
        let args: Value = serde_json::from_str(&call.arguments).expect("tool arguments");
        let path = super::argument(&args, &["path", "filePath", "file_path", "absolute_path"]);
        if call.tool == "read" {
            let path = path.expect("Read path");
            return match self.0.read_file(&path) {
                Ok(content) => {
                    let mut message = ChatMessage::tool_result(id, &call.tool, content);
                    message.source_read = Some(
                        json!({"path": path, "success": true, "complete": true, "format": "raw"}),
                    );
                    message
                }
                Err(error) => ChatMessage::tool_result_error(id, &call.tool, error.to_string()),
            };
        }
        let result = match call.tool.as_str() {
            "write" => {
                let path = path.expect("Write path");
                let content =
                    super::argument(&args, &["content", "contents", "text", "new_string"])
                        .expect("Write content");
                self.0.create_file(&path, &content);
                match self.0.read_file(&path) {
                    Ok(actual) if actual == content => {
                        json!({"success": true, "path": path, "bytes": actual.len()})
                    }
                    other => {
                        json!({"is_error": true, "error": format!("Write not observed: {other:?}")})
                    }
                }
            }
            "bash" => {
                let command = args["command"].as_str().expect("Bash command");
                self.0.run_command(command);
                self.0.last_command_result().map_or_else(
                    || json!({"is_error": true, "error": "command produced no observation"}),
                    |result| {
                        json!({"stdout": result.stdout, "stderr": result.stderr,
                        "exit_code": result.status_code, "timed_out": result.timed_out,
                        "complete": !result.timed_out})
                    },
                )
            }
            _ => json!({"is_error": true, "error": "unsupported fixture tool"}),
        };
        ChatMessage::tool_result(id, &call.tool, result.to_string())
    }
}

impl Drop for ToolWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.root());
    }
}
