//! Regression coverage for issue #905: tool failures are observations, not
//! completed steps, and verification evidence must match the requested bytes.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{AnthropicMessagesRequest, ChatMessage, ResponsesRequest, ToolCall};
use std::fs;
use std::path::{Path, PathBuf};

const PROMPT: &str = "Create a file hello.txt containing exactly: Hello World";
const TOOLS: [&str; 3] = ["read_file", "write_file", "run_command"];

use crate::observed_plan_event;

fn before_target(prompt: &str) -> (Vec<ChatMessage>, PlannedToolCall) {
    observed_plan_event::before_target(prompt, &TOOLS)
}

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

fn advance_with_tools(tools: &[&str]) -> (Vec<ChatMessage>, PlannedToolCall) {
    let (mut messages, target_write) = observed_plan_event::before_target(PROMPT, tools);
    assert_eq!(target_write.tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&target_write.arguments).expect("write arguments");
    assert_eq!(arguments["path"], "hello.txt");
    assert_eq!(arguments["content"], "Hello World");
    record(&mut messages, &target_write, r#"{"success":true}"#);

    let verification = observed_plan_event::next_call(&messages, tools);
    (messages, verification)
}
fn advance_to_verification() -> (Vec<ChatMessage>, PlannedToolCall) {
    let result = advance_with_tools(&TOOLS);
    assert_eq!(result.1.tool, "read_file");
    result
}

#[test]
fn exact_modifier_is_not_written_as_part_of_the_payload() {
    let plan = formal_ai::agentic_coding::general_planner::compose_general_change_plan(PROMPT)
        .expect("literal plan")
        .links_notation();
    assert!(plan.contains("expected_evidence \"Hello World\""), "{plan}");
    assert!(
        !plan.contains("expected_evidence \"exactly: Hello World\""),
        "{plan}"
    );
    let (_, call) = before_target(PROMPT);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&call.arguments).unwrap()["content"],
        "Hello World"
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
        let (_, call) = before_target(prompt);
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("arguments");
        let target_bytes = arguments["content"].as_str().expect("actual target bytes");
        let plan = formal_ai::agentic_coding::general_planner::compose_general_change_plan(prompt)
            .expect("literal plan");
        assert_eq!(target_bytes, plan.content, "case {index} exact target");
        let plan = plan.links_notation();
        assert!(
            !plan.contains("expected_evidence: exactly"),
            "case {index}: {plan}"
        );
    }
}

// Exclusive Create failures are terminal observations, not overwrite-retry authority.
// Separate Set-contents coverage in observed-auxiliary-contracts.test.mjs retains
// the one read-before-write retry and its second-failure bound.
#[test]
fn failed_exclusive_creation_does_not_retry_or_read_an_absent_target() {
    let (mut messages, call) = before_target(PROMPT);
    record(
        &mut messages,
        &call,
        r#"{"is_error":true,"content":"File must be read before writing. Use read_file first."}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("failed exclusive creation must terminate without overwrite recovery");
    };
    assert!(
        answer.contains("File must be read before writing"),
        "{answer}"
    );
    assert!(!answer.contains("Completed"), "{answer}");
}

#[test]
fn repeated_planning_after_failed_creation_never_retries_the_write() {
    let (mut messages, call) = before_target(PROMPT);
    record(
        &mut messages,
        &call,
        r#"{"is_error":true,"content":"Write transport remains unavailable"}"#,
    );
    for _ in 0..2 {
        let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
            panic!("failed exclusive creation must stay terminal");
        };
        assert!(
            answer.contains("Write transport remains unavailable"),
            "{answer}"
        );
        assert!(!answer.contains("Completed"), "{answer}");
    }
}

#[test]
fn an_unrecoverable_write_reports_the_transport_without_inventing_a_check() {
    let tools = ["write_file", "run_command"];
    let (mut messages, call) = observed_plan_event::before_target(PROMPT, &tools);
    assert_eq!(call.tool, "write_file");
    record(
        &mut messages,
        &call,
        "Error: write_stdin failed: Unknown process id 0",
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("the actual failed write must terminate");
    };
    assert!(answer.contains("Unknown process id 0"), "{answer}");
    assert!(!answer.contains("verified it"), "{answer}");
    assert!(!answer.contains("Completed"), "{answer}");
    assert!(!answer.contains("Exit Code:"), "{answer}");
}

#[test]
fn write_only_target_failure_is_terminal_and_reports_the_actual_error() {
    let tools = ["write_file"];
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let call = observed_plan_event::next_call(&messages, &tools);
    let arguments: serde_json::Value = serde_json::from_str(&call.arguments).expect("target write");
    assert_eq!(arguments["path"], "hello.txt");
    assert_eq!(arguments["content"], "Hello World");
    record(
        &mut messages,
        &call,
        "<tool_use_error>Error writing file</tool_use_error>",
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("the real target transport failure must terminate honestly");
    };
    assert!(answer.contains("Error writing file"), "{answer}");
    assert!(!answer.contains("Completed"), "{answer}");
}

