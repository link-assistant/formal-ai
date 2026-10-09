//! A task naming two artifacts is not finished after the first (issue #1099).
//!
//! The reported session got one prompt naming two files, edited the first, and
//! answered `Final("Added \"Gemfile.lock\" to the list ... and observed the
//! result.")` five seconds in, with the second file never written. The whole
//! prompt had arrived -- the trace shows it -- so the second clause was read
//! and dropped, not truncated in transport.

use std::collections::HashMap;

use formal_ai::agentic_coding::task_obligations::obligations;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::obligation_ledger::ObligationExpectation;
use formal_ai::protocol::{ChatMessage, ToolCall};

#[path = "../issue_1066_ladder_capability/tool_workspace.rs"]
mod observed_tool_workspace;

const TOOLS: [&str; 4] = ["read", "grep", "write", "bash"];

/// Issue #1099's shape -- two artifacts in one prompt, introduced by
/// enumeration cues -- written in the phrasing the general composer reads.
///
/// The issue's literal prompt ("edit the tracked file X: add "Y" to the Z
/// list ... write a new file W whose first three lines are exactly ---")
/// composes to nothing today: neither clause matches the composer's
/// `create file PATH containing TEXT` shape, so the request reaches this
/// route with no artifact at all. That is a separate limit of the write-request
/// reader, tracked as its own defect; what #1099 reported and what this file
/// pins is the structural bug behind it -- that a request naming two artifacts
/// was planned, and answered, as if it named one.
const TWO_FILES: &str = "Two files. First, create file notes/attribution.md containing \
    Gemfile.lock. Second, create file changelog/fragment.md containing bump patch.";

/// Drive the loop, answering each planned call, and collect what was written.
fn written_paths(
    task: &str,
    turns: usize,
) -> (Vec<String>, HashMap<String, String>, Option<String>) {
    let mut messages = vec![ChatMessage::user(task)];
    let mut written = Vec::new();
    let mut observed = HashMap::new();
    let mut workspace = observed_tool_workspace::ToolWorkspace::new(task);
    for turn in 0..turns {
        match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                let call = &calls[0];
                let id = format!("step-{turn}");
                let result = workspace.execute(&id, call);
                if call.tool == "write" {
                    let arguments: serde_json::Value =
                        serde_json::from_str(&call.arguments).expect("planned Write arguments");
                    let path = ["path", "filePath", "file_path"]
                        .iter()
                        .find_map(|key| arguments.get(*key).and_then(serde_json::Value::as_str))
                        .expect("planned Write target");
                    let content = ["content", "contents", "text"]
                        .iter()
                        .find_map(|key| arguments.get(*key).and_then(serde_json::Value::as_str))
                        .expect("planned Write content");
                    let actual = workspace.read(path).expect("physically written target");
                    assert_eq!(
                        actual, content,
                        "the actual artifact must match all planned bytes"
                    );
                    written.push(path.to_owned());
                    observed.insert(path.to_owned(), actual);
                }
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    &id,
                    &call.tool,
                    call.arguments.clone(),
                )]));
                messages.push(result);
            }
            Some(AgenticPlan::Final(answer)) => return (written, observed, Some(answer)),
            None => return (written, observed, None),
        }
    }
    (written, observed, None)
}

#[test]
fn a_prompt_naming_two_files_yields_two_obligations() {
    let found = obligations(TWO_FILES).expect("two named artifacts are two obligations");
    let targets = file_targets(&found);
    assert_eq!(
        targets,
        vec!["notes/attribution.md", "changelog/fragment.md"],
        "both artifacts, in the order the request names them: {found:#?}"
    );
}

/// A cue inside a clause describes something; it does not open an obligation.
/// Issue #1099's own prompt says "whose first three lines are exactly ---" in
/// its second clause, so reading every "first" as an enumeration would invent
/// a third artifact out of a sentence about a file's contents.
#[test]
fn a_cue_inside_a_clause_does_not_open_another_obligation() {
    let task = "First, create file notes/a.txt containing alpha, whose first line is \
                exactly alpha. Second, create file notes/b.txt containing beta.";
    let found = obligations(task).expect("two obligations");
    let targets = file_targets(&found);
    assert_eq!(
        targets,
        vec!["notes/a.txt", "notes/b.txt"],
        "a described `first line` is not a third artifact: {found:#?}"
    );
}

