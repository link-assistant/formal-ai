//! Issue #1180 (E145): repository history as formal context for reasoning.
//!
//! Covers the requirements the issue pins: commit formalization with
//! trailers and changed symbols (R1180-1/R1180-2), the issue/PR/comment/
//! review importers over github-logs JSON shapes (R1180-3/R1180-4), the
//! CI-run importer with failing steps (R1180-5), watermark incrementalism
//! and idempotence (R1180-6), `MemoryEvent` storage with no schema change
//! (R1180-7), and the `check-self-development-release.rs` lineage on this
//! repository's real history (R1180-8) — queried through the existing
//! memory query language.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::history_context::{self, HistoryRules, RepositoryHistoryCursor};
use formal_ai::memory::MemoryStore;
use formal_ai::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};
use formal_ai::memory_query_language::{
    MemoryQueryValue, QueryDialect, compile_memory_query, execute_memory_query,
};

const LIMITS: MemoryProgramLimits = MemoryProgramLimits {
    max_matches: 32,
    max_iterations: 4,
};

static SEQ: AtomicUsize = AtomicUsize::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-history-{}-{}-{}",
        tag,
        std::process::id(),
        SEQ.fetch_add(1, Ordering::SeqCst),
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
}

fn commit(dir: &Path, path: &str, contents: &str, subject: &str, body: Option<&str>) -> String {
    let file = dir.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).expect("create fixture directory");
    }
    std::fs::write(file, contents).expect("write fixture file");
    git(dir, &["add", path]);
    let mut args: Vec<&str> = vec!["-c", "commit.gpgsign=false", "commit", "-q", "-m", subject];
    if let Some(body) = body {
        args.push("-m");
        args.push(body);
    }
    git(dir, &args);
    git(dir, &["rev-parse", "HEAD"])
}

#[test]
fn commit_formalization_extracts_trailers_and_changed_symbols() {
    let repo = temp_dir("trailers");
    init_repo(&repo);
    commit(
        &repo,
        "src/lib.rs",
        "pub fn alpha() {}\n",
        "add alpha",
        None,
    );
    let second = commit(
        &repo,
        "src/lib.rs",
        "pub fn alpha() {}\npub fn beta() {}\n",
        "add beta",
        Some("Refs #42"),
    );

    let rules = HistoryRules::defaults();
    let events =
        history_context::import_commits(&repo, None, &rules).expect("import fixture commits");
    assert_eq!(events.len(), 2, "both commits import");

    let event = events
        .iter()
        .find(|event| event.id == format!("commit:{second}"))
        .expect("second commit present");
    assert_eq!(event.kind.as_deref(), Some("commit"));
    assert_eq!(event.role.as_deref(), Some("author"));
    assert_eq!(event.conversation_id.as_deref(), Some("issue-42"));
    assert!(
        event
            .evidence
            .iter()
            .any(|evidence| evidence == "path:src/lib.rs")
    );
    assert!(
        event
            .evidence
            .iter()
            .any(|evidence| evidence == "symbol:function_item"),
        "the added function is a changed symbol: {:?}",
        event.evidence
    );
    assert!(
        event.sent_at.is_some(),
        "the committer date formalizes as sent_at"
    );
    let content = event.content.as_deref().unwrap_or_default();
    assert!(content.starts_with("add beta"));
    assert!(content.contains("Refs #42"));
    assert!(
        event
            .content
            .as_deref()
            .is_some_and(|text| !text.is_empty()),
        "subject and body both formalize into content"
    );
}

