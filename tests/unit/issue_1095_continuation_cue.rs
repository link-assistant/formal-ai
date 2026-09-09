//! A turn that is only a continuation cue resumes the task (issue #1095).
//!
//! `@link-assistant/agent` sends "Continue if you have next steps" after a tool
//! result. Eight ladder leaves ended in web searches for the words "continue"
//! and "next step", because the only recovery path looked for a compaction
//! envelope that an ordinary tool loop never has, and because the phrase was
//! matched in English only. The cue is a seed role now, in four languages, and
//! a cue after a tool result resumes the last user turn that was not a cue.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

const TASK: &str = "In the file src/queue.rs, replace \"pending\" with \"waiting\". \
                    Change only that file and keep it valid Rust.";
const TOOLS: [&str; 5] = ["read", "grep", "write", "bash", "websearch"];

/// One tool loop: the task, the planner's first step, its result, then the cue.
fn session_with_cue(cue: &str) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::user(TASK)];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("the edit task must open with a tool step");
    };
    let first = &calls[0];
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "step-0",
        &first.tool,
        first.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "step-0",
        &first.tool,
        "src/queue.rs:12:    let state = \"pending\";\n",
    ));
    messages.push(ChatMessage::user(cue));
    messages
}

fn assert_resumes_the_task(cue: &str) {
    let messages = session_with_cue(cue);
    let plan = plan_chat_step(&messages, &TOOLS)
        .unwrap_or_else(|| panic!("the cue {cue:?} must continue the task, not leave the router"));
    match plan {
        AgenticPlan::ToolCalls(calls) => {
            for call in &calls {
                assert_ne!(
                    call.tool, "websearch",
                    "cue {cue:?} opened a web search: {calls:#?}"
                );
                let arguments = call.arguments.to_lowercase();
                assert!(
                    !arguments.contains("next steps") && !arguments.contains("продолж"),
                    "the cue itself became the request for {cue:?}: {calls:#?}"
                );
            }
            assert!(
                calls.iter().any(|call| call.arguments.contains("queue.rs")),
                "the established task was not resumed for {cue:?}: {calls:#?}"
            );
        }
        AgenticPlan::Final(answer) => {
            assert!(
                answer.contains("queue.rs"),
                "a final answer to the cue {cue:?} must be about the task: {answer}"
            );
        }
    }
}

#[test]
fn a_continuation_cue_after_a_tool_result_resumes_the_task_in_english() {
    assert_resumes_the_task("Continue if you have next steps");
    assert_resumes_the_task("continue");
}

#[test]
fn a_continuation_cue_after_a_tool_result_resumes_the_task_in_russian() {
    assert_resumes_the_task("Продолжай");
    assert_resumes_the_task("Продолжи, если есть следующие шаги.");
}

#[test]
fn a_continuation_cue_after_a_tool_result_resumes_the_task_in_hindi() {
    assert_resumes_the_task("जारी रखें");
}

#[test]
fn a_continuation_cue_after_a_tool_result_resumes_the_task_in_chinese() {
    assert_resumes_the_task("继续");
    assert_resumes_the_task("如果还有下一步，请继续。");
}

#[test]
fn a_request_that_merely_contains_the_word_keeps_its_own_meaning() {
    let messages = vec![ChatMessage::user(
        "Continue the migration: in the file src/queue.rs, replace \"pending\" with \
         \"waiting\". Change only that file and keep it valid Rust.",
    )];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("an ordinary request that contains the word must still be planned");
    };
    assert!(
        calls.iter().any(|call| call.arguments.contains("queue.rs")),
        "the request was not read as its own task: {calls:#?}"
    );
}

#[test]
fn a_cue_with_nothing_before_it_is_not_a_task() {
    // No established task: the cue cannot resume anything, and must not be
    // turned into one either. Whatever the router does, it does not search the
    // web for the words of the cue.
    let messages = vec![ChatMessage::user("Continue if you have next steps")];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) {
        assert!(
            calls.iter().all(|call| call.tool != "websearch"),
            "a bare cue opened a web search: {calls:#?}"
        );
    }
}
