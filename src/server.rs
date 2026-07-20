use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use serde_json::{json, Value};

use crate::anthropic::{
    anthropic_message_sse, create_anthropic_message_with_solver_and_memory, AnthropicContentBlock,
    AnthropicMessagesRequest,
};
use crate::context_capacity::ContextCapacity;
use crate::engine::{knowledge_graph, render_thinking_steps};
use crate::gemini::{
    create_gemini_generate_content_response_with_solver_and_memory, gemini_model_list,
    gemini_model_metadata, gemini_response_sse, vertex_model_list, GeminiGenerateContentRequest,
};
use crate::mcp::handle_mcp_request;
use crate::memory_sync::SyncStore;
use crate::network_endpoint::{handle_links_query_request, handle_network_request};
use crate::protocol::{
    chat_exchange_to_record, chat_tool_executions, create_chat_completion_with_solver_and_memory,
    create_response_with_solver_and_memory, messages_exchange_to_record,
    responses_exchange_to_record, ChatCompletion, ChatCompletionRequest, ResponsesRequest,
};
use crate::responses_stream::responses_sse_response;
use crate::seed::{canonical_model_id, merged_bundle, try_resolve_model_id};
use crate::solver::{ExecutionSurface, SolverConfig, UniversalSolver};
use crate::telegram::handle_telegram_webhook;

static HTTP_AGENT_MODE_FORCED: AtomicBool = AtomicBool::new(false);

pub const ADVERTISED_MAX_OUTPUT_TOKENS: i64 = 8_192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiHttpResponse {
    pub status_code: u16,
    pub content_type: &'static str,
    pub body: String,
    /// Whether this response was served through a deprecated route alias.
    ///
    /// Set for the legacy `/v1/graph` links-network alias so the wire response
    /// carries a `deprecation` marker in its metadata (an HTTP `deprecation`
    /// header plus a successor `link` to the canonical `/v1/network` endpoint)
    /// while the JSON payload stays byte-for-byte identical to the canonical one.
    pub deprecated: bool,
}

