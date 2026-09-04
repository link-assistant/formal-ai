//! Issue #1066: `verify-node.sh` decides a node's outcome, not the agent.
//!
//! A node used to pass on three facts: the Agent CLI exited zero, a proof file
//! existed, and its first line named the node. None of them is the task. These
//! tests execute the committed verifier against fixture repositories that each
//! isolate one way a node can look finished without being finished -- a proof
//! with no repository effect, a composite that drops a child's result, a leaf
//! change that destroys the anchor it was told to keep -- so the completion
//! criterion is evaluated by the harness and demonstrated to be.
//!
//! They live beside `issue_1066_agent_ladder`, which proves the tree the ladder
//! generates, and share its fixture helpers.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::issue_1066_agent_ladder::{git, root, temporary_directory};

const NODE_VERIFIER: &str = "experiments/issue_1028_agent_cli_ladder/verify-node.sh";

/// The change a leaf fixture is expected to make: absent from the commit,
/// present in the worktree afterwards.
const LEAF_MARKER: &str = "pub max_depth: u8";
/// The anchor that must survive it, so a node cannot pass by rewriting the file
/// down to its marker.
const LEAF_ANCHOR: &str = "storage_field=children";

/// Make the fixture's tracked source carry the leaf's change, exactly as a node
/// that did the work would leave the worktree.
fn apply_leaf_change(directory: &Path) {
    let target = directory.join("README.md");
    let source = fs::read_to_string(&target).expect("fixture source");
    fs::write(&target, format!("{source}{LEAF_MARKER},\n")).expect("apply the leaf change");
}

/// Write the node's leaf effect file with the given `result=` line.
fn write_leaf_effect(directory: &Path, result: &str) {
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        format!("node_path=1.1\nnode_depth=5\nnode_kind=leaf\nresult={result}\n"),
    )
    .expect("write leaf effect");
}

fn node_verifier_fixture(label: &str) -> (PathBuf, PathBuf) {
    let directory = temporary_directory(label);
    git(&directory, &["init", "--quiet"]);
    git(&directory, &["config", "user.name", "Ladder Fixture"]);
    git(
        &directory,
        &["config", "user.email", "ladder@example.invalid"],
    );
    fs::write(
        directory.join("README.md"),
        "fixture\nstorage_field=children\npub children: Vec<Self>,\n",
    )
    .expect("write fixture seed");
    fs::write(directory.join("NOTES.md"), "unrelated tracked file\n").expect("write bystander");
    fs::write(
        directory.join("fixture.rs"),
        "pub struct Node {\n    pub children: Vec<Node>,\n}\n",
    )
    .expect("write Rust fixture");
    git(&directory, &["add", "README.md", "NOTES.md", "fixture.rs"]);
    git(&directory, &["commit", "--quiet", "-m", "fixture"]);
    let proof = directory.join(".agent-ladder/node-1.1-proof.md");
    fs::create_dir_all(proof.parent().expect("proof parent")).expect("create proof directory");
    fs::write(
        &proof,
        "node_path=1.1\nThe repository inspection produced a concrete recorded result.\n",
    )
    .expect("write proof");
    (directory, proof)
}

fn commit_verified_child_effects(directory: &Path, left_result: &str, right_result: &str) {
    let child_directory = directory.join(".agent-ladder/verified-children");
    fs::create_dir_all(&child_directory).expect("create verified-child directory");
    fs::write(
        child_directory.join("node-1.1.1.lino"),
        format!("node_path=1.1.1\nresult={left_result}\n"),
    )
    .expect("write left child effect");
    fs::write(
        child_directory.join("node-1.1.2.lino"),
        format!("node_path=1.1.2\nresult={right_result}\n"),
    )
    .expect("write right child effect");
    git(directory, &["add", ".agent-ladder/verified-children"]);
    git(
        directory,
        &["commit", "--quiet", "-m", "verified child effects"],
    );
}

/// The external completion criterion a leaf is measured against: the file its
/// task named, the text that must appear there, and the text that must survive
/// the change. Passing it as one value keeps the three from drifting apart at a
/// call site, where they are meaningless individually.
struct Criterion<'a> {
    path: &'a str,
    marker: &'a str,
    guard: &'a str,
}

/// The criterion the leaf fixtures below carry.
const LEAF_CRITERION: Criterion<'static> = Criterion {
    path: "README.md",
    marker: LEAF_MARKER,
    guard: LEAF_ANCHOR,
};

/// A composite is measured against its children's recorded effects rather than
/// against a file, so it leaves the criterion slots empty.
const NO_CRITERION: Criterion<'static> = Criterion {
    path: "",
    marker: "",
    guard: "",
};

/// The two child nodes a composite composes; empty for a leaf, which has none.
const NO_CHILDREN: (&str, &str) = ("", "");
const BOTH_CHILDREN: (&str, &str) = ("1.1.1", "1.1.2");

