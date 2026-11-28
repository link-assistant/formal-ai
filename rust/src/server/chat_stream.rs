use serde_json::{Value, json};

use super::ApiHttpResponse;
use crate::engine::render_thinking_steps;
use crate::protocol::ChatCompletion;

const SSE_DATA_PREFIX: &str = "data: ";

/// Serialise a completed [`ChatCompletion`] as an OpenAI-compatible
/// `chat.completion.chunk` SSE stream.
///
/// The Vercel AI SDK's `@ai-sdk/openai-compatible` provider (and every other
/// OpenAI-compatible streaming parser) expects incremental `chat.completion.chunk`
/// events carrying `choices[].delta` — not a single `chat.completion` payload.
/// Shipping the non-streaming shape "worked" for text (the SDK falls back to
/// scraping content out of the raw SSE stream, which is where the CLI's
/// *"AI SDK dropped token data"* warning comes from) but silently dropped
/// `tool_calls`, so the agent CLI never actually invoked the tool the planner
/// requested. Emit a `role` delta, one delta per tool call (full arguments in one
/// frame), a final `finish_reason` chunk carrying any `output_projection`, an
/// optional `stream_options.include_usage` chunk, and the `[DONE]` sentinel.
pub(super) fn chat_completion_sse_response(
    completion: &ChatCompletion,
    include_usage: bool,
) -> ApiHttpResponse {
    let mut body = String::new();
    let mut base = json!({
        "id": completion.id,
        "object": "chat.completion.chunk",
        "created": completion.created,
        "model": completion.model,
    });

    let choice = completion.choices.first();

    // Chunk 1: role delta.
    let role_delta = json!({
        "index": 0,
        "delta": { "role": "assistant" },
        "finish_reason": null,
    });
    body.push_str(&sse_chunk(&base, &role_delta));

    // Chunk 2..N: reasoning, content, or tool_call deltas.
    if let Some(choice) = choice {
        let reasoning = if choice.message.reasoning_content.is_empty() {
            render_thinking_steps(&choice.message.thinking_steps)
        } else {
            choice.message.reasoning_content.clone()
        };
        if !reasoning.is_empty() {
            let delta = json!({
                "index": 0,
                "delta": {
                    "reasoning_content": reasoning,
                    "reasoning": reasoning,
                },
                "finish_reason": null,
            });
            body.push_str(&sse_chunk(&base, &delta));
        }
        let text = choice.message.content.plain_text();
        if !text.is_empty() {
            let delta = json!({
                "index": 0,
                "delta": { "content": text },
                "finish_reason": null,
            });
            body.push_str(&sse_chunk(&base, &delta));
        }
        if let Some(refusal) = &choice.message.refusal {
            let delta = json!({
                "index": 0,
                "delta": { "refusal": refusal },
                "finish_reason": null,
            });
            body.push_str(&sse_chunk(&base, &delta));
        }
        for (index, call) in choice.message.tool_calls.iter().enumerate() {
            let delta = json!({
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": index,
                        "id": call.id,
                        "type": call.kind,
                        "function": {
                            "name": call.function.name,
                            "arguments": call.function.arguments,
                        }
                    }]
                },
                "finish_reason": null,
            });
            body.push_str(&sse_chunk(&base, &delta));
        }
    }

    // Final chunk: finish_reason.
    let finish_reason = choice.map_or_else(
        || String::from("stop"),
        |choice| choice.finish_reason.clone(),
    );
    let mut final_chunk = json!({
        "index": 0,
        "delta": {},
        "finish_reason": finish_reason,
    });
    if let Some(metadata) = choice.and_then(|choice| choice.message.formal_ai.as_ref()) {
        final_chunk["delta"]["formal_ai"] = json!(metadata);
    }
    if let Some(projection) = &completion.output_projection {
        base["output_projection"] = projection.clone();
    }
    if let Some(format) = &completion.response_format {
        base["response_format"] = format.clone();
    }
    if let Some(identifier) = &completion.derivation_identifier {
        base["derivation_id"] = json!(identifier);
        base["evidence_links"] = json!(completion.evidence_links);
        if let Some(choice) = choice {
            final_chunk["delta"]["thinking_steps"] = json!(choice.message.thinking_steps);
        }
    }
    body.push_str(&sse_chunk(&base, &final_chunk));

    // Optional usage chunk — the AI SDK reads token counts from here when
    // `stream_options.include_usage` is set (per OpenAI's spec).
    if include_usage {
        let usage_payload = json!({
            "id": completion.id,
            "object": "chat.completion.chunk",
            "created": completion.created,
            "model": completion.model,
            "choices": [],
            "usage": {
                "prompt_tokens": completion.usage.prompt_tokens,
                "completion_tokens": completion.usage.completion_tokens,
                "total_tokens": completion.usage.total_tokens,
            }
        });
        body.push_str(SSE_DATA_PREFIX);
        body.push_str(&usage_payload.to_string());
        body.push_str("\n\n");
    }

    body.push_str(SSE_DATA_PREFIX);
    body.push_str("[DONE]\n\n");

    ApiHttpResponse {
        status_code: 200,
        content_type: "text/event-stream",
        body,
        deprecated: false,
    }
}

/// Serialise a single OpenAI streaming chunk: merge `base` (id/object/created/model)
/// with a `choices` entry and emit it as an SSE `data:` frame.
fn sse_chunk(base: &Value, choice: &Value) -> String {
    let mut merged = base.clone();
    if let Value::Object(map) = &mut merged {
        map.insert(String::from("choices"), Value::Array(vec![choice.clone()]));
    }
    format!("{SSE_DATA_PREFIX}{merged}\n\n")
}
