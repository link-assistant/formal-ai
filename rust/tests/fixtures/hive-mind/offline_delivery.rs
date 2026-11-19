//! Offline Hive Mind delivery harness shared by the issue #1157, #1158 and
//! #1160 unit tests (sub-issues of #1162).
//!
//! One Hello World work item is driven end to end by the Rust planner
//! (`plan_chat_step`) with no LLM: every planned tool call runs against a real
//! temporary git repository with a bare `origin`, and `gh` is the fake in
//! `fake-gh.cjs` (shared with `experiments/issue-1162/offline-hive-end-to-end.mjs`),
//! which serves the issue, records every pull request call and simulates CI
//! from the pushed workflows. No network and no credentials are used, so this
//! is local end-to-end evidence, not hosted CI evidence.
//!
//! Tool results use the Agent CLI shapes the JS driver
//! (`experiments/js_dogfood/drive.mjs`) produces: `Output: …\nExit Code: n`
//! for bash, a numbered `<file>` frame for read, and an empty result for a
//! successful write or edit.
#![allow(dead_code)]

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const ISSUE: &str = "https://github.com/example/test-hello-world/issues/1";
pub const PR: &str = "https://github.com/example/test-hello-world/pull/2";
pub const BRANCH: &str = "issue-1-0123456789ab";
pub const TOOLS: [&str; 4] = ["bash", "read", "write", "edit"];

/// Hive Mind's formal-ai prompt, verbatim from the 2026-09-27 Kotlin run.
pub const FORMAL_AI_PROMPT: &str = include_str!("formal-ai-prompt.txt");
/// Hive Mind's regular prompt ("Issue to solve: … Your prepared Pull Request: …").
pub const REGULAR_PROMPT: &str = include_str!("regular-prompt.txt");
/// Hive Mind's real auto-restart prompt after a session left `?? Main.jar`.
pub const RESTART_PROMPT: &str = include_str!("restart-uncommitted-main-jar.txt");
const FAKE_GH: &str = include_str!("fake-gh.cjs");
const ISSUE_BODY: &str = include_str!("../issue-1162/hello-world-kotlin-issue.md");
const RECORDED_REPOSITORY: &str =
    "https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a";
const RECORDED_BRANCH: &str = "issue-1-604f2202fd18";

/// A recorded prompt with its work item URLs pointed at the offline fixture.
#[must_use]
pub fn offline(prompt: &str) -> String {
    prompt
        .replace(&format!("{RECORDED_REPOSITORY}/issues/1"), ISSUE)
        .replace(&format!("{RECORDED_REPOSITORY}/pull/2"), PR)
        .replace(RECORDED_BRANCH, BRANCH)
}

