//! One slow request must not stop the server answering others (issue #1106).
//!
//! The reported symptom was a permanent wedge: a chat completion never
//! returned, and afterwards even `GET /v1/models` — which had answered a
//! second earlier — stopped responding, while the process stayed alive.
//!
//! It was not a deadlock. Every chat completion opens the memory store and
//! hands its events to the solver, so its cost grows with accumulated history;
//! against an 11 MB store one `Hi` projects to about sixteen minutes. What
//! turned that slowness into an outage was the accept loop, which handled each
//! connection inline, so the request in flight blocked every other caller.
//!
//! This pins the part that must never regress: while one request is still
//! being answered, an unrelated request is served promptly.

use std::time::{Duration, Instant};

use super::http_server::{
    http_get_json, http_post_json_with_read_timeout, reserve_loopback_port,
    spawn_formal_ai_server_with_env,
};

const TOKEN: &str = "sk-local-agentic-tools";

#[test]
fn an_unrelated_request_is_answered_while_a_completion_is_in_flight() {
    // A completion against an empty store is fast, and a fast request cannot
    // demonstrate the outage. The reported failure needs an accumulated store:
    // the per-request cost grows with it. 400 recorded exchanges make one
    // completion take seconds -- enough for the unrelated request below to be
    // blocked behind it before the fix -- while still finishing inside the
    // test's own budget. (2 000 takes over a minute, which is the defect this
    // number is chosen to stay just clear of.)
    let port = reserve_loopback_port();
    let store = std::env::temp_dir().join(format!(
        "formal-ai-issue-1106-{}-{port}.lino",
        std::process::id()
    ));
    let mut document = String::from("demo_memory\n");
    for index in 0..400 {
        document.push_str("  event \"chat_user_");
        document.push_str(&format!("{index:08x}"));
        document.push_str("\"\n    kind \"message\"\n    role \"user\"\n    content \"recorded ");
        document.push_str(&format!("exchange {index} about solving something"));
        document.push_str("\"\n    writeCount \"1\"\n");
    }
    std::fs::write(&store, &document).expect("seed the memory store");
    let store_path = store.to_string_lossy().into_owned();

    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_MEMORY_PATH", store_path.as_str())]);

    let completion = std::thread::spawn(move || {
        http_post_json_with_read_timeout(
            port,
            "/api/openai/v1/chat/completions",
            Some(TOKEN),
            &serde_json::json!({
                "model": "formal-ai",
                "messages": [{"role": "user", "content": "Hi"}],
            }),
            Duration::from_secs(120),
        )
    });

    // Give the completion time to be accepted and start solving, then ask for
    // something unrelated. Before the fix this blocked until the completion
    // finished -- which, on a large store, is never in any useful sense.
    std::thread::sleep(Duration::from_millis(300));
    let started = Instant::now();
    let models = http_get_json(port, "/api/openai/v1/models", Some(TOKEN));
    let waited = started.elapsed();

    assert_eq!(
        models["object"], "list",
        "the models endpoint must answer while a completion is still being solved"
    );
    assert!(
        waited < Duration::from_secs(20),
        "an unrelated request waited {waited:?} behind a completion; a slow answer must not \
         become an outage for every other caller (issue #1106)"
    );

    let completion = completion.join().expect("completion thread");
    assert_eq!(
        completion["object"], "chat.completion",
        "the completion itself must still be answered"
    );
    let _ = std::fs::remove_file(&store);
}
