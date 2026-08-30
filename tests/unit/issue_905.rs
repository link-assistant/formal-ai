//! Regression coverage for issue #905: tool failures are observations, not
//! completed steps, and verification evidence must match the requested bytes.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{AnthropicMessagesRequest, ChatMessage, ResponsesRequest, ToolCall};
use std::fs;

const PROMPT: &str = "Create a file hello.txt containing exactly: Hello World";
const TOOLS: [&str; 3] = ["read_file", "write_file", "run_command"];

fn next_call(messages: &[ChatMessage]) -> PlannedToolCall {
    match plan_chat_step(messages, &TOOLS) {
        Some(AgenticPlan::ToolCalls(mut calls)) if calls.len() == 1 => calls.remove(0),
        other => panic!("expected one tool call, got {other:?}"),
    }
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

fn advance_to_verification() -> (Vec<ChatMessage>, PlannedToolCall) {
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let plan_write = next_call(&messages);
    assert_eq!(plan_write.tool, "write_file");
    record(&mut messages, &plan_write, r#"{"success":true}"#);

    let target_write = next_call(&messages);
    assert_eq!(target_write.tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&target_write.arguments).expect("write arguments");
    assert_eq!(arguments["path"], "hello.txt");
    assert_eq!(arguments["content"], "Hello World");
    record(&mut messages, &target_write, r#"{"success":true}"#);

    let verification = next_call(&messages);
    assert_eq!(verification.tool, "run_command");
    (messages, verification)
}

#[test]
fn exact_modifier_is_not_written_as_part_of_the_payload() {
    let messages = vec![ChatMessage::user(PROMPT)];
    let call = next_call(&messages);
    let arguments: serde_json::Value = serde_json::from_str(&call.arguments).expect("arguments");
    let plan = arguments["content"].as_str().expect("plan content");
    assert!(plan.contains("expected_evidence \"Hello World\""), "{plan}");
    assert!(
        !plan.contains("expected_evidence \"exactly: Hello World\""),
        "{plan}"
    );
}

#[test]
fn exact_content_parsing_generalizes_across_payloads_and_punctuation() {
    for (index, prompt) in [
        "Create a file a.txt containing exactly: alpha",
        "Create file b.txt containing exactly beta",
        "Please create the file c.txt containing exactly: two words",
        "Create a file d.txt containing exactly, punctuation stays!",
        "Create a file e.txt containing exactly `code bytes`",
        "Create a file f.txt containing exactly: 12345",
        "Create a file g.txt containing exactly: Mixed CASE",
        "Create a file h.txt containing exactly: a:b:c",
        "Create a file i.txt containing exactly: braces {stay}",
        "Create a file j.txt containing exactly: symbols #!?",
        "Create a file k.txt containing exactly: tabs are words",
        "Create a file l.txt containing exactly: slash/value",
        "Create a file m.txt containing exactly: dot.value",
        "Create a file n.txt containing exactly: under_score",
        "Create a file o.txt containing exactly: dash-value",
        "Create a file p.txt containing exactly: quoted value",
        "Create a file q.txt containing exactly: unicode café",
        "Create a file r.txt containing exactly: JSON-ish [1,2]",
        "Create a file s.txt containing exactly: equals=a",
        "Create a file t.txt containing exactly: final payload",
    ]
    .iter()
    .enumerate()
    {
        let call = next_call(&[ChatMessage::user(*prompt)]);
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("arguments");
        let plan = arguments["content"].as_str().expect("plan content");
        assert!(
            !plan.contains("expected_evidence: exactly"),
            "case {index}: {plan}"
        );
    }
}

#[test]
fn failed_write_is_followed_by_read_then_one_write_retry() {
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let plan_write = next_call(&messages);
    record(
        &mut messages,
        &plan_write,
        r#"{"is_error":true,"content":"File must be read before writing. Use read_file first."}"#,
    );

    let read = next_call(&messages);
    assert_eq!(read.tool, "read_file");
    record(
        &mut messages,
        &read,
        r#"{"is_error":true,"content":"File hello.txt not found"}"#,
    );

    let retry = next_call(&messages);
    assert_eq!(retry.tool, "write_file");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&retry.arguments).unwrap()["path"],
        ".formal-ai/general-change-plan.lino"
    );
}

#[test]
fn a_second_failed_write_for_the_same_path_stops_retrying() {
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let first = next_call(&messages);
    record(
        &mut messages,
        &first,
        r#"{"is_error":true,"content":"Read the file before writing"}"#,
    );
    let read = next_call(&messages);
    record(
        &mut messages,
        &read,
        r#"{"is_error":true,"content":"File not found"}"#,
    );
    let retry = next_call(&messages);
    record(
        &mut messages,
        &retry,
        r#"{"is_error":true,"content":"Write transport remains unavailable"}"#,
    );

    // Retrying stops, but the plan still asks the workspace: the check the
    // plan itself named runs once, and its status is what gets reported.
    let verification = next_call(&messages);
    assert_eq!(verification.tool, "run_command");
    record(
        &mut messages,
        &verification,
        "Command: cat hello.txt\nOutput: (empty)\nError: cat: hello.txt: No such file or directory\nExit Code: 1\n",
    );

    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("the bounded retry must stop after the second failure");
    };
    assert!(
        !answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(answer.contains('1'), "{answer}");
    assert!(answer.contains("hello.txt"), "{answer}");
}

/// Issue #916 rung R916-01: the write transport fault of issue #905 leaves the
/// plan without a recovery path, and the report must carry what the workspace
/// answered — the check the plan named, and the status it exited with — rather
/// than the transport message alone.
#[test]
fn an_unrecoverable_write_still_asks_the_workspace_before_reporting() {
    let tools = ["write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let mut guard = 0;
    let answer = loop {
        guard += 1;
        assert!(guard < 8, "the plan must terminate");
        match plan_chat_step(&messages, &tools) {
            Some(AgenticPlan::ToolCalls(mut calls)) => {
                let call = calls.remove(0);
                let result = if call.tool == "write_file" {
                    "Error: write_stdin failed: Unknown process id 0".to_owned()
                } else {
                    "Command: cat hello.txt\nOutput: (empty)\nError: cat: hello.txt: No such file \
                     or directory\nExit Code: 1\n"
                        .to_owned()
                };
                record(&mut messages, &call, &result);
            }
            Some(AgenticPlan::Final(answer)) => break answer,
            other => panic!("unexpected plan {other:?}"),
        }
    };
    assert!(!answer.contains("verified it"), "{answer}");
    assert!(
        !answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(answer.contains('1'), "{answer}");
}

#[test]
fn failed_auxiliary_plan_write_without_read_still_attempts_the_target() {
    let tools = ["write_file"];
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let Some(AgenticPlan::ToolCalls(mut calls)) = plan_chat_step(&messages, &tools) else {
        panic!("expected the auxiliary plan write");
    };
    let plan_write = calls.remove(0);
    record(
        &mut messages,
        &plan_write,
        "<tool_use_error>Error writing file</tool_use_error>",
    );

    let Some(AgenticPlan::ToolCalls(mut calls)) = plan_chat_step(&messages, &tools) else {
        panic!("the unavailable auxiliary plan must not swallow the user's write");
    };
    let target_write = calls.remove(0);
    let arguments: serde_json::Value =
        serde_json::from_str(&target_write.arguments).expect("target write arguments");
    assert_eq!(arguments["path"], "hello.txt");
    assert_eq!(arguments["content"], "Hello World");
}

#[test]
fn explicit_error_verification_cannot_produce_a_success_claim() {
    let (mut messages, verification) = advance_to_verification();
    record(
        &mut messages,
        &verification,
        r#"{"is_error":true,"content":"cat: hello.txt: No such file or directory\nExit Code: 1"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected an honest terminal report");
    };
    assert!(
        !answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(!answer.contains("verified it"), "{answer}");
    assert!(answer.contains("No such file or directory"), "{answer}");
}

#[test]
fn nonzero_exit_verification_cannot_produce_a_success_claim() {
    let (mut messages, verification) = advance_to_verification();
    record(
        &mut messages,
        &verification,
        r#"{"exit_code":1,"status":"failed","output":"cat: hello.txt: No such file"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected an honest terminal report");
    };
    assert!(
        !answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(answer.contains("No such file"), "{answer}");
}

#[test]
fn successful_command_with_wrong_evidence_is_not_verified() {
    let (mut messages, verification) = advance_to_verification();
    record(
        &mut messages,
        &verification,
        r#"{"exit_code":0,"status":"completed","output":"Goodbye World"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected a mismatch report");
    };
    assert!(
        !answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(answer.contains("Goodbye World"), "{answer}");
}

#[test]
fn matching_observed_evidence_allows_completion() {
    let (mut messages, verification) = advance_to_verification();
    record(
        &mut messages,
        &verification,
        r#"{"exit_code":0,"status":"completed","output":"Hello World\n"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected completion");
    };
    assert!(
        answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(answer.contains("verified it"), "{answer}");
}

#[test]
fn requested_failure_vocabulary_is_valid_successful_evidence() {
    let prompt = "Create a file status.txt containing exactly: failed";
    let mut messages = vec![ChatMessage::user(prompt)];
    let plan_write = next_call(&messages);
    record(&mut messages, &plan_write, r#"{"success":true}"#);
    let target_write = next_call(&messages);
    record(&mut messages, &target_write, r#"{"success":true}"#);
    let verification = next_call(&messages);
    record(
        &mut messages,
        &verification,
        r#"{"exit_code":0,"status":"completed","output":"failed\n"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected completion");
    };
    assert!(
        answer.contains("Completed the general change request"),
        "{answer}"
    );
}

#[test]
fn tool_error_metadata_survives_openai_compatible_deserialization() {
    for key in ["is_error", "isError"] {
        let value = serde_json::json!({
            "role": "tool",
            "content": "provider-owned failure detail",
            key: true,
        });
        let message: ChatMessage = serde_json::from_value(value).expect("tool result message");
        assert!(
            message.is_error,
            "{key} must remain available to the planner"
        );
    }
}

#[test]
fn anthropic_and_responses_adapters_preserve_error_metadata() {
    let anthropic: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "messages": [
            {"role":"assistant", "content":[{"type":"tool_use", "id":"a1", "name":"write_file", "input":{}}]},
            {"role":"user", "content":[{"type":"tool_result", "tool_use_id":"a1", "content":"denied", "is_error":true}]}
        ]
    }))
    .expect("Anthropic request");
    let anthropic_messages = anthropic.to_chat_completion_request().messages;
    assert!(anthropic_messages.last().expect("tool result").is_error);

    let responses: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "input": [
            {"type":"function_call", "call_id":"r1", "name":"write_file", "arguments":"{}"},
            {"type":"function_call_output", "call_id":"r1", "output":"denied", "status":"failed"}
        ]
    }))
    .expect("Responses request");
    let responses_messages = responses.to_chat_completion_request().messages;
    assert!(responses_messages.last().expect("tool result").is_error);
}

#[test]
fn verification_failure_responses_cover_every_supported_language() {
    // Coverage matrix: language: "en", language: "ru", language: "hi",
    // language: "zh", language: "es".
    for language in ["en", "ru", "hi", "zh", "es"] {
        for intent in [
            "general_plan_verification_mismatch",
            "general_plan_unverified",
            // The completion claim is seeded too: the sentence this issue
            // quotes as the false report must not be English typed into Rust.
            "general_plan_completed",
        ] {
            let response = formal_ai::seed::response_for(intent, language)
                .unwrap_or_else(|| panic!("missing {intent} response for language {language}"));
            assert!(
                response.contains("{target}"),
                "{intent}/{language}: {response}"
            );
            assert!(
                !response.trim().is_empty(),
                "{intent}/{language} must not be empty"
            );
        }
    }
}

#[test]
fn issue_905_case_study_and_self_authorship_are_preserved() {
    let root = env!("CARGO_MANIFEST_DIR");
    let case = format!("{root}/docs/case-studies/issue-905");
    let read = |path: &str| fs::read_to_string(format!("{case}/{path}")).expect(path);

    assert!(read("README.md").contains("ses_034e9dafeffe7nxeTkFhmHLmZN"));
    assert!(read("requirements.md").contains("1/5 (20%)"));
    assert!(read("raw-data/direct-codex3.log").contains("Unknown process id 0"));
    assert!(read("raw-data/direct-qwen.log").contains("is_error"));
    assert!(read("test-evidence/regression-red.log").contains("6 failed"));
    assert!(
        read("self-hosting-authorship/agent-cli.log").contains("ses_034e9dafeffe7nxeTkFhmHLmZN")
    );
    assert!(
        read("self-hosting-fixture-refresh/agent-cli.log")
            .contains("ses_faf5f322effeicCLioH0QBuTdQ")
    );

    let authored = fs::read(format!(
        "{case}/self-hosting-authorship/tool-result-evidence-invariant.lino"
    ))
    .expect("Agent CLI authored invariant");
    let canonical = fs::read(format!(
        "{root}/data/meta/tool-result-evidence-invariant.lino"
    ))
    .expect("canonical invariant");
    assert_eq!(authored, canonical);

    let refreshed = fs::read(format!(
        "{case}/self-hosting-fixture-refresh/self-healing-case.lino"
    ))
    .expect("Agent CLI refreshed self-healing fixture");
    let canonical = fs::read(format!("{root}/data/meta/self-healing-case.lino"))
        .expect("canonical self-healing fixture");
    assert_eq!(refreshed, canonical);
}
