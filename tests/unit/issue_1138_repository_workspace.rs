//! Issue #1138 B7 (plan 03, L2, L4, L5): one workspace, at an exact commit.
//!
//! A branch name is not a commit and is refused before anything is created; the
//! tree that comes back is at the commit that was asked for; writing in the
//! workspace leaves the ambient checkout byte-identical; and the diff a task is
//! judged by round-trips — applying it to a second clone at the same base
//! reproduces the edited tree.

use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::repository_workspace::clone::{WorkspaceSpec, clone_at_base};
use formal_ai::repository_workspace::{RepositoryWorkspace, WorkspaceError};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn temp_root(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("formal-ai-issue-1138-workspace-{tag}"))
}

/// The ambient checkout's own HEAD, so the fixtures clone something real and
/// local rather than reaching the network.
fn head_commit() -> String {
    let output = Command::new("git")
        .args(["-C", &repo_root().display().to_string(), "rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse should run");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn spec_at(commit: &str) -> WorkspaceSpec {
    WorkspaceSpec {
        origin: repo_root().display().to_string(),
        base_commit: commit.to_owned(),
        sparse_paths: Vec::new(),
    }
}

/// A moving reference is not a task definition.
#[test]
fn clone_at_base_refuses_a_branch_name() {
    let root = temp_root("branch");
    let _ = std::fs::remove_dir_all(&root);
    let outcome = clone_at_base(&spec_at("main"), &root);
    assert_eq!(
        outcome,
        Err(WorkspaceError::NotACommit {
            given: String::from("main"),
        }),
        "a branch name must be refused rather than resolved to whatever it points at today"
    );
    assert!(
        !root.exists(),
        "a refused clone creates no directory"
    );
}

/// What comes back is checked out at exactly the requested commit.
#[test]
fn clone_at_base_checks_out_the_exact_commit() {
    let commit = head_commit();
    let root = temp_root("exact");
    let _ = std::fs::remove_dir_all(&root);
    let workspace =
        RepositoryWorkspace::open(&spec_at(&commit), &root).expect("a local clone should open");

    assert_eq!(
        workspace.base_commit(),
        commit,
        "the workspace reports the commit it was asked for"
    );
    let observed = Command::new("git")
        .args([
            "-C",
            &workspace.root().display().to_string(),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git rev-parse should run in the clone");
    assert_eq!(
        String::from_utf8_lossy(&observed.stdout).trim(),
        commit,
        "the tree on disk agrees with the spec"
    );
}

/// The workspace is somewhere else. Writing in it may not touch the checkout the
/// test itself is running from.
#[test]
fn workspace_is_isolated_from_the_ambient_checkout() {
    let commit = head_commit();
    let root = temp_root("isolation");
    let _ = std::fs::remove_dir_all(&root);
    let ambient = std::fs::read_to_string(repo_root().join("Cargo.toml"))
        .expect("the ambient manifest should be readable");

    let mut workspace =
        RepositoryWorkspace::open(&spec_at(&commit), &root).expect("a local clone should open");
    workspace
        .write("Cargo.toml", "# rewritten inside the workspace only\n")
        .expect("writing inside the workspace is allowed");

    assert_eq!(
        std::fs::read_to_string(repo_root().join("Cargo.toml"))
            .expect("the ambient manifest should still be readable"),
        ambient,
        "the ambient checkout must be byte-identical after a workspace write"
    );
    assert!(
        !workspace.root().starts_with(repo_root()),
        "the workspace root must live outside the ambient checkout"
    );
}

/// Nothing was changed, so there is nothing to show.
#[test]
fn diff_is_empty_for_an_untouched_clone() {
    let commit = head_commit();
    let root = temp_root("empty-diff");
    let _ = std::fs::remove_dir_all(&root);
    let workspace =
        RepositoryWorkspace::open(&spec_at(&commit), &root).expect("a local clone should open");
    assert_eq!(
        workspace.diff().expect("an untouched clone diffs cleanly"),
        "",
        "an untouched clone produces an empty diff"
    );
}

/// The diff is the deliverable, so it has to apply.
#[test]
fn diff_round_trips_through_git_apply() {
    let commit = head_commit();
    let first_root = temp_root("round-trip-a");
    let second_root = temp_root("round-trip-b");
    let _ = std::fs::remove_dir_all(&first_root);
    let _ = std::fs::remove_dir_all(&second_root);

    let mut first =
        RepositoryWorkspace::open(&spec_at(&commit), &first_root).expect("first clone opens");
    first
        .write("NOTICE-issue-1138.txt", "round trip\n")
        .expect("writing inside the workspace is allowed");
    let patch = first.diff().expect("the edited clone diffs");
    assert!(!patch.is_empty(), "an edited clone produces a diff");

    let second =
        RepositoryWorkspace::open(&spec_at(&commit), &second_root).expect("second clone opens");
    let patch_file = second_root.join("issue-1138.patch");
    std::fs::write(&patch_file, patch.as_bytes()).expect("the patch should be writable");
    let applied = Command::new("git")
        .args([
            "-C",
            &second.root().display().to_string(),
            "apply",
            &patch_file.display().to_string(),
        ])
        .output()
        .expect("git apply should run");
    assert!(
        applied.status.success(),
        "the produced diff must apply to a clean clone at the same base: {}",
        String::from_utf8_lossy(&applied.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(second.root().join("NOTICE-issue-1138.txt"))
            .expect("the applied file should exist"),
        "round trip\n",
        "the applied diff reproduces the edited tree byte for byte"
    );
}
