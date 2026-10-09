//! Test-only actual workspace adapter; process observations never come from canned text.
use formal_ai::agent::{AgentError, AgentWorkspace, AgentWorkspaceConfig};
use formal_ai::agentic_coding::PlannedToolCall;
use formal_ai::agentic_coding::planner::{Capability, tool_capability};
use formal_ai::protocol::ChatMessage;
use serde_json::{Value, json};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub(crate) struct ToolWorkspace {
    workspace: AgentWorkspace,
    next_run: u64,
}

fn argument(args: &Value, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| args[*name].as_str().map(str::to_owned))
}

impl ToolWorkspace {
    pub(crate) fn new(prompt: &str) -> Self {
        let config = AgentWorkspaceConfig {
            time_budget: Duration::from_secs(2),
            ..AgentWorkspaceConfig::default()
        };
        Self {
            workspace: AgentWorkspace::for_prompt(prompt, &config).expect("fixture workspace"),
            next_run: 0,
        }
    }

    pub(crate) fn write_initial(&mut self, path: &str, content: &str) -> Result<(), AgentError> {
        self.write(path, content)?;
        if self.read(path)? != content {
            return Err(std::io::Error::other("physical write bytes differ").into());
        }
        Ok(())
    }

    fn write(&self, path: &str, content: &str) -> Result<(), AgentError> {
        let target = std::path::Path::new(path);
        if target.as_os_str().is_empty()
            || target.is_absolute()
            || !target
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)))
        {
            return Err(AgentError::PathEscapesWorkspace(path.to_owned()));
        }
        let target = self.workspace.root().join(target);
        std::fs::create_dir_all(target.parent().expect("workspace file parent"))?;
        std::fs::write(target, content)?;
        Ok(())
    }

    pub(crate) fn read(&self, path: &str) -> Result<String, AgentError> {
        self.workspace.read_file(path)
    }

    fn run_shell(&mut self, command: &str) -> Value {
        self.next_run += 1;
        let stdout_path = self
            .workspace
            .root()
            .join(format!(".tool-stdout-{}", self.next_run));
        let stderr_path = self
            .workspace
            .root()
            .join(format!(".tool-stderr-{}", self.next_run));
        let stdout = std::fs::File::create(&stdout_path).expect("stdout capture");
        let stderr = std::fs::File::create(&stderr_path).expect("stderr capture");
        // File-backed streams avoid waiting on a timed-out descendant holding a pipe open.
        let spawned = Command::new("sh")
            .args(["-c", command])
            .current_dir(self.workspace.root())
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => {
                return json!({"is_error": true, "error": error.to_string(), "complete": false});
            }
        };
        let started = Instant::now();
        let (status, timed_out) = loop {
            match child.try_wait() {
                Ok(Some(status)) => break (status, false),
                Ok(None) if started.elapsed() >= Duration::from_secs(2) => {
                    let _ = child.kill();
                    match child.wait() {
                        Ok(status) => {
                            let timed_out = !status.success();
                            break (status, timed_out);
                        }
                        Err(error) => {
                            return json!({"is_error": true, "error": error.to_string(), "complete": false});
                        }
                    }
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return json!({"is_error": true, "error": error.to_string(), "complete": false});
                }
            }
        };
        let streams = std::fs::read_to_string(&stdout_path).and_then(|stdout| {
            std::fs::read_to_string(&stderr_path).map(|stderr| (stdout, stderr))
        });
        let _ = std::fs::remove_file(stdout_path);
        let _ = std::fs::remove_file(stderr_path);
        match streams {
            Ok((stdout, stderr)) => json!({"stdout": stdout, "stderr": stderr,
                "exit_code": status.code(), "timed_out": timed_out, "complete": !timed_out}),
            Err(error) => {
                json!({"is_error": true, "error": error.to_string(), "exit_code": status.code(), "complete": false})
            }
        }
    }

    pub(crate) fn execute(&mut self, id: &str, call: &PlannedToolCall) -> ChatMessage {
        let args: Value = serde_json::from_str(&call.arguments).expect("tool arguments");
        let path = argument(&args, &["path", "filePath", "file_path", "absolute_path"]);
        let capability = tool_capability(&call.tool);
        if capability == Some(Capability::Read) {
            let path = path.expect("Read path");
            return match self.read(&path) {
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
        let result = match capability {
            Some(Capability::Write) => {
                let path = path.expect("Write path");
                let content = argument(&args, &["content", "contents", "text", "new_string"])
                    .expect("Write content");
                if let Err(error) = self.write_initial(&path, &content) {
                    return ChatMessage::tool_result_error(id, &call.tool, error.to_string());
                }
                match self.read(&path) {
                    Ok(actual) if actual == content => {
                        json!({"success": true, "path": path, "bytes": actual.len()})
                    }
                    other => {
                        json!({"is_error": true, "error": format!("Write not observed: {other:?}")})
                    }
                }
            }
            Some(Capability::Run) => self.run_shell(
                &argument(&args, &["command", "cmd", "script", "shell_command"])
                    .expect("Run command"),
            ),
            _ => json!({"is_error": true, "error": "unsupported fixture tool"}),
        };
        ChatMessage::tool_result(id, &call.tool, result.to_string())
    }
}

impl Drop for ToolWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.workspace.root());
    }
}