impl ApiHttpResponse {
    /// Flag this response as served through a deprecated route alias so the wire
    /// layer emits the `deprecation` / successor-`link` metadata. The payload is
    /// left untouched, keeping the alias byte-for-byte identical to the canonical
    /// endpoint.
    #[must_use]
    const fn into_deprecated_alias(mut self) -> Self {
        self.deprecated = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApiAuthConfig {
    pub bearer_token: Option<String>,
}

struct ParsedHttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl ApiAuthConfig {
    #[must_use]
    pub fn bearer_token(token: impl Into<String>) -> Self {
        Self {
            bearer_token: Some(token.into()),
        }
    }

    #[must_use]
    pub fn from_env() -> Self {
        Self {
            bearer_token: first_non_empty_env(&[
                "FORMAL_AI_API_BEARER_TOKEN",
                "FORMAL_AI_HTTP_BEARER_TOKEN",
                "FORMAL_AI_API_TOKEN",
            ]),
        }
    }

    #[must_use]
    pub fn allows(&self, headers: &[(&str, &str)]) -> bool {
        let Some(expected) = self.bearer_token.as_deref() else {
            return true;
        };
        bearer_token_from_headers(headers).is_some_and(|actual| actual == expected)
            || api_key_from_headers(headers).is_some_and(|actual| actual == expected)
    }
}

#[must_use]
pub fn handle_api_request(method: &str, path: &str, body: &str) -> ApiHttpResponse {
    handle_api_request_with_auth(method, path, &[], body, &ApiAuthConfig::from_env())
}

#[must_use]
pub fn handle_api_request_with_headers(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> ApiHttpResponse {
    handle_api_request_with_auth(method, path, headers, body, &ApiAuthConfig::from_env())
}

/// Record a live chat exchange into the shared memory log (issue #540), never
/// failing the request over it: a write error is logged and swallowed so the
/// answer still reaches the client.
fn record_exchange_best_effort(
    store: &mut SyncStore,
    exchange: Option<(String, String)>,
    tools: &[crate::memory_sync::RecordedToolExecution],
) {
    let Some((prompt, answer)) = exchange else {
        return;
    };
    if let Err(error) = store.record_chat_exchange_with_tools(&prompt, &answer, tools) {
        eprintln!("[memory] failed to record live chat exchange: {error}");
    }
}

#[must_use]
pub fn handle_api_request_with_auth(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
    auth: &ApiAuthConfig,
) -> ApiHttpResponse {
    let normalized_path = path.split('?').next().unwrap_or(path);
    let authorized = !requires_bearer_auth(method, normalized_path) || auth.allows(headers);
    let response = dispatch_api_request_with_auth(method, path, headers, body, auth);
    crate::dialog_log::record_api_exchange_if_enabled(
        method, path, headers, body, &response, authorized,
    );
    response
}

fn dispatch_api_request_with_auth(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
    auth: &ApiAuthConfig,
) -> ApiHttpResponse {
    let _foreground_activity = crate::dreaming_runtime::ForegroundActivity::begin();
    let normalized_path = path.split('?').next().unwrap_or(path);
    let query = path.split_once('?').map_or("", |(_, q)| q);

    if normalized_path == "/mcp" && !mcp_origin_allowed(headers) {
        return error_response(403, "MCP origin is not allowed");
    }
    if requires_bearer_auth(method, normalized_path) && !auth.allows(headers) {
        return error_response(401, "missing or invalid bearer token");
    }

    crate::dialog_log::trace_request_if_enabled(method, normalized_path, body);

    if let Some(response) = handle_dynamic_protocol_route(method, normalized_path, body) {
        return response;
    }

    match (method, normalized_path) {
        ("OPTIONS", _) => ApiHttpResponse {
            status_code: 204,
            content_type: "application/json",
            body: String::new(),
            deprecated: false,
        },
        // Reachability preflight. Claude Code opens every session with
        // `HEAD <base-url>` before its first `/v1/messages` POST; the issue-#671
        // matrix recorded that probe coming back 404 from `/api/anthropic` while
        // the conversation itself worked, which reads in a transcript exactly
        // like a misconfigured base URL. A base path we serve is reachable, and
        // saying so costs one branch.
        (
            "HEAD",
            "/" | "/health" | "/api/anthropic" | "/api/openai" | "/api/gemini" | "/api/formal-ai"
            | "/api/vertex",
        ) => ApiHttpResponse {
            status_code: 200,
            content_type: "application/json",
            body: String::new(),
            deprecated: false,
        },
        ("GET", "/health") => json_response(
            200,
            &json!({
                "status": "ok",
                "model": canonical_model_id(),
            }),
        ),
        ("GET", "/v1/models" | "/api/openai/v1/models") => handle_openai_models_request(),
        ("GET", "/v1/network" | "/api/formal-ai/v1/network") => handle_network_request(query),
        // Deprecated alias: the project's associative vocabulary is a *links
        // network*, not a graph (issue #664). `/v1/graph` keeps working for
        // existing desktop / VS Code / e2e clients but returns the same payload
        // flagged deprecated so the wire response points at `/v1/network`.
        ("GET", "/v1/graph" | "/api/formal-ai/v1/graph") => {
            handle_network_request(query).into_deprecated_alias()
        }
        ("GET", "/v1/bundle" | "/api/formal-ai/v1/bundle") => {
            links_notation_response(200, merged_bundle())
        }
        ("GET", "/v1/links" | "/api/formal-ai/v1/links") => {
            links_notation_response(200, knowledge_graph().to_links_notation())
        }
        ("POST", "/v1/links/query" | "/api/formal-ai/v1/links/query") => {
            handle_links_query_request(body)
        }
        ("GET", "/v1/memory" | "/api/formal-ai/v1/memory") => {
            links_notation_response(200, SyncStore::open().to_links_notation())
        }
        ("GET", "/v1/memory/since" | "/api/formal-ai/v1/memory/since") => {
            handle_memory_since_request(query)
        }
        ("POST", "/v1/memory/import" | "/api/formal-ai/v1/memory/import") => {
            handle_memory_import_request(body)
        }
        ("POST", "/v1/messages" | "/api/anthropic/v1/messages") => {
            handle_anthropic_messages_request(body)
        }
        ("POST", "/v1/chat/completions" | "/api/openai/v1/chat/completions") => {
            match serde_json::from_str::<ChatCompletionRequest>(body) {
                Ok(request) => {
                    if let Some(response) = unsupported_model_response(request.model.as_deref()) {
                        return response;
                    }
                    let solver = http_solver();
                    let mut store = SyncStore::open();
                    let completion = create_chat_completion_with_solver_and_memory(
                        &request,
                        &solver,
                        store.events(),
                    );
                    record_exchange_best_effort(
                        &mut store,
                        chat_exchange_to_record(&request, &completion),
                        &chat_tool_executions(&request.messages),
                    );
                    if request.stream {
                        let include_usage = request
                            .stream_options
                            .is_some_and(|options| options.include_usage);
                        chat_completion_sse_response(&completion, include_usage)
                    } else {
                        json_response(200, &completion)
                    }
                }
                Err(error) => error_response(400, &format!("invalid chat request: {error}")),
            }
        }
        ("POST", "/v1/responses" | "/api/openai/v1/responses") => {
            match serde_json::from_str::<ResponsesRequest>(body) {
                Ok(request) => {
                    if let Some(response) = unsupported_model_response(request.model.as_deref()) {
                        return response;
                    }
                    let solver = http_solver();
                    let mut store = SyncStore::open();
                    let response =
                        create_response_with_solver_and_memory(&request, &solver, store.events());
                    record_exchange_best_effort(
                        &mut store,
                        responses_exchange_to_record(&request, &response),
                        &chat_tool_executions(&request.to_chat_completion_request().messages),
                    );
                    if request.stream {
                        responses_sse_response(&response)
                    } else {
                        json_response(200, &response)
                    }
                }
                Err(error) => error_response(400, &format!("invalid responses request: {error}")),
            }
        }
        ("POST", "/mcp") => handle_mcp_request(body, &http_solver()),
        ("GET", "/mcp") => error_response(405, "MCP SSE streams are not supported"),
        ("POST", "/telegram/webhook") => match handle_telegram_webhook(body) {
            Ok(Some(reply)) => json_response(200, &reply),
            Ok(None) => ApiHttpResponse {
                status_code: 200,
                content_type: "application/json",
                body: String::new(),
                deprecated: false,
            },
            Err(error) => error_response(400, &error.to_string()),
        },
        _ => error_response(404, "route not found"),
    }
}

fn requires_bearer_auth(method: &str, normalized_path: &str) -> bool {
    method != "OPTIONS"
        && (normalized_path == "/mcp"
            || normalized_path.starts_with("/v1/")
            || normalized_path.starts_with("/api/"))
}

fn mcp_origin_allowed(headers: &[(&str, &str)]) -> bool {
    let Some(origin) = headers
        .iter()
        .find_map(|(name, value)| name.eq_ignore_ascii_case("origin").then_some(*value))
    else {
        return true;
    };
    let Some(host) = headers
        .iter()
        .find_map(|(name, value)| name.eq_ignore_ascii_case("host").then_some(*value))
    else {
        return false;
    };
    let origin = origin.trim_end_matches('/');
    origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .is_some_and(|authority| authority.eq_ignore_ascii_case(host))
}

fn handle_dynamic_protocol_route(
    method: &str,
    normalized_path: &str,
    body: &str,
) -> Option<ApiHttpResponse> {
    if method == "GET" && normalized_path == "/api/gemini/v1beta/models" {
        return Some(json_response(200, &gemini_model_list()));
    }
    if method == "GET" {
        if let Some(model) = gemini_model_metadata_path(normalized_path) {
            return Some(json_response(
                200,
                &gemini_model_metadata(&format!("models/{model}")),
            ));
        }
        if let Some((project, location)) = vertex_models_path(normalized_path) {
            return Some(json_response(200, &vertex_model_list(&project, &location)));
        }
    }
    if method == "POST" {
        if let Some(model) = gemini_model_action_path(normalized_path, "generateContent") {
            return Some(handle_gemini_generate_content_request(&model, false, body));
        }
        if let Some(model) = gemini_model_action_path(normalized_path, "streamGenerateContent") {
            return Some(handle_gemini_generate_content_request(&model, true, body));
        }
        if let Some(route) = vertex_model_action_path(normalized_path, "generateContent") {
            return Some(handle_gemini_generate_content_request(
                &route.model,
                false,
                body,
            ));
        }
        if let Some(route) = vertex_model_action_path(normalized_path, "streamGenerateContent") {
            return Some(handle_gemini_generate_content_request(
                &route.model,
                true,
                body,
            ));
        }
    }
    None
}

fn handle_openai_models_request() -> ApiHttpResponse {
    let model_id = canonical_model_id();
    let context = match ContextCapacity::current() {
        Ok(context) => context,
        Err(error) => return error_response(500, &error.to_string()),
    };
    let context_metadata = json!(context);
    json_response(
        200,
        &json!({
            "object": "list",
            "data": [{
                "id": model_id,
                "slug": model_id,
                "object": "model",
                "owned_by": "link-assistant",
                "context_window": context.context_window_tokens,
                "context_window_tokens": context.context_window_tokens,
                "context_used_tokens": context.context_used_tokens,
                "context_used_fraction": context.context_used_fraction,
                "disk_free_bytes": context.disk_free_bytes,
                "memory_used_bytes": context.memory_used_bytes,
                "avg_utf8_bytes_per_char": context.avg_utf8_bytes_per_char,
                "context": context_metadata
            }],
            "models": [{
                "id": model_id,
                "slug": model_id,
                "name": model_id,
                "context_window": context.context_window_tokens,
                "max_output_tokens": ADVERTISED_MAX_OUTPUT_TOKENS,
                "context_window_tokens": context.context_window_tokens,
                "context_used_tokens": context.context_used_tokens,
                "context_used_fraction": context.context_used_fraction,
                "disk_free_bytes": context.disk_free_bytes,
                "memory_used_bytes": context.memory_used_bytes,
                "avg_utf8_bytes_per_char": context.avg_utf8_bytes_per_char,
                "context": context_metadata
            }],
            "rate_limit": {
                "requests_per_minute": 60,
                "tokens_per_minute": 60_000
            }
        }),
    )
}

fn handle_gemini_generate_content_request(
    model: &str,
    stream: bool,
    body: &str,
) -> ApiHttpResponse {
    let mut model = normalize_protocol_model_id(model);
    if is_vendor_hardcoded_gemini_model(&model) {
        // Answer as ourselves, not as the model the CLI asked for: a transcript
        // that echoed `gemini-3-flash-preview` back would claim a provenance
        // this server does not have.
        model = canonical_model_id().to_owned();
    } else if let Some(response) = unsupported_model_response(Some(&model)) {
        return response;
    }
    match serde_json::from_str::<GeminiGenerateContentRequest>(body) {
        Ok(request) => {
            let solver = http_solver();
            let mut store = SyncStore::open();
            let chat_request = request.to_chat_completion_request(&model);
            let response = create_gemini_generate_content_response_with_solver_and_memory(
                &request,
                &model,
                &solver,
                store.events(),
            );
            let answer = response["candidates"][0]["content"]["parts"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n");
            record_exchange_best_effort(
                &mut store,
                messages_exchange_to_record(&chat_request.messages, &answer),
                &chat_tool_executions(&chat_request.messages),
            );
            if stream {
                ApiHttpResponse {
                    status_code: 200,
                    content_type: "text/event-stream",
                    body: gemini_response_sse(&response),
                    deprecated: false,
                }
            } else {
                json_response(200, &response)
            }
        }
        Err(error) => error_response(400, &format!("invalid generateContent request: {error}")),
    }
}

fn normalize_protocol_model_id(model: &str) -> String {
    model
        .strip_prefix("models/")
        .unwrap_or(model)
        .trim()
        .to_owned()
}

fn gemini_model_metadata_path(path: &str) -> Option<String> {
    let model = path.strip_prefix("/api/gemini/v1beta/models/")?;
    (!model.contains(':')).then(|| normalize_protocol_model_id(model))
}

fn gemini_model_action_path(path: &str, action: &str) -> Option<String> {
    let model = path.strip_prefix("/api/gemini/v1beta/models/")?;
    let suffix = format!(":{action}");
    model.strip_suffix(&suffix).map(normalize_protocol_model_id)
}

fn vertex_models_path(path: &str) -> Option<(String, String)> {
    let route = path.strip_prefix("/api/vertex/v1/projects/")?;
    let (project, route) = route.split_once("/locations/")?;
    let (location, tail) = route.split_once("/publishers/google/models")?;
    tail.is_empty()
        .then(|| (project.to_owned(), location.to_owned()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VertexModelRoute {
    model: String,
}

fn vertex_model_action_path(path: &str, action: &str) -> Option<VertexModelRoute> {
    let route = path.strip_prefix("/api/vertex/v1/projects/")?;
    let (_project, route) = route.split_once("/locations/")?;
    let (_location, model) = route.split_once("/publishers/google/models/")?;
    let suffix = format!(":{action}");
    let model = model.strip_suffix(&suffix)?;
    Some(VertexModelRoute {
        model: normalize_protocol_model_id(model),
    })
}

fn first_non_empty_env(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        let value = std::env::var(name).ok()?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn bearer_token_from_headers<'a>(headers: &'a [(&str, &str)]) -> Option<&'a str> {
    headers.iter().find_map(|(name, value)| {
        if name.eq_ignore_ascii_case("authorization") {
            parse_bearer_token(value)
        } else {
            None
        }
    })
}

fn api_key_from_headers<'a>(headers: &'a [(&str, &str)]) -> Option<&'a str> {
    headers.iter().find_map(|(name, value)| {
        (name.eq_ignore_ascii_case("x-api-key")
            || name.eq_ignore_ascii_case("x-goog-api-key")
            || name.eq_ignore_ascii_case("anthropic-api-key"))
        .then(|| value.trim())
        .filter(|value| !value.is_empty())
    })
}

fn parse_bearer_token(value: &str) -> Option<&str> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    Some(token)
}

/// Whether a Gemini-protocol model id is one Gemini CLI hardcodes for its own
/// internal calls.
///
/// The CLI routes the *user's* turns through whatever `with-formal-ai`
/// configures (`models/formal-ai:streamGenerateContent` in the transcripts), but
/// its utility calls — the next-speaker check and the web-search fallback —
/// name a flash model no setting overrides. Rejecting those with a 400 broke the
/// CLI mid-conversation while the main channel looked healthy; the issue-#671
/// matrix caught it on the `gemini` leg's "search online" prompt, where
/// `gemini-3-flash-preview:generateContent` came back
/// `unsupported model … use \`formal-ai\``.
///
/// We are the only backend behind that base URL, so the honest answer is to
/// serve the request with the canonical model rather than to fail it. The prefix
/// keeps the exception to ids the vendor itself ships, and it survives the next
/// flash release; a mistyped or foreign id on this path is still a 400.
fn is_vendor_hardcoded_gemini_model(model: &str) -> bool {
    model.trim().to_ascii_lowercase().starts_with("gemini-")
}

fn unsupported_model_response(model: Option<&str>) -> Option<ApiHttpResponse> {
    let model = model.map(str::trim).filter(|model| !model.is_empty())?;
    if try_resolve_model_id(Some(model)).is_some() {
        None
    } else {
        Some(error_response(
            400,
            &format!(
                "unsupported model `{model}`; use `{}` or a configured alias",
                canonical_model_id()
            ),
        ))
    }
}

/// Enable agent-mode tool calls for HTTP solver instances created by this
/// process, independent of `FORMAL_AI_AGENT_MODE`.
///
/// This is used by `formal-ai serve --agent-mode` so operators have an explicit
/// command-line opt-in instead of relying only on an environment variable.
pub fn enable_http_agent_mode_for_current_process() {
    HTTP_AGENT_MODE_FORCED.store(true, Ordering::Relaxed);
}

fn http_solver() -> UniversalSolver {
    let mut config = SolverConfig::from_env();
    if HTTP_AGENT_MODE_FORCED.load(Ordering::Relaxed) {
        config.agent_mode = true;
    }
    config.execution_surface = ExecutionSurface::HttpServer;
    UniversalSolver::new(config)
}

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
/// requested. Emit chunks: an initial `role` delta, one delta per tool call
/// (with full `function.arguments` in a single frame — the SDK stitches them
/// back together), a final `finish_reason` chunk, an optional usage chunk when
/// the client asks for it via `stream_options.include_usage`, and the closing
/// `[DONE]` sentinel.
fn chat_completion_sse_response(
    completion: &ChatCompletion,
    include_usage: bool,
) -> ApiHttpResponse {
    let mut body = String::new();
    let base = json!({
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
    let final_chunk = json!({
        "index": 0,
        "delta": {},
        "finish_reason": finish_reason,
    });
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
        body.push_str("data: ");
        body.push_str(&usage_payload.to_string());
        body.push_str("\n\n");
    }

    body.push_str("data: [DONE]\n\n");

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
    format!("data: {merged}\n\n")
}

/// Translate an Anthropic Messages request (`POST /v1/messages`) so the `claude`
/// CLI can target the local server directly (R4 / ROADMAP D4). The underlying
/// reasoning is the same OpenAI-compatible solver (R7); only the envelope is
/// translated, plus an Anthropic SSE stream when `stream: true`.
fn handle_anthropic_messages_request(body: &str) -> ApiHttpResponse {
    match serde_json::from_str::<AnthropicMessagesRequest>(body) {
        Ok(request) => {
            if let Some(response) = unsupported_model_response(request.model.as_deref()) {
                return response;
            }
            let solver = http_solver();
            let mut store = SyncStore::open();
            let chat_request = request.to_chat_completion_request();
            let message =
                create_anthropic_message_with_solver_and_memory(&request, &solver, store.events());
            let answer = message
                .content
                .iter()
                .filter_map(|block| match block {
                    AnthropicContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            record_exchange_best_effort(
                &mut store,
                messages_exchange_to_record(&chat_request.messages, &answer),
                &chat_tool_executions(&chat_request.messages),
            );
            if request.stream {
                ApiHttpResponse {
                    status_code: 200,
                    content_type: "text/event-stream",
                    body: anthropic_message_sse(&message),
                    deprecated: false,
                }
            } else {
                json_response(200, &message)
            }
        }
        Err(error) => error_response(400, &format!("invalid messages request: {error}")),
    }
}

/// Evaluate a LinksQL query (`POST /v1/links/query`, ROADMAP D3 / R6). The body
/// is a Links-Notation envelope carrying the query string; the response is the
/// matched nodes/edges as a Links-Notation envelope (R7 keeps this internal
/// channel Links-native rather than introducing a non-OpenAI JSON REST surface).
/// Return the memory delta after a given event id (`GET /v1/memory/since?event=<id>`,
/// ROADMAP D1 / R5c). The payload is `demo_memory` Links Notation (R7).
fn handle_memory_since_request(query: &str) -> ApiHttpResponse {
    let last_seen = query_param(query, "event");
    let store = SyncStore::open();
    links_notation_response(200, store.delta_links_notation(last_seen.as_deref()))
}

/// Merge an inbound `demo_memory` document into the shared store
/// (`POST /v1/memory/import`, ROADMAP D1 / R5c).
fn handle_memory_import_request(body: &str) -> ApiHttpResponse {
    let mut store = SyncStore::open();
    match store.import_links_notation(body) {
        Ok(added) => json_response(
            200,
            &json!({
                "object": "memory.import",
                "added": added,
                "total": store.events().len(),
            }),
        ),
        Err(error) => error_response(500, &format!("failed to persist memory: {error}")),
    }
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .find_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            (name == key).then(|| value.to_owned())
        })
}

pub(crate) const fn links_notation_response(status_code: u16, body: String) -> ApiHttpResponse {
    ApiHttpResponse {
        status_code,
        content_type: "text/plain",
        body,
        deprecated: false,
    }
}

pub fn serve(address: &str) -> std::io::Result<()> {
    crate::dreaming_runtime::start_core_dreaming();
    eprintln!(
        "formal-ai shared memory: {}",
        crate::shared_memory::shared_memory_path().display()
    );
    let listener = TcpListener::bind(address)?;
    eprintln!("formal-ai server listening on http://{address}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }

    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> std::io::Result<()> {
    let Some(request) = read_request(stream)? else {
        return Ok(());
    };
    let headers = request
        .headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    let response =
        handle_api_request_with_headers(&request.method, &request.path, &headers, &request.body);
    write_response(stream, &response)
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<ParsedHttpRequest>> {
    let mut buffer = [0_u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(None);
    }

    let mut request_bytes = buffer[..bytes_read].to_vec();
    let header_end = loop {
        if let Some(position) = find_header_end(&request_bytes) {
            break position;
        }
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        request_bytes.extend_from_slice(&buffer[..bytes_read]);
    };

    let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
    let content_length = content_length(&header_text);
    let body_start = header_end + 4;

    while request_bytes.len() < body_start.saturating_add(content_length) {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        request_bytes.extend_from_slice(&buffer[..bytes_read]);
    }

    let request_line = header_text.lines().next().unwrap_or_default();
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let path = request_parts.next().unwrap_or_default().to_owned();
    let headers = request_headers(&header_text);
    let body_end = body_start
        .saturating_add(content_length)
        .min(request_bytes.len());
    let body = String::from_utf8_lossy(&request_bytes[body_start..body_end]).to_string();

    Ok(Some(ParsedHttpRequest {
        method,
        path,
        headers,
        body,
    }))
}

fn write_response(stream: &mut TcpStream, response: &ApiHttpResponse) -> std::io::Result<()> {
    let status_text = match response.status_code {
        200 => "200 OK",
        204 => "204 No Content",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        405 => "405 Method Not Allowed",
        _ => "500 Internal Server Error",
    };

    // A response served through a deprecated route alias carries a wire-layer
    // deprecation note so clients can migrate without inspecting the (byte-identical)
    // body. The canonical `/v1/network` endpoint never emits it.
    let deprecation_header = if response.deprecated {
        "deprecation: true\r\nlink: </v1/network>; rel=\"successor-version\"\r\n"
    } else {
        ""
    };

    write!(
        stream,
        "HTTP/1.1 {status_text}\r\n\
         content-type: {}\r\n\
         content-length: {}\r\n\
         access-control-allow-origin: *\r\n\
         access-control-allow-methods: GET,POST,OPTIONS\r\n\
         access-control-allow-headers: content-type,authorization,x-api-key,x-goog-api-key,anthropic-api-key\r\n\
         {deprecation_header}\
         connection: close\r\n\
         \r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    )
}

pub(crate) fn json_response<T: Serialize>(status_code: u16, value: &T) -> ApiHttpResponse {
    match serde_json::to_string_pretty(value) {
        Ok(body) => ApiHttpResponse {
            status_code,
            content_type: "application/json",
            body,
            deprecated: false,
        },
        Err(error) => error_response(500, &format!("failed to serialize response: {error}")),
    }
}

pub(crate) fn error_response(status_code: u16, message: &str) -> ApiHttpResponse {
    ApiHttpResponse {
        status_code,
        content_type: "application/json",
        body: json!({
            "error": {
                "message": message,
                "type": "formal_ai_error"
            }
        })
        .to_string(),
        deprecated: false,
    }
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn request_headers(headers: &str) -> Vec<(String, String)> {
    headers
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

fn content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}
