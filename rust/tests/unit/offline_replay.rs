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
    in_child_with_cache(test, |_| {})
}

/// Seed an exclusive cache in the parent before replaying an offline test.
pub fn in_child_with_cache(test: &str, seed: impl FnOnce(&std::path::Path)) -> bool {
    replay_in_child(test, seed, &[("FORMAL_AI_OFFLINE", "1")])
}

/// Replay a test whose contract was recorded with read-only source retrieval
/// denied but the solver itself *not* in offline mode: source clients see an
/// empty, exclusive cache and report `offline_cache_miss`, while routing keeps
/// the default `SolverConfig::offline == false`. Read access became default-on
/// in c0e480a36 (#1582), so without this a CI runner's real network outcome
/// (`unreachable`, or a persisted `unreachable_cached`) leaks into the answer.
pub fn in_network_denied_child(test: &str) -> bool {
    replay_in_child(test, |_| {}, &[("FORMAL_AI_NETWORK", "denied")])
}

/// Replay a network-denied test against an operator-reviewed approved-lessons
/// document. Since 289c28b88 the live ledger is empty unless
/// `FORMAL_AI_APPROVED_LESSONS` names such a document, and the ledger is a
/// process-wide `OnceLock`, so the variable must be set in a child process.
pub fn in_child_with_approved_lessons(test: &str, lessons: &str) -> bool {
    replay_in_child(
        test,
        |_| {},
        &[
            ("FORMAL_AI_NETWORK", "denied"),
            ("FORMAL_AI_APPROVED_LESSONS", lessons),
        ],
    )
}

fn replay_in_child(
    test: &str,
    seed: impl FnOnce(&std::path::Path),
    environment: &[(&str, &str)],
) -> bool {
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
    seed(&cache);
    let result = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env(MARKER, test)
        .envs(environment.iter().copied())
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
