//! Run one issue-#1028 ladder node against the planner offline (issue #1066).
//!
//! `experiments/issue_1028_agent_cli_ladder/run.sh` is the ground truth: it
//! stands up `formal-ai serve --agent-mode` and drives it with the real
//! `@link-assistant/agent` CLI. That loop needs a release build, a free port and
//! about a minute per node, which is too slow to diagnose a routing gap with.
//!
//! This harness plays the CLI's part in-process. It advertises the same fourteen
//! tool names the Agent CLI advertises, executes each planned call against a
//! throwaway copy of the repository, and feeds the result back, so the planner
//! sees exactly the conversation it sees in the real run. Web tools answer with
//! an explicit unavailability error rather than a fabricated page: the harness is
//! offline and says so.
//!
//! Usage:
//!   cargo run --example issue_1066_ladder_node_offline -- \
//!       [--task <text>] [--node <path>] [--depth <n>] [--criterion <text>] \
//!       [--prompt <text>] [--workspace <dir>] [--turns <n>]
//!
//! With no arguments it runs leaf `1.1.1.1.1` of the committed ladder.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

/// The tool names `@link-assistant/agent` advertises to the server, in the
/// order the live trace recorded them.
const LADDER_TOOLS: [&str; 14] = [
    "bash",
    "batch",
    "codesearch",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "task",
    "todoread",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];

fn main() {
    let options = Options::parse();
    let workspace = options.workspace();
    println!("workspace: {}", workspace.display());
    println!("=== prompt ===\n{}\n", options.prompt);

    let mut messages = vec![ChatMessage::user(&options.prompt)];
    let mut answer = None;
    for turn in 0..options.turns {
        let Some(plan) = plan_chat_step(&messages, &LADDER_TOOLS) else {
            println!("turn {turn}: no plan (the planner declined the request)");
            break;
        };
        match plan {
            AgenticPlan::Final(text) => {
                answer = Some(text);
                break;
            }
            AgenticPlan::ToolCalls(calls) => {
                for (index, call) in calls.into_iter().enumerate() {
                    let result = execute(&call.tool, &call.arguments, &workspace);
                    println!(
                        "turn {turn}.{index}: {} {}\n  -> {}\n",
                        call.tool,
                        call.arguments,
                        first_lines(&result, 12)
                    );
                    let id = format!("call_{turn}_{index}");
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        id.clone(),
                        call.tool.clone(),
                        call.arguments.clone(),
                    )]));
                    messages.push(ChatMessage::tool_result(id, &call.tool, &result));
                }
            }
        }
    }

    match answer {
        Some(text) => println!("=== final ===\n{text}\n"),
        None => println!("=== no final answer within {} turns ===\n", options.turns),
    }
    report_evidence(&workspace);
}

/// Print every file the run left under the ladder's evidence directory, which
/// is what `run.sh` checks after the CLI exits.
fn report_evidence(workspace: &Path) {
    let evidence = workspace.join(".agent-ladder");
    let Ok(entries) = std::fs::read_dir(&evidence) else {
        println!("=== evidence ===\n(no {} directory)", evidence.display());
        return;
    };
    let mut names: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    names.sort();
    println!("=== evidence ===");
    for path in names {
        let body = std::fs::read_to_string(&path).unwrap_or_default();
        println!("--- {} ---\n{}", path.display(), first_lines(&body, 20));
    }
}

/// The head of a tool result, capped by line count *and* by line width.
///
/// A repository-wide grep can return a single line of several megabytes (one
/// minified JSON fixture is enough), which drowns the routing decision this
/// harness exists to show.
fn first_lines(text: &str, limit: usize) -> String {
    const WIDTH: usize = 200;
    let mut out: Vec<String> = text
        .lines()
        .take(limit)
        .map(|line| match line.char_indices().nth(WIDTH) {
            Some((cut, _)) => format!("{}…", &line[..cut]),
            None => line.to_owned(),
        })
        .collect();
    if text.lines().count() > limit {
        out.push("…".to_owned());
    }
    out.join("\n")
}

struct Options {
    prompt: String,
    turns: usize,
    workspace: Option<PathBuf>,
}

impl Options {
    fn parse() -> Self {
        let mut named: BTreeMap<String, String> = BTreeMap::new();
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let Some(key) = flag.strip_prefix("--") else {
                continue;
            };
            if let Some(value) = args.next() {
                named.insert(key.to_owned(), value);
            }
        }
        let node = named
            .get("node")
            .cloned()
            .unwrap_or_else(|| String::from("1.1.1.1.1"));
        let depth = named
            .get("depth")
            .cloned()
            .unwrap_or_else(|| String::from("5"));
        let criterion = named
            .get("criterion")
            .cloned()
            .unwrap_or_else(|| String::from("observable evidence exists"));
        let task = named.get("task").cloned().unwrap_or_else(|| {
            String::from(
                "Atomic task L01: Inspect the existing task-decomposition data model and \
                 identify where a node stores its children.",
            )
        });
        let prompt = named
            .get("prompt")
            .cloned()
            .unwrap_or_else(|| ladder_node_prompt(&task, &node, &depth, &criterion));
        Self {
            prompt,
            turns: named
                .get("turns")
                .and_then(|value| value.parse().ok())
                .unwrap_or(12),
            workspace: named.get("workspace").map(PathBuf::from),
        }
    }

    /// The node's working directory: the supplied one, or a throwaway copy of
    /// the committed tree, built the same way `run.sh` builds it.
    fn workspace(&self) -> PathBuf {
        if let Some(directory) = &self.workspace {
            std::fs::create_dir_all(directory).expect("create the requested workspace");
            return directory.clone();
        }
        let root = std::env::temp_dir().join(format!("formal-ai-ladder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create the throwaway workspace");
        let status = Command::new("bash")
            .args(["-c", "git archive HEAD | tar -x -C \"$1\"", "--"])
            .arg(&root)
            .status()
            .expect("copy the committed tree");
        assert!(status.success(), "git archive HEAD failed");
        std::fs::create_dir_all(root.join(".agent-ladder")).expect("create the evidence directory");
        root
    }
}

