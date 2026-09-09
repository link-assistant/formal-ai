//! Recording an exchange must not rebuild the whole projection (issue #1106).
//!
//! The wedge in `issue_1106_concurrent_requests` was the accept loop; this is
//! the cost that loop was blocking on. Every chat completion persists the
//! memory, and persisting replaced the native link-cli graph in full: a
//! 400-event store rebuilt all 400 events to append three, which measured 51.8
//! seconds of a 53-second request and grew with the square of the history.
//!
//! The projection now records how many events it holds, so a completion appends
//! only what is new. The saved graph is identical either way -- the addresses
//! link-cli handed out are stored beside the database rather than recomputed --
//! so what needs pinning is that the second completion does not pay for the
//! first one's history.

use std::time::{Duration, Instant};

use super::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_with_env,
};

const TOKEN: &str = "sk-local-agentic-tools";

/// The store size is the same 400 events the sibling test seeds, for the same
/// reason: enough accumulated history that a full rebuild is unmistakably slow.
const SEEDED_EVENTS: usize = 400;

/// Generous next to the measured 0.35 s, and far under the 46 s a rebuild costs.
/// The gap between those two numbers is wide enough that this does not depend on
/// how fast the machine running it happens to be.
const APPEND_BUDGET: Duration = Duration::from_secs(20);

#[test]
fn a_second_completion_does_not_rebuild_the_whole_projection() {
    let port = reserve_loopback_port();
    let store = std::env::temp_dir().join(format!(
        "formal-ai-issue-1106-reuse-{}-{port}.lino",
        std::process::id()
    ));
    let mut document = String::from("demo_memory\n");
    for index in 0..SEEDED_EVENTS {
        use std::fmt::Write as _;
        let _ = write!(
            document,
            "  event \"chat_user_{index:08x}\"\n    kind \"message\"\n    role \"user\"\n    \
             content \"recorded exchange {index} about solving something\"\n    \
             writeCount \"1\"\n"
        );
    }
    std::fs::write(&store, &document).expect("seed the memory store");
    let store_path = store.to_string_lossy().into_owned();

    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_MEMORY_PATH", store_path.as_str())]);

    // The first completion may legitimately rebuild: a store opened for the
    // first time has no record of what its projection contains, and rebuilding
    // is what makes `.lino` a deterministic recovery source. Its duration is
    // deliberately not asserted on.
    let first = complete(port, "first");
    assert_eq!(first["object"], "chat.completion");

    // The second must not. By now the projection's contents are known, so this
    // appends a handful of events instead of replaying the history.
    let started = Instant::now();
    let second = complete(port, "second");
    let elapsed = started.elapsed();

    assert_eq!(second["object"], "chat.completion");
    assert!(
        elapsed < APPEND_BUDGET,
        "a completion against an already-projected store must append rather than rebuild it, \
         but this one took {elapsed:?}"
    );
}

fn complete(port: u16, prompt: &str) -> serde_json::Value {
    http_post_json_with_read_timeout(
        port,
        "/api/openai/v1/chat/completions",
        Some(TOKEN),
        &serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": prompt}],
        }),
        // The first completion against a fresh store rebuilds the projection
        // once, which is slow by design and is not what this test measures.
        // Sibling cases seed stores of their own concurrently, so this budget
        // is set well clear of that rather than close to it.
        Duration::from_secs(600),
    )
}

/// The saved address map is line-based and escapes what would break that, so a
/// recorded exchange whose text contains a newline or the two characters `\n`
/// must still be appended onto rather than silently rebuilt -- and must survive
/// the round trip. A decoder that did not also unescape the escape character
/// would fold these two distinct names together.
#[test]
fn a_prefix_containing_escaped_characters_is_restored_exactly() {
    let port = reserve_loopback_port();
    let store = std::env::temp_dir().join(format!(
        "formal-ai-issue-1106-escape-{}-{port}.lino",
        std::process::id()
    ));
    // Written through the `.lino` quoting these characters already require, so
    // the store round-trips them before the projection ever sees them.
    let document = String::from(
        "demo_memory\n\
         \x20 event \"chat_user_00000001\"\n\
         \x20   kind \"message\"\n\
         \x20   role \"user\"\n\
         \x20   content \"a literal backslash-n \\\\n and a tab-free line\"\n\
         \x20   writeCount \"1\"\n\
         \x20 event \"chat_user_00000002\"\n\
         \x20   kind \"message\"\n\
         \x20   role \"user\"\n\
         \x20   content \"a plain line with no escapes at all\"\n\
         \x20   writeCount \"1\"\n",
    );
    std::fs::write(&store, &document).expect("seed the memory store");
    let store_path = store.to_string_lossy().into_owned();

    let _server =
        spawn_formal_ai_server_with_env(port, &[("FORMAL_AI_MEMORY_PATH", store_path.as_str())]);

    assert_eq!(complete(port, "first")["object"], "chat.completion");
    assert_eq!(complete(port, "second")["object"], "chat.completion");

    // Both events must still be there and still distinct. Losing one, or
    // merging them, is what a mismatched escape pair would cause.
    let persisted = std::fs::read_to_string(&store).expect("read the memory store");
    assert!(
        persisted.contains("a literal backslash-n"),
        "the escaped event must survive the projection round trip: {persisted}"
    );
    assert!(
        persisted.contains("a plain line with no escapes at all"),
        "the unescaped event must survive alongside it: {persisted}"
    );
}