#[test]
fn incremental_import_is_idempotent() {
    let repo = temp_dir("idempotent");
    init_repo(&repo);
    commit(&repo, "a.rs", "pub fn one() {}\n", "one", None);
    commit(
        &repo,
        "a.rs",
        "pub fn one() {}\npub fn two() {}\n",
        "two",
        None,
    );

    let memory = temp_dir("idempotent-memory");
    let rules = HistoryRules::defaults();
    let first =
        history_context::import_incremental(&repo, None, &memory, &rules).expect("first import");
    assert_eq!(first, 2, "both commits append");

    let second =
        history_context::import_incremental(&repo, None, &memory, &rules).expect("second import");
    assert_eq!(second, 0, "no new history appends nothing");

    let (store_path, cursor_path) = history_context::store_paths(&memory, &repo);
    let cursor_before = std::fs::read_to_string(&cursor_path).expect("cursor written");
    assert!(cursor_before.contains("last_commit_sha"));

    let third =
        history_context::import_incremental(&repo, None, &memory, &rules).expect("third import");
    assert_eq!(third, 0);
    let cursor_after = std::fs::read_to_string(&cursor_path).expect("cursor stable");
    assert_eq!(cursor_before, cursor_after);

    let store = MemoryStore::load_from_file(&store_path).expect("store reloads");
    assert_eq!(store.events().len(), 2, "no duplicates in the store");
}

#[test]
fn incremental_import_only_advances_past_the_watermark() {
    let repo = temp_dir("watermark");
    init_repo(&repo);
    let first = commit(&repo, "a.rs", "pub fn one() {}\n", "one", None);
    let second = commit(
        &repo,
        "a.rs",
        "pub fn one() {}\npub fn two() {}\n",
        "two",
        None,
    );
    let third = commit(
        &repo,
        "a.rs",
        "pub fn one() {}\npub fn two() {}\npub fn three() {}\n",
        "three",
        None,
    );
    assert_ne!(first, second);
    assert_ne!(second, third);

    let rules = HistoryRules::defaults();
    let events = history_context::import_commits(&repo, Some(&second), &rules)
        .expect("watermark-bounded import");
    assert_eq!(events.len(), 1, "only the commit after the watermark");
    assert_eq!(events[0].id, format!("commit:{third}"));

    let memory = temp_dir("watermark-memory");
    let (_, cursor_path) = history_context::store_paths(&memory, &repo);
    RepositoryHistoryCursor {
        last_commit_sha: Some(second),
        ..RepositoryHistoryCursor::default()
    }
    .write(&cursor_path)
    .expect("seed cursor");
    let appended = history_context::import_incremental(&repo, None, &memory, &rules)
        .expect("incremental from seeded cursor");
    assert_eq!(appended, 1, "the seeded cursor bounds the import");
}

