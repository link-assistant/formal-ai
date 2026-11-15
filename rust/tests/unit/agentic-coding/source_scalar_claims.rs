use super::{source_scalar_assessment, source_scalar_question};
use crate::agentic_coding::progress::Progress;
use crate::agentic_coding::tool_result::complete_owned_source_read_frame;
use crate::protocol::{ChatMessage, ToolCall};
use serde_json::{Value, json};

fn question(expected: &str) -> String {
    format!(
        "Read facts.json. Is the declared value of enabled {expected}? Identify the source evidence and uncertainty."
    )
}
fn observed(request: &str, source: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::user(request),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "observed-read",
            "read",
            json!({"path":"facts.json"}).to_string(),
        )]),
        ChatMessage::tool_result(
            "observed-read",
            "read",
            complete_owned_source_read_frame(source),
        ),
    ]
}
fn assessment(request: &str, messages: &[ChatMessage]) -> Value {
    source_scalar_assessment(request, "facts.json", "enabled", &Progress::scan(messages))
        .expect("The source-value question has a candidate frame")
}

#[test]
fn complete_actual_read_assesses_closed_source_scalar_claims() {
    for (expected, source, status) in [
        ("true", r#"{"enabled":true}"#, "supported"),
        ("false", r#"{"enabled":true}"#, "contradicted"),
        ("not false", r#"{"enabled":true}"#, "supported"),
        ("not true", r#"{"enabled":true}"#, "contradicted"),
        ("1.0", r#"{"enabled":1}"#, "supported"),
        ("1", r#"{"enabled":1.0}"#, "supported"),
        ("-0", r#"{"enabled":0}"#, "supported"),
        ("0", r#"{"enabled":-0.0}"#, "supported"),
        ("1.25", r#"{"enabled":1.25}"#, "supported"),
        ("null", r#"{"enabled":null}"#, "supported"),
        (r#""true""#, r#"{"enabled":true}"#, "contradicted"),
        (
            r#""?. false or true""#,
            r#"{"enabled":"?. false or true"}"#,
            "supported",
        ),
        (
            r#""Error: failed""#,
            r#"{"enabled":"Error: failed"}"#,
            "supported",
        ),
        ("true", r#"{"other":true}"#, "unknown"),
        ("true", r#"{"enabled":true,"enabled":false}"#, "unknown"),
        ("true", r#"{"enabled":[]}"#, "unknown"),
        ("true", r#"{"enabled":9007199254740993}"#, "unknown"),
        ("false or true", r#"{"enabled":true}"#, "unknown"),
    ] {
        let request = question(expected);
        let result = assessment(&request, &observed(&request, source));
        assert_eq!(result["status"], status, "{expected}: {result}");
        if status != "unknown" {
            assert_eq!(
                result["evidence"][0]["documentSpan"],
                json!([0, source.len()])
            );
            assert_eq!(result["evidence"][0]["spanScope"], "complete-document");
            assert_eq!(
                result["evidence"][0]["sourceIdentity"],
                crate::source_fetch::sha256_hex(source.as_bytes())
            );
        }
    }
}

#[test]
fn stale_pending_failed_and_incomplete_read_evidence_stays_unknown() {
    let request = question("true");
    for tool in ["write", "edit", "bash"] {
        let mut messages = observed(&request, r#"{"enabled":true}"#);
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "later-effect",
            tool,
            if tool == "bash" {
                json!({"command":"unknown-effect"}).to_string()
            } else {
                json!({"path":"foreign.txt","content":"changed","oldString":"a","newString":"b"})
                    .to_string()
            },
        )]));
        messages.push(ChatMessage::tool_result("later-effect", tool, "ok"));
        assert_eq!(
            assessment(&request, &messages)["status"],
            "unknown",
            "{tool}"
        );
    }
    let mut pending = observed(&request, r#"{"enabled":true}"#);
    pending.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "pending-effect",
        "write",
        json!({"path":"foreign.txt","content":"changed"}).to_string(),
    )]));
    assert_eq!(assessment(&request, &pending)["status"], "unknown");
    for failed in [false, true] {
        let mut messages = observed(&request, r#"{"enabled":true}"#);
        messages.pop();
        messages.push(if failed {
            ChatMessage::tool_result_error("observed-read", "read", "Error: failed")
        } else {
            ChatMessage::tool_result("observed-read", "read", r#"{"enabled":true}"#)
        });
        assert_eq!(assessment(&request, &messages)["status"], "unknown");
    }
}

#[test]
fn question_operand_spans_preserve_unicode_and_refuse_unowned_negation() {
    for request in [
        "Read facts.json. What is the value of enabled?",
        "Read facts.json and tell me the value of enabled.",
    ] {
        assert!(source_scalar_question(request, "enabled").is_none());
    }
    let request = r#"Read facts.json. Is the value of ключ "?. false or true"?"#;
    let claim = source_scalar_question(request, "ключ")
        .flatten()
        .expect("Owned scalar claim");
    let start = claim["keySpan"][0].as_u64().unwrap() as usize;
    let end = claim["keySpan"][1].as_u64().unwrap() as usize;
    assert_eq!(&request[start..end], "ключ");
    assert_eq!(claim["expected"], "?. false or true");
    assert!(
        source_scalar_question(
            "Read facts.json. Is it not the case that the declared value of enabled false?",
            "enabled",
        )
        .is_some_and(|claim| claim.is_none())
    );
    assert!(
        source_scalar_question(
            "Read facts.json. Example: «Is the value of enabled true?»",
            "enabled",
        )
        .is_none()
    );
}
