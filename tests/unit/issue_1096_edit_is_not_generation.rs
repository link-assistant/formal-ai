//! A replacement in an existing file is an edit, not a generation (issue #1096).
//!
//! `plan_generated_source_step` claimed `In the file src/x.rs, replace "A" with
//! "B". Change only that file and keep it valid Rust.` because "keep it valid
//! Rust" reads as a program request. That path synthesises its own artifact and
//! verifies the agent's write against it -- so the agent's correct edit of the
//! real file failed with "the observed bytes differ". Seven ladder leaves failed
//! that way. The generation step now declines anything the edit reader
//! (`compose_edit_request`) recognises, and the edit route claims it.

use formal_ai::agentic_coding::general_planner::compose_edit_request;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ChatMessage;

const TOOLS: [&str; 4] = ["read", "grep", "write", "bash"];
const TARGET: &str = "src/web_search_fusion_core.rs";

fn first_step(task: &str) -> Option<(String, String)> {
    match plan_chat_step(&[ChatMessage::user(task)], &TOOLS)? {
        AgenticPlan::ToolCalls(calls) => calls
            .into_iter()
            .next()
            .map(|call| (call.tool, call.arguments)),
        AgenticPlan::Final(_) => None,
    }
}

/// The generated-source step writes a synthesised body as its very first step.
/// An edit observes the target first. Whatever else the router does with a
/// phrasing, it must never open with a synthesised write of the named file.
fn assert_not_generated(task: &str) {
    if let Some((tool, arguments)) = first_step(task) {
        assert!(
            !(tool == "write" && arguments.contains(TARGET)),
            "the first step wrote a generated file over the one to be edited: {task}\n{tool} {arguments}"
        );
    }
}

/// English and Russian phrasings are read as edits and open by observing the
/// target, which is the ladder's leaf shape.
fn assert_edit(task: &str) {
    assert!(
        compose_edit_request(task).is_some(),
        "the edit reader must recognise the leaf phrasing: {task}"
    );
    let (tool, arguments) = first_step(task).expect("an edit opens with a tool step");
    assert_ne!(
        tool, "write",
        "an edit observes the file before writing: {task}"
    );
    assert!(
        arguments.contains(TARGET),
        "the edit targets the named file: {task}\n{arguments}"
    );
    assert_not_generated(task);
}

#[test]
fn a_replacement_in_a_tracked_file_is_planned_as_an_edit_in_english() {
    assert_edit(
        "In the file src/web_search_fusion_core.rs, replace \"fusion\" with \"merge\". \
         Change only that file and keep it valid Rust.",
    );
}

#[test]
fn a_replacement_in_a_tracked_file_is_planned_as_an_edit_in_russian() {
    assert_edit(
        "В файле src/web_search_fusion_core.rs замени \"fusion\" на \"merge\". \
         Измени только этот файл и оставь его валидным Rust.",
    );
}

/// Hindi puts the verb last (`... से बदलें`) and Chinese is not whitespace
/// tokenised, so `compose_edit_request` does not yet read these as edits --
/// a reader limit, tracked separately. What #1096 guarantees for them already
/// is the half that failed the ladder: they are not claimed as generation.
#[test]
fn a_replacement_in_a_tracked_file_is_not_generated_in_hindi() {
    assert_not_generated(
        "फ़ाइल src/web_search_fusion_core.rs में \"fusion\" को \"merge\" से बदलें। \
         केवल वही फ़ाइल बदलें और उसे वैध Rust रखें।",
    );
}

#[test]
fn a_replacement_in_a_tracked_file_is_not_generated_in_chinese() {
    assert_not_generated(
        "在文件 src/web_search_fusion_core.rs 中，将 \"fusion\" 替换为 \"merge\"。\
         只修改该文件，并保持其为有效的 Rust。",
    );
}

/// The boundary the decline relies on: a new-file program request is not an
/// edit request, so the generation route still owns it.
#[test]
fn a_new_file_program_request_is_not_read_as_an_edit() {
    for task in [
        "Write a new file src/generated_answer.rs with a pub function answer that returns 42. \
         Keep it valid Rust.",
        "Create src/generated_answer.rs containing a function answer that returns 42.",
    ] {
        assert!(
            compose_edit_request(task).is_none(),
            "a new-file request must not be read as an edit: {task}"
        );
    }
}
