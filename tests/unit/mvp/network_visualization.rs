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
// Active expectations: the prototype already exposes Links Notation traces.
// ---------------------------------------------------------------------------

#[test]
fn answers_already_carry_links_notation_for_visualization() {
    let response = answer("Hi");
    assert!(!response.links_notation.is_empty());
}

// ---------------------------------------------------------------------------
// MVP expectations.
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
#[ignore = "MVP-target: web demo must render the graph alongside chat but never block chat replies"]
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
