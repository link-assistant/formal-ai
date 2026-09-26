//! What a client asks before it asks anything (issue #1021).
//!
//! An agentic CLI does not open with its first turn. It opens with reachability
//! probes against the base URL it was handed, and a probe that comes back `404`
//! is indistinguishable, in a transcript, from a base URL that points nowhere.
//! The issue-#671 matrix asserts that no exchange in a leg fails, so every
//! probe a pinned client makes is part of the contract this server has to keep.
//!
//! `@anthropic-ai/claude-code` 2.1.238 added a second probe on top of the
//! `HEAD <base-url>` one it already made: a once-per-session connection warm-up
//! that sends `HEAD $ANTHROPIC_BASE_URL/api/hello` and discards the answer,
//! which against the base URL our wrapper writes arrives as
//! `/api/anthropic/api/hello`. The doubled `/api` belongs to neither side:
//! `https://api.anthropic.com/api/hello` is Anthropic's own endpoint and
//! answers `200 {"message": "hello"}` to `GET` and `200` with an empty body to
//! `HEAD`, so an Anthropic-compatible surface answers it too.

use formal_ai::handle_api_request;

/// Every base path this server publishes, in the form a client is handed it.
const BASE_PATHS: [&str; 7] = [
    "/",
    "/health",
    "/api/anthropic",
    "/api/openai",
    "/api/gemini",
    "/api/formal-ai",
    "/api/vertex",
];

#[test]
fn every_published_base_path_answers_a_reachability_probe() {
    for path in BASE_PATHS {
        let response = handle_api_request("HEAD", path, "");
        assert_eq!(
            response.status_code, 200,
            "HEAD {path} must report the base path as reachable"
        );
        assert!(
            response.body.is_empty(),
            "HEAD {path} must answer with an empty body, not {:?}",
            response.body
        );
    }
}

#[test]
fn the_anthropic_hello_probe_is_answered_under_the_base_path_a_client_is_given() {
    // Both spellings: the bare endpoint, and the one a client reaches by
    // appending `/api/hello` to the `/api/anthropic` base URL our wrapper
    // writes into `ANTHROPIC_BASE_URL`.
    for path in ["/api/hello", "/api/anthropic/api/hello"] {
        let head = handle_api_request("HEAD", path, "");
        assert_eq!(head.status_code, 200, "HEAD {path}");
        assert!(head.body.is_empty(), "HEAD {path} body: {:?}", head.body);

        let get = handle_api_request("GET", path, "");
        assert_eq!(get.status_code, 200, "GET {path}");
        let json: serde_json::Value =
            serde_json::from_str(&get.body).unwrap_or_else(|error| panic!("GET {path}: {error}"));
        // The payload is upstream's, verbatim -- a client that parses it gets
        // the same shape from us that it gets from `api.anthropic.com`.
        assert_eq!(json["message"], "hello", "GET {path} body: {}", get.body);
    }
}

/// The probe is a probe, not a route that swallows everything beneath it.
#[test]
fn the_hello_probe_does_not_answer_for_paths_it_does_not_own() {
    for path in ["/api/hello/world", "/api/anthropic/api/hello/world"] {
        assert_eq!(
            handle_api_request("GET", path, "").status_code,
            404,
            "GET {path} must not be absorbed by the reachability probe"
        );
    }
}