#[test]
fn issue_and_pr_import_links_commit_to_source_issue() {
    let logs = temp_dir("logs");
    std::fs::write(
        logs.join("issue-1014.json"),
        r#"{
            "number": 1014,
            "title": "release gate keeps failing",
            "body": "The self-development release gate fails on every run.",
            "state": "closed",
            "labels": [{"name": "bug"}],
            "createdAt": "2026-09-07T00:00:00Z",
            "updatedAt": "2026-09-07T00:00:00Z",
            "url": "https://github.com/link-assistant/formal-ai/issues/1014",
            "author": {"login": "konard"}
        }"#,
    )
    .expect("write issue fixture");
    std::fs::write(
        logs.join("pr-1015.json"),
        r#"{
            "number": 1015,
            "title": "fix the release gate",
            "body": "Closes #1014",
            "state": "merged",
            "labels": [],
            "createdAt": "2026-09-08T00:00:00Z",
            "updatedAt": "2026-09-09T00:00:00Z",
            "url": "https://github.com/link-assistant/formal-ai/pull/1015",
            "author": {"login": "konard"}
        }"#,
    )
    .expect("write pr fixture");
    std::fs::write(
        logs.join("pr-1015-reviews.json"),
        r#"[
            {"id": 777, "user": {"login": "konard"}, "state": "APPROVED",
             "body": "", "submittedAt": "2026-09-08T01:00:00Z"}
        ]"#,
    )
    .expect("write review fixture");
    std::fs::write(
        logs.join("issue-1014-comments.json"),
        r#"[
            {"id": 888, "author": {"login": "konard"}, "body": "done",
             "createdAt": "2026-09-08T02:00:00Z"}
        ]"#,
    )
    .expect("write comment fixture");

    let rules = HistoryRules::defaults();
    let events = history_context::import_issues_and_pulls(&logs, None, &rules)
        .expect("import github-logs fixtures");
    let by_id = |id: &str| {
        events
            .iter()
            .find(|event| event.id == id)
            .unwrap_or_else(|| {
                let known: Vec<&str> = events.iter().map(|event| event.id.as_str()).collect();
                panic!("missing {id}: {known:?}")
            })
    };

    let issue = by_id("issue:1014");
    assert_eq!(issue.kind.as_deref(), Some("issue"));
    assert_eq!(issue.conversation_id.as_deref(), Some("issue-1014"));
    assert_eq!(issue.intent.as_deref(), Some("closed"));
    assert!(
        issue.evidence.iter().any(|e| e == "label:bug"),
        "labels formalize as evidence: {:?}",
        issue.evidence
    );

    let pull = by_id("pull:1015");
    assert_eq!(pull.kind.as_deref(), Some("pull_request"));
    assert!(
        pull.evidence.iter().any(|e| e == "issue:1014"),
        "the PR body's Closes trailer links its issue: {:?}",
        pull.evidence
    );

    let review = events
        .iter()
        .find(|event| event.intent.as_deref() == Some("review_decision"))
        .expect("review decision present");
    assert_eq!(review.kind.as_deref(), Some("review"));
    assert_eq!(review.conversation_id.as_deref(), Some("pr-1015"));
    assert!(review.evidence.iter().any(|e| e == "pr:1015"));

    let comment = events
        .iter()
        .find(|event| event.intent.as_deref() == Some("comment"))
        .expect("issue comment present");
    assert_eq!(comment.conversation_id.as_deref(), Some("issue-1014"));

    // The watermark keeps only records strictly newer than it.
    let filtered =
        history_context::import_issues_and_pulls(&logs, Some("2026-09-08T00:00:00Z"), &rules)
            .expect("filtered import");
    let ids: Vec<&str> = filtered.iter().map(|event| event.id.as_str()).collect();
    assert!(
        !ids.contains(&"issue:1014"),
        "updatedAt equal to the watermark is excluded: {ids:?}"
    );
    assert!(ids.contains(&"pull:1015"), "newer records pass: {ids:?}");

    // Everything is queryable through the existing memory query language.
    let mut store = MemoryStore::from_events(events);
    let query = compile_memory_query(
        // `CONTAINS` is not ANSI, and the exact sql-ansi grammar rejects it;
        // `LIKE` with `%` wildcards is the standard spelling of the same test.
        "SELECT id FROM memory WHERE kind = 'pull_request' AND evidence LIKE '%issue:1014%'",
        QueryDialect::SqlAnsi,
        LIMITS,
    )
    .expect("compile lineage query");
    let outcome = execute_memory_query(&query, &mut store, MemoryProgramAuthorization::ReadOnly);
    assert!(
        outcome.rows.iter().any(|row| {
            matches!(
                row.get("id"),
                Some(MemoryQueryValue::Text(text)) if text == "pull:1015"
            )
        }),
        "lineage query answers from repository history: {:?}",
        outcome.rows
    );
}

