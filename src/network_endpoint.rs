//! HTTP handlers for the links-network view of the knowledge store.
//!
//! These endpoints expose the associative *links network* (issue #664) — the
//! canonical `GET /v1/network` projection and the LinksQL query surface — kept
//! together in one module so the server's request router stays lean and the
//! network-facing surface has a single home. The legacy `/v1/graph` alias is
//! served by the router by flagging [`handle_network_request`]'s response as a
//! deprecated alias; the payload is byte-for-byte identical to `/v1/network`.

use crate::engine::{is_known_trace_id, knowledge_graph, knowledge_graph_dot};
use crate::links_query::run_links_query;
use crate::server::{error_response, json_response, links_notation_response, ApiHttpResponse};

/// Serve the links-network view of the knowledge store — the canonical
/// `/v1/network` endpoint (and, flagged deprecated, its `/v1/graph` alias).
pub fn handle_network_request(query: &str) -> ApiHttpResponse {
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
            deprecated: false,
        };
    }

    json_response(200, &knowledge_graph())
}

/// Answer a LinksQL query against the links network
/// (`POST /v1/links/query`). The response is Links-Notation text.
pub fn handle_links_query_request(body: &str) -> ApiHttpResponse {
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
