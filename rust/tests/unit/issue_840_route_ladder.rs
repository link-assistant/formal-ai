//! Route-only replay of the issue #840 task ladder (plan 10, leaf 21).
//!
//! The live ladder (`experiments/issue_840_task_ladder/run_ladder.sh`) boots a
//! server and judges each node on answer text *and* route. The answer half
//! needs the live round trip and stays there; the route half — which tools the
//! planner asks for and the shape of the commands it emits — is deterministic
//! and belongs in the `rust` CI stage, where every push re-checks it instead of
//! waiting for the scheduled ladder run. This replay is `ladder.py` ported
//! one-to-one: the same chat-completions seam (an in-process agent-mode server
//! via `create_chat_completion_with_solver`) with the same advertised tool
//! definitions (`bash`, `websearch`, descriptions included), the same
//! client-side execution contract (`/bin/sh -c` in the sandbox with
//! `FORMAL_AI_DESKTOP_DIR` exported, keyword-matched web fixtures), the same
//! four-step cap, and the same route-level judge fields
//! (`expect_tool`, `forbid_tool`, `command_forbid`). The prose fields
//! (`expect`, `expect_any`, `forbid`, refusals, capability menus) are the live
//! ladder's to judge and are deliberately not re-checked here.

use std::fs;
use std::path::Path;
use std::process::Command;

use formal_ai::protocol::{ChatCompletionRequest, ChatMessage, create_chat_completion_with_solver};
use formal_ai::solver::{ExecutionSurface, SolverConfig, UniversalSolver};
use serde_json::{Value, json};

/// The tool definitions the live ladder advertises — `ladder.py`'s `TOOLS`
/// list, verbatim down to the descriptions the server sees.
fn ladder_tool_definitions() -> Vec<Value> {
    ["bash", "websearch"]
        .into_iter()
        .map(|name| {
            let (description, property) = if name == "bash" {
                ("Execute a shell command", "command")
            } else {
                ("Search the web", "query")
            };
            json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": {
                        "type": "object",
                        "properties": { property: { "type": "string" } },
                        "required": [property],
                    },
                },
            })
        })
        .collect()
}

/// The live ladder's step cap: an agent loop that keeps calling tools past
/// four round trips has not converged, and its route is judged on what ran.
const MAX_STEPS: usize = 4;

fn tasks_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join("experiments/issue_840_task_ladder/tasks.json")
}

/// One replayed node: what the planner asked for, exactly as the live
/// harness collects it.
struct Replay {
    tools_called: Vec<String>,
    bash_commands: Vec<String>,
}

/// Execute one planned call the way `AgentLoop.execute` does, and return the
/// textual result to feed back to the planner.
fn execute_call(
    name: &str,
    arguments: &Value,
    sandbox: &Path,
    documents: &[Value],
    replay: &mut Replay,
) -> String {
    if name == "bash" {
        let Some(command) = arguments.get("command").and_then(Value::as_str) else {
            return String::from("(tool not executed by harness)");
        };
        replay.bash_commands.push(command.to_owned());
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .current_dir(sandbox)
            .env("FORMAL_AI_DESKTOP_DIR", sandbox.join("Desktop"))
            .output();
        return match output {
            Ok(output) => {
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    String::from("(no output)")
                } else {
                    trimmed.to_owned()
                }
            }
            Err(error) => format!("(command failed to start: {error})"),
        };
    }
    if name == "websearch" {
        let query = arguments
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_lowercase();
        for document in documents {
            let keywords = document
                .get("keywords")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let matched = keywords
                .iter()
                .filter_map(Value::as_str)
                .all(|keyword| query.contains(keyword.to_lowercase().as_str()));
            if matched {
                return document
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
            }
        }
        return String::from("(no fixture for this query)");
    }
    String::from("(tool not executed by harness)")
}

