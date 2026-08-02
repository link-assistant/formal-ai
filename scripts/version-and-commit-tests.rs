//! Tests for `version-and-commit.rs`.
//!
//! These live beside the script rather than inside it: the release script had
//! grown past the 1000-line limit `scripts/check-file-size.rs` enforces, and the
//! test module was the half that no release run executes. `super::` still names
//! the script, so the tests read exactly as they did inline.

use super::collect_changelog_with_date;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("version-and-commit-{name}-{nanos}"));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn changelog_collection_consumes_fragments_once_and_keeps_readme() {
    let repo = temp_dir("changelog-cleanup");
    let changelog_dir = repo.join("changelog.d");
    fs::create_dir_all(&changelog_dir).unwrap();

    let first_fragment = changelog_dir.join("20260714_fix_release_loop.md");
    let second_fragment = changelog_dir.join("20260714_note_release_loop.md");
    fs::write(
        &first_fragment,
        r#"---
bump: patch
---

### Fixed
- Prevent already-collected changelog fragments from triggering another release.
"#,
    )
    .unwrap();
    fs::write(
        &second_fragment,
        r#"### Changed
- Keep release commits from leaving stale changelog fragments behind.
"#,
    )
    .unwrap();
    fs::write(changelog_dir.join("README.md"), "Fragment instructions\n").unwrap();

    let changelog = repo.join("CHANGELOG.md");
    fs::write(
        &changelog,
        r#"# Changelog

## [0.1.0] - 2026-01-01

### Added
- Initial release
"#,
    )
    .unwrap();

    collect_changelog_with_date(
        changelog_dir.to_str().unwrap(),
        changelog.to_str().unwrap(),
        "0.2.0",
        "2026-07-14",
    );

    let after_first_release = fs::read_to_string(&changelog).unwrap();
    assert!(after_first_release.contains("## [0.2.0] - 2026-07-14"));
    assert!(after_first_release.contains("Prevent already-collected"));
    assert!(after_first_release.contains("Keep release commits"));
    assert!(!after_first_release.contains("bump: patch"));
    assert!(!first_fragment.exists());
    assert!(!second_fragment.exists());
    assert!(changelog_dir.join("README.md").exists());

    collect_changelog_with_date(
        changelog_dir.to_str().unwrap(),
        changelog.to_str().unwrap(),
        "0.3.0",
        "2026-07-15",
    );
    assert_eq!(fs::read_to_string(&changelog).unwrap(), after_first_release);
}

/// The release commit must write CHANGELOG.md in exactly the shape
/// `experiments/issue_711_rebuild_changelog.mjs --check` reconstructs from
/// git history, byte for byte. It did not: the entry was spliced in before
/// the first `## [` line, but `lines[..idx]` already ends with the blank
/// line that follows the insert marker and `new_entry` opened with another
/// `\n`, so every release left a second blank line after the marker. And
/// `lines()` drops the trailing newline that `join("\n")` never restores,
/// so every release also stripped the final newline. Both survived because
/// the check only runs when the lint job's path filter fires, which a
/// release commit does not trigger -- so `main` went red on the next
/// unrelated PR and someone hand-fixed it ("refresh reconstructed release
/// artifacts", repeatedly).
#[test]
fn release_writes_the_changelog_exactly_as_reconstruction_expects() {
    let repo = temp_dir("changelog-canonical-shape");
    let changelog_dir = repo.join("changelog.d");
    fs::create_dir_all(&changelog_dir).unwrap();
    fs::write(
        changelog_dir.join("fragment.md"),
        "---\nbump: patch\n---\n\n### Fixed\n- A representative fragment.\n",
    )
    .unwrap();

    // The canonical shape: marker, exactly one blank line, newest section,
    // one blank line between sections, single trailing newline.
    let changelog = repo.join("CHANGELOG.md");
    let before = "\
# Changelog

<!-- changelog-insert-here -->

## [0.1.0] - 2026-01-01

### Added
- Initial release
";
    fs::write(&changelog, before).unwrap();

    collect_changelog_with_date(
        changelog_dir.to_str().unwrap(),
        changelog.to_str().unwrap(),
        "0.2.0",
        "2026-07-16",
    );

    let expected = "\
# Changelog

<!-- changelog-insert-here -->

## [0.2.0] - 2026-07-16

### Fixed
- A representative fragment.

## [0.1.0] - 2026-01-01

### Added
- Initial release
";
    assert_eq!(fs::read_to_string(&changelog).unwrap(), expected);
}

fn git_ok(repo: &Path, args: &[&str]) {
    super::git(repo, args).unwrap_or_else(|e| panic!("git {:?} failed: {}", args, e));
}

