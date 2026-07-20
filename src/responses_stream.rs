use serde_json::{json, Value};

use crate::protocol::{
    ResponseFunctionToolCall, ResponseObject, ResponseOutputItem, ResponseOutputMessage,
    ResponseReasoningItem,
};
use crate::server::ApiHttpResponse;

pub fn responses_sse_response(response: &ResponseObject) -> ApiHttpResponse {
    let mut body = String::new();
    let mut sequence_number = 0u64;

    push_response_stream_event(
        &mut body,
        "response.created",
        &json!({
            "type": "response.created",
            "sequence_number": next_response_sequence(&mut sequence_number),
            "response": response_snapshot(response, "in_progress", false),
        }),
    );
    push_response_stream_event(
        &mut body,
        "response.in_progress",
        &json!({
            "type": "response.in_progress",
            "sequence_number": next_response_sequence(&mut sequence_number),
            "response": response_snapshot(response, "in_progress", false),
        }),
    );

    for (output_index, item) in response.output.iter().enumerate() {
        if matches!(item, ResponseOutputItem::Reasoning(_)) {
            push_response_output_item_events(&mut body, &mut sequence_number, output_index, item);
        }
    }
    for (output_index, item) in response.output.iter().enumerate() {
        if !matches!(item, ResponseOutputItem::Reasoning(_)) {
            push_response_output_item_events(&mut body, &mut sequence_number, output_index, item);
        }
    }

    push_response_stream_event(
        &mut body,
        "response.completed",
        &json!({
            "type": "response.completed",
            "sequence_number": next_response_sequence(&mut sequence_number),
            "response": response_snapshot(response, "completed", true),
        }),
    );

    ApiHttpResponse {
        status_code: 200,
        content_type: "text/event-stream",
        body,
        deprecated: false,
    }
}

fn push_response_output_item_events(
    body: &mut String,
    sequence_number: &mut u64,
    output_index: usize,
    item: &ResponseOutputItem,
) {
    push_response_stream_event(
        body,
        "response.output_item.added",
        &json!({
            "type": "response.output_item.added",
            "sequence_number": next_response_sequence(sequence_number),
            "output_index": output_index,
            "item": response_output_item_started(item),
        }),
    );

    match item {
        ResponseOutputItem::Message(message) => {
            push_response_message_events(body, sequence_number, output_index, message);
        }
        ResponseOutputItem::FunctionCall(call) => {
            push_response_function_call_events(body, sequence_number, output_index, call);
        }
        ResponseOutputItem::WebSearchCall(call) => {
            for state in ["in_progress", "searching", "completed"] {
                push_response_stream_event(
                    body,
                    &format!("response.web_search_call.{state}"),
                    &json!({
                        "type": format!("response.web_search_call.{state}"),
                        "sequence_number": next_response_sequence(sequence_number),
                        "item_id": &call.id,
                        "output_index": output_index,
                    }),
                );
            }
        }
        ResponseOutputItem::Reasoning(reasoning) => {
            push_response_reasoning_events(body, sequence_number, output_index, reasoning);
        }
    }

    push_response_stream_event(
        body,
        "response.output_item.done",
        &json!({
            "type": "response.output_item.done",
            "sequence_number": next_response_sequence(sequence_number),
            "output_index": output_index,
            "item": item,
        }),
    );
}

fn push_response_message_events(
    body: &mut String,
    sequence_number: &mut u64,
    output_index: usize,
    message: &ResponseOutputMessage,
) {
    for (content_index, content) in message.content.iter().enumerate() {
        push_response_stream_event(
            body,
            "response.content_part.added",
            &json!({
                "type": "response.content_part.added",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &message.id,
                "output_index": output_index,
                "content_index": content_index,
                "part": {"type": &content.kind, "text": ""},
            }),
        );
        push_response_stream_event(
            body,
            "response.output_text.delta",
            &json!({
                "type": "response.output_text.delta",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &message.id,
                "output_index": output_index,
                "content_index": content_index,
                "delta": &content.text,
            }),
        );
        push_response_stream_event(
            body,
            "response.output_text.done",
            &json!({
                "type": "response.output_text.done",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &message.id,
                "output_index": output_index,
                "content_index": content_index,
                "text": &content.text,
            }),
        );
        push_response_stream_event(
            body,
            "response.content_part.done",
            &json!({
                "type": "response.content_part.done",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &message.id,
                "output_index": output_index,
                "content_index": content_index,
                "part": content,
            }),
        );
    }
}

