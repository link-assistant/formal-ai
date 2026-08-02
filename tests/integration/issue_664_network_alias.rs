//! Issue #664: the associative vocabulary is a *links network*, not a graph.
//!
//! `/v1/network` is the canonical endpoint for the links-network view. The
//! historical `/v1/graph` route stays a *deprecated alias* so existing desktop /
//! VS Code / e2e clients keep working — it returns the exact same payload but
//! advertises its deprecation over the wire (an HTTP `deprecation` header plus a
//! successor `link` pointing at `/v1/network`). These tests exercise the real
//! loopback HTTP server so the wire-layer metadata is asserted end-to-end.

use crate::http_server::{http_request, reserve_loopback_port, spawn_formal_ai_server};

const TOKEN: &str = "sk-local-agentic-tools";

#[test]
fn network_endpoint_and_graph_alias_serve_identical_payloads() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);

    let network = http_request("GET", port, "/v1/network", Some(TOKEN), None)
        .expect("GET /v1/network should complete");
    let graph = http_request("GET", port, "/v1/graph", Some(TOKEN), None)
        .expect("GET /v1/graph should complete");

    assert_eq!(network.status_code, 200);
    assert_eq!(graph.status_code, 200);
    assert_eq!(
        network.body, graph.body,
        "the deprecated /v1/graph alias must serve a byte-identical payload",
    );
}

#[test]
fn graph_alias_advertises_deprecation_over_the_wire() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);

    let graph = http_request("GET", port, "/v1/graph", Some(TOKEN), None)
        .expect("GET /v1/graph should complete");

    assert_eq!(
        graph.header("deprecation"),
        Some("true"),
        "the legacy /v1/graph alias must carry a deprecation marker",
    );
    let successor = graph
        .header("link")
        .expect("the deprecated alias must point at its successor version");
    assert!(
        successor.contains("/v1/network") && successor.contains("successor-version"),
        "successor link should reference /v1/network, got {successor:?}",
    );
}

#[test]
fn canonical_network_endpoint_is_not_marked_deprecated() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);

    let network = http_request("GET", port, "/v1/network", Some(TOKEN), None)
        .expect("GET /v1/network should complete");

    assert_eq!(
        network.header("deprecation"),
        None,
        "the canonical /v1/network endpoint must not advertise deprecation",
    );
}
