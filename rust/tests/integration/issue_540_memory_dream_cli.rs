//! Issue #540: the `formal-ai memory dream` command.
//!
//! It is the inspectable maintenance interface (R545) and non-destructive by
//! default (R539): it prints the plan, including each replayed candidate
//! task's pass/fail evidence (R415), and changes the memory file only through
//! `--apply --confirm`, writing the optional `--backup` bundle first.

use std::path::Path;
use std::process::{Command, Output};

use formal_ai::{MemoryEvent, MemoryStore, RetainedAmendment, replay_answer_with_amendments};

const RULE: &str = "Always include a runnable test with Rust changes.";

fn fixture_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-540-dream-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("create fixture directory");
    path
}

fn cache_event(id: &str, payload: &str) -> MemoryEvent {
    MemoryEvent {
        id: String::from(id),
        kind: Some(String::from("source:http")),
        role: Some(String::from("cache")),
        tool: Some(String::from("web_search")),
        content: Some(String::from(payload)),
        outputs: Some(String::from(payload)),
        evidence: vec![String::from("rediscover:https://doc.rust-lang.org/std/")],
        ..MemoryEvent::default()
    }
}

/// Seed a requirement, its replay-verified run, two duplicate caches, and a raw message.
fn seed_memory(path: &Path) {
    let input = "refactor rust parser safely";
    let payload = "same cache payload".repeat(20);
    MemoryStore::from_events(vec![
        MemoryEvent {
            id: String::from("req-1"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from(RULE)),
            conversation_title: Some(String::from("rust")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("run-1"),
            kind: Some(String::from("test_run")),
            role: Some(String::from("derived")),
            content: Some(String::from(input)),
            inputs: Some(String::from(input)),
            outputs: Some(replay_answer_with_amendments(
                input,
                &[RetainedAmendment {
                    id: String::from("run-1-amendment"),
                    topic: String::from("rust"),
                    rule: String::from(RULE),
                }],
            )),
            conversation_title: Some(String::from("rust")),
            ..MemoryEvent::default()
        },
        cache_event("cache-low-use", &payload),
        cache_event("cache-high-use", &payload),
        MemoryEvent {
            id: String::from("raw-user-message"),
            kind: Some(String::from("message")),
            role: Some(String::from("user")),
            content: Some(String::from("raw event stays")),
            evidence: vec![String::from("source:http:cache-high-use")],
            ..MemoryEvent::default()
        },
    ])
    .save_to_file(path)
    .expect("write dreaming fixture");
}

fn dream(path: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "dream",
            "--path",
            path.to_str().expect("utf-8 memory path"),
            "--storage-capacity-bytes",
            "1000000",
            "--free-bytes",
            "1000000",
        ])
        .args(extra)
        .env("FORMAL_AI_DREAMING", "0")
        .output()
        .expect("run formal-ai memory dream")
}

fn event_ids(path: &Path) -> Vec<String> {
    MemoryStore::load_from_file(path)
        .expect("load memory")
        .events()
        .iter()
        .map(|event| event.id.clone())
        .collect()
}

#[test]
fn memory_dream_prints_the_plan_and_changes_nothing_without_apply_confirm() {
    let dir = fixture_dir("dry-run");
    let memory_path = dir.join("memory.lino");
    seed_memory(&memory_path);
    let before = std::fs::read(&memory_path).expect("read seeded memory");

    let plan = dream(&memory_path, &[]);
    assert!(
        plan.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&plan.stderr)
    );
    let stdout = String::from_utf8_lossy(&plan.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.first().copied(), Some("memory_dreaming_plan"));
    for expected in [
        "  enabled: true",
        "  target_free_ratio_percent: 20",
        "  storage_capacity_bytes: 1000000",
        "  free_bytes: 1000000",
        "  required_reclaim_bytes: 0",
        "  requires_bigger_storage: false",
        "  candidate_task topic=rust source=run-1 passed=true",
    ] {
        assert!(
            lines.contains(&expected),
            "missing {expected:?} in\n{stdout}"
        );
    }
    assert!(
        lines
            .iter()
            .any(|line| line
                .starts_with("  action RemoveDuplicateRecomputable event=cache-low-use ")),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read(&memory_path).expect("read memory after the plan"),
        before,
        "printing the plan must not touch memory"
    );

    let refused = dream(&memory_path, &["--apply"]);
    assert!(
        !refused.status.success(),
        "--apply without --confirm must fail"
    );
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains(
            "Refusing to apply dreaming memory plan because this operation is irreversible."
        ),
        "{}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert_eq!(
        std::fs::read(&memory_path).expect("read memory after the refusal"),
        before,
        "a refused apply must not touch memory"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn memory_dream_apply_confirm_writes_the_backup_then_removes_only_the_duplicate() {
    let dir = fixture_dir("apply");
    let memory_path = dir.join("memory.lino");
    let backup_path = dir.join("backup.lino");
    seed_memory(&memory_path);

    let applied = dream(
        &memory_path,
        &[
            "--apply",
            "--confirm",
            "--backup",
            backup_path.to_str().expect("utf-8 backup path"),
        ],
    );
    assert!(
        applied.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&applied.stderr)
    );
    let backup = std::fs::read_to_string(&backup_path).expect("the backup bundle is written");
    assert!(
        backup.contains("cache-low-use"),
        "the backup keeps the removed event"
    );

    let ids = event_ids(&memory_path);
    assert!(!ids.iter().any(|id| id == "cache-low-use"), "{ids:?}");
    for kept in ["req-1", "run-1", "cache-high-use", "raw-user-message"] {
        assert!(ids.iter().any(|id| id == kept), "{kept} must stay: {ids:?}");
    }
    let store = MemoryStore::load_from_file(&memory_path).expect("load dreamed memory");
    assert!(
        store
            .events()
            .iter()
            .any(|event| event.kind.as_deref() == Some("meta_algorithm_amendment")),
        "applying bakes the learned amendment"
    );
    let _ = std::fs::remove_dir_all(dir);
}