#[test]
fn the_session_does_not_finish_before_the_second_file_is_written() {
    let (written, observed, answer) = written_paths(TWO_FILES, 8);
    assert_eq!(
        observed.get("notes/attribution.md").map(String::as_str),
        Some("Gemfile.lock.")
    );
    assert_eq!(
        observed.get("changelog/fragment.md").map(String::as_str),
        Some("bump patch.")
    );
    assert!(
        written.iter().any(|path| path.contains("fragment.md")),
        "the second named artifact was never written; wrote {written:?}, answered {answer:?}"
    );
    if let Some(answer) = answer {
        assert!(
            written.len() >= 2,
            "a final answer arrived with only {} artifact(s) written: {answer}",
            written.len()
        );
    }
}

/// A request naming one artifact keeps the single-target path exactly as it
/// was: reading one obligation as two would invent work nobody asked for.
#[test]
fn a_single_artifact_request_is_unchanged() {
    for task in [
        "Create file notes/general-demo.txt containing planner fallback works",
        "Write file artifacts/unseen-case.md with text capability composed plan",
    ] {
        assert!(
            obligations(task).is_none(),
            "one named artifact must not split into several: {task}"
        );
    }
}

#[test]
fn two_obligations_are_recognised_in_russian() {
    let task = "Сначала создай файл notes/a.txt с текстом альфа. \
                Затем создай файл notes/b.txt с текстом бета.";
    let found = obligations(task).expect("two obligations in Russian");
    let targets = file_targets(&found);
    assert_eq!(targets, vec!["notes/a.txt", "notes/b.txt"], "{found:#?}");
}

fn file_targets(
    obligations: &[formal_ai::agentic_coding::task_obligations::Obligation],
) -> Vec<&str> {
    obligations
        .iter()
        .filter_map(|obligation| match &obligation.expectation {
            ObligationExpectation::FileBytes { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn explicit_file_receipts_advance_the_declared_leaf_without_hashing_shell_metadata() {
    use formal_ai::agentic_coding::task_obligations::next_step;
    use formal_ai::obligation_ledger::ObligationStep;
    let task =
        "First, create file a.txt containing alpha. Second, create file b.txt containing beta.";
    let receipt = |messages: &mut Vec<ChatMessage>, command: &str, raw: &str| {
        let id = format!("receipt-{}", messages.len());
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            "bash",
            serde_json::json!({"command":command}).to_string(),
        )]));
        messages.push(ChatMessage::tool_result(id, "bash", raw));
    };
    for raw in ["Output: alpha.\nExit Code: 0", "alpha."] {
        let mut messages = vec![ChatMessage::user(task)];
        receipt(&mut messages, "cat a.txt", raw);
        let Some(ObligationStep::Observe(node)) = next_step(task, &messages) else {
            panic!("second file remains open");
        };
        assert!(
            matches!(node.expectation, ObligationExpectation::FileBytes { ref path, .. } if path == "b.txt")
        );
    }
    for (command, raw) in [
        ("cat a.txt", "Output: wrong\nExit Code: 0"),
        ("cat a.txt", "Output: alpha.\nExit Code: 1"),
        ("cat other-a.txt", "Output: alpha.\nExit Code: 0"),
        ("cat other-a.txt", "alpha."),
        ("printf a.txt", "alpha."),
    ] {
        let mut messages = vec![ChatMessage::user(task)];
        receipt(&mut messages, command, raw);
        let Some(ObligationStep::Observe(node)) = next_step(task, &messages) else {
            panic!("first file remains open");
        };
        assert!(
            matches!(node.expectation, ObligationExpectation::FileBytes { ref path, .. } if path == "a.txt")
        );
    }
    let mut messages = vec![ChatMessage::user(task)];
    receipt(&mut messages, "cat a.txt", "Output: alpha.\nExit Code: 0");
    receipt(&mut messages, "cat a.txt", "Output: wrong\nExit Code: 1");
    let Some(ObligationStep::Observe(node)) = next_step(task, &messages) else {
        panic!("failed latest receipt must remain open");
    };
    assert!(
        matches!(node.expectation, ObligationExpectation::FileBytes { ref path, .. } if path == "a.txt")
    );
}

#[path = "../../fixtures/literal-punctuation-contract.rs"]
mod literal_punctuation_contract;

#[path = "../../fixtures/literal-obligation-transaction.rs"]
mod literal_obligation_transaction;

#[path = "../../fixtures/literal-action-ownership.rs"]
mod literal_action_ownership;
