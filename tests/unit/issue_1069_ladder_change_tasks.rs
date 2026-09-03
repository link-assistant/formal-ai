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
//! all". These tests drive the same 32 leaves through `plan_chat_step` against
//! the same real repository bytes, answering the read step exactly as a client's
//! read tool would. A leaf that the planner cannot turn into an effecting step
//! fails here, in a second, instead of failing halfway through a long run.
//!
//! What they drive is the *whole node prompt*, not the task sentence. An earlier
//! version of this file sent only the sentence, reported all 32 leaves green,
//! and the very first real run of node `1.1.2.2.1` then read the node id as if
//! it were a file: `Error: File not found: /tmp/tmp.QBPcFTv2tg/1.1.2.2.1`. The
//! sentence never contains a node id, so no sentence-shaped test could have
//! caught that. A fast test is only worth having when it is faithful, so the
//! prompt is assembled here exactly as `run.sh` assembles it, and
//! `issue_1069_the_node_prompt_still_reads_as_the_ladder_writes_it` fails if the
//! wording drifts apart.

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

/// The five fragments `run.sh` builds a depth-5 node prompt out of, in the order
/// it concatenates them. Held here as literals so the assembly below is
/// readable, and checked against the script itself by
/// `issue_1069_the_node_prompt_still_reads_as_the_ladder_writes_it`.
const LEAF_TASK_PREFIX: &str = "Atomic task L";
const LEAF_CRITERION: &str = "tracked_source_change";
const EFFECT_CONTRACT: &str = "Apply the change to the tracked file `";
const NODE_PREAMBLE: &str = "This is recursive binary-tree node ";
const NODE_EPILOGUE: &str = "The harness rejects proof without the separate Git effect.";

/// The node id `run.sh` gives leaf number `index` (1-based).
///
/// `run.sh` walks the tree from the root and maps a path back to a leaf with
/// `bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))`, so the
/// inverse is the 5-bit big-endian spelling of `index - 1` with `0` reading as
/// branch 1 and `1` as branch 2.
fn node_id(index: usize) -> String {
    (0..5)
        .map(|bit| {
            if (index - 1) >> (4 - bit) & 1 == 0 {
                "1"
            } else {
                "2"
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// The prompt the Agent CLI actually receives for a leaf, assembled the way
/// `run.sh` assembles it.
fn node_prompt(leaf: &Leaf, index: usize) -> String {
    let id = node_id(index);
    let effect_contract = format!(
        "{EFFECT_CONTRACT}{}` itself -- the file has to end up modified in the Git worktree, and \
         nothing else may change. Then create `agent-ladder-effects/node-{id}.lino` with these \
         exact field lines: `node_path={id}`, `node_depth=5`, `node_kind=leaf`, and `result=` \
         followed by at least four words that state the change you made and that contain the \
         exact text {}.",
        leaf.path, leaf.marker,
    );
    format!(
        "{LEAF_TASK_PREFIX}{index:02}: {}\n\n{NODE_PREAMBLE}{id} at depth 5. Solve only this \
         node's task in this fresh temporary repository. Its harness-evaluated completion \
         criterion is: {LEAF_CRITERION}. {effect_contract} Leave supporting evidence in \
         .agent-ladder/node-{id}-proof.md. The first line must be exactly node_path={id} and the \
         body must state the concrete result. {NODE_EPILOGUE} Use web research when it materially \
         improves factual accuracy. Do not claim success without evidence.\n",
        leaf.task,
    )
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

/// Drive the leaf's whole node prompt until the planner produces an effect,
/// answering every read with the repository's real bytes.
fn effect_of(leaf: &Leaf, prompt: &str, source: &str) -> Result<Effect, String> {
    let mut messages = vec![ChatMessage::user(prompt)];
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
    for (offset, leaf) in leaves().into_iter().enumerate() {
        let source = read(&leaf.path);
        let prompt = node_prompt(&leaf, offset + 1);
        match effect_of(&leaf, &prompt, &source) {
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

/// The prompt assembled above is only a faithful stand-in while it still reads
/// like the one `run.sh` sends. Every fragment it builds from is checked back
/// against the committed script, so a reworded prompt fails here rather than
/// silently turning these tests into a test of nothing.
#[test]
fn issue_1069_the_node_prompt_still_reads_as_the_ladder_writes_it() {
    let script = read(LADDER);
    for fragment in [
        LEAF_TASK_PREFIX,
        LEAF_CRITERION,
        EFFECT_CONTRACT,
        NODE_PREAMBLE,
        NODE_EPILOGUE,
    ] {
        assert!(
            script.contains(fragment),
            "the ladder no longer writes {fragment:?}; re-derive node_prompt from run.sh",
        );
    }
}

/// Issue #1069: a recursive node id is not a file.
///
/// The first real Agent CLI run of leaf L07 opened with
/// `read {"filePath": "/tmp/tmp.QBPcFTv2tg/1.1.2.2.1"}` and never recovered:
/// having read nothing, the planner then built an edit whose operands were
/// sentences lifted out of its own prompt, and finally wrote the resulting error
/// message into `src/engine_responses.rs`. Every one of those steps followed
/// from resolving the target path to the node id, so this pins the first one.
#[test]
fn issue_1069_a_node_id_is_not_the_file_to_read() {
    let leaves = leaves();
    let leaf = &leaves[6];
    assert_eq!(leaf.id, "L07", "L07 is the leaf the first real run failed on");
    let prompt = node_prompt(leaf, 7);
    assert!(
        prompt.contains("1.1.2.2.1"),
        "L07 is node 1.1.2.2.1; the prompt has to carry the id that caused the failure",
    );
    let call = match plan_chat_step(&[ChatMessage::user(&prompt)], &TOOLS) {
        Some(AgenticPlan::ToolCalls(mut calls)) if calls.len() == 1 => calls.remove(0),
        other => panic!("expected one tool call, got {other:?}"),
    };
    assert_eq!(
        argument(&call, "path").or_else(|| argument(&call, "filePath")),
        Some(leaf.path.clone()),
        "the planner must open the file the task names, not the node id",
    );
}
