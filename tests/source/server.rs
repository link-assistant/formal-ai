use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::Serialize;
use serde_json::json;

use crate::anthropic::{
    anthropic_message_sse, create_anthropic_message_with_solver, AnthropicMessagesRequest,
};
use crate::engine::{is_known_trace_id, knowledge_graph, knowledge_graph_dot, DEFAULT_MODEL};
use crate::links_query::run_links_query;
use crate::memory_sync::SyncStore;
use crate::protocol::{
    create_chat_completion_with_solver, create_response_with_solver, ChatCompletionRequest,
    ResponsesRequest,
};
use crate::seed::merged_bundle;
use crate::solver::{ExecutionSurface, SolverConfig, UniversalSolver};
use crate::telegram::handle_telegram_webhook;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiHttpResponse {
    pub status_code: u16,
    pub content_type: &'static str,
    pub body: String,
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

#[must_use]
pub fn handle_api_request_with_auth(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
    auth: &ApiAuthConfig,
) -> ApiHttpResponse {
    let normalized_path = path.split('?').next().unwrap_or(path);
    let query = path.split_once('?').map_or("", |(_, q)| q);

    if requires_bearer_auth(method, normalized_path) && !auth.allows(headers) {
        return error_response(401, "missing or invalid bearer token");
    }

    match (method, normalized_path) {
        ("OPTIONS", _) => ApiHttpResponse {
            status_code: 204,
            content_type: "application/json",
            body: String::new(),
        },
        ("GET", "/health") => json_response(
            200,
            &json!({
                "status": "ok",
                "model": DEFAULT_MODEL,
            }),
        ),
        ("GET", "/v1/models") => json_response(
            200,
            &json!({
                "object": "list",
                "data": [{
                    "id": DEFAULT_MODEL,
                    "slug": DEFAULT_MODEL,
                    "object": "model",
                    "created": 0,
                    "owned_by": "link-assistant"
                }],
                "rate_limit": {
                    "requests_per_minute": 60,
                    "tokens_per_minute": 60_000
                }
            }),
        ),
        ("GET", "/v1/graph") => handle_graph_request(query),
        ("GET", "/v1/bundle") => links_notation_response(200, merged_bundle()),
        ("GET", "/v1/links") => links_notation_response(200, knowledge_graph().to_links_notation()),
        ("POST", "/v1/links/query") => handle_links_query_request(body),
        ("GET", "/v1/memory") => {
            links_notation_response(200, SyncStore::open().to_links_notation())
        }
        ("GET", "/v1/memory/since") => handle_memory_since_request(query),
        ("POST", "/v1/memory/import") => handle_memory_import_request(body),
        ("POST", "/v1/messages") => handle_anthropic_messages_request(body),
        ("POST", "/v1/chat/completions") => {
            match serde_json::from_str::<ChatCompletionRequest>(body) {
                Ok(request) => {
                    let solver = http_solver();
                    if request.stream {
                        sse_response(&create_chat_completion_with_solver(&request, &solver))
                    } else {
                        json_response(200, &create_chat_completion_with_solver(&request, &solver))
                    }
                }
                Err(error) => error_response(400, &format!("invalid chat request: {error}")),
            }
        }
        ("POST", "/v1/responses") => match serde_json::from_str::<ResponsesRequest>(body) {
            Ok(request) => {
                let solver = http_solver();
                json_response(200, &create_response_with_solver(&request, &solver))
            }
            Err(error) => error_response(400, &format!("invalid responses request: {error}")),
        },
        ("POST", "/telegram/webhook") => match handle_telegram_webhook(body) {
            Ok(Some(reply)) => json_response(200, &reply),
            Ok(None) => ApiHttpResponse {
                status_code: 200,
                content_type: "application/json",
                body: String::new(),
            },
            Err(error) => error_response(400, &error.to_string()),
        },
        _ => error_response(404, "route not found"),
    }
}

fn requires_bearer_auth(method: &str, normalized_path: &str) -> bool {
    method != "OPTIONS" && normalized_path.starts_with("/v1/")
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

fn parse_bearer_token(value: &str) -> Option<&str> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    Some(token)
}

fn http_solver() -> UniversalSolver {
    let mut config = SolverConfig::from_env();
    config.execution_surface = ExecutionSurface::HttpServer;
    UniversalSolver::new(config)
}

fn handle_graph_request(query: &str) -> ApiHttpResponse {
    let mut trace: Option<&str> = None;
    let mut format: Option<&str> = None;
    for pair in query.split('&').filter(|part| !part.is_empty()) {
        if let Some((key, value)) = pair.split_once('=') {
            match key {
                "trace" => trace = Some(value),
                "format" => format = Some(value),
                _ => {}
            }
        }
    }

    if let Some(trace_id) = trace {
        if !is_known_trace_id(trace_id) {
            return error_response(404, "unknown trace id");
        }
    }

    if format == Some("dot") {
        return ApiHttpResponse {
            status_code: 200,
            content_type: "text/plain",
            body: knowledge_graph_dot(),
        };
    }

    json_response(200, &knowledge_graph())
}

fn sse_response<T: Serialize>(value: &T) -> ApiHttpResponse {
    let payload = serde_json::to_string(value).unwrap_or_default();
    let body = format!("data: {payload}\n\ndata: [DONE]\n\n");
    ApiHttpResponse {
        status_code: 200,
        content_type: "text/event-stream",
        body,
    }
}

/// Translate an Anthropic Messages request (`POST /v1/messages`) so the `claude`
/// CLI can target the local server directly (R4 / ROADMAP D4). The underlying
/// reasoning is the same OpenAI-compatible solver (R7); only the envelope is
/// translated, plus an Anthropic SSE stream when `stream: true`.
fn handle_anthropic_messages_request(body: &str) -> ApiHttpResponse {
    match serde_json::from_str::<AnthropicMessagesRequest>(body) {
        Ok(request) => {
            let solver = http_solver();
            let message = create_anthropic_message_with_solver(&request, &solver);
            if request.stream {
                ApiHttpResponse {
                    status_code: 200,
                    content_type: "text/event-stream",
                    body: anthropic_message_sse(&message),
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
fn handle_links_query_request(body: &str) -> ApiHttpResponse {
    let Some(query) = parse_links_query_body(body) else {
        return error_response(400, "request must provide a `query` string");
    };
    match run_links_query(&query) {
        Ok(result) => links_notation_response(200, result.to_links_notation()),
        Err(error) => error_response(400, &format!("invalid LinksQL query: {error}")),
    }
}

/// Extract the LinksQL query string from a request body. Accepts either a JSON
/// object (`{"query": "..."}`, for tooling convenience) or a Links-Notation
/// envelope (`links_query`\n`  query "..."`).
fn parse_links_query_body(body: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(query) = value.get("query").and_then(|item| item.as_str()) {
            return Some(query.to_owned());
        }
    }
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("query ") {
            let unquoted = rest.trim().trim_matches('"');
            if !unquoted.is_empty() {
                return Some(unquoted.replace("\\\"", "\""));
            }
        }
    }
    None
}

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

const fn links_notation_response(status_code: u16, body: String) -> ApiHttpResponse {
    ApiHttpResponse {
        status_code,
        content_type: "text/plain",
        body,
    }
}

pub fn serve(address: &str) -> std::io::Result<()> {
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
        404 => "404 Not Found",
        _ => "500 Internal Server Error",
    };

    write!(
        stream,
        "HTTP/1.1 {status_text}\r\n\
         content-type: {}\r\n\
         content-length: {}\r\n\
         access-control-allow-origin: *\r\n\
         access-control-allow-methods: GET,POST,OPTIONS\r\n\
         access-control-allow-headers: content-type,authorization\r\n\
         connection: close\r\n\
         \r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    )
}

fn json_response<T: Serialize>(status_code: u16, value: &T) -> ApiHttpResponse {
    match serde_json::to_string_pretty(value) {
        Ok(body) => ApiHttpResponse {
            status_code,
            content_type: "application/json",
            body,
        },
        Err(error) => error_response(500, &format!("failed to serialize response: {error}")),
    }
}

fn error_response(status_code: u16, message: &str) -> ApiHttpResponse {
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
