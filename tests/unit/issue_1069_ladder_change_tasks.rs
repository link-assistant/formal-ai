//! Issue #1069: every ladder leaf must be a change Formal AI can actually plan.
//!
//! The issue #1028 ladder used to ask each of its 32 leaves to inspect a tracked
//! file and record what it saw, so every rung was satisfiable by writing a
//! self-describing side file and the ladder proved nothing about coding. The
//! leaves are now change-shaped -- a member insertion, a literal replacement or
//! an identifier rename in a real tracked source -- and `verify-node.sh` demands
//! the resulting diff.
//!
//! A full ladder run is over half an hour of real Agent CLI turns, which is far
//! too slow a feedback loop for the question "can the planner reach this edit at
//! all". These tests drive the same 32 task sentences through `plan_chat_step`
//! against the same real repository bytes, answering the read step exactly as a
//! client's read tool would. A leaf that the planner cannot turn into an
//! effecting step fails here, in a second, instead of failing halfway through a
//! long run.

use std::fs;
use std::path::PathBuf;

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const LADDER: &str = "experiments/issue_1028_agent_cli_ladder/run.sh";
const LEAF_COUNT: usize = 32;
const TOOLS: [&str; 4] = ["read_file", "write_file", "edit_file", "run_shell_command"];
/// A leaf is one edit; a planner that has not reached it within this many turns
/// is looping rather than working.
const MAX_STEPS: usize = 6;

struct Leaf {
    id: String,
    task: String,
    path: String,
    marker: String,
    guard: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// The leaves the workflow actually runs, lifted out of the committed script so
/// this test can never drift from the ladder.
fn leaves() -> Vec<Leaf> {
    let script = read(LADDER);
    let body = script
        .split_once("cat > \"$OUT/leaves.tsv\" <<'EOF'\n")
        .expect("the ladder writes its leaves")
        .1
        .split_once("\nEOF\n")
        .expect("the leaves heredoc is closed")
        .0;
    let leaves = body
        .lines()
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 5, "every leaf row has five fields: {line:?}");
            Leaf {
                id: fields[0].to_owned(),
                task: fields[1].to_owned(),
                path: fields[2].to_owned(),
                marker: fields[3].to_owned(),
                guard: fields[4].to_owned(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        leaves.len(),
        LEAF_COUNT,
        "the ladder has {LEAF_COUNT} leaves"
    );
    leaves
}

fn argument(call: &PlannedToolCall, key: &str) -> Option<String> {
    let arguments: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
    Some(arguments[key].as_str()?.to_owned())
}

/// The first of several spellings a client might use for the same argument.
/// An edit call carries every alias so that whichever key the client reads is
/// present, and the value behind all of them is the same.
fn first_argument(call: &PlannedToolCall, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| argument(call, key))
}

fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

/// What the planner did that would change the target's bytes: either the file
/// it wrote, or the shell command it asked the client to run. Both are real
/// effects; which one a change family uses is the planner's business, so the
/// leaf contract is stated over either.
enum Effect {
    Wrote(String),
    Commanded(String),
}