fn run_node_verifier(
    directory: &Path,
    proof: &Path,
    depth: &str,
    children: (&str, &str),
    criterion: &Criterion,
) -> std::process::Output {
    Command::new(root().join(NODE_VERIFIER))
        .arg(directory)
        .arg(proof)
        .arg("1.1")
        .arg(depth)
        .arg(children.0)
        .arg(children.1)
        .arg(criterion.path)
        .arg(criterion.marker)
        .arg(criterion.guard)
        .output()
        .expect("run the committed node verifier")
}

#[test]
fn the_node_verifier_rejects_a_proof_without_a_repository_effect() {
    let (directory, proof) = node_verifier_fixture("proof-only");

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "missing_effect"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_accepts_a_non_hollow_proof_and_new_leaf_effect() {
    let (directory, proof) = node_verifier_fixture("leaf-effect");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=5\nnode_kind=leaf\nresult=The tracked source now declares pub max_depth: u8.\n",
    )
    .expect("write leaf effect");
    apply_leaf_change(&directory);

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_does_not_treat_source_generics_as_placeholders() {
    let (directory, proof) = node_verifier_fixture("source-generics");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=5\nnode_kind=leaf\nresult=Line 79 now reads pub extras: Vec<Self>,\n",
    )
    .expect("write leaf effect containing Rust generics");
    let target = directory.join("README.md");
    let source = fs::read_to_string(&target).expect("fixture source");
    fs::write(&target, format!("{source}pub extras: Vec<Self>,\n")).expect("apply the change");

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        NO_CHILDREN,
        &Criterion {
            path: "README.md",
            marker: "pub extras: Vec<Self>",
            guard: LEAF_ANCHOR,
        },
    );

    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_rejects_an_entire_placeholder_result() {
    let (directory, proof) = node_verifier_fixture("placeholder-result");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=5\nnode_kind=leaf\nresult=<task result goes here>\n",
    )
    .expect("write placeholder leaf effect");
    apply_leaf_change(&directory);

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "placeholder_effect_result"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_requires_both_children_in_a_composite_effect() {
    let (directory, proof) = node_verifier_fixture("composite-effect");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=2\nnode_kind=composite\nleft_child=1.1.1\nresult=The requested child results were composed into this checked effect.\n",
    )
    .expect("write incomplete composite effect");

    let output = run_node_verifier(&directory, &proof, "2", BOTH_CHILDREN, &NO_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "missing_right_child"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_rejects_a_composite_without_verified_child_effects() {
    let (directory, proof) = node_verifier_fixture("composite-without-effects");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=2\nnode_kind=composite\nleft_child=1.1.1\nright_child=1.1.2\nresult=Both named child tasks compose into this checked result.\n",
    )
    .expect("write structurally complete composite effect");

    let output = run_node_verifier(&directory, &proof, "2", BOTH_CHILDREN, &NO_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "missing_child_effect"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_rejects_a_composite_that_does_not_copy_a_child_result() {
    let (directory, proof) = node_verifier_fixture("composite-wrong-child-result");
    let left_result = "Left child verified the children storage field.";
    let right_result = "Right child verified the recursive depth bound.";
    commit_verified_child_effects(&directory, left_result, right_result);
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        format!(
            "node_path=1.1\nnode_depth=2\nnode_kind=composite\nleft_child=1.1.1\nright_child=1.1.2\nleft_result=An unrelated left result.\nright_result={right_result}\nresult=An unrelated left result; {right_result}\n"
        ),
    )
    .expect("write composite with wrong left result");

    let output = run_node_verifier(&directory, &proof, "2", BOTH_CHILDREN, &NO_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "unverified_left_child_result"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_accepts_a_composite_of_both_verified_child_results() {
    let (directory, proof) = node_verifier_fixture("verified-composite");
    let left_result = "Left child verified the children storage field.";
    let right_result = "Right child verified the recursive depth bound.";
    commit_verified_child_effects(&directory, left_result, right_result);
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        format!(
            "node_path=1.1\nnode_depth=2\nnode_kind=composite\nleft_child=1.1.1\nright_child=1.1.2\nleft_result={left_result}\nright_result={right_result}\nresult={left_result} {right_result}\n"
        ),
    )
    .expect("write verified composite effect");

    let output = run_node_verifier(&directory, &proof, "2", BOTH_CHILDREN, &NO_CRITERION);

    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_rejects_a_child_effect_modified_after_fixture_commit() {
    let (directory, proof) = node_verifier_fixture("modified-child-effect");
    let left_result = "Left child verified the children storage field.";
    let right_result = "Right child verified the recursive depth bound.";
    commit_verified_child_effects(&directory, left_result, right_result);
    fs::write(
        directory.join(".agent-ladder/verified-children/node-1.1.1.lino"),
        "node_path=1.1.1\nresult=The child result was changed after verification.\n",
    )
    .expect("modify committed child effect");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        format!(
            "node_path=1.1\nnode_depth=2\nnode_kind=composite\nleft_child=1.1.1\nright_child=1.1.2\nleft_result=The child result was changed after verification.\nright_result={right_result}\nresult=The child result was changed after verification. {right_result}\n"
        ),
    )
    .expect("write composite from modified child effect");

    let output = run_node_verifier(&directory, &proof, "2", BOTH_CHILDREN, &NO_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "modified_child_effect"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_rejects_a_delivery_status_as_the_task_result() {
    let (directory, proof) = node_verifier_fixture("delivery-status");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=5\nnode_kind=leaf\nresult=Recorded the findings in the requested proof file.\n",
    )
    .expect("write status-only effect");
    apply_leaf_change(&directory);

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "status_only_effect_result"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_node_verifier_rejects_a_leaf_effect_unrelated_to_its_external_criterion() {
    let (directory, proof) = node_verifier_fixture("unrelated-leaf-effect");
    let effect = directory.join("agent-ladder-effects/node-1.1.lino");
    fs::create_dir_all(effect.parent().expect("effect parent")).expect("create effect directory");
    fs::write(
        effect,
        "node_path=1.1\nnode_depth=5\nnode_kind=leaf\nresult=The package contains several unrelated source modules.\n",
    )
    .expect("write unrelated leaf effect");
    apply_leaf_change(&directory);

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "unverified_leaf_result"
    );
    let _ = fs::remove_dir_all(directory);
}

/// Writing a convincing effect file is not doing the work: with the tracked
/// source untouched the node has produced evidence and nothing else. This is
/// the exact shape every pre-#1069 leaf had, so it must now fail.
#[test]
fn the_node_verifier_rejects_a_leaf_effect_without_the_tracked_change() {
    let (directory, proof) = node_verifier_fixture("effect-without-change");
    write_leaf_effect(
        &directory,
        "The tracked source now declares pub max_depth: u8.",
    );

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "missing_leaf_change"
    );
    let _ = fs::remove_dir_all(directory);
}

/// A marker that is already committed would let a node pass by finding text
/// rather than by writing it, which is the observation contract again.
#[test]
fn the_node_verifier_rejects_a_change_marker_that_is_already_committed() {
    let (directory, proof) = node_verifier_fixture("preexisting-change");
    write_leaf_effect(
        &directory,
        "The source already contains storage_field=children.",
    );

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        NO_CHILDREN,
        &Criterion {
            path: "README.md",
            marker: LEAF_ANCHOR,
            guard: LEAF_ANCHOR,
        },
    );

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "preexisting_leaf_change"
    );
    let _ = fs::remove_dir_all(directory);
}

