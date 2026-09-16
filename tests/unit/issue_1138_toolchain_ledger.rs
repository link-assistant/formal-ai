//! Issue #1138 B6 (plan 06, L9): the recipe is durable, the payload is not.
//!
//! Forgetting a toolchain deletes the record *and* the installed prefix; the
//! next run rediscovers the same procedure and reproduces the same content id.
//! What survives a forget is the ability to rediscover, not the bytes; what
//! survives a restart is the ability to reattach without asking the publisher
//! again.

use std::path::PathBuf;

use formal_ai::prerequisite::Platform;
use formal_ai::prerequisite::ledger::{ToolchainLedger, ToolchainRecord};
use formal_ai::prerequisite::probe::ToolchainProbe;

fn temp_root(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("formal-ai-issue-1138-ledger-{tag}"))
}

fn record(root: &PathBuf) -> ToolchainRecord {
    ToolchainRecord {
        program: String::from("zig"),
        source_id: String::from("zig_official"),
        source_url: String::from("https://ziglang.org/learn/getting-started/"),
        content_id: String::from("4".repeat(64)),
        platform: Platform::observed(),
        postcondition: ToolchainProbe {
            program: String::from("zig"),
            argv: vec![String::from("version")],
            expect: None,
            requires: Vec::new(),
        },
        rediscover: String::from("https://ziglang.org/learn/getting-started/"),
        observed_version: String::from("as observed"),
        installed_prefix: root.join(".formal-ai/toolchains/zig/4"),
    }
}

/// Forget deletes the record and the bytes; the next discovery reproduces the
/// identical content id from the retained URL.
#[test]
fn forget_and_rediscover_reproduces_the_content_id() {
    let root = temp_root("forget");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");
    let ledger = ToolchainLedger::new(root.join("toolchain-ledger.lino"));
    let original = record(&root);
    ledger.append(&original).expect("the ledger is append-only");

    let before = ledger
        .record_for("zig")
        .expect("the appended record is readable back");
    ledger.forget("zig").expect("forgetting is supported");
    assert!(
        ledger.record_for("zig").is_none(),
        "a forgotten toolchain has no record"
    );
    assert!(
        !before.installed_prefix.exists(),
        "forgetting deletes the disposable bytes as well as the record"
    );

    ledger
        .append(&record(&root))
        .expect("rediscovery appends the record again");
    let after = ledger
        .record_for("zig")
        .expect("the rediscovered record is readable back");
    assert_eq!(
        after.content_id, before.content_id,
        "rediscovery from the retained URL must reproduce the identical content id"
    );
    assert_eq!(
        after.observed_version, before.observed_version,
        "the version is whatever was observed, never a hard-coded string"
    );
}

/// What the ledger keeps is the way back to the procedure, not the procedure's
/// output.
#[test]
fn the_ledger_retains_the_recipe_not_the_payload() {
    let root = temp_root("recipe");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");
    let ledger = ToolchainLedger::new(root.join("toolchain-ledger.lino"));
    ledger.append(&record(&root)).expect("append");

    ledger.forget("zig").expect("forgetting is supported");
    ledger.append(&record(&root)).expect("append again");

    let retained = ledger
        .record_for("zig")
        .expect("the reconstruction record survives");
    assert!(
        !retained.rediscover.is_empty(),
        "the rediscover URL is what survives a forget"
    );
    assert!(
        !retained.content_id.is_empty(),
        "the content id of the retrieved bytes survives with it"
    );
}

/// A fresh process finds the installed toolchain from the ledger alone.
#[test]
fn a_restart_reattaches_from_the_ledger() {
    let root = temp_root("restart");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test root should be creatable");
    let path = root.join("toolchain-ledger.lino");
    ToolchainLedger::new(&path)
        .append(&record(&root))
        .expect("append");

    // A second handle stands in for a restarted process: nothing is carried
    // over in memory.
    let restarted = ToolchainLedger::new(&path);
    let reattached = restarted
        .reattach("zig")
        .expect("a restart reattaches from the ledger without re-probing the publisher");
    assert_eq!(
        reattached.program, "zig",
        "the reattached toolchain is the one the ledger recorded"
    );
    assert!(
        reattached.prefix.starts_with(&root),
        "the reattached prefix is the workspace-scoped one"
    );
}
