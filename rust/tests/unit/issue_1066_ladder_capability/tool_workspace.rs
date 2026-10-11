//! Test-only actual workspace adapter; process observations never come from canned text.
use formal_ai::agent::{AgentError, AgentWorkspace, AgentWorkspaceConfig};
use formal_ai::agentic_coding::PlannedToolCall;
use formal_ai::agentic_coding::planner::{Capability, tool_capability};
use formal_ai::protocol::ChatMessage;
use serde_json::{Value, json};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct ToolWorkspace {
    workspace: AgentWorkspace,
    next_run: u64,
}

fn argument(args: &Value, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| args[*name].as_str().map(str::to_owned))
}

impl ToolWorkspace {
    pub fn new(prompt: &str) -> Self {
        let config = AgentWorkspaceConfig {
            time_budget: Duration::from_secs(2),
            ..AgentWorkspaceConfig::default()
        };
        Self {
            workspace: AgentWorkspace::for_prompt(prompt, &config).expect("fixture workspace"),
            next_run: 0,
        }
    }

    pub fn write_initial(&self, path: &str, content: &str) -> Result<(), AgentError> {
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

    pub fn read(&self, path: &str) -> Result<String, AgentError> {
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

    pub fn execute(&mut self, id: &str, call: &PlannedToolCall) -> ChatMessage {
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
                Err(error) => {
                    let missing = matches!(&error, AgentError::Io(error)
                        if error.kind() == std::io::ErrorKind::NotFound);
                    let mut message =
                        ChatMessage::tool_result_error(id, &call.tool, error.to_string());
                    message.source_read = Some(json!({
                        "path": path, "success": false, "complete": false, "format": "raw",
                        "error_code": if missing { "ENOENT" } else { "UNKNOWN" },
                    }));
                    message
                }
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
            Some(Capability::Edit) => {
                let edited = (|| -> Result<String, AgentError> {
                    let path = path
                        .as_deref()
                        .ok_or_else(|| std::io::Error::other("Edit path is missing"))?;
                    let old = argument(&args, &["oldString", "old_string", "old_str", "old"])
                        .ok_or_else(|| std::io::Error::other("Edit anchor is missing"))?;
                    let new = argument(&args, &["newString", "new_string", "new_str", "new"])
                        .ok_or_else(|| std::io::Error::other("Edit replacement is missing"))?;
                    let replace_all = match args.get("replaceAll") {
                        None => false,
                        Some(Value::Bool(value)) => *value,
                        Some(_) => return Err(std::io::Error::other("Invalid replaceAll").into()),
                    };
                    if old.is_empty() {
                        self.write_initial(path, &new)?;
                        return Ok(new);
                    }
                    let current = self.read(path)?;
                    let first = current.find(&old);
                    if first.is_none() || (!replace_all && first != current.rfind(&old)) {
                        return Err(
                            std::io::Error::other("Edit anchor missing or ambiguous").into()
                        );
                    }
                    let updated = if replace_all {
                        current.replace(&old, &new)
                    } else {
                        current.replacen(&old, &new, 1)
                    };
                    self.write_initial(path, &updated)?;
                    Ok(updated)
                })();
                match edited {
                    Ok(actual) => json!({"success": true, "bytes": actual.len()}),
                    Err(error) => json!({"is_error": true, "error": error.to_string()}),
                }
            }
            Some(Capability::Run) => self.run_shell(
                &argument(&args, &["command", "cmd", "script", "shell_command"])
                    .expect("Run command"),
            ),
            _ => json!({"is_error": true, "error": "unsupported fixture tool"}),
        };
        let failed = result["is_error"].as_bool() == Some(true)
            || result["timed_out"].as_bool() == Some(true)
            || result
                .get("exit_code")
                .is_some_and(|status| status.as_i64() != Some(0));
        if failed {
            ChatMessage::tool_result_error(id, &call.tool, result.to_string())
        } else {
            ChatMessage::tool_result(id, &call.tool, result.to_string())
        }
    }
}

impl Drop for ToolWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.workspace.root());
    }
}

#[cfg(test)]
mod process_observation_tests {
    use super::*;

    #[test]
    fn nonzero_exit_marks_failure_and_preserves_both_streams() {
        let mut workspace = ToolWorkspace::new("observe a failing command");
        let call = PlannedToolCall {
            tool: "run_shell_command".to_owned(),
            arguments:
                json!({"command": "printf 'actual stdout'; printf 'actual stderr' >&2; exit 7"})
                    .to_string(),
        };
        let observed = workspace.execute("failed-command", &call);
        assert!(observed.is_error);
        let receipt: Value = serde_json::from_str(&observed.content.plain_text()).unwrap();
        assert_eq!(receipt["exit_code"], 7);
        assert_eq!(receipt["stdout"], "actual stdout");
        assert_eq!(receipt["stderr"], "actual stderr");
        assert_eq!(receipt["timed_out"], false);
        assert_eq!(receipt["complete"], true);
    }

    #[test]
    fn successful_error_named_stdout_is_data_not_a_failure() {
        let mut workspace = ToolWorkspace::new("observe successful authored text");
        let call = PlannedToolCall {
            tool: "run_shell_command".to_owned(),
            arguments: json!({"command": "printf 'Error: authored text'"}).to_string(),
        };
        let observed = workspace.execute("successful-command", &call);
        assert!(!observed.is_error);
        let receipt: Value = serde_json::from_str(&observed.content.plain_text()).unwrap();
        assert_eq!(receipt["exit_code"], 0);
        assert_eq!(receipt["stdout"], "Error: authored text");
        assert_eq!(receipt["stderr"], "");
    }

    #[test]
    fn unsupported_tool_is_explicitly_a_failed_observation() {
        let mut workspace = ToolWorkspace::new("observe an unsupported tool");
        let call = PlannedToolCall {
            tool: "unknown_fixture_provider".to_owned(),
            arguments: "{}".to_owned(),
        };
        let observed = workspace.execute("unsupported-command", &call);
        assert!(observed.is_error);
        let receipt: Value = serde_json::from_str(&observed.content.plain_text()).unwrap();
        assert_eq!(receipt["error"], "unsupported fixture tool");
    }
}
