//! Offline agentic driver — the in-repo "agentic CLI" that drives the server.
//!
//! This is the *client* side of issue #468's loop. The maintainer's framing:
//! *"our Formal AI system should have enough skills (meta algorithm, rust code) to
//! actually call all the tools from any agentic CLI, understand errors from tools,
//! and so on, call bash commands, do web fetch and web search, to actually
//! complete the task."* An external agentic CLI (link-assistant/agent, gemini-cli,
//! …) would normally play this role against our OpenAI-compatible server; this
//! module plays it in-repo so the whole loop runs offline and deterministically in
//! CI.
//!
//! It advertises a tool set, sends a chat request, and whenever the server answers
//! with `tool_calls` it **executes** each call — `web_search` / `web_fetch`
//! against the offline [`corpus`], `write_file` / `run_command` against a single
//! reused, sandboxed [`AgentWorkspace`] — feeds each result back as a `tool`
//! message, and loops until the server answers with `finish_reason: "stop"`. The
//! loop is bounded by a hard turn cap, so unbounded reasoning stays a NON-GOAL and
//! neural inference is never involved.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::corpus;
use crate::agent::{AgentCommandResult, AgentError, AgentWorkspace, AgentWorkspaceConfig};
use crate::protocol::{
    create_chat_completion_with_solver, ChatCompletionRequest, ChatMessage, ToolCall,
};
use crate::solver::{SolverConfig, UniversalSolver};

/// The tool set the driver advertises — the four capabilities the planner's recipe
/// relies on (`web_search` → `web_fetch` → `write_file` → `run_command`).
pub const DRIVER_TOOLS: [&str; 4] = ["web_search", "web_fetch", "write_file", "run_command"];

/// A hard cap on agentic turns (server round-trips). The recipe needs five; the
/// cap is generous but finite so the loop can never run away.
const MAX_TURNS: usize = 12;

/// One executed tool call, recorded for the transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverToolStep {
    /// The tool name the server requested.
    pub tool: String,
    /// The JSON-encoded arguments the server requested.
    pub arguments: String,
    /// The result the driver fed back as a `tool` message.
    pub result: String,
}

/// The outcome of an agentic run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverOutcome {
    /// The task the driver was asked to solve.
    pub task: String,
    /// Every tool call the driver executed, in order.
    pub steps: Vec<DriverToolStep>,
    /// The server's final assistant text (the knowledge base inline).
    pub final_answer: String,
    /// How many server round-trips the loop took.
    pub turns: usize,
    /// Whether the loop stopped at `MAX_TURNS` rather than a final answer.
    pub hit_turn_cap: bool,
}

impl DriverOutcome {
    /// A human-readable transcript of the tool calls and how the loop ended.
    #[must_use]
    pub fn transcript(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "task: {}", self.task);
        let _ = writeln!(
            out,
            "turns: {} (tool calls: {})",
            self.turns,
            self.steps.len()
        );
        for (index, step) in self.steps.iter().enumerate() {
            let _ = writeln!(out, "  [{}] {} {}", index + 1, step.tool, step.arguments);
            let _ = writeln!(out, "      -> {}", preview(&step.result, 200));
        }
        if self.hit_turn_cap {
            let _ = writeln!(out, "(stopped at the {MAX_TURNS}-turn safety cap)");
        }
        out
    }

    /// A stable, replayable JSON record of the whole agentic session — the task,
    /// the tools the CLI advertised, every executed tool call with its arguments
    /// and result, the turn count, and the final answer. This is the "Agent CLI
    /// session that solved the task" artifact: it is deterministic (no clock, no
    /// randomness, no network), so committing it to the repo documents exactly how
    /// Formal AI drove its own CLI to the solution.
    #[must_use]
    pub fn session_json(&self) -> Value {
        json!({
            "task": self.task,
            "driver": "formal-ai in-repo agentic CLI",
            "server": "formal-ai OpenAI-compatible chat completions",
            "tools_advertised": DRIVER_TOOLS,
            "turns": self.turns,
            "hit_turn_cap": self.hit_turn_cap,
            "steps": self.steps.iter().map(|step| json!({
                "tool": step.tool,
                "arguments": serde_json::from_str::<Value>(&step.arguments)
                    .unwrap_or_else(|_| Value::String(step.arguments.clone())),
                "result": step.result,
            })).collect::<Vec<_>>(),
            "final_answer": self.final_answer,
        })
    }
}

/// Drive the agentic loop for `task` using a default sandbox workspace.
///
/// # Errors
///
/// Returns an [`AgentError`] if the isolated workspace cannot be created.
pub fn run_agentic_task(task: &str) -> Result<DriverOutcome, AgentError> {
    run_agentic_task_in(task, &AgentWorkspaceConfig::default())
}

