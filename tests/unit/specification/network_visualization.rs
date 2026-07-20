//! Network-visualization tests.
//!
//! `VISION.md` and `REQUIREMENTS.md` call for an optional graph view of
//! the link network displayed alongside the chat — so the user can see how a
//! reply was derived. The visualization is *optional*: it must never be
//! required for the chat surface to work.

use formal_ai::{handle_api_request, FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: the implementation already exposes Links Notation traces.
// ---------------------------------------------------------------------------

#[test]
fn answers_already_carry_links_notation_for_visualization() {
    let response = answer("Hi");
    assert!(!response.links_notation.is_empty());
}

// ---------------------------------------------------------------------------
// Issue #258 graduated expectations.
// ---------------------------------------------------------------------------

#[test]
fn graph_endpoint_returns_nodes_and_edges() {
    let response = handle_api_request("GET", "/v1/graph", "");
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert!(json["nodes"].is_array());
    assert!(json["edges"].is_array());
}

#[test]
fn graph_nodes_carry_minimum_metadata() {
    let response = handle_api_request("GET", "/v1/graph", "");
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let node = &json["nodes"][0];
    assert!(node["id"].is_string());
    assert!(node["label"].is_string());
    assert!(node["links_notation"].is_string());
}

#[test]
fn graph_edges_describe_doublet_links() {
    let response = handle_api_request("GET", "/v1/graph", "");
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let edge = &json["edges"][0];
    assert!(edge["from"].is_string());
    assert!(edge["to"].is_string());
    assert!(edge["role"].is_string());
}

#[test]
fn graph_endpoint_filters_by_trace_id() {
    let response = handle_api_request("GET", "/v1/graph?trace=answer_greeting_hi", "");
    assert_eq!(response.status_code, 200);
}

#[test]
fn graph_endpoint_returns_404_for_unknown_trace() {
    let response = handle_api_request("GET", "/v1/graph?trace=does_not_exist", "");
    assert_eq!(response.status_code, 404);
}

#[test]
fn web_demo_chat_works_even_when_graph_is_disabled() {
    std::env::set_var("FORMAL_AI_DISABLE_GRAPH", "1");
    let response = answer("Hi");
    std::env::remove_var("FORMAL_AI_DISABLE_GRAPH");
    assert_eq!(response.intent, "greeting");
    assert!(!response.answer.is_empty());
}

#[test]
fn graph_endpoint_supports_dot_export() {
    let response = handle_api_request("GET", "/v1/graph?format=dot", "");
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("digraph"));
}

// ---------------------------------------------------------------------------
// Issue #664: the associative vocabulary is a *links network*, not a graph.
// `/v1/network` is the canonical endpoint; `/v1/graph` stays a deprecated alias
// so existing desktop / VS Code / e2e clients keep working, returning the exact
// same payload but flagged deprecated in its response metadata.
// ---------------------------------------------------------------------------

#[test]
fn network_endpoint_returns_nodes_and_edges() {
    let response = handle_api_request("GET", "/v1/network", "");
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert!(json["nodes"].is_array());
    assert!(json["edges"].is_array());
}

#[test]
fn network_endpoint_is_the_canonical_route_and_not_deprecated() {
    let response = handle_api_request("GET", "/v1/network", "");
    assert!(
        !response.deprecated,
        "the canonical /v1/network endpoint must not be flagged deprecated",
    );
}

#[test]
fn graph_alias_returns_identical_payload_to_network() {
    let network = handle_api_request("GET", "/v1/network", "");
    let graph = handle_api_request("GET", "/v1/graph", "");
    assert_eq!(network.status_code, graph.status_code);
    assert_eq!(network.content_type, graph.content_type);
    assert_eq!(
        network.body, graph.body,
        "the deprecated /v1/graph alias must serve a byte-identical payload",
    );
}

#[test]
fn graph_alias_is_flagged_deprecated_while_network_is_not() {
    let network = handle_api_request("GET", "/v1/network", "");
    let graph = handle_api_request("GET", "/v1/graph", "");
    assert!(!network.deprecated);
    assert!(
        graph.deprecated,
        "the legacy /v1/graph alias must carry a deprecation marker in its metadata",
    );
}

#[test]
fn graph_alias_payload_matches_network_for_dot_and_trace_variants() {
    for query in ["", "?format=dot", "?trace=answer_greeting_hi"] {
        let network = handle_api_request("GET", &format!("/v1/network{query}"), "");
        let graph = handle_api_request("GET", &format!("/v1/graph{query}"), "");
        assert_eq!(network.body, graph.body, "payload diverged for {query:?}");
        assert!(!network.deprecated);
        assert!(
            graph.deprecated,
            "alias not flagged deprecated for {query:?}"
        );
    }
}
