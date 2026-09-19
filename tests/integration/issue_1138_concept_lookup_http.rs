//! Issue #1138, plan 01 L15 — the concept lookup through a real server process.
//!
//! `tests/unit/issue_1138_concept_lookup.rs` proves the retrieval contract on
//! the native path and `tests/web/issue-1138-concept-lookup.test.mjs` proves the
//! browser worker resolves the same senses. This file is the third surface: a
//! real `formal-ai serve` process, asked in natural language over HTTP,
//! answering from the committed captures under `tests/fixtures/issue-1138-b1/`
//! with no network at all.
//!
//! Each test asks a *different* question (CONTRIBUTING rule 4): a word the
//! dictionary defines, the same question with the dictionary opted out, and a
//! word no trusted source defines.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::http_server::{http_post_json, reserve_loopback_port, spawn_formal_ai_server_with_env};

const TOKEN: Option<&str> = Some("sk-local-agentic-tools");

/// The committed capture tree the server replays.
const FIXTURE_DIR: &str = "tests/fixtures/issue-1138-b1";

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
    fn checkout(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after the unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("formal-ai-issue-1138-http-{name}-{nanos}"));
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

#[test]
fn chat_completions_resolves_an_unknown_word_from_the_committed_captures() {
    let cache = CaptureCache::checkout("resolve");
    let port = reserve_loopback_port();
    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_SOURCE_CACHE_DIR", cache.as_str())]);

    let answer = ask(port, "What does isogram mean?");

    // Which source answers is decided by registry order, not by this test: the
    // Free Dictionary API endpoint the wiktionary row leads with has no entry
    // for the held-out word (plan 01 L6 records the measurement), so the first
    // declared source that actually answers is wordnet, and the answer carries
    // wordnet's exact page, digest and license. Changing the registry order
    // changes this expectation with it.
    assert!(
        answer.contains("https://en-word.net/api/lemma/isogram"),
        "the answer cites the exact page the meaning was read from: {answer}"
    );
    assert!(
        answer.contains("sha256 "),
        "the retrieved bytes are fingerprinted in the answer: {answer}"
    );
    assert!(
        answer.contains("numerical value"),
        "the reported meaning is the retrieved gloss: {answer}"
    );
}

#[test]
fn chat_completions_honours_a_dictionary_opt_out() {
    let cache = CaptureCache::checkout("optout");
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_with_env(
        port,
        &[
            ("FORMAL_AI_SOURCE_CACHE_DIR", cache.as_str()),
            ("FORMAL_AI_DISABLED_SERVICES", "externalServiceWiktionary"),
        ],
    );

    let answer = ask(port, "Explain the word lipogram.");

    assert!(
        !answer.contains("https://en.wiktionary.org/"),
        "an opted-out dictionary contributes nothing over the wire either: {answer}"
    );
    assert!(
        answer.contains("wiktionary"),
        "the disabled service is still reported as disabled: {answer}"
    );
}

#[test]
fn chat_completions_reports_an_unresolved_word_instead_of_inventing_a_meaning() {
    let cache = CaptureCache::checkout("unresolved");
    let port = reserve_loopback_port();
    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_SOURCE_CACHE_DIR", cache.as_str())]);

    let answer = ask(port, "What does blorptide mean?");

    assert!(
        !answer.contains("blorptide is a"),
        "a word no source defines never acquires a meaning: {answer}"
    );
    assert!(
        answer.contains("wiktionary") && answer.contains("wikipedia"),
        "the honest refusal names every source it consulted: {answer}"
    );
}
