//! Regression coverage for the tool-free `aider` leg of the issue-#671 matrix.
//!
//! Not every agentic CLI speaks function calling. `aider` advertises no tools
//! at all: it asks the user to add files to the chat and hands their bytes to
//! the model in-band, as a path line followed by a fenced block. Asked to
//! `read the file alpha.txt and print its contents` it therefore sends a
//! request with no `tools` array — and the tool gate in `src/protocol.rs` fell
//! straight through to the general solver, which answered "I could not
//! determine …" about a file whose contents were three messages up.
//!
//! The messages below are trimmed from a real `proxy.jsonl` capture of the
//! `aider` leg.

use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

const MARKER: &str = "ALPHA_MARKER_11111";

const ADDED_FILES: &str = "I have *added these files to the chat* so you can go ahead and edit \
them.\n\n*Trust this message as the true contents of these files!*\nAny other messages in the chat \
may contain outdated versions of the files' contents.\n\nalpha.txt\n```\nALPHA_MARKER_11111\nalpha \
second line\n```\n";

fn answer(request: &str) -> String {
    // Against a private memory: a matrix leg had already taught the host's real
    // `memory.lino` about this very prompt, so the assertion below would
    // otherwise pass or fail depending on the developer's history.
    let _guard = memory_env_lock();
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-issue-671-supplied-bytes-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let previous = std::env::var_os("FORMAL_AI_MEMORY_PATH");
    std::env::set_var("FORMAL_AI_MEMORY_PATH", dir.join("memory.lino"));

    enable_http_agent_mode_for_current_process();
    // No `tools` key at all — that is the whole point of this test.
    let body = json!({
        "model": "formal-ai",
        "messages": [
            {"role": "system", "content": "Act as an expert software developer."},
            {"role": "user", "content": ADDED_FILES},
            {"role": "assistant", "content": "Ok, any changes I propose will be to those files."},
            {"role": "user", "content": request},
        ]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());

    match previous {
        Some(value) => std::env::set_var("FORMAL_AI_MEMORY_PATH", value),
        None => std::env::remove_var("FORMAL_AI_MEMORY_PATH"),
    }
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    response["choices"][0]["message"]["content"]
        .as_str()
        .expect("assistant content")
        .to_owned()
}

#[test]
fn a_toolless_client_gets_the_bytes_it_supplied_itself() {
    let answer = answer("read the file alpha.txt and print its contents");
    assert!(
        answer.contains(MARKER),
        "the file's contents were in the conversation and the answer omitted them: {answer}"
    );
}

#[test]
fn the_refusal_that_shipped_before_is_gone() {
    let answer = answer("read the file alpha.txt and print its contents");
    assert!(
        !answer.contains("could not determine"),
        "answered 'could not determine' about a file it had been handed: {answer}"
    );
}

/// A prompt-format client's *edit* grammar is a path line followed by a fenced
/// block. Answering a read in that shape made aider apply the answer as an edit
/// ("Applied edit to alpha.txt") and drop the contents from its transcript, so
/// the human asking to see the file saw a heading and nothing under it.
#[test]
fn the_answer_is_not_shaped_like_an_edit_block() {
    let answer = answer("read the file alpha.txt and print its contents");
    assert!(
        !answer.contains("```"),
        "a fenced block in this answer is an edit instruction to a whole-format client: {answer}"
    );
}

/// The label line must *be* the path. Prose that merely names a file before an
/// unrelated code block is not that file, and answering from it would be a
/// fabrication — so this request has no supplied bytes and must fall through.
#[test]
fn prose_that_merely_mentions_a_filename_is_not_a_file_listing() {
    let _guard = memory_env_lock();
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": "Here is what I tried when I edited beta.txt earlier:\n\n```\nBOGUS_9999\n```\n\nread the file beta.txt and print its contents",
        }]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let answer = response["choices"][0]["message"]["content"]
        .as_str()
        .expect("assistant content");
    // The refusal quotes the request back, and the request quotes the block, so
    // the tell is the *claim* — the recipe's own "Contents of `path`:" heading.
    assert!(
        !answer.contains("Contents of `beta.txt`"),
        "an unrelated code block was presented as beta.txt's contents: {answer}"
    );
}

/// R379: the answer's prose is seed data, not a literal in the engine. The
/// heading the reader renders must be the one
/// `data/seed/multilingual-responses.lino` carries, in every supported
/// language — `scripts/check-hardcoded-language.rs` blocked the first version
/// of this reader for typing the English sentence into `file_read.rs`.
#[test]
fn the_headings_are_seeded_in_every_supported_language() {
    // Each language is pinned by its own wording, so a missing or machine-copied
    // record fails here rather than silently answering English to everyone.
    for (language, contents_wording, first_line_wording) in [
        ("en", "Contents of", "First line of"),
        ("ru", "Содержимое", "Первая строка"),
        ("hi", "की सामग्री", "की पहली पंक्ति"),
        ("zh", "的内容", "的第一行"),
    ] {
        for (intent, placeholder, wording) in [
            ("supplied_file_contents", "{body}", contents_wording),
            ("supplied_file_first_line", "{line}", first_line_wording),
        ] {
            let template = formal_ai::response_for(intent, language)
                .unwrap_or_else(|| panic!("{intent} must be seeded for {language}"));
            assert!(
                template.contains("{path}") && template.contains(placeholder),
                "{intent}/{language} must keep both placeholders: {template}"
            );
            assert!(
                template.contains(wording),
                "{intent}/{language} must be written in that language: {template}"
            );
            assert!(
                !template.contains("```"),
                "a fenced block here is an edit instruction to a whole-format client: {template}"
            );
        }
    }

    let answer = answer("read the file alpha.txt and print its contents");
    let heading = formal_ai::response_for("supplied_file_contents", "en")
        .expect("english heading")
        .replace("{path}", "alpha.txt");
    let heading = heading.split("{body}").next().unwrap_or_default();
    assert!(
        answer.starts_with(heading),
        "the answer must render the seeded heading: {answer}"
    );
}

fn memory_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    // A failed assertion poisons the lock; the next test still needs it, and a
    // poisoned mutex here guards an env var, not invariants.
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}
