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

/// A node prompt states three obligations, not one: change the tracked source,
/// create the structured effects record, and leave the proof note. Reaching the
/// first one is not finishing the node, so this bound covers all three.
const MAX_NODE_STEPS: usize = 32;

/// The files a node run leaves behind, in write order.
type Workspace = Vec<(String, String)>;

fn workspace_get<'a>(workspace: &'a Workspace, path: &str) -> Option<&'a str> {
    workspace
        .iter()
        .rev()
        .find(|(held, _)| held == path)
        .map(|(_, content)| content.as_str())
}

fn workspace_put(workspace: &mut Workspace, path: &str, content: &str) {
    if let Some(slot) = workspace
        .iter_mut()
        .find(|(held, _)| held == path)
        .map(|(_, held)| held)
    {
        *slot = content.to_owned();
        return;
    }
    workspace.push((path.to_owned(), content.to_owned()));
}

/// The two commands the planner's change routes emit, executed over the
/// simulated workspace. Anything else is reported rather than silently ignored,
/// so this harness can never pass a node by pretending to run a command it does
/// not understand.
fn run_command(workspace: &mut Workspace, command: &str) -> Result<String, String> {
    if let Some(path) = command.strip_prefix("cat ") {
        let path = path.trim();
        return Ok(workspace_get(workspace, path).unwrap_or_default().to_owned());
    }
    if let Some(rest) = command.strip_prefix("sed -i 's/") {
        let (script, target) = rest
            .split_once("' -- ")
            .ok_or_else(|| format!("unparsed sed command: {command}"))?;
        let (pattern, replacement) = script
            .trim_end_matches("/g")
            .split_once('/')
            .ok_or_else(|| format!("unparsed sed script: {command}"))?;
        let target = target.trim();
        let source = workspace_get(workspace, target)
            .ok_or_else(|| format!("sed on a file that is not there: {target}"))?
            .to_owned();
        // `\b` is a word boundary; the planner only ever asks for it around the
        // whole pattern, so honouring it means rewriting whole words only.
        let bare = pattern.trim_start_matches("\\b").trim_end_matches("\\b");
        let word_scoped = bare != pattern;
        let mut updated = String::with_capacity(source.len());
        let mut rest = source.as_str();
        while let Some(hit) = rest.find(bare) {
            let (before, after) = rest.split_at(hit);
            let following = &after[bare.len()..];
            let boundary = |text: &str, at_end: bool| {
                let character = if at_end {
                    text.chars().next()
                } else {
                    text.chars().next_back()
                };
                character.is_none_or(|character| {
                    !character.is_alphanumeric() && character != '_'
                })
            };
            updated.push_str(before);
            if !word_scoped || (boundary(before, false) && boundary(following, true)) {
                updated.push_str(replacement);
            } else {
                updated.push_str(bare);
            }
            rest = following;
        }
        updated.push_str(rest);
        workspace_put(workspace, target, &updated);
        return Ok(String::new());
    }
    Err(format!("unhandled command: {command}"))
}

/// Replay a whole node prompt the way the Agent CLI replays it, against a
/// workspace holding the leaf's real committed bytes, until the planner says the
/// node is finished.
///
/// Unlike [`effect_of`], this does not stop at the first effect. Stopping there
/// is exactly the blind spot that let node `1.1.2.2.1` report green here and
/// then fail a real run with `missing_proof`: the planner made the edit, called
/// the request served, and never wrote the two records the same prompt asked
/// for. A node is finished when every file it was told to produce is there.
fn run_node(leaf: &Leaf, prompt: &str, source: &str) -> Result<Workspace, String> {
    let mut workspace: Workspace = vec![(leaf.path.clone(), source.to_owned())];
    let mut messages = vec![ChatMessage::user(prompt)];
    for _ in 0..MAX_NODE_STEPS {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::Final(_)) => return Ok(workspace),
            Some(AgenticPlan::ToolCalls(calls)) if !calls.is_empty() => calls,
            other => return Err(format!("expected tool calls or a final answer, got {other:?}")),
        };
        for call in &calls {
            let result = match call.tool.as_str() {
                "read_file" => {
                    let path = first_argument(call, &["path", "filePath", "file_path"])
                        .ok_or_else(|| format!("read without a path: {}", call.arguments))?;
                    workspace_get(&workspace, &path)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("Error: File not found: {path}"))
                }
                "write_file" => {
                    let path = first_argument(call, &["path", "filePath", "file_path"])
                        .ok_or_else(|| format!("write without a path: {}", call.arguments))?;
                    let content = first_argument(call, &["content", "contents", "text"])
                        .ok_or_else(|| format!("write without content: {}", call.arguments))?;
                    workspace_put(&mut workspace, &path, &content);
                    "ok".to_owned()
                }
                "edit_file" => {
                    let path = first_argument(call, &["path", "filePath", "file_path"])
                        .ok_or_else(|| format!("edit without a path: {}", call.arguments))?;
                    let old = first_argument(call, &["old_string", "oldString", "old_str", "old"])
                        .ok_or_else(|| format!("edit without an old value: {}", call.arguments))?;
                    let new = first_argument(call, &["new_string", "newString", "new_str", "new"])
                        .ok_or_else(|| format!("edit without a new value: {}", call.arguments))?;
                    let held = workspace_get(&workspace, &path)
                        .ok_or_else(|| format!("edit on a file that is not there: {path}"))?;
                    if !held.contains(&old) {
                        return Err(format!("edit replaces {old:?}, which {path} does not hold"));
                    }
                    let updated = held.replace(&old, &new);
                    workspace_put(&mut workspace, &path, &updated);
                    "ok".to_owned()
                }
                "run_shell_command" => {
                    let command = argument(call, "command")
                        .ok_or_else(|| format!("command without a command: {}", call.arguments))?;
                    run_command(&mut workspace, &command)?
                }
                other => return Err(format!("unexpected tool {other}")),
            };
            record(&mut messages, call, &result);
        }
    }
    Err(format!("did not finish within {MAX_NODE_STEPS} steps"))
}