/// Replay one node's prompt through the same seam the live ladder measures —
/// an OpenAI chat-completions request with the ladder's tools against the
/// agent-mode solver — executing each planned call with the harness's
/// client-side contract, up to the live ladder's step cap.
fn replay_node(prompt: &str, sandbox: &Path, documents: &[Value]) -> Replay {
    let mut replay = Replay {
        tools_called: Vec::new(),
        bash_commands: Vec::new(),
    };
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        execution_surface: ExecutionSurface::HttpServer,
        ..SolverConfig::default()
    });
    let tools = ladder_tool_definitions();
    let mut messages = vec![ChatMessage::user(prompt)];
    for step in 0..MAX_STEPS {
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
            break;
        };
        let mut message = choice.message;
        let calls = std::mem::take(&mut message.tool_calls);
        if calls.is_empty() {
            break;
        }
        let ids: Vec<String> = calls
            .iter()
            .enumerate()
            .map(|(index, _)| format!("call_{step}_{index}"))
            .collect();
        messages.push(message);
        for (call, id) in calls.iter().zip(ids.iter()) {
            replay.tools_called.push(call.function.name.clone());
            let arguments: Value =
                serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);
            let result = execute_call(
                &call.function.name,
                &arguments,
                sandbox,
                documents,
                &mut replay,
            );
            messages.push(ChatMessage::tool_result(
                id.clone(),
                call.function.name.clone(),
                result,
            ));
        }
    }
    replay
}

struct Violation {
    field: &'static str,
    detail: String,
}

/// The route-level half of `ladder.py`'s `judge`: tool presence/absence and
/// forbidden command fragments. Casefolded on both sides, like the live judge.
fn judge_route(node: &Value, replay: &Replay) -> Vec<Violation> {
    let mut violations = Vec::new();
    let called: Vec<String> = replay
        .tools_called
        .iter()
        .map(|name| name.to_lowercase())
        .collect();
    for name in node
        .get("expect_tool")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let name = name.as_str().unwrap_or_default().to_lowercase();
        if !called.contains(&name) {
            violations.push(Violation {
                field: "expect_tool",
                detail: format!("planner never called {name}"),
            });
        }
    }
    for name in node
        .get("forbid_tool")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let name = name.as_str().unwrap_or_default().to_lowercase();
        if called.contains(&name) {
            violations.push(Violation {
                field: "forbid_tool",
                detail: format!("planner called forbidden {name}"),
            });
        }
    }
    let command_text = replay.bash_commands.join("\n").to_lowercase();
    for fragment in node
        .get("command_forbid")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let fragment = fragment.as_str().unwrap_or_default().to_lowercase();
        if command_text.contains(&fragment) {
            violations.push(Violation {
                field: "command_forbid",
                detail: format!("command used forbidden fragment {fragment:?}"),
            });
        }
    }
    violations
}

/// Materialize the dataset's sandbox spec (empty dirs and files reproducing
/// the #838 desktop layout), the way `run_ladder.sh` does before booting.
fn build_sandbox(spec: &Value, root: &Path) {
    for dir in spec
        .get("dirs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let Some(dir) = dir.as_str() else { continue };
        fs::create_dir_all(root.join(dir)).expect("sandbox dir");
    }
    for file in spec
        .get("files")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let Some(file) = file.as_str() else { continue };
        let path = root.join(file);
        fs::create_dir_all(path.parent().expect("sandbox file parent")).expect("sandbox parent");
        fs::File::create(path).expect("sandbox file");
    }
}

#[test]
fn route_only_ladder_replay_holds_for_every_node() {
    let dataset: Value =
        serde_json::from_str(&fs::read_to_string(tasks_path()).expect("tasks.json"))
            .expect("tasks.json parses");
    let nodes = dataset
        .get("tasks")
        .and_then(Value::as_array)
        .expect("tasks array")
        .clone();
    assert!(
        nodes.len() >= 40,
        "the ladder dataset should still carry every stable node, found {}",
        nodes.len()
    );

    let sandbox_root = std::env::temp_dir().join("formal-ai-route-ladder-sandbox");
    let _ = fs::remove_dir_all(&sandbox_root);
    fs::create_dir_all(&sandbox_root).expect("sandbox root");
    build_sandbox(
        dataset.get("sandbox").unwrap_or(&Value::Null),
        &sandbox_root,
    );
    let documents: Vec<Value> = serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("the repository root sits one level above the crate")
                .join("experiments/issue_840_task_ladder/web_fixtures.json"),
        )
        .expect("web_fixtures.json"),
    )
    .map(|fixtures: Value| {
        fixtures
            .get("documents")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    })
    .unwrap_or_default();

    let mut failures = Vec::new();
    for node in &nodes {
        let id = node.get("id").and_then(Value::as_str).unwrap_or("?");
        let prompt = node
            .get("prompt")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let replay = replay_node(prompt, &sandbox_root, &documents);
        for violation in judge_route(node, &replay) {
            failures.push(format!(
                "{id}: {} — {} (tools called: {:?}, commands: {:?})",
                violation.field, violation.detail, replay.tools_called, replay.bash_commands
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "route-only ladder replay failed for {} node(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
