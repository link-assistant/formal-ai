//! Summarise a proxied response body into the fields `proxy.jsonl` records.
//!
//! The recorder has to read four vendor shapes — OpenAI chat/Responses, Gemini
//! and Anthropic — in both their whole-body and their server-sent-event forms,
//! which is most of the proxy's code and none of its socket handling. It lives
//! here so `proxy.rs` stays the transport.

use std::collections::BTreeMap;

use serde_json::Value;

use super::ProxyToolCallLog;

#[derive(Debug, Default)]
pub struct ResponseSummary {
    pub model: Option<String>,
    pub tool_calls: Vec<ProxyToolCallLog>,
    pub content: String,
}

#[derive(Debug, Default)]
struct StreamingChatAccumulator {
    model: Option<String>,
    content: String,
    tool_calls: BTreeMap<u64, StreamingToolCall>,
}

/// The Anthropic streaming shape, which spreads one tool call across three
/// events: `content_block_start` names it, `content_block_delta` carries its
/// arguments as `partial_json`, and `content_block_stop` ends it.
///
/// Without this the `claude` leg of the issue-#671 matrix logged
/// `"response_tool_calls": []` for an exchange that plainly planned `Read` —
/// the transcript is the evidence a recorded session is made of, so a blank
/// field there reads as "the client answered from prose" and failed the leg
/// over a defect in the recorder rather than in the server.
#[derive(Debug, Default)]
struct StreamingAnthropicAccumulator {
    model: Option<String>,
    content: String,
    tool_calls: BTreeMap<u64, StreamingToolCall>,
}

#[derive(Debug, Default)]
struct StreamingToolCall {
    name: String,
    arguments: String,
}

#[derive(Debug)]
struct SseEvent {
    data: String,
}

pub fn summarize_sse_response(body: &str) -> ResponseSummary {
    let events = parse_sse_events(body);
    for event in &events {
        let data = event.data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("response.completed") {
            if let Some(response) = value.get("response") {
                return summarize_response_value(response);
            }
        }
    }

    let mut chat = StreamingChatAccumulator::default();
    let mut anthropic = StreamingAnthropicAccumulator::default();
    let mut summary = ResponseSummary::default();
    for event in events {
        let data = event.data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if chat.apply_chunk(&value) || anthropic.apply_event(&value) {
            continue;
        }
        merge_response_summary(&mut summary, summarize_response_value(&value));
    }
    merge_response_summary(&mut summary, anthropic.finish());
    merge_response_summary(&mut summary, chat.finish());
    summary
}

fn parse_sse_events(body: &str) -> Vec<SseEvent> {
    let normalized = body.replace("\r\n", "\n");
    normalized
        .split("\n\n")
        .filter_map(|block| {
            let mut data = String::new();
            for line in block.lines() {
                if let Some(value) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(value.trim_start());
                }
            }
            (!data.is_empty()).then_some(SseEvent { data })
        })
        .collect()
}

impl StreamingChatAccumulator {
    fn apply_chunk(&mut self, value: &Value) -> bool {
        let Some(choices) = value.get("choices").and_then(Value::as_array) else {
            return false;
        };
        if self.model.is_none() {
            self.model = value
                .get("model")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
        }
        for choice in choices {
            let Some(delta) = choice.get("delta") else {
                continue;
            };
            if let Some(content) = delta.get("content").and_then(Value::as_str) {
                self.content.push_str(content);
            }
            if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
                for call in tool_calls {
                    let index = call.get("index").and_then(Value::as_u64).unwrap_or(0);
                    let entry = self.tool_calls.entry(index).or_default();
                    if let Some(name) = call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                    {
                        entry.name.push_str(name);
                    }
                    if let Some(arguments) = call
                        .get("function")
                        .and_then(|function| function.get("arguments"))
                        .and_then(Value::as_str)
                    {
                        entry.arguments.push_str(arguments);
                    }
                }
            }
        }
        true
    }

    fn finish(self) -> ResponseSummary {
        ResponseSummary {
            model: self.model,
            tool_calls: self
                .tool_calls
                .into_values()
                .filter(|call| !call.name.is_empty())
                .map(|call| ProxyToolCallLog {
                    name: call.name,
                    arguments: arguments_from_str(&call.arguments),
                })
                .collect(),
            content: self.content,
        }
    }
}

