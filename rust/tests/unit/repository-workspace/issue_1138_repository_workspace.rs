//! Issue #1138 B7 (plan 03, L2, L4, L5): one workspace, at an exact commit.
//!
//! A branch name is not a commit and is refused before anything is created; the
//! tree that comes back is at the commit that was asked for; writing in the
//! workspace leaves the ambient checkout byte-identical; and the diff a task is
//! judged by round-trips — applying it to a second clone at the same base
//! reproduces the edited tree.

use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::execution_evidence::ObservationKind;
use formal_ai::meta_frame::NeedStatus;
use formal_ai::repository_workspace::clone::{WorkspaceSpec, clone_at_base};
use formal_ai::repository_workspace::edit::{Change, apply_change};
use formal_ai::repository_workspace::{
    RepositoryTask, RepositoryWorkspace, WorkspaceError, WorkspaceProtocol,
    render_protocol_template_from,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(tag: &str) -> Self {
        Self(std::env::temp_dir().join(format!("formal-ai-issue-1138-workspace-{tag}")))
    }
}

impl std::ops::Deref for TempRoot {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for TempRoot {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_root(tag: &str) -> TempRoot {
    TempRoot::new(tag)
}

/// The ambient checkout's own HEAD, so the fixtures clone something real and
/// local rather than reaching the network.
fn head_commit() -> String {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root().display().to_string(),
            "rev-parse",
            "HEAD",
        ])
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

#[test]
fn repository_messages_change_when_protocol_data_changes() {
    let first = "repository_template message\n  record_type \"runtime_template\"\n  text \"before {value}\"\n";
    let second = first.replace("before", "after");

    assert_eq!(
        render_protocol_template_from(first, "message", &[("value", "edit")]).as_deref(),
        Some("before edit")
    );
    assert_eq!(
        render_protocol_template_from(&second, "message", &[("value", "edit")]).as_deref(),
        Some("after edit")
    );
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
    assert!(!root.exists(), "a refused clone creates no directory");
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

/// Materialising one exact spec twice produces the same source tree. A task is
/// defined by immutable repository bytes, not by which clone happened first.
#[test]
fn clone_at_base_is_deterministic_for_the_same_spec() {
    let commit = head_commit();
    let first_root = temp_root("deterministic-a");
    let second_root = temp_root("deterministic-b");
    let _ = std::fs::remove_dir_all(&first_root);
    let _ = std::fs::remove_dir_all(&second_root);
    let first =
        RepositoryWorkspace::open(&spec_at(&commit), &first_root).expect("first clone opens");
    let second =
        RepositoryWorkspace::open(&spec_at(&commit), &second_root).expect("second clone opens");

    let tree = |workspace: &RepositoryWorkspace| {
        let output = Command::new("git")
            .args([
                "-C",
                &workspace.root().display().to_string(),
                "rev-parse",
                "HEAD^{tree}",
            ])
            .output()
            .expect("git tree identity");
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    };
    assert_eq!(
        tree(&first),
        tree(&second),
        "the immutable Git tree must be identical in both materialisations"
    );
}

/// The workspace is somewhere else. Writing in it may not touch the checkout the
/// test itself is running from.
#[test]
fn workspace_is_isolated_from_the_ambient_checkout() {
    let commit = head_commit();
    let root = temp_root("isolation");
    let _ = std::fs::remove_dir_all(&root);
    let ambient = std::fs::read_to_string(repo_root().join("rust/Cargo.toml"))
        .expect("the ambient manifest should be readable");

    let mut workspace =
        RepositoryWorkspace::open(&spec_at(&commit), &root).expect("a local clone should open");
    workspace
        .write("Cargo.toml", "# rewritten inside the workspace only\n")
        .expect("writing inside the workspace is allowed");

    assert_eq!(
        std::fs::read_to_string(repo_root().join("rust/Cargo.toml"))
            .expect("the ambient manifest should still be readable"),
        ambient,
        "the ambient checkout must be byte-identical after a workspace write"
    );
    assert!(
        !workspace.root().starts_with(repo_root()),
        "the workspace root must live outside the ambient checkout"
    );
}

/// An edit is complete only after the workspace reads the bytes back and binds
/// that observation to the changed path.
#[test]
fn apply_change_returns_evidence_for_the_observed_bytes() {
    let commit = head_commit();
    let root = temp_root("edit-evidence");
    let _ = std::fs::remove_dir_all(&root);
    let mut workspace =
        RepositoryWorkspace::open(&spec_at(&commit), &root).expect("a local clone should open");
    let change = Change {
        relative_path: String::from("NOTICE-issue-1138.txt"),
        contents: String::from("observed edit\n"),
    };

    let evidence = apply_change(&mut workspace, &change).expect("the edit should be observed");
    assert_eq!(evidence.kind, ObservationKind::FileBytes);
    assert_eq!(evidence.for_need, change.relative_path);
    assert_eq!(evidence.observed_byte_length, change.contents.len());
    assert_eq!(
        evidence.observed_output_sha256,
        formal_ai::source_fetch::sha256_hex(change.contents.as_bytes()),
        "the evidence hashes the bytes read back from disk"
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

/// A repository task is allowed to mutate only after both the locator and the
/// structural editor independently derive one unambiguous target from the
/// requirement and the tree. Quoted members are data, not fixture-specific
/// answers embedded in the protocol.
#[test]
fn protocol_derives_and_observes_a_structural_member_edit() {
    let commit = head_commit();
    let root = temp_root("protocol-edit");
    let _ = std::fs::remove_dir_all(&root);
    let spec = spec_at(&commit);
    let mut workspace = RepositoryWorkspace::open(&spec, &root).expect("a local clone should open");
    let outcome = WorkspaceProtocol::load().execute(
        &mut workspace,
        &RepositoryTask {
            requirement: String::from("Add \"wikiquote\" to the list of trusted search providers."),
            clone: spec,
            tests: None,
        },
    );

    assert!(
        outcome.open.is_empty(),
        "an evidenced structural edit must not leave an authoring obligation open: {:?}",
        outcome.open
    );
    assert!(
        outcome.diff.contains("+    \"wikiquote\","),
        "the deliverable is the observed repository diff: {}",
        outcome.diff
    );
    assert_eq!(
        outcome.edited,
        vec![String::from("rust/src/web_search_core.rs")],
        "only the located source file is an authored edit"
    );

    let verify = outcome
        .need_ledger
        .rows
        .iter()
        .find(|row| row.route.as_deref() == Some("verify"))
        .expect("the protocol ledger has one row per declared step");
    assert_eq!(
        verify.status,
        NeedStatus::Planned,
        "a skipped optional verification has no execution record and cannot be satisfied"
    );
    for row in outcome
        .need_ledger
        .rows
        .iter()
        .filter(|row| row.status == NeedStatus::Satisfied)
    {
        assert!(
            outcome
                .observations
                .iter()
                .any(|evidence| evidence.for_need == row.need_id),
            "a satisfied step must link to a concrete execution record: {row:?}"
        );
    }
}

/// Stopping at location leaves every later step planned. Merely loading the
/// protocol is never evidence that a step ran.
#[test]
fn a_step_that_did_not_run_remains_planned() {
    let commit = head_commit();
    let root = temp_root("protocol-stopped");
    let _ = std::fs::remove_dir_all(&root);
    let spec = spec_at(&commit);
    let mut workspace = RepositoryWorkspace::open(&spec, &root).expect("a local clone should open");
    let outcome = WorkspaceProtocol::load().execute(
        &mut workspace,
        &RepositoryTask {
            requirement: String::from("Change an unnamed concept that this tree does not contain."),
            clone: spec,
            tests: None,
        },
    );

    assert_eq!(
        outcome.stopped_at.as_ref().map(|step| step.id.as_str()),
        Some("locate")
    );
    for id in ["read", "edit", "verify", "diff"] {
        assert_eq!(
            outcome
                .need_ledger
                .rows
                .iter()
                .find(|row| row.route.as_deref() == Some(id))
                .map(|row| row.status),
            Some(NeedStatus::Planned),
            "the unexecuted `{id}` step must remain planned"
        );
    }
}