#[test]
fn ci_run_import_carries_failing_steps() {
    let logs = temp_dir("runs");
    std::fs::write(
        logs.join("run-36266423193.json"),
        r#"{
            "databaseId": 36266423193,
            "workflowName": "regenerate-self-ast-census",
            "status": "completed",
            "conclusion": "failure",
            "createdAt": "2026-09-26T21:00:00Z",
            "updatedAt": "2026-09-26T21:10:00Z",
            "headSha": "e4193a615c2a0788a2f7e1067a1a55d4c5d7912d",
            "event": "push",
            "url": "https://github.com/link-assistant/formal-ai/actions/runs/36266423193",
            "jobs": [
                {"name": "census", "conclusion": "failure",
                 "steps": [{"name": "regenerate", "conclusion": "failure"}]}
            ]
        }"#,
    )
    .expect("write run fixture");
    std::fs::write(
        logs.join("actions-runs-recent.json"),
        r#"[
            {"databaseId": 36266423194, "workflowName": "release",
             "status": "completed", "conclusion": "success",
             "createdAt": "2026-09-27T21:00:00Z", "updatedAt": "2026-09-27T21:10:00Z",
             "headSha": "0000000000000000000000000000000000000000",
             "event": "push", "url": "", "jobs": []}
        ]"#,
    )
    .expect("write run list fixture");

    let rules = HistoryRules::defaults();
    let events = history_context::import_ci_runs(&logs, None, &rules).expect("import runs");
    assert_eq!(events.len(), 2);

    let failed = events
        .iter()
        .find(|event| event.id == "ci_run:36266423193")
        .expect("failed run present");
    assert_eq!(failed.kind.as_deref(), Some("ci_run"));
    let content = failed.content.as_deref().unwrap_or_default();
    assert!(
        content.starts_with("regenerate-self-ast-census: failure"),
        "workflow and conclusion formalize: {content}"
    );
    assert!(
        content.contains("census/regenerate"),
        "the failing step formalizes: {content}"
    );
    assert!(
        failed
            .evidence
            .iter()
            .any(|e| e == "commit:e4193a615c2a0788a2f7e1067a1a55d4c5d7912d")
    );

    let newer_only = history_context::import_ci_runs(&logs, Some(36_266_423_193), &rules)
        .expect("filtered runs");
    let ids: Vec<&str> = newer_only.iter().map(|event| event.id.as_str()).collect();
    assert!(
        !ids.contains(&"ci_run:36266423193"),
        "databaseId equal to the watermark is excluded: {ids:?}"
    );
    assert!(ids.contains(&"ci_run:36266423194"));
}

#[test]
fn rules_seed_round_trips_through_the_parser() {
    let seed = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../data/seed/history-formalization.lino"
    ))
    .expect("seed on disk");
    let rules = HistoryRules::from_seed_text(&seed);
    assert_eq!(rules, HistoryRules::defaults());
}

#[test]
fn on_this_repo_check_self_development_release_lineage() {
    let repo_root =
        std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("..")).expect("repo root");
    let shallow = Command::new("git")
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(&repo_root)
        .output()
        .expect("probe repository shape");
    if String::from_utf8_lossy(&shallow.stdout).trim() == "true" {
        eprintln!("skip: shallow clone has no followable history");
        return;
    }

    let rules = HistoryRules::load(Some(&repo_root));
    let events = history_context::import_commits_for_path(
        &repo_root,
        None,
        "scripts/check-self-development-release.rs",
        &rules,
    )
    .expect("import the pinned path's lineage");
    assert!(!events.is_empty(), "the path has history");

    let ids: Vec<&str> = events.iter().map(|event| event.id.as_str()).collect();
    for prefix in [
        "commit:88d368a5b",
        "commit:02f590106",
        "commit:96f405e0d",
        "commit:aa6a5d085",
        "commit:9efeb735c",
        "commit:33451d012",
    ] {
        assert!(
            ids.iter().any(|id| id.starts_with(prefix)),
            "lineage is missing {prefix}: {ids:?}"
        );
    }

    let first = &events[0];
    assert!(
        first.id.starts_with("commit:88d368a5b"),
        "the introducing commit comes first chronologically: {}",
        first.id
    );
    assert_eq!(first.conversation_id.as_deref(), Some("issue-1014"));
    assert!(
        first.evidence.iter().any(|e| e == "pr:1015"),
        "the delivering merge formalizes: {:?}",
        first.evidence
    );
    assert!(
        first
            .evidence
            .iter()
            .any(|e| e == "path:scripts/check-self-development-release.rs")
    );
}