/// Set up a bare origin plus a clone whose identity is configured, so the
/// release helper can commit without inheriting the developer's git config.
fn repo_with_origin(name: &str) -> (PathBuf, PathBuf) {
    let root = temp_dir(name);
    let origin = root.join("origin.git");
    let work = root.join("work");
    git_ok(
        &root,
        &["init", "--bare", "--initial-branch=main", origin.to_str().unwrap()],
    );
    git_ok(&root, &["clone", origin.to_str().unwrap(), work.to_str().unwrap()]);
    git_ok(&work, &["config", "user.email", "ci@example.com"]);
    git_ok(&work, &["config", "user.name", "CI"]);
    fs::write(work.join("Cargo.toml"), "version = \"0.1.0\"\n").unwrap();
    git_ok(&work, &["add", "-A"]);
    git_ok(&work, &["commit", "-m", "init"]);
    git_ok(&work, &["push", "-u", "origin", "main"]);
    (root, work)
}

/// Land a commit on origin/main from an independent clone, simulating a
/// concurrent release that pushes while this job is running.
fn push_concurrent_commit(root: &Path, subject: &str) {
    let other = root.join("other");
    git_ok(
        root,
        &[
            "clone",
            root.join("origin.git").to_str().unwrap(),
            other.to_str().unwrap(),
        ],
    );
    git_ok(&other, &["config", "user.email", "other@example.com"]);
    git_ok(&other, &["config", "user.name", "Other"]);
    fs::write(other.join("NOTES.md"), "concurrent\n").unwrap();
    git_ok(&other, &["add", "-A"]);
    git_ok(&other, &["commit", "-m", subject]);
    git_ok(&other, &["push", "origin", "main"]);
}

/// Regression test for the release failure in CI run 29484631709:
/// "Error rebasing onto origin/main: Command failed: error: cannot rebase:
/// Your index contains uncommitted changes." A concurrent release pushed to
/// origin/main while this job had already staged its version bump, so the
/// sync must happen before anything is written and staged.
#[test]
fn syncs_with_concurrent_release_then_commits_bump_on_top() {
    let (root, work) = repo_with_origin("concurrent-release");
    push_concurrent_commit(&root, "concurrent change");

    // Order under test: sync while clean, then bump, stage and commit.
    super::sync_with_remote(&work, "main").expect("sync must succeed against an advanced remote");
    fs::write(work.join("Cargo.toml"), "version = \"0.2.0\"\n").unwrap();
    git_ok(&work, &["add", "Cargo.toml"]);
    git_ok(&work, &["commit", "-m", "chore: release v0.2.0"]);

    // The release commit sits on top of the concurrent one, and both survive.
    let log = super::git(&work, &["log", "--format=%s"]).unwrap();
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        vec!["chore: release v0.2.0", "concurrent change", "init"]
    );
    assert_eq!(
        fs::read_to_string(work.join("Cargo.toml")).unwrap(),
        "version = \"0.2.0\"\n"
    );
    assert_eq!(fs::read_to_string(work.join("NOTES.md")).unwrap(), "concurrent\n");
    assert!(super::git(&work, &["status", "--porcelain"]).unwrap().is_empty());
}

/// Pins the ordering itself: syncing after the bump is staged is exactly the
/// failure CI hit, so this must stay impossible.
#[test]
fn syncing_after_staging_the_bump_fails() {
    let (root, work) = repo_with_origin("sync-after-staging");
    push_concurrent_commit(&root, "concurrent change");

    fs::write(work.join("Cargo.toml"), "version = \"0.2.0\"\n").unwrap();
    git_ok(&work, &["add", "Cargo.toml"]);

    let err = super::sync_with_remote(&work, "main")
        .expect_err("rebase must refuse to run against a staged version bump");
    assert!(err.contains("cannot rebase"), "unexpected error: {}", err);
}

/// Being *ahead* of origin is not a reason to rebase. The old
/// `local != remote` check treated ahead and diverged as "behind".
#[test]
fn does_not_rebase_when_only_ahead_of_remote() {
    let (_root, work) = repo_with_origin("ahead-of-remote");

    fs::write(work.join("Cargo.toml"), "version = \"0.2.0\"\n").unwrap();
    git_ok(&work, &["add", "Cargo.toml"]);
    git_ok(&work, &["commit", "-m", "chore: release v0.2.0"]);

    assert_eq!(super::commits_behind(&work, "main"), 0);
    // Clean no-op even though HEAD != origin/main.
    super::sync_with_remote(&work, "main").expect("being ahead must not trigger a rebase");
    let log = super::git(&work, &["log", "--format=%s"]).unwrap();
    assert_eq!(log.lines().collect::<Vec<_>>(), vec!["chore: release v0.2.0", "init"]);
}
