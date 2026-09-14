//! A task naming two artifacts is not finished after the first (issue #1099).
//!
//! The reported session got one prompt naming two files, edited the first, and
//! answered `Final("Added \"Gemfile.lock\" to the list ... and observed the
//! result.")` five seconds in, with the second file never written. The whole
//! prompt had arrived -- the trace shows it -- so the second clause was read
//! and dropped, not truncated in transport.

use formal_ai::agentic_coding::task_obligations::obligations;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

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
fn written_paths(task: &str, turns: usize) -> (Vec<String>, Option<String>) {
    let mut messages = vec![ChatMessage::user(task)];
    let mut written = Vec::new();
    for turn in 0..turns {
        match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                let call = &calls[0];
                if call.tool == "write"
                    && let Ok(arguments) =
                        serde_json::from_str::<serde_json::Value>(&call.arguments)
                {
                    for key in ["path", "filePath", "file_path"] {
                        if let Some(path) = arguments.get(key).and_then(|v| v.as_str()) {
                            written.push(path.to_owned());
                            break;
                        }
                    }
                }
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    format!("step-{turn}"),
                    &call.tool,
                    call.arguments.clone(),
                )]));
                messages.push(ChatMessage::tool_result(
                    format!("step-{turn}"),
                    &call.tool,
                    "ok",
                ));
            }
            Some(AgenticPlan::Final(answer)) => return (written, Some(answer)),
            None => return (written, None),
        }
    }
    (written, None)
}

#[test]
fn a_prompt_naming_two_files_yields_two_obligations() {
    let found = obligations(TWO_FILES).expect("two named artifacts are two obligations");
    let targets: Vec<&str> = found.iter().map(|o| o.target.as_str()).collect();
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
    let targets: Vec<&str> = found.iter().map(|o| o.target.as_str()).collect();
    assert_eq!(
        targets,
        vec!["notes/a.txt", "notes/b.txt"],
        "a described `first line` is not a third artifact: {found:#?}"
    );
}

#[test]
fn the_session_does_not_finish_before_the_second_file_is_written() {
    let (written, answer) = written_paths(TWO_FILES, 8);
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
    let targets: Vec<&str> = found.iter().map(|o| o.target.as_str()).collect();
    assert_eq!(targets, vec!["notes/a.txt", "notes/b.txt"], "{found:#?}");
}