/// Whether every command is on PATH; tests skip (and say so) otherwise.
#[must_use]
pub fn available(commands: &[&str]) -> bool {
    let missing: Vec<&str> = commands
        .iter()
        .copied()
        .filter(|command| {
            !Command::new("sh")
                .args(["-c", &format!("command -v {command} >/dev/null 2>&1")])
                .status()
                .is_ok_and(|status| status.success())
        })
        .collect();
    if !missing.is_empty() {
        eprintln!("skipped: needs {} on PATH", missing.join(", "));
    }
    missing.is_empty()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// A temporary checkout on `BRANCH` with a bare origin and the fake `gh`.
pub struct Workspace {
    pub root: PathBuf,
    pub dir: PathBuf,
    pub origin: PathBuf,
    pub bin: PathBuf,
    pub state: PathBuf,
    branch: String,
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

impl Workspace {
    #[must_use]
    pub fn new(language: &str, label: &str) -> Self {
        Self::for_work_item(language, label, BRANCH, PR)
    }

    pub fn for_work_item(language: &str, label: &str, branch: &str, pr: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "formal-ai-offline-delivery-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("work");
        let origin = root.join("origin.git");
        let bin = root.join("bin");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&bin).unwrap();
        git(&root, &["init", "-q", "--bare", origin.to_str().unwrap()]);
        git(&dir, &["init", "-q", "-b", branch]);
        git(&dir, &["config", "user.name", "Offline fixture"]);
        git(&dir, &["config", "user.email", "fixture@example.invalid"]);
        std::fs::write(dir.join("README.md"), "# test-hello-world\n").unwrap();
        git(&dir, &["add", "--", "README.md"]);
        git(&dir, &["commit", "-qm", "Initial commit"]);
        git(&dir, &["remote", "add", "origin", origin.to_str().unwrap()]);
        git(&dir, &["push", "-q", "-u", "origin", branch]);
        let state = root.join("gh-state.json");
        let body = ISSUE_BODY.replace("Kotlin", language);
        std::fs::write(
            &state,
            serde_json::to_string_pretty(&serde_json::json!({
                "calls": [],
                "issue": { "title": format!("Implement Hello World in {language}"), "body": body },
                "branch": branch,
                "pr": { "title": "[WIP] Implement Hello World", "body": "Work in progress", "isDraft": true, "url": pr },
            }))
            .unwrap(),
        )
        .unwrap();
        let gh = bin.join("gh");
        std::fs::write(&gh, FAKE_GH).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self {
            root,
            dir,
            origin,
            bin,
            state,
            branch: branch.to_owned(),
        }
    }

    fn within(&self, path: &str) -> PathBuf {
        let full = self.dir.join(path);
        assert!(
            !path.split('/').any(|part| part == "..") && full.starts_with(&self.dir),
            "path escapes the sandbox: {path}"
        );
        full
    }

    fn bash(&self, command: &str) -> String {
        let path = format!(
            "{}:{}",
            self.bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let output = Command::new("timeout")
            .args(["300", "/bin/sh", "-c", command])
            .current_dir(&self.dir)
            .env("PATH", path)
            .env("FAKE_GH_STATE", &self.state)
            .output()
            .expect("sh runs");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status = output.status.code().unwrap_or(-1);
        if status == 0 && stderr.is_empty() {
            format!("Output: {stdout}\nExit Code: 0")
        } else {
            format!("Output: {stdout}\nError: {stderr}\nExit Code: {status}")
        }
    }

    fn failure(message: &str) -> String {
        serde_json::json!({ "is_error": true, "error": message }).to_string()
    }

    /// Execute one tool call the way the Agent CLI does.
    #[must_use]
    pub fn execute(&self, tool: &str, args: &Value) -> String {
        let path = ["filePath", "file_path", "path"]
            .iter()
            .find_map(|key| args[*key].as_str())
            .unwrap_or_default();
        match tool {
            "bash" => self.bash(args["command"].as_str().unwrap_or_default()),
            "read" => match std::fs::read_to_string(self.within(path)) {
                Ok(text) => {
                    let lines: Vec<&str> = text.split('\n').collect();
                    let body = lines
                        .iter()
                        .enumerate()
                        .map(|(index, line)| format!("{:05}| {line}", index + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!(
                        "<file>\n{body}\n\n(End of file - total {} lines)\n</file>",
                        lines.len()
                    )
                }
                Err(error) => Self::failure(&error.to_string()),
            },
            "write" => {
                if !args["append_mode"].is_null() {
                    return Self::failure("append requires the explicit receipt adapter");
                }
                let target = self.within(path);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::write(target, args["content"].as_str().unwrap_or_default()).unwrap();
                String::new()
            }
            "edit" => {
                let target = self.within(path);
                let text = std::fs::read_to_string(&target).unwrap_or_default();
                let old = args["oldString"]
                    .as_str()
                    .or_else(|| args["old_string"].as_str())
                    .unwrap_or_default();
                let new = args["newString"]
                    .as_str()
                    .or_else(|| args["new_string"].as_str())
                    .unwrap_or_default();
                if old.is_empty() {
                    std::fs::write(target, new).unwrap();
                    return String::new();
                }
                if !text.contains(old) {
                    return Self::failure("oldString not found in content");
                }
                let all = args["replaceAll"].as_bool().unwrap_or(false);
                if !all && text.matches(old).count() > 1 {
                    return Self::failure("Found multiple matches for oldString.");
                }
                let next = if all {
                    text.replace(old, new)
                } else {
                    text.replacen(old, new, 1)
                };
                std::fs::write(target, next).unwrap();
                String::new()
            }
            other => Self::failure(&format!("the offline driver does not execute {other}")),
        }
    }

    /// Drive one session to its final answer (or `steps` tool calls).
    #[must_use]
    pub fn session(&self, prompt: &str, steps: usize) -> Session {
        let mut messages = vec![ChatMessage::user(prompt)];
        let mut calls = Vec::new();
        for step in 0..steps {
            match plan_chat_step(&messages, &TOOLS) {
                Some(AgenticPlan::ToolCalls(planned)) => {
                    let call = planned[0].clone();
                    let args: Value =
                        serde_json::from_str(&call.arguments).expect("call arguments");
                    let result = self.execute(&call.tool, &args);
                    let id = format!("offline-{step}");
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        id.clone(),
                        call.tool.clone(),
                        call.arguments.clone(),
                    )]));
                    messages.push(ChatMessage::tool_result(id, &call.tool, &result));
                    calls.push(Call {
                        tool: call.tool,
                        args,
                        result,
                    });
                }
                Some(AgenticPlan::Final(answer)) => {
                    return Session {
                        calls,
                        answer: Some(answer),
                    };
                }
                None => break,
            }
        }
        Session {
            calls,
            answer: None,
        }
    }

    /// What the session left behind: the pull request, every `gh` call, the
    /// working tree and what origin holds.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        let state: Value =
            serde_json::from_str(&std::fs::read_to_string(&self.state).unwrap()).unwrap();
        let gh_calls = state["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|call| {
                call.as_array()
                    .unwrap()
                    .iter()
                    .take(2)
                    .map(|part| part.as_str().unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        let lines = |text: String| -> Vec<String> {
            text.lines()
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect()
        };
        Outcome {
            pr: state["pr"].clone(),
            gh_calls,
            status: git(&self.dir, &["status", "--porcelain"]),
            tracked: lines(git(&self.dir, &["ls-files"])),
            pushed: lines(git(
                &self.origin,
                &["ls-tree", "-r", "--name-only", &self.branch],
            )),
            head: git(&self.dir, &["rev-parse", "HEAD"]),
            pushed_head: git(&self.origin, &["rev-parse", &self.branch]),
            commits: git(&self.dir, &["rev-list", "--count", "HEAD"])
                .parse()
                .unwrap(),
        }
    }
}

pub struct Call {
    pub tool: String,
    pub args: Value,
    pub result: String,
}

impl Call {
    #[must_use]
    pub fn command(&self) -> &str {
        self.args["command"].as_str().unwrap_or_default()
    }
}

pub struct Session {
    pub calls: Vec<Call>,
    pub answer: Option<String>,
}

impl Session {
    #[must_use]
    pub fn answer(&self) -> &str {
        self.answer.as_deref().unwrap_or_else(|| {
            panic!(
                "no final answer after {} calls; last: {:?}",
                self.calls.len(),
                self.calls
                    .last()
                    .map(|call| (&call.tool, call.command(), &call.result))
            )
        })
    }

    /// Every bash command, in order.
    pub fn commands(&self) -> impl Iterator<Item = &str> {
        self.calls
            .iter()
            .filter(|call| call.tool == "bash")
            .map(Call::command)
    }
}

pub struct Outcome {
    pub pr: Value,
    pub gh_calls: Vec<String>,
    pub status: String,
    pub tracked: Vec<String>,
    pub pushed: Vec<String>,
    pub commits: usize,
    pub head: String,
    pub pushed_head: String,
}

/// The delivered state every Hive Mind client must reach (issue #1162 acceptance 3).
pub fn assert_delivered(workspace: &Workspace, language: &str, program: &str) -> Outcome {
    let outcome = workspace.outcome();
    assert_eq!(
        outcome.pr["title"],
        format!("Implement Hello World in {language}")
    );
    assert_eq!(outcome.pr["isDraft"], false, "the pull request is ready");
    let body = outcome.pr["body"].as_str().unwrap_or_default();
    assert!(
        body.starts_with(&format!("Verified artifact: {program}\n")),
        "{body}"
    );
    assert!(body.contains("- `sh tests/verify-output.sh`"), "{body}");
    assert!(body.ends_with(&format!("Resolves {ISSUE}")), "{body}");
    assert_eq!(outcome.status, "", "the working tree is clean");
    let delivered = [
        ".github/workflows/test-hello-world.yml",
        "README.md",
        program,
        "tests/verify-output.sh",
    ];
    for file in delivered {
        assert!(
            outcome.pushed.iter().any(|pushed| pushed == file),
            "{file} is pushed: {:?}",
            outcome.pushed
        );
    }
    assert!(
        outcome
            .pushed
            .iter()
            .all(|file| delivered.contains(&file.as_str()) || file == ".gitignore"),
        "{:?}",
        outcome.pushed
    );
    outcome
}
