//! Complete multi-exchange transcript regressions for issue #822.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::conversation_context::load_conversation_context_from;
use formal_ai::dialog_log::DialogExchangeLog;
use formal_ai::proxy::summarize_proxy_exchange;
use serde_json::{json, Value};

const DIALOG_ID: &str = "issue-822-multi-exchange";

fn temporary_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-issue-822-complete-transcript-{}-{nonce}",
        std::process::id()
    ))
}

fn exchange(
    timestamp_unix_ms: u128,
    request_id: &str,
    request_body: &Value,
    response_body: &Value,
) -> DialogExchangeLog {
    let request_body = request_body.to_string();
    let response_body = response_body.to_string();
    DialogExchangeLog {
        timestamp_unix_ms,
        dialog_id: DIALOG_ID.to_owned(),
        request_id: request_id.to_owned(),
        exchange: summarize_proxy_exchange(
            "POST",
            "/v1/chat/completions",
            request_body.as_bytes(),
            200,
            "application/json",
            response_body.as_bytes(),
            true,
        ),
    }
}

fn write_exchanges(directory: &std::path::Path, exchanges: &[DialogExchangeLog]) {
    fs::create_dir_all(directory).expect("create fixture directory");
    let mut jsonl = exchanges
        .iter()
        .map(|exchange| serde_json::to_string(exchange).expect("serialize exchange"))
        .collect::<Vec<_>>()
        .join("\n");
    jsonl.push('\n');
    fs::write(directory.join(format!("{DIALOG_ID}.jsonl")), jsonl).expect("write dialog fixture");
}

fn chat_response(message: &Value) -> Value {
    json!({
        "id": "response",
        "choices": [{"index": 0, "message": message, "finish_reason": "stop"}]
    })
}

#[test]
fn export_merges_incremental_and_cumulative_requests_with_every_response() {
    let directory = temporary_directory();
    let system = json!({"role": "system", "content": "Use all context"});
    let first_user = json!({"role": "user", "content": "First question"});
    let first_assistant = json!({"role": "assistant", "content": "First answer"});
    let second_user = json!({"role": "user", "content": "Second question"});
    let second_assistant = json!({
        "role": "assistant",
        "content": null,
        "tool_calls": [{
            "id": "call_1",
            "type": "function",
            "function": {"name": "bash", "arguments": "{\"command\":\"pwd\"}"}
        }]
    });
    let tool_result = json!({
        "role": "tool",
        "tool_call_id": "call_1",
        "content": "/workspace"
    });
    let final_assistant = json!({"role": "assistant", "content": "Complete"});

    let exchanges = vec![
        exchange(
            10,
            "request-z",
            &json!({"model": "formal-ai", "messages": [system, first_user]}),
            &chat_response(&first_assistant),
        ),
        exchange(
            10,
            "request-a",
            &json!({"model": "formal-ai", "messages": [second_user]}),
            &chat_response(&second_assistant),
        ),
        exchange(
            11,
            "request-m",
            &json!({
                "model": "formal-ai",
                "messages": [
                    system,
                    first_user,
                    first_assistant,
                    second_user,
                    second_assistant,
                    tool_result
                ]
            }),
            &chat_response(&final_assistant),
        ),
    ];
    write_exchanges(&directory, &exchanges);

    let context =
        load_conversation_context_from(&directory, DIALOG_ID).expect("load complete context");
    let messages = context["messages"].as_array().expect("message list");
    let contents = messages
        .iter()
        .filter_map(|message| message.get("content"))
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(messages.len(), 7, "{context:#}");
    assert_eq!(
        contents,
        [
            "Use all context",
            "First question",
            "First answer",
            "Second question",
            "/workspace",
            "Complete"
        ],
        "{context:#}"
    );
    assert_eq!(
        messages
            .iter()
            .filter(|message| message.get("tool_calls").is_some())
            .count(),
        1,
        "overlapping cumulative history must not duplicate tool calls"
    );
    assert_eq!(context["metadata"]["exchange_count"], 3);
    assert_eq!(context["server_logs"][0]["request_id"], "request-z");
    assert_eq!(context["server_logs"][1]["request_id"], "request-a");

    fs::remove_dir_all(directory).expect("remove fixture directory");
}