impl StreamingAnthropicAccumulator {
    /// Whether this event belongs to an Anthropic message stream. Events it does
    /// not recognise fall through to the generic summariser, so an OpenAI or
    /// Gemini stream is unaffected.
    fn apply_event(&mut self, value: &Value) -> bool {
        let Some(kind) = value.get("type").and_then(Value::as_str) else {
            return false;
        };
        let index = value.get("index").and_then(Value::as_u64).unwrap_or(0);
        match kind {
            "message_start" => {
                if self.model.is_none() {
                    self.model = value
                        .get("message")
                        .and_then(|message| message.get("model"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                }
            }
            "content_block_start" => {
                let block = value.get("content_block");
                if block
                    .and_then(|block| block.get("type"))
                    .and_then(Value::as_str)
                    == Some("tool_use")
                {
                    if let Some(name) = block
                        .and_then(|block| block.get("name"))
                        .and_then(Value::as_str)
                    {
                        self.tool_calls
                            .entry(index)
                            .or_default()
                            .name
                            .push_str(name);
                    }
                }
            }
            "content_block_delta" => {
                let delta = value.get("delta");
                if let Some(text) = delta
                    .and_then(|delta| delta.get("text"))
                    .and_then(Value::as_str)
                {
                    self.content.push_str(text);
                }
                if let Some(partial) = delta
                    .and_then(|delta| delta.get("partial_json"))
                    .and_then(Value::as_str)
                {
                    self.tool_calls
                        .entry(index)
                        .or_default()
                        .arguments
                        .push_str(partial);
                }
            }
            "content_block_stop" | "message_delta" | "message_stop" | "ping" => {}
            _ => return false,
        }
        true
    }

    fn finish(self) -> ResponseSummary {
        ResponseSummary {
            model: self.model,
            tool_calls: self
                .tool_calls
                .into_values()
                .filter(|call| !call.name.is_empty())
                .map(|call| ProxyToolCallLog {
                    name: call.name,
                    arguments: arguments_from_str(&call.arguments),
                })
                .collect(),
            content: self.content,
        }
    }
}

pub fn summarize_response_value(value: &Value) -> ResponseSummary {
    let mut summary = ResponseSummary::default();
    apply_response_value(value, &mut summary);
    summary
}

fn merge_response_summary(target: &mut ResponseSummary, source: ResponseSummary) {
    if target.model.is_none() {
        target.model = source.model;
    }
    target.tool_calls.extend(source.tool_calls);
    target.content.push_str(&source.content);
}

fn apply_response_value(value: &Value, summary: &mut ResponseSummary) {
    set_model(summary, value.get("model").and_then(Value::as_str));
    set_model(summary, value.get("modelVersion").and_then(Value::as_str));

    if let Some(response) = value.get("response") {
        apply_response_value(response, summary);
    }
    if let Some(item) = value.get("item") {
        apply_response_item(item, summary);
    }
    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            if let Some(message) = choice.get("message") {
                apply_chat_message(message, summary);
            }
        }
    }
    if let Some(output) = value.get("output").and_then(Value::as_array) {
        for item in output {
            apply_response_item(item, summary);
        }
    }
    if value.get("type").and_then(Value::as_str) == Some("function_call") {
        apply_response_item(value, summary);
    }
    if let Some(candidates) = value.get("candidates").and_then(Value::as_array) {
        for candidate in candidates {
            if let Some(parts) = candidate
                .get("content")
                .and_then(|content| content.get("parts"))
                .and_then(Value::as_array)
            {
                for part in parts {
                    apply_gemini_part(part, summary);
                }
            }
        }
    }
    if let Some(content) = value.get("content").and_then(Value::as_array) {
        for block in content {
            apply_anthropic_content_block(block, summary);
        }
    }
}

fn set_model(summary: &mut ResponseSummary, model: Option<&str>) {
    if summary.model.is_none() {
        summary.model = model.map(ToOwned::to_owned);
    }
}

fn apply_chat_message(message: &Value, summary: &mut ResponseSummary) {
    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in tool_calls {
            if let Some(function) = call.get("function") {
                append_function_call(function, summary);
            }
        }
    }
    if let Some(function) = message.get("function_call") {
        append_function_call(function, summary);
    }
    append_content_value(message.get("content"), summary);
}

fn apply_response_item(item: &Value, summary: &mut ResponseSummary) {
    match item.get("type").and_then(Value::as_str) {
        Some("function_call") => {
            if let Some(name) = item.get("name").and_then(Value::as_str) {
                summary.tool_calls.push(ProxyToolCallLog {
                    name: name.to_owned(),
                    arguments: arguments_from_value(item.get("arguments")),
                });
            }
        }
        Some("message") => {
            if let Some(content) = item.get("content").and_then(Value::as_array) {
                for part in content {
                    append_content_value(part.get("text"), summary);
                }
            }
        }
        _ => {}
    }
}

fn apply_gemini_part(part: &Value, summary: &mut ResponseSummary) {
    if let Some(call) = part.get("functionCall") {
        if let Some(name) = call.get("name").and_then(Value::as_str) {
            summary.tool_calls.push(ProxyToolCallLog {
                name: name.to_owned(),
                arguments: arguments_from_value(call.get("args")),
            });
        }
    }
    append_content_value(part.get("text"), summary);
}

fn apply_anthropic_content_block(block: &Value, summary: &mut ResponseSummary) {
    match block.get("type").and_then(Value::as_str) {
        Some("tool_use") => {
            if let Some(name) = block.get("name").and_then(Value::as_str) {
                summary.tool_calls.push(ProxyToolCallLog {
                    name: name.to_owned(),
                    arguments: arguments_from_value(block.get("input")),
                });
            }
        }
        Some("text") => append_content_value(block.get("text"), summary),
        _ => {}
    }
}

fn append_function_call(function: &Value, summary: &mut ResponseSummary) {
    if let Some(name) = function.get("name").and_then(Value::as_str) {
        summary.tool_calls.push(ProxyToolCallLog {
            name: name.to_owned(),
            arguments: arguments_from_value(function.get("arguments")),
        });
    }
}

fn append_content_value(value: Option<&Value>, summary: &mut ResponseSummary) {
    match value {
        Some(Value::String(text)) => summary.content.push_str(text),
        Some(Value::Array(parts)) => {
            for part in parts {
                append_content_value(part.get("text"), summary);
            }
        }
        _ => {}
    }
}

fn arguments_from_value(value: Option<&Value>) -> Value {
    match value {
        Some(Value::String(arguments)) => arguments_from_str(arguments),
        Some(value) => value.clone(),
        None => Value::Null,
    }
}

fn arguments_from_str(arguments: &str) -> Value {
    serde_json::from_str(arguments).unwrap_or_else(|_| Value::String(arguments.to_owned()))
}