fn push_response_function_call_events(
    body: &mut String,
    sequence_number: &mut u64,
    output_index: usize,
    call: &ResponseFunctionToolCall,
) {
    push_response_stream_event(
        body,
        "response.function_call_arguments.delta",
        &json!({
            "type": "response.function_call_arguments.delta",
            "sequence_number": next_response_sequence(sequence_number),
            "item_id": &call.id,
            "output_index": output_index,
            "delta": &call.arguments,
        }),
    );
    push_response_stream_event(
        body,
        "response.function_call_arguments.done",
        &json!({
            "type": "response.function_call_arguments.done",
            "sequence_number": next_response_sequence(sequence_number),
            "item_id": &call.id,
            "output_index": output_index,
            "arguments": &call.arguments,
        }),
    );
}

fn push_response_reasoning_events(
    body: &mut String,
    sequence_number: &mut u64,
    output_index: usize,
    reasoning: &ResponseReasoningItem,
) {
    for (summary_index, summary) in reasoning.summary.iter().enumerate() {
        push_response_stream_event(
            body,
            "response.reasoning_summary_part.added",
            &json!({
                "type": "response.reasoning_summary_part.added",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &reasoning.id,
                "output_index": output_index,
                "summary_index": summary_index,
                "part": {"type": &summary.kind, "text": ""},
            }),
        );
        push_response_stream_event(
            body,
            "response.reasoning_summary_text.delta",
            &json!({
                "type": "response.reasoning_summary_text.delta",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &reasoning.id,
                "output_index": output_index,
                "summary_index": summary_index,
                "delta": &summary.text,
            }),
        );
        push_response_stream_event(
            body,
            "response.reasoning_summary_text.done",
            &json!({
                "type": "response.reasoning_summary_text.done",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &reasoning.id,
                "output_index": output_index,
                "summary_index": summary_index,
                "text": &summary.text,
            }),
        );
        push_response_stream_event(
            body,
            "response.reasoning_summary_part.done",
            &json!({
                "type": "response.reasoning_summary_part.done",
                "sequence_number": next_response_sequence(sequence_number),
                "item_id": &reasoning.id,
                "output_index": output_index,
                "summary_index": summary_index,
                "part": summary,
            }),
        );
    }
}

fn response_output_item_started(item: &ResponseOutputItem) -> Value {
    match item {
        ResponseOutputItem::Message(message) => json!({
            "id": &message.id,
            "type": &message.kind,
            "role": &message.role,
            "content": [],
        }),
        ResponseOutputItem::FunctionCall(call) => json!(call),
        ResponseOutputItem::WebSearchCall(call) => json!(call),
        ResponseOutputItem::Reasoning(reasoning) => json!({
            "id": &reasoning.id,
            "type": &reasoning.kind,
            "summary": [],
        }),
    }
}

fn response_snapshot(response: &ResponseObject, status: &str, include_output: bool) -> Value {
    let mut snapshot = serde_json::to_value(response).unwrap_or_else(|_| json!({}));
    if let Value::Object(map) = &mut snapshot {
        map.insert(String::from("status"), Value::String(status.to_owned()));
        if !include_output {
            map.insert(String::from("output"), Value::Array(Vec::new()));
        }
    }
    snapshot
}

const fn next_response_sequence(sequence_number: &mut u64) -> u64 {
    let current = *sequence_number;
    *sequence_number = current.saturating_add(1);
    current
}

fn push_response_stream_event(body: &mut String, event: &str, data: &Value) {
    body.push_str("event: ");
    body.push_str(event);
    body.push('\n');
    body.push_str("data: ");
    body.push_str(&data.to_string());
    body.push_str("\n\n");
}
