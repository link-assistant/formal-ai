//! Regression coverage for the `aider` leg of the issue-#671 agentic matrix.
//!
//! Aider appends a reminder to the *user* turn, repeated verbatim from its own
//! system prompt: the "*file listing* format" block, complete with the example
//! `// entire file content ...`. Asked to `read the file alpha.txt and print
//! its contents`, the server answered about the example instead —
//! `// \n// // entire file content ...\n// // ... goes in between` — because
//! the appended block, not the request, drove the plan.
//!
//! Qwen Code marks the same kind of block with `<system-reminder>`, which the
//! server already strips. Aider marks it with nothing, so the tell is the
//! duplication itself: text the client already said as the system prompt is the
//! client talking, not the user.
//!
//! Both strings below are trimmed from a real `proxy.jsonl` capture of the
//! `aider` leg.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{Value, json};

const REMINDER: &str = "To suggest changes to a file you MUST return the entire content of the \
updated file.\nYou MUST use this *file listing* format:\n\npath/to/filename.js\n```\n// entire \
file content ...\n// ... goes in between\n```\n\nEvery *file listing* MUST use this format:\n- \
First line: the filename with any originally provided path; no extra markup, punctuation, \
comments, etc. JUST the filename with path.";

const SYSTEM: &str = "Act as an expert software developer.\nTake requests for changes to the \
supplied code.\nIf the request is ambiguous, ask questions.\n\nTo suggest changes to a file you \
MUST return the entire content of the updated file.\nYou MUST use this *file listing* format:\n\n\
path/to/filename.js\n```\n// entire file content ...\n// ... goes in between\n```\n\nEvery *file \
listing* MUST use this format:\n- First line: the filename with any originally provided path; no \
extra markup, punctuation, comments, etc. JUST the filename with path.";

fn answer(request: &str) -> String {
    let response = with_private_memory(|| {
        enable_http_agent_mode_for_current_process();
        let body = json!({
            "model": "formal-ai",
            "messages": [
                {"role": "system", "content": SYSTEM},
                {"role": "user", "content": format!("{request}\n\n{REMINDER}")},
            ]
        });
        handle_api_request("POST", "/v1/chat/completions", &body.to_string())
    });

    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    response["choices"][0]["message"]["content"]
        .as_str()
        .expect("assistant content")
        .to_owned()
}

/// Run `body` against a private, empty memory. The server learns from what it
/// serves, and the real `aider` matrix leg had already written this very
/// reminder into `$HOME/.formal-ai/memory.lino` as a standing requirement — so
/// the assertions below passed or failed depending on whether the developer had
/// ever run the leg. A regression test must not read the host.
///
/// Edition 2024 made `std::env::set_var` unsafe, and rightly: `cargo test` runs
/// these on threads that share one environment. This crate forbids unsafe code,
/// so the override is scoped by `temp-env`, whose reentrant lock is held for the
/// closure — which is what now serialises these tests, in place of the mutex
/// this file used to hold by hand.
fn with_private_memory<R>(body: impl FnOnce() -> R) -> R {
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-issue-671-aider-echo-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let result = temp_env::with_var("FORMAL_AI_MEMORY_PATH", Some(dir.join("memory.lino")), body);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

#[test]
fn an_appended_system_echo_does_not_become_the_request() {
    let answer = answer("read the file alpha.txt and print its contents");
    assert!(
        !answer.contains("entire file content"),
        "the client's own format reminder was answered as the task: {answer}"
    );
    assert!(
        !answer.contains("goes in between"),
        "the client's own format reminder was answered as the task: {answer}"
    );
}

#[test]
fn the_request_itself_still_reaches_the_planner() {
    let answer = answer("read the file alpha.txt and print its contents");
    assert!(
        answer.to_lowercase().contains("alpha.txt"),
        "the request named alpha.txt and the answer never mentioned it: {answer}"
    );
}

/// The stripping is line-aligned and verbatim, so a user who quotes a phrase
/// from the system prompt mid-sentence keeps their whole request.
#[test]
fn a_request_that_merely_quotes_the_system_prompt_is_untouched() {
    let answer = answer("explain what you mean by an expert software developer");
    assert!(
        !answer.trim().is_empty(),
        "a request quoting the system prompt was stripped to nothing"
    );
}