/// Drive the agentic loop for `task` using the given workspace `config`.
///
/// # Errors
///
/// Returns an [`AgentError`] if the isolated workspace cannot be created.
pub fn run_agentic_task_in(
    task: &str,
    config: &AgentWorkspaceConfig,
) -> Result<DriverOutcome, AgentError> {
    // Agent mode is the explicit opt-in the server's tool gate requires; without
    // it the server refuses every tool. The driver is that isolated execution
    // environment, so it opts in.
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let tools = tool_definitions(&DRIVER_TOOLS);
    let mut workspace = AgentWorkspace::for_prompt(task, config)?;
    let mut messages = vec![ChatMessage::user(task)];
    let mut steps = Vec::new();
    let mut turns = 0usize;

    loop {
        if turns >= MAX_TURNS {
            return Ok(DriverOutcome {
                task: task.to_owned(),
                steps,
                final_answer: String::new(),
                turns,
                hit_turn_cap: true,
            });
        }
        turns += 1;

        let request = ChatCompletionRequest {
            model: None,
            messages: messages.clone(),
            temperature: None,
            stream: false,
            tools: tools.clone(),
            tool_choice: None,
            functions: Vec::new(),
            function_call: None,
            stream_options: None,
        };
        let completion = create_chat_completion_with_solver(&request, &solver);
        let Some(choice) = completion.choices.into_iter().next() else {
            return Ok(DriverOutcome {
                task: task.to_owned(),
                steps,
                final_answer: String::new(),
                turns,
                hit_turn_cap: false,
            });
        };

        let requested_tools =
            choice.finish_reason == "tool_calls" && !choice.message.tool_calls.is_empty();
        if !requested_tools {
            return Ok(DriverOutcome {
                task: task.to_owned(),
                steps,
                final_answer: choice.message.content.plain_text(),
                turns,
                hit_turn_cap: false,
            });
        }

        // Execute the requested tool calls, then append the assistant turn
        // followed by the `tool` results (order matters: the planner maps each
        // result's `tool_call_id` back to a *prior* assistant `tool_calls` turn).
        let assistant = choice.message;
        let mut results = Vec::with_capacity(assistant.tool_calls.len());
        for call in &assistant.tool_calls {
            let result = execute_tool_call(call, &mut workspace);
            steps.push(DriverToolStep {
                tool: call.function.name.clone(),
                arguments: call.function.arguments.clone(),
                result: result.clone(),
            });
            results.push(ChatMessage::tool_result(
                call.id.clone(),
                call.function.name.clone(),
                result,
            ));
        }
        messages.push(assistant);
        messages.extend(results);
    }
}

/// Execute one tool call against the offline corpus or the sandbox workspace and
/// return the textual result to feed back to the server.
fn execute_tool_call(call: &ToolCall, workspace: &mut AgentWorkspace) -> String {
    let arguments: Value = serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);
    match call.function.name.as_str() {
        "web_search" => corpus::web_search(arg_str(&arguments, "query")),
        "web_fetch" => corpus::web_fetch(arg_str(&arguments, "url")),
        "write_file" => {
            let path = arg_str(&arguments, "path");
            let content = arg_str(&arguments, "content");
            workspace.create_file(path, content);
            format!("wrote {} byte(s) to {path}", content.len())
        }
        "run_command" => {
            let command = arg_str(&arguments, "command");
            workspace.run_command(command);
            workspace.last_command_result().map_or_else(
                || format!("run_command produced no result for {command:?}"),
                format_command_result,
            )
        }
        other => format!("error: unsupported tool {other}"),
    }
}

/// OpenAI-shaped function tool definitions for the advertised tool `names`.
fn tool_definitions(names: &[&str]) -> Vec<Value> {
    names
        .iter()
        .map(|name| {
            json!({
                "type": "function",
                "function": { "name": name, "description": tool_description(name) },
            })
        })
        .collect()
}

fn tool_description(name: &str) -> &'static str {
    match name {
        "web_search" => "Search the web for sources. Arguments: {\"query\": string}.",
        "web_fetch" => "Fetch the text at a URL. Arguments: {\"url\": string}.",
        "write_file" => {
            "Write a workspace file. Arguments: {\"path\": string, \"content\": string}."
        }
        "run_command" => {
            "Run an allowlisted command in the workspace. Arguments: {\"command\": string}."
        }
        _ => "Tool.",
    }
}

/// Render a command result the way an agentic CLI would surface it: stdout on
/// success, an annotated error otherwise (so the agent can "understand errors").
fn format_command_result(result: &AgentCommandResult) -> String {
    if result.timed_out {
        return format!("command timed out: {}", result.command);
    }
    match result.status_code {
        Some(0) => result.stdout.clone(),
        Some(code) => format!(
            "command exited with status {code}\nstdout:\n{}\nstderr:\n{}",
            result.stdout, result.stderr
        ),
        None => format!(
            "command terminated without an exit status\nstderr:\n{}",
            result.stderr
        ),
    }
}

fn arg_str<'a>(arguments: &'a Value, key: &str) -> &'a str {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// Collapse whitespace and truncate `text` to `max` characters for the transcript.
fn preview(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max {
        return collapsed;
    }
    let truncated: String = collapsed.chars().take(max).collect();
    format!("{truncated}…")
}
