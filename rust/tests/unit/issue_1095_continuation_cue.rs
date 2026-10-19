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

fn session_with_complete_source() -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::user(TASK)];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("the held-out edit must start with an observed read");
    };
    assert_eq!(calls.len(), 1);
    let first = &calls[0];
    assert_eq!(first.tool, "read");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "complete-source",
        &first.tool,
        first.arguments.clone(),
    )]));
    let mut result = ChatMessage::tool_result(
        "complete-source",
        &first.tool,
        "fn update() { let state = \"pending\"; }\n",
    );
    result.source_read = Some(serde_json::json!({
        "path": "src/queue.rs", "success": true, "complete": true, "format": "raw"
    }));
    messages.push(result);
    messages
}

fn planned_calls(messages: &[ChatMessage]) -> Vec<formal_ai::agentic_coding::PlannedToolCall> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(messages, &TOOLS) else {
        panic!("the source edit must retain a concrete tool plan");
    };
    calls
}

#[test]
fn repeated_exact_objective_preserves_the_bound_source_and_next_edit() {
    let messages = session_with_complete_source();
    let original = planned_calls(&messages);
    assert_eq!(original.len(), 1);
    assert_eq!(original[0].tool, "write");
    for suffix in [vec![TASK], vec![TASK, TASK], vec!["continue", TASK]] {
        let mut continued = messages.clone();
        continued.extend(suffix.into_iter().map(ChatMessage::user));
        let next = planned_calls(&continued);
        assert_eq!(next.len(), original.len());
        assert_eq!(next[0].tool, original[0].tool);
        assert_eq!(next[0].arguments, original[0].arguments);
    }
}

#[test]
fn changed_or_intervening_objectives_do_not_reuse_the_prior_source() {
    for suffix in [
        vec![TASK.to_owned() + " "],
        vec![TASK.replace("waiting", "queued")],
        vec!["An unrelated request.".to_owned(), TASK.to_owned()],
    ] {
        let mut messages = session_with_complete_source();
        messages.extend(suffix.into_iter().map(ChatMessage::user));
        assert_eq!(planned_calls(&messages)[0].tool, "read");
    }
}

#[test]
fn multipart_objectives_and_multipart_cues_always_need_a_fresh_read() {
    for text in [TASK, "continue"] {
        let mut messages = session_with_complete_source();
        messages.push(
            serde_json::from_value(serde_json::json!({
                "role": "user", "content": [{"type": "text", "text": text},
                    {"type": "image_url", "image_url": {"url": "fixture:held-out-image"}}]
            }))
            .expect("a valid protocol multipart request"),
        );
        assert_eq!(planned_calls(&messages)[0].tool, "read");
    }
}

#[test]
fn repeated_objective_does_not_authorize_an_unbound_read_receipt() {
    let mut messages = session_with_complete_source();
    messages[2].tool_call_id = Some("foreign-source".to_owned());
    messages.push(ChatMessage::user(TASK));
    assert_eq!(planned_calls(&messages)[0].tool, "read");
}
