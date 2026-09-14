//! Where a test binary keeps its memory store.
//!
//! With no `FORMAL_AI_MEMORY_PATH`, `shared_memory_path()` used to resolve
//! `$HOME/.formal-ai/memory.lino` even under `cargo test`. Every test in a
//! binary then shared one store, the parallel rebuilds of `<memory>.seed.links`
//! in `seed_links::mirror()` interleaved, the size fields of the size-balanced
//! tree stopped agreeing, and `fix_size` (`left + right + 1`) overflowed inside
//! `platform-trees` -- aborting the process during unwinding and blaming
//! whichever test happened to be running, which then passed in isolation.
//!
//! It also left a 64 MB `memory.links` in the home directory of the machine
//! that ran the suite.

use std::path::{Path, PathBuf};

#[test]
fn both_test_layouts_are_recognised() {
    for test_binary in [
        // A plain `cargo test`.
        "/repo/target/debug/deps/unit-70a1c6defe99c803",
        "/repo/target/release/deps/integration-abc123",
        // The prebuilt binaries issue #1055 introduced, run by
        // `scripts/run-prebuilt-tests.sh`. This is the exact path that failed
        // in CI when the check looked only for `target/*/deps`.
        "/home/runner/work/formal-ai/formal-ai/dist/tests/unit",
        "/home/runner/work/formal-ai/formal-ai/dist/tests/integration",
        "/home/runner/work/formal-ai/formal-ai/dist/tests/source",
    ] {
        assert!(
            formal_ai::is_test_executable(Path::new(test_binary)),
            "{test_binary} was not recognised as a test binary"
        );
    }
}

#[test]
fn a_shipped_binary_is_not_recognised_as_a_test() {
    // The shipped paths a release actually runs from. None of them may send the
    // memory store into a temporary directory: doing so would silently discard
    // a user's memory between runs.
    for shipped in [
        "/usr/local/bin/formal-ai",
        "/repo/target/release/formal-ai",
        "/repo/target/debug/formal-ai",
        "/home/user/.cargo/bin/formal-ai",
        "/opt/formal-ai/bin/formal-ai",
    ] {
        assert!(
            !formal_ai::is_test_executable(Path::new(shipped)),
            "{shipped} was mistaken for a test binary"
        );
    }
}

#[test]
fn this_very_binary_is_recognised_as_a_test() {
    // The measurement that matters. An earlier version of this guard keyed on
    // `CARGO_TARGET_TMPDIR`, which cargo passes at *compile* time and which is
    // absent from the runtime environment, and on `cfg!(test)`, which is false
    // for the library when it is built as a dependency of an integration test.
    // Both compiled, both were wrong, and the isolation silently did nothing.
    assert!(
        formal_ai::running_under_cargo_test(),
        "current_exe {:?} was not recognised as a cargo test binary",
        std::env::current_exe()
    );
    assert!(formal_ai::test_memory_directory().is_some());
}

#[test]
fn the_store_this_binary_resolves_is_not_in_the_home_directory() {
    // Only meaningful when the environment has not pinned a path already.
    if std::env::var_os(formal_ai::MEMORY_PATH_ENV)
        .is_some_and(|value| !value.to_string_lossy().trim().is_empty())
    {
        return;
    }
    let resolved = formal_ai::shared_memory_path();
    let home_store = std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(formal_ai::shared_memory::MEMORY_DIRECTORY_NAME)
            .join(formal_ai::shared_memory::MEMORY_FILE_NAME)
    });
    if let Some(home_store) = home_store {
        assert_ne!(
            resolved, home_store,
            "the test suite resolved the developer's own memory store"
        );
    }
}

#[test]
fn an_explicit_memory_path_still_wins() {
    // `tests/issue_756.rs` pins the exact path `resolve_memory_path_from`
    // derives from a given HOME, so the test fallback must never reach it.
    let explicit = std::ffi::OsStr::new("/tmp/explicit-store.lino");
    assert_eq!(
        formal_ai::resolve_memory_path_from(
            Some(explicit),
            Some(std::ffi::OsStr::new("/home/user")),
            None,
            false
        ),
        PathBuf::from("/tmp/explicit-store.lino")
    );
}

#[test]
fn a_test_that_relocates_home_keeps_the_store_it_asked_for() {
    // `tests/issue_756.rs` moves `HOME` to a temporary directory and asserts
    // the store is created under it. Redirecting that too would break a test
    // that is already isolating itself correctly, so the redirect covers only
    // the unmanaged default. Regression pin: an earlier version of this guard
    // overrode a relocated `HOME` and failed exactly that test.
    let relocated = std::env::temp_dir().join("formal-ai-relocated-home");
    let overrides: [(&str, Option<&std::ffi::OsStr>); 2] = [
        (formal_ai::MEMORY_PATH_ENV, None),
        ("HOME", Some(relocated.as_os_str())),
    ];
    temp_env::with_vars(overrides, || {
        assert_eq!(
            formal_ai::shared_memory_path(),
            relocated
                .join(formal_ai::shared_memory::MEMORY_DIRECTORY_NAME)
                .join(formal_ai::shared_memory::MEMORY_FILE_NAME),
            "a relocated HOME must win over the test-binary redirect"
        );
    });
}
