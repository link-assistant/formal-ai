//! Issue #991: the multi-source how-to synthesis must reach a caller over the
//! wire, not only inside the library.
//!
//! `tests/unit/issue_991_how_to_synthesis.rs` proves the synthesis contract on
//! the native path and `tests/web/issue-991-how-to-synthesis.test.mjs` proves
//! the browser worker executes the same contract. This file is the third
//! surface: a real `formal-ai serve` process, asked in natural language over
//! HTTP, answering from the same committed capture tree with no network at all.
//!
//! Determinism comes from the environment the server is spawned with —
//! `FORMAL_AI_SOURCE_CACHE_DIR` pointed at a copy of `tests/fixtures/issue-991`
//! and no `FORMAL_AI_LIVE_FETCH`, so the transport is disabled and every byte
//! the answer cites came from the committed captures. The tree is copied to a
//! temporary directory because the server writes its service-accessibility
//! record back into the cache directory; the committed fixture stays untouched.
//!
//! Each test asks about a *different* task (CONTRIBUTING rule 4) — a documented
//! one, one asked with a service opted out, and one no service documents — so a
//! passing run shows the route is general rather than pinned to one request.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::http_server::{http_post_json, reserve_loopback_port, spawn_formal_ai_server_with_env};

const TOKEN: Option<&str> = Some("sk-local-agentic-tools");

/// The committed capture tree the server replays.
const FIXTURE_DIR: &str = "tests/fixtures/issue-991";

/// A private copy of the capture tree, removed when the test finishes.
struct CaptureCache {
    path: PathBuf,
}

impl Drop for CaptureCache {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

impl CaptureCache {
    /// Copy the committed captures somewhere the server may also write to.
    fn checkout(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after the unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("formal-ai-issue-991-http-{name}-{nanos}"));
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR),
            &path,
        );
        Self { path }
    }

    fn as_str(&self) -> &str {
        self.path.to_str().expect("temporary path is valid UTF-8")
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create the capture cache copy");
    for entry in fs::read_dir(from).expect("read the committed capture tree") {
        let entry = entry.expect("read a capture tree entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy a capture");
        }
    }
}

/// Ask the running server a question the way an OpenAI-compatible client does.
fn ask(port: u16, question: &str) -> String {
    let response = http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "stream": false,
            "messages": [{ "role": "user", "content": question }]
        }),
    );
    response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

/// The whole contract on one request: registry-selected sources, steps that
/// carry their exact provenance, and the declared bounds, all offline.
#[test]
fn chat_completions_answers_a_how_to_request_from_the_committed_captures() {
    let cache = CaptureCache::checkout("guide");
    let port = reserve_loopback_port();
    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_SOURCE_CACHE_DIR", cache.as_str())]);

    let answer = ask(port, "how to make pancakes?");

    assert!(
        answer.contains("## How to make pancakes"),
        "the answer should be the synthesised guide: {answer}"
    );
    assert!(
        answer.contains("1. ") && answer.contains("2. "),
        "the guide should assert at least the minimum number of steps: {answer}"
    );
    assert!(
        answer.contains("sha256 "),
        "every step should cite the digest of the bytes it came from: {answer}"
    );
    assert!(
        answer.contains("https://www.wikihow.com/"),
        "the guide should cite the exact source URL it captured: {answer}"
    );
    assert!(
        answer.contains("Bounds: max_depth=2 max_pages_per_service=4 max_services=4 max_steps=12"),
        "the guide should state the bounds it ran under: {answer}"
    );
}

/// A settings opt-out is authoritative over the wire too: the disabled service
/// is reported as disabled, contributes nothing, and the enabled services still
/// answer.
#[test]
fn chat_completions_honours_a_service_opt_out() {
    let cache = CaptureCache::checkout("optout");
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_with_env(
        port,
        &[
            ("FORMAL_AI_SOURCE_CACHE_DIR", cache.as_str()),
            (
                "FORMAL_AI_DISABLED_SERVICES",
                "externalServiceStackExchange",
            ),
        ],
    );

    let answer = ask(port, "how to make pancakes?");

    assert!(
        answer.contains(
            "- stackexchange — disabled (0 page(s), 0 step(s)): externalServiceStackExchange"
        ),
        "the opted-out service should be reported as disabled, naming the setting: {answer}"
    );
    assert!(
        !answer.contains("api.stackexchange.com"),
        "an opted-out service must never be contacted, cached or not: {answer}"
    );
    assert!(
        answer.contains("## How to make pancakes") && answer.contains("sha256 "),
        "the services that stayed enabled must still answer: {answer}"
    );
}

/// No service documents the task, so the server reports the shortfall and falls
/// back to the discovery plan instead of inventing a procedure.
#[test]
fn chat_completions_reports_insufficient_evidence_instead_of_inventing_steps() {
    let cache = CaptureCache::checkout("insufficient");
    let port = reserve_loopback_port();
    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_SOURCE_CACHE_DIR", cache.as_str())]);

    let answer = ask(port, "how to build a nonexistent quantum flux capacitor?");

    assert!(
        !answer.contains("sha256 "),
        "no step may be asserted when no source documents the task: {answer}"
    );
    assert!(
        !answer.contains("## How to build a nonexistent quantum flux capacitor"),
        "an unsupported guide must not be rendered as if it had steps: {answer}"
    );
    assert!(
        answer.contains("Procedural discovery plan"),
        "the truthful fallback must still describe what was checked: {answer}"
    );
}