#[test]
fn write_only_success_is_explicitly_unverified_without_readback() {
    let tools = ["write_file"];
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let call = observed_plan_event::next_call(&messages, &tools);
    let arguments: serde_json::Value = serde_json::from_str(&call.arguments).expect("target write");
    assert_eq!(arguments["path"], "hello.txt");
    assert_eq!(arguments["content"], "Hello World");
    record(&mut messages, &call, r#"{"success":true}"#);
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("write-only clients cannot fabricate missing verification");
    };
    assert_eq!(
        answer,
        formal_ai::seed::localized_response("coding_workspace_written_unverified", "en")
            .expect("maintained unverified response")
            .replace("{path}", "hello.txt")
    );
    assert!(!answer.contains(".formal-ai/general-change-plan.lino"));
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
    assert!(!answer.contains("Completed"), "{answer}");
    assert!(!answer.contains("verified it"), "{answer}");
    assert!(answer.contains("No such file or directory"), "{answer}");
}

#[test]
fn nonzero_exit_verification_cannot_produce_a_success_claim() {
    let tools = ["write_file", "run_command"];
    let (mut messages, verification) = advance_with_tools(&tools);
    assert_eq!(verification.tool, "run_command");
    record(
        &mut messages,
        &verification,
        r#"{"exit_code":1,"status":"failed","output":"cat: hello.txt: No such file"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("expected an honest terminal report");
    };
    assert!(!answer.contains("Completed"), "{answer}");
    assert!(answer.contains("No such file"), "{answer}");
}

#[test]
fn successful_command_with_wrong_evidence_is_not_verified() {
    let tools = ["write_file", "run_command"];
    let (mut messages, verification) = advance_with_tools(&tools);
    assert_eq!(verification.tool, "run_command");
    record(
        &mut messages,
        &verification,
        r#"{"exit_code":0,"status":"completed","output":"(Goodbye World)"}"#,
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("expected a mismatch report");
    };
    assert!(!answer.contains("Completed"), "{answer}");
    assert!(answer.contains("Goodbye World"), "{answer}");
}

#[test]
fn matching_observed_evidence_allows_completion() {
    let (mut messages, verification) = advance_to_verification();
    record(
        &mut messages,
        &verification,
        &formal_ai::agentic_coding::tool_result::complete_owned_source_read_frame("Hello World"),
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected completion");
    };
    assert_eq!(
        answer,
        formal_ai::seed::localized_response("coding_workspace_effect_observed", "en")
            .expect("maintained completion")
            .replace("{path}", "hello.txt")
    );
}

#[test]
fn requested_failure_vocabulary_is_valid_successful_evidence() {
    let prompt = "Create a file status.txt containing exactly: failed";
    let (mut messages, target_write) = before_target(prompt);
    record(&mut messages, &target_write, r#"{"success":true}"#);
    let verification = next_call(&messages);
    record(
        &mut messages,
        &verification,
        &formal_ai::agentic_coding::tool_result::complete_owned_source_read_frame("failed"),
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("expected completion");
    };
    assert_eq!(
        answer,
        formal_ai::seed::localized_response("coding_workspace_effect_observed", "en")
            .expect("maintained completion")
            .replace("{path}", "status.txt")
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
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
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
            .contains("ses_f31609a51ffeU6RSdVVZNdp0XX")
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

    let refreshed = repair_case_without_source_counts(&PathBuf::from(format!(
        "{case}/self-hosting-fixture-refresh/self-healing-case.lino"
    )));
    let canonical = repair_case_without_source_counts(&PathBuf::from(format!(
        "{root}/data/meta/self-healing-case.lino"
    )));
    assert_eq!(refreshed, canonical);
}

/// The repair case with the two counts that track the planner source removed.
///
/// The committed artifact is what a real Agent CLI run produced, and it may
/// not be rewritten: it is the evidence for that run. The canonical document
/// is generated from `src/agentic_coding/planner.rs` and
/// `tests/unit/agentic-coding/issue_558_self_healing.rs` fails when it drifts from that
/// source. Byte equality between the two therefore held only while the planner
/// was frozen; issue #1085 moved eleven handlers out of Rust and the planner's
/// link count moved with it. What still has to hold -- and does -- is that
/// every clause the run authored is the clause the recipe still produces.
fn repair_case_without_source_counts(path: &Path) -> String {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("total_link_count ") || trimmed.starts_with("named_node_count ")
            {
                let indent = &line[..line.len() - trimmed.len()];
                let key = trimmed.split_whitespace().next().unwrap_or_default();
                format!("{indent}{key} <tracks the planner source>")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