#[test]
fn export_reconstructs_the_final_streamed_assistant_message() {
    let directory = temporary_directory();
    let request = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Run a command"}],
        "stream": true
    });
    let stream = concat!(
        "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"Done \"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"now\",\"tool_calls\":[{",
        "\"index\":0,\"id\":\"call_1\",\"type\":\"function\",",
        "\"function\":{\"name\":\"bash\",\"arguments\":\"{\\\"command\\\":\"}}]}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{",
        "\"index\":0,\"function\":{\"arguments\":\"\\\"pwd\\\"}\"}}]}}]}\n\n",
        "data: [DONE]\n\n"
    );
    let request_body = request.to_string();
    let record = DialogExchangeLog {
        timestamp_unix_ms: 20,
        dialog_id: DIALOG_ID.to_owned(),
        request_id: String::from("request-stream"),
        exchange: summarize_proxy_exchange(
            "POST",
            "/v1/chat/completions",
            request_body.as_bytes(),
            200,
            "text/event-stream",
            stream.as_bytes(),
            true,
        ),
    };
    write_exchanges(&directory, &[record]);

    let context =
        load_conversation_context_from(&directory, DIALOG_ID).expect("load streaming context");
    let messages = context["messages"].as_array().expect("message list");

    assert_eq!(messages.len(), 2, "{context:#}");
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "Run a command");
    assert_eq!(messages[1]["role"], "assistant");
    assert_eq!(messages[1]["content"], "Done now");
    assert_eq!(messages[1]["tool_calls"][0]["id"], "call_1");
    assert_eq!(
        messages[1]["tool_calls"][0]["function"]["arguments"],
        "{\"command\":\"pwd\"}"
    );

    fs::remove_dir_all(directory).expect("remove fixture directory");
}

#[test]
fn export_collects_responses_gemini_and_direct_assistant_envelopes() {
    let directory = temporary_directory();
    let responses_assistant = json!({
        "type": "message",
        "role": "assistant",
        "content": [{"type": "output_text", "text": "Responses answer"}]
    });
    let gemini_user = json!({
        "role": "user",
        "parts": [{"text": "Gemini question"}]
    });
    let gemini_assistant = json!({
        "role": "model",
        "parts": [{"text": "Gemini answer"}]
    });

    let exchanges = vec![
        exchange(
            30,
            "request-responses",
            &json!({
                "model": "formal-ai",
                "input": "Responses question"
            }),
            &json!({
                "id": "resp_1",
                "output": [
                    {
                        "type": "reasoning",
                        "summary": [{"type": "summary_text", "text": "internal"}]
                    },
                    responses_assistant
                ]
            }),
        ),
        exchange(
            31,
            "request-gemini",
            &json!({
                "model": "formal-ai",
                "contents": [gemini_user]
            }),
            &json!({
                "candidates": [{
                    "content": gemini_assistant,
                    "finishReason": "STOP"
                }]
            }),
        ),
        exchange(
            32,
            "request-direct",
            &json!({
                "model": "formal-ai",
                "messages": [{"role": "user", "content": "Direct question"}]
            }),
            &json!({"role": "assistant", "content": "Direct answer"}),
        ),
    ];
    write_exchanges(&directory, &exchanges);

    let context =
        load_conversation_context_from(&directory, DIALOG_ID).expect("load response envelopes");
    let messages = context["messages"].as_array().expect("message list");

    assert_eq!(messages.len(), 6, "{context:#}");
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "Responses question");
    assert_eq!(messages[1]["role"], "assistant");
    assert_eq!(messages[1]["content"][0]["text"], "Responses answer");
    assert_eq!(messages[2]["role"], "user");
    assert_eq!(messages[2]["parts"][0]["text"], "Gemini question");
    assert_eq!(messages[3]["role"], "model");
    assert_eq!(messages[3]["parts"][0]["text"], "Gemini answer");
    assert_eq!(messages[4]["content"], "Direct question");
    assert_eq!(messages[5]["content"], "Direct answer");
    assert!(
        messages
            .iter()
            .all(|message| message.get("type") != Some(&Value::String(String::from("reasoning")))),
        "reasoning envelope items are server metadata, not conversation messages: {context:#}"
    );
    assert_eq!(context["metadata"]["first_timestamp_unix_ms"], 30);
    assert_eq!(context["metadata"]["last_timestamp_unix_ms"], 32);

    fs::remove_dir_all(directory).expect("remove fixture directory");
}
