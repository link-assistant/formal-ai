//! Diagnostics for issue #1031: sccache sees one compile request per job.
//!
//! `Test (ubuntu-latest / full)` compiled 514 crates on run 32500015304 while
//! sccache reported `Compile requests 1`, zero misses and zero write errors. A
//! wrapper that is never asked cannot miss, so that counter describes neither a
//! cold cache nor a broken backend.
//!
//! The configuration is not at fault: `RUSTC_WRAPPER`, `SCCACHE_GHA_ENABLED`
//! and `incremental = false` on both profiles are exactly what sccache needs.
//! What was missing is the evidence to tell a lazily spawned server that dies
//! with its step apart from a cache that was merely cold -- so the server is
//! started explicitly and the counters are read between steps.

use super::workflow_fixtures::{job_block, release_workflow};
use std::fs;

/// sccache reported `Compile requests 1` against 514 compiled crates on run
/// 32500015304 -- one request, zero misses, zero write errors. A wrapper that
/// is never asked cannot miss, so the counter says the compilations went
/// somewhere else rather than that the cache was cold (#1029).
///
/// Two things make that answerable on the next run instead of the next guess:
/// the server is started explicitly before any cargo step, so a lazily spawned
/// one that dies with its step cannot be mistaken for a cold cache; and the
/// counters are read straight after the step that compiles, not only in the
/// post-job summary, which separates "never saw the work" from "was reset".
#[test]
fn sccache_is_started_explicitly_and_reports_between_steps() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let action = fs::read_to_string(format!(
        "{manifest_dir}/.github/actions/setup-sccache/action.yml"
    ))
    .expect("the sccache setup action should be readable");

    assert!(
        action.contains("sccache --start-server"),
        "the setup action must start the server before any cargo step, or a \
         server spawned lazily by the first rustc call cannot outlive it"
    );
    assert!(
        action.contains("sccache --show-stats"),
        "the setup action must report the counters it starts from"
    );

    let workflow = release_workflow();
    let test = job_block(&workflow, "test");
    assert!(
        test.matches("sccache --show-stats").count() >= 1,
        "the test job must read the counters after its compiling step, not \
         only in the post-job summary"
    );
}
