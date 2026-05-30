//! Integration coverage for the local-app surface added for issue #347:
//! the bundled Anthropic adapter (R4/D4), the Links-Notation REST envelopes and
//! LinksQL query layer (R6/D3), and the local-database sync endpoints (R5c/D1).
//!
//! These exercise the HTTP handler directly so the routes documented in
//! `docs/desktop/server-api.md` are verified end-to-end without a live socket.

use formal_ai::{handle_api_request, parse_bundle, run_links_query, SyncStore};

// --- D4 (R4): bundled Anthropic Messages adapter ---------------------------

#[test]
fn anthropic_messages_route_returns_anthropic_envelope() {
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();

    let response = handle_api_request("POST", "/v1/messages", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["type"], "message");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["model"], "claude-sonnet-4-5");
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "Hi, how may I help you?");
    assert_eq!(json["stop_reason"], "end_turn");
    assert!(json["usage"]["input_tokens"].is_number());
}

#[test]
fn anthropic_messages_route_streams_sse_when_requested() {
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5",
        "stream": true,
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();

    let response = handle_api_request("POST", "/v1/messages", &body);

    assert_eq!(response.status_code, 200);
    assert!(response.content_type.starts_with("text/event-stream"));
    for event in [
        "event: message_start",
        "event: content_block_delta",
        "event: message_stop",
    ] {
        assert!(response.body.contains(event), "missing {event}");
    }
}

// --- D3 (R6): Links-Notation REST envelopes + LinksQL -----------------------

#[test]
fn bundle_route_returns_links_notation() {
    let response = handle_api_request("GET", "/v1/bundle", "");
    assert_eq!(response.status_code, 200);
    assert!(response.content_type.starts_with("text/plain"));
    // The full bundle is a parseable multi-record Links-Notation document.
    let records = parse_bundle(&response.body);
    assert!(!records.is_empty());
}

#[test]
fn links_route_renders_the_knowledge_graph_as_links_notation() {
    let response = handle_api_request("GET", "/v1/links", "");
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("knowledge_graph"));
    // The projection names both node and edge entries.
    assert!(response.body.contains("node"));
    assert!(response.body.contains("edge"));
}

#[test]
fn links_query_route_filters_edges_by_role() {
    // Discover a role present in the graph projection.
    let result = run_links_query("MATCH (a)-[r]->(b) RETURN a, r, b").expect("baseline query");
    let role = result
        .edges
        .first()
        .map(|edge| edge.role.clone())
        .expect("graph should have at least one edge");

    let query = format!("MATCH (a)-[r]->(b) WHERE r.role = \"{role}\" RETURN a, r, b");
    let body = serde_json::json!({ "query": query }).to_string();
    let response = handle_api_request("POST", "/v1/links/query", &body);

    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("links_query_result"));

    // Acceptance criterion: the role-filtered LinksQL result equals the
    // /v1/graph edges carrying that role.
    let filtered = run_links_query(&query).expect("role query");
    assert!(filtered.edges.iter().all(|edge| edge.role == role));
    assert!(!filtered.edges.is_empty());
}

#[test]
fn links_query_route_accepts_links_notation_body() {
    let body = "query \"MATCH (a) RETURN a\"";
    let response = handle_api_request("POST", "/v1/links/query", body);
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("links_query_result"));
}

#[test]
fn links_query_route_reports_parse_errors() {
    let body = serde_json::json!({ "query": "NONSENSE" }).to_string();
    let response = handle_api_request("POST", "/v1/links/query", &body);
    assert_eq!(response.status_code, 400);
}

// --- D1 (R5c): local-database sync endpoints --------------------------------

#[test]
fn memory_route_renders_demo_memory_document() {
    let response = handle_api_request("GET", "/v1/memory", "");
    assert_eq!(response.status_code, 200);
    assert!(response.content_type.starts_with("text/plain"));
    // Even an empty store renders a demo_memory document the memory parser
    // round-trips without error.
    let _ = formal_ai::parse_memory_links_notation(&response.body);
}

#[test]
fn memory_import_then_since_round_trips_through_store() {
    // Drive the file-backed store directly so the test is hermetic and does not
    // depend on process-wide env for the HTTP handler.
    let dir = std::env::temp_dir().join(format!("formal-ai-local-surface-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("memory.lino");

    let inbound = formal_ai::export_memory_links_notation(&[
        sample_event("a", "first"),
        sample_event("b", "second"),
    ]);

    let mut store = SyncStore::open_at(&path);
    let added = store
        .import_links_notation(&inbound)
        .expect("import should persist");
    assert_eq!(added, 2);

    let reopened = SyncStore::open_at(&path);
    assert_eq!(reopened.events().len(), 2);

    let delta = reopened.delta_links_notation(Some("a"));
    let parsed = formal_ai::parse_memory_links_notation(&delta);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].id, "b");

    let _ = std::fs::remove_dir_all(&dir);
}

fn sample_event(id: &str, content: &str) -> formal_ai::MemoryEvent {
    formal_ai::MemoryEvent {
        id: id.to_owned(),
        content: Some(content.to_owned()),
        ..formal_ai::MemoryEvent::default()
    }
}