/// Everything a node prompt asks for has to be there when the node reports done.
///
/// The ladder's verifier demands the tracked change *and* the structured effect
/// *and* the proof note. `issue_1069_every_ladder_leaf_reaches_a_real_change`
/// only demands the first, which is why a green suite still produced
/// `1.1.2.2.1 FAIL missing_proof` on a real run.
#[test]
fn issue_1069_every_ladder_node_satisfies_every_obligation_it_was_given() {
    let mut failures = Vec::new();
    for (offset, leaf) in leaves().into_iter().enumerate() {
        let index = offset + 1;
        let id = node_id(index);
        let source = read(&leaf.path);
        let prompt = node_prompt(&leaf, index);
        let workspace = match run_node(&leaf, &prompt, &source) {
            Ok(workspace) => workspace,
            Err(reason) => {
                failures.push(format!("{} ({id}): {reason}", leaf.id));
                continue;
            }
        };
        let mut report = |reason: String| failures.push(format!("{} ({id}): {reason}", leaf.id));

        match workspace_get(&workspace, &leaf.path) {
            None => report(format!("lost {}", leaf.path)),
            Some(changed) if changed == source => {
                report(format!("left {} unchanged", leaf.path))
            }
            Some(changed) if !changed.contains(&leaf.marker) => {
                report(format!("{} lacks {:?}", leaf.path, leaf.marker))
            }
            Some(changed) if !changed.contains(&leaf.guard) => {
                report(format!("{} lost {:?}", leaf.path, leaf.guard))
            }
            Some(_) => {}
        }

        let effects_path = format!("agent-ladder-effects/node-{id}.lino");
        match workspace_get(&workspace, &effects_path) {
            None => report(format!("never wrote {effects_path}")),
            Some(effects) => {
                for field in [
                    format!("node_path={id}"),
                    "node_depth=5".to_owned(),
                    "node_kind=leaf".to_owned(),
                ] {
                    if !effects.lines().any(|line| line.trim() == field) {
                        report(format!("{effects_path} lacks the line {field:?}"));
                    }
                }
                match effects.lines().find_map(|line| line.strip_prefix("result=")) {
                    None => report(format!("{effects_path} states no result=")),
                    Some(result) if !result.contains(&leaf.marker) => report(format!(
                        "{effects_path} result= omits {:?}: {result:?}",
                        leaf.marker
                    )),
                    Some(result) if result.split_whitespace().count() < 4 => {
                        report(format!("{effects_path} result= is under four words: {result:?}"))
                    }
                    Some(_) => {}
                }
            }
        }

        let proof_path = format!(".agent-ladder/node-{id}-proof.md");
        match workspace_get(&workspace, &proof_path) {
            None => report(format!("never wrote {proof_path}")),
            Some(proof) if proof.lines().next() != Some(format!("node_path={id}").as_str()) => {
                report(format!(
                    "{proof_path} does not open with node_path={id}: {:?}",
                    proof.lines().next()
                ))
            }
            Some(_) => {}
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {LEAF_COUNT} nodes left an obligation unmet:\n{}",
        failures
            .iter()
            .filter_map(|failure| failure.split(' ').next())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        failures.join("\n")
    );
}