/// Replacing the file with its marker satisfies "the marker is present" while
/// destroying everything the marker was supposed to join.
#[test]
fn the_node_verifier_rejects_a_change_that_destroyed_its_anchor() {
    let (directory, proof) = node_verifier_fixture("destroyed-anchor");
    write_leaf_effect(
        &directory,
        "The tracked source now declares pub max_depth: u8.",
    );
    fs::write(directory.join("README.md"), format!("{LEAF_MARKER},\n")).expect("overwrite");

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "destroyed_leaf_anchor"
    );
    let _ = fs::remove_dir_all(directory);
}

/// "Change only that file" is part of the task, so a node that also edited an
/// unrelated tracked source has not done the task it was given.
#[test]
fn the_node_verifier_rejects_collateral_changes_to_other_sources() {
    let (directory, proof) = node_verifier_fixture("collateral-change");
    write_leaf_effect(
        &directory,
        "The tracked source now declares pub max_depth: u8.",
    );
    apply_leaf_change(&directory);
    fs::write(directory.join("NOTES.md"), "edited by mistake\n").expect("collateral edit");

    let output = run_node_verifier(&directory, &proof, "5", NO_CHILDREN, &LEAF_CRITERION);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "unexpected_tracked_changes"
    );
    let _ = fs::remove_dir_all(directory);
}

/// A Rust change a reviewer could not even parse is not a change.
#[test]
fn the_node_verifier_rejects_a_rust_change_that_no_longer_parses() {
    if Command::new("rustfmt").arg("--version").output().is_err() {
        return;
    }
    let (directory, proof) = node_verifier_fixture("unparsable-change");
    write_leaf_effect(&directory, "The struct now declares pub max_depth: u8.");
    fs::write(
        directory.join("fixture.rs"),
        "pub struct Node {\n    pub children: Vec<Node>,\n    pub max_depth: u8\n",
    )
    .expect("write unbalanced Rust");

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        NO_CHILDREN,
        &Criterion {
            path: "fixture.rs",
            marker: LEAF_MARKER,
            guard: "pub struct Node",
        },
    );

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "unparsable_leaf_change"
    );
    let _ = fs::remove_dir_all(directory);
}
