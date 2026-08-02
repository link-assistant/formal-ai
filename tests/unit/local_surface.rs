//! Integration coverage for the local-app surface added for issue #347:
//! the bundled Anthropic adapter (R4/D4), the Links-Notation REST envelopes and
//! LinksQL query layer (R6/D3), and the local-database sync endpoints (R5c/D1).
//!
//! These exercise the HTTP handler directly so the routes documented in
//! `docs/desktop/server-api.md` are verified end-to-end without a live socket.

use std::sync::{Mutex, MutexGuard, OnceLock};

use formal_ai::{
    export_memory_links_notation, handle_api_request, parse_bundle, run_links_query, MemoryEvent,
    SyncStore,
};

// --- D4 (R4): bundled Anthropic Messages adapter ---------------------------

#[test]
fn anthropic_messages_route_returns_anthropic_envelope() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();

    let response = handle_api_request("POST", "/v1/messages", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["type"], "message");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["model"], "formal-ai");
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "Hi, how may I help you?");
    assert_eq!(json["stop_reason"], "end_turn");
    assert!(json["usage"]["input_tokens"].is_number());
}

#[test]
fn anthropic_messages_route_queries_persisted_memory_with_natural_language() {
    let response = with_recall_memory(|| {
        let body = serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": "Find Rust in another conversation"}]
        })
        .to_string();
        handle_api_request("POST", "/v1/messages", &body)
    });

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    let content = json["content"][0]["text"].as_str().unwrap_or_default();
    assert!(content.contains("Rust Notes"), "{content}");
    assert!(content.contains("user: What is Rust?"), "{content}");
    assert!(
        !content.contains("What is Wikipedia?"),
        "query should not include unrelated memory: {content}"
    );
}

#[test]
fn anthropic_messages_route_streams_sse_when_requested() {
    let body = serde_json::json!({
        "model": "formal-ai",
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

#[test]
fn chat_completions_route_records_live_exchange_into_configured_memory() {
    // Issue #540 §1: a POSTed chat must land in the persistent memory log so
    // background dreaming has organic data to learn from — proven end-to-end
    // through the HTTP handler against a temp FORMAL_AI_MEMORY_PATH.
    let _guard = memory_env_lock();
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-local-surface-chat-record-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("memory.lino");

    let previous = std::env::var_os("FORMAL_AI_MEMORY_PATH");
    std::env::set_var("FORMAL_AI_MEMORY_PATH", &path);
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    match previous {
        Some(value) => std::env::set_var("FORMAL_AI_MEMORY_PATH", value),
        None => std::env::remove_var("FORMAL_AI_MEMORY_PATH"),
    }

    assert_eq!(response.status_code, 200);
    let recorded = SyncStore::open_at(&path);
    let user_event = recorded
        .events()
        .iter()
        .find(|event| event.role.as_deref() == Some("user"))
        .expect("POSTed user prompt must be recorded");
    assert_eq!(user_event.content.as_deref(), Some("Hi"));
    let task_event = recorded
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("task"))
        .expect("solved exchange must be recorded as a task event");
    assert_eq!(task_event.inputs.as_deref(), Some("Hi"));
    assert_eq!(
        task_event.outputs.as_deref(),
        Some("Hi, how may I help you?")
    );
    assert!(
        task_event.evidence.contains(&user_event.id),
        "task must cite the recorded user prompt"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn sample_event(id: &str, content: &str) -> formal_ai::MemoryEvent {
    formal_ai::MemoryEvent {
        id: id.to_owned(),
        content: Some(content.to_owned()),
        ..formal_ai::MemoryEvent::default()
    }
}

fn with_recall_memory<T>(run: impl FnOnce() -> T) -> T {
    let _guard = memory_env_lock();
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-local-surface-memory-query-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("memory.lino");
    let memory = export_memory_links_notation(&[
        recall_event("a1", "user", "conv-a", "Rust Notes", "What is Rust?"),
        recall_event(
            "a2",
            "assistant",
            "conv-a",
            "Rust Notes",
            "Rust is a systems programming language.",
        ),
        recall_event(
            "b1",
            "user",
            "conv-b",
            "Wikipedia Notes",
            "What is Wikipedia?",
        ),
    ]);
    std::fs::write(&path, memory).expect("write memory");

    let previous = std::env::var_os("FORMAL_AI_MEMORY_PATH");
    std::env::set_var("FORMAL_AI_MEMORY_PATH", &path);
    let result = run();
    match previous {
        Some(value) => std::env::set_var("FORMAL_AI_MEMORY_PATH", value),
        None => std::env::remove_var("FORMAL_AI_MEMORY_PATH"),
    }
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn recall_event(
    id: &str,
    role: &str,
    conversation_id: &str,
    conversation_title: &str,
    content: &str,
) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(String::from("message")),
        role: Some(role.to_owned()),
        content: Some(content.to_owned()),
        conversation_id: Some(conversation_id.to_owned()),
        conversation_title: Some(conversation_title.to_owned()),
        ..MemoryEvent::default()
    }
}

fn memory_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}