/// Drive the leaf's own task sentence until the planner produces an effect,
/// answering every read with the repository's real bytes.
fn effect_of(leaf: &Leaf, source: &str) -> Result<Effect, String> {
    let mut messages = vec![ChatMessage::user(&leaf.task)];
    for _ in 0..MAX_STEPS {
        let call = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(mut calls)) if calls.len() == 1 => calls.remove(0),
            other => return Err(format!("expected one tool call, got {other:?}")),
        };
        match call.tool.as_str() {
            "read_file" => {
                let path = argument(&call, "path").unwrap_or_default();
                if path != leaf.path {
                    return Err(format!("read {path:?} instead of {:?}", leaf.path));
                }
                record(&mut messages, &call, source);
            }
            "write_file" | "edit_file" => {
                // A write carries the whole file; an edit carries the operand
                // pair and leaves the substitution to the client. Applying it
                // here is exactly what the client's edit tool does, so both
                // shapes are compared against the same finished bytes.
                if let Some(content) = argument(&call, "content") {
                    return Ok(Effect::Wrote(content));
                }
                let old = first_argument(&call, &["old_string", "oldString", "old_str", "old"])
                    .ok_or_else(|| format!("edit without an old value: {}", call.arguments))?;
                let new = first_argument(&call, &["new_string", "newString", "new_str", "new"])
                    .ok_or_else(|| format!("edit without a new value: {}", call.arguments))?;
                if !source.contains(&old) {
                    return Err(format!(
                        "edit replaces {old:?}, which the file does not hold"
                    ));
                }
                return Ok(Effect::Wrote(source.replace(&old, &new)));
            }
            "run_shell_command" => {
                let command = argument(&call, "command")
                    .ok_or_else(|| format!("command without a command: {}", call.arguments))?;
                return Ok(Effect::Commanded(command));
            }
            other => return Err(format!("unexpected tool {other}")),
        }
    }
    Err(format!("no effect within {MAX_STEPS} steps"))
}

/// The ladder's whole point after issue #1069: every leaf reaches a real edit.
#[test]
fn issue_1069_every_ladder_leaf_reaches_a_real_change() {
    let mut failures = Vec::new();
    for leaf in leaves() {
        let source = read(&leaf.path);
        match effect_of(&leaf, &source) {
            Err(reason) => failures.push(format!("{} ({}): {reason}", leaf.id, leaf.path)),
            Ok(Effect::Wrote(content)) => {
                if !content.contains(&leaf.marker) {
                    failures.push(format!("{}: written bytes lack {:?}", leaf.id, leaf.marker));
                } else if !content.contains(&leaf.guard) {
                    failures.push(format!("{}: written bytes lost {:?}", leaf.id, leaf.guard));
                } else if content == source {
                    failures.push(format!("{}: wrote the file back unchanged", leaf.id));
                }
            }
            Ok(Effect::Commanded(command)) => {
                if !command.contains(&leaf.marker) {
                    failures.push(format!(
                        "{}: command lacks {:?}: {command}",
                        leaf.id, leaf.marker
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "leaves without a change:\n{}",
        failures.join("\n")
    );
}

/// The contract the verifier enforces has to be satisfiable from the committed
/// tree: the marker must be absent before the run and the anchor present, or
/// the leaf would pass or fail for reasons that have nothing to do with coding.
#[test]
fn issue_1069_every_leaf_contract_is_grounded_in_the_committed_tree() {
    for leaf in leaves() {
        let source = read(&leaf.path);
        assert!(
            !source.contains(&leaf.marker),
            "{}: {:?} already exists in {}",
            leaf.id,
            leaf.marker,
            leaf.path,
        );
        assert!(
            source.contains(&leaf.guard),
            "{}: anchor {:?} is missing from {}",
            leaf.id,
            leaf.guard,
            leaf.path,
        );
    }
}

/// Three change families, none of them a single repeated shape.
#[test]
fn issue_1069_the_ladder_covers_every_change_family() {
    let leaves = leaves();
    let insertions = leaves
        .iter()
        .filter(|leaf| leaf.task.contains(" list."))
        .count();
    let replacements = leaves
        .iter()
        .filter(|leaf| leaf.task.contains(", replace "))
        .count();
    let renames = leaves
        .iter()
        .filter(|leaf| leaf.task.contains(", rename the constant "))
        .count();
    assert_eq!(
        insertions + replacements + renames,
        LEAF_COUNT,
        "every leaf belongs to a named change family",
    );
    for (family, count) in [
        ("member insertion", insertions),
        ("literal replacement", replacements),
        ("identifier rename", renames),
    ] {
        assert!(count >= 8, "{family} is exercised by only {count} leaves");
    }
}
