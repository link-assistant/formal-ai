//! Isolate historical offline contracts from the developer's live source cache.
//!
//! Discovery has its own source-transport fixtures. A frozen catalog replay
//! cannot silently depend on whether today's network or a prior run supplied a
//! better attributed example. Environment overrides belong to a child process,
//! never to concurrently executing test threads.
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

/// Return true in the parent after validating the isolated replay; the child
/// returns false so the original assertions execute unchanged.
pub fn in_child(test: &str) -> bool {
    const MARKER: &str = "FORMAL_AI_ISOLATED_OFFLINE_REPLAY";
    if std::env::var(MARKER).as_deref() == Ok(test) {
        return false;
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cache = std::env::temp_dir().join(format!(
        "formal-ai-offline-replay-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&cache).expect("create an exclusive empty source cache");
    let result = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env(MARKER, test)
        .env("FORMAL_AI_OFFLINE", "1")
        .env("FORMAL_AI_SOURCE_CACHE_DIR", &cache)
        .output();
    fs::remove_dir_all(&cache).expect("remove only this replay's temporary cache");
    let result = result.expect("start the offline replay child");
    assert!(
        result.status.success(),
        "offline replay {test} failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    true
}