/// Reproduce the instruction `experiments/issue_1028_agent_cli_ladder/run.sh`
/// sends to one node, verbatim in shape.
fn ladder_node_prompt(task: &str, id: &str, depth: &str, criterion: &str) -> String {
    format!(
        "{task}\n\nThis is recursive binary-tree node {id} at depth {depth}. Solve only this \
         node's task in this fresh temporary repository. Its completion criterion is: \
         {criterion}. Leave observable evidence in .agent-ladder/node-{id}-proof.md. The first \
         line must be exactly node_path={id}. Use web research when it materially improves \
         factual accuracy. Do not claim success without evidence.\n"
    )
}

/// Execute one planned call the way the Agent CLI would, against `workspace`.
fn execute(tool: &str, arguments: &str, workspace: &Path) -> String {
    let value: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
    let string = |keys: &[&str]| -> Option<String> {
        keys.iter()
            .find_map(|key| value.get(*key).and_then(|entry| entry.as_str()))
            .map(str::to_owned)
    };
    let resolve = |path: &str| -> PathBuf {
        let candidate = Path::new(path);
        if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            workspace.join(candidate)
        }
    };
    match tool {
        "read" => {
            let Some(path) = string(&["filePath", "file_path", "path"]) else {
                return String::from("Error: no path argument");
            };
            std::fs::read_to_string(resolve(&path))
                .unwrap_or_else(|error| format!("Error: {path}: {error}"))
        }
        "write" => {
            let Some(path) = string(&["filePath", "file_path", "path"]) else {
                return String::from("Error: no path argument");
            };
            let content = string(&["content", "contents", "text"]).unwrap_or_default();
            let target = resolve(&path);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&target, content) {
                Ok(()) => format!("{{\"success\":true,\"path\":\"{path}\"}}"),
                Err(error) => format!("Error: {path}: {error}"),
            }
        }
        "edit" => {
            let Some(path) = string(&["filePath", "file_path", "path"]) else {
                return String::from("Error: no path argument");
            };
            let old = string(&["oldString", "old_string", "old"]).unwrap_or_default();
            let new = string(&["newString", "new_string", "new"]).unwrap_or_default();
            let target = resolve(&path);
            let Ok(body) = std::fs::read_to_string(&target) else {
                return format!("Error: File not found: {path}");
            };
            if !body.contains(&old) {
                return format!("Error: oldString not found in {path}");
            }
            match std::fs::write(&target, body.replacen(&old, &new, 1)) {
                Ok(()) => format!("{{\"success\":true,\"path\":\"{path}\"}}"),
                Err(error) => format!("Error: {path}: {error}"),
            }
        }
        "list" => shell(
            workspace,
            &format!(
                "ls -la {}",
                quote(&string(&["path", "directory"]).unwrap_or_else(|| String::from(".")))
            ),
        ),
        "glob" => shell(
            workspace,
            &format!(
                "find . -path {} -not -path './.git/*' | head -50",
                quote(&string(&["pattern", "glob", "query"]).unwrap_or_else(|| String::from("*")))
            ),
        ),
        "grep" | "codesearch" => {
            let Some(pattern) = string(&["pattern", "query", "regex"]) else {
                return String::from("Error: no pattern argument");
            };
            shell(
                workspace,
                &format!(
                    "grep -rn --binary-files=without-match --exclude-dir=.git -- {} . | head -50",
                    quote(&pattern)
                ),
            )
        }
        "bash" => {
            let Some(command) = string(&["command", "cmd", "script"]) else {
                return String::from("Error: no command argument");
            };
            shell(workspace, &command)
        }
        "websearch" | "webfetch" => {
            String::from("Error: this offline harness has no network access; no result was fetched")
        }
        "todoread" => String::from("[]"),
        "todowrite" | "task" | "batch" => String::from("{\"success\":true}"),
        other => format!("Error: unknown tool {other}"),
    }
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn shell(workspace: &Path, command: &str) -> String {
    let output = Command::new("bash")
        .args(["-c", command])
        .current_dir(workspace)
        .output();
    match output {
        Ok(output) => {
            let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
            let errors = String::from_utf8_lossy(&output.stderr);
            if !errors.trim().is_empty() {
                text.push_str(&errors);
            }
            if text.trim().is_empty() {
                String::from("(no output)")
            } else {
                text
            }
        }
        Err(error) => format!("Error: {error}"),
    }
}
