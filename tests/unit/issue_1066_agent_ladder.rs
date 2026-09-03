//! Issue #1066: the issue #1028 binary-tree ladder must actually run its tree.
//!
//! The ladder was rewritten from a flat 32-leaf list into a complete binary tree
//! in `eb16ec1d0`. The rewrite never executed a single node: the generator wrote
//! its rows with `'\t'.join(r)` while `depth` in the row is an `int`, so it died
//! with `TypeError: sequence item 1: expected str instance, int found` before
//! `selected.tsv` existed. Two further defects sat behind that crash — failure
//! rows written with `echo "$id\tFAIL\t…"`, which bash emits as one literal
//! backslash-t blob, and node instructions interpolated into a double-quoted
//! string where `\n` is not a newline.
//!
//! These tests execute the committed generator rather than describing it, so the
//! tree shape is proven from the harness the workflow runs.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LADDER: &str = "experiments/issue_1028_agent_cli_ladder/run.sh";
const NODE_VERIFIER: &str = "experiments/issue_1028_agent_cli_ladder/verify-node.sh";
const EXPERIMENT: &str =
    "experiments/issue_1066_self_development/reproduce-ladder-tree-generation.sh";

/// A complete binary tree of depth five: 1 + 2 + 4 + 8 + 16 + 32.
const NODE_COUNT: usize = 63;
const LEAF_COUNT: usize = 32;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// The body of a heredoc the ladder writes for itself, lifted from the script so
/// a test can never drift from what the workflow runs.
fn heredoc(script: &str, opening: &str, terminator: &str) -> String {
    let body = script
        .split_once(opening)
        .unwrap_or_else(|| panic!("ladder no longer opens `{opening}`"))
        .1;
    body.split_once(terminator)
        .unwrap_or_else(|| panic!("`{opening}` heredoc is not closed"))
        .0
        .to_owned()
}

fn python_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

#[derive(Debug)]
struct Node {
    path: String,
    depth: usize,
    text: String,
    criterion: String,
    left: String,
    right: String,
    criterion_path: String,
    criterion_marker: String,
    criterion_guard: String,
}

/// Run the committed generator over the committed leaves and read back the tree.
fn generate_tree(directory: &Path) -> Vec<Node> {
    let script = read(LADDER);
    let leaves = heredoc(&script, "cat > \"$OUT/leaves.tsv\" <<'EOF'\n", "\nEOF\n");
    let generator = heredoc(
        &script,
        "python3 - \"$OUT/leaves.tsv\" \"$NODES\" <<'PY'\n",
        "\nPY\n",
    );
    assert_eq!(
        leaves.lines().count(),
        LEAF_COUNT,
        "the ladder must formulate exactly {LEAF_COUNT} atomic leaves",
    );

    let leaves_path = directory.join("leaves.tsv");
    let generator_path = directory.join("generate.py");
    let tree_path = directory.join("tree.tsv");
    fs::write(&leaves_path, format!("{leaves}\n")).expect("write leaves");
    fs::write(&generator_path, generator).expect("write generator");

    let output = Command::new("python3")
        .arg(&generator_path)
        .arg(&leaves_path)
        .arg(&tree_path)
        .output()
        .expect("run the ladder tree generator");
    assert!(
        output.status.success(),
        "the ladder tree generator failed:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );

    fs::read_to_string(&tree_path)
        .expect("generated tree")
        .lines()
        .map(|line| {
            let mut fields = line.split('\t').collect::<Vec<_>>();
            assert!(
                (6..=9).contains(&fields.len()),
                "every node row has six required and at most three optional fields: {line:?}",
            );
            fields.resize(9, "");
            Node {
                path: fields[0].to_owned(),
                depth: fields[1]
                    .parse()
                    .unwrap_or_else(|_| panic!("node depth must be a number: {line:?}")),
                text: fields[2].to_owned(),
                criterion: fields[3].to_owned(),
                left: fields[4].to_owned(),
                right: fields[5].to_owned(),
                criterion_path: fields[6].to_owned(),
                criterion_marker: fields[7].to_owned(),
                criterion_guard: fields[8].to_owned(),
            }
        })
        .collect()
}

fn temporary_directory(label: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("issue-1066-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("temporary directory");
    directory
}

fn git(directory: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .output()
        .expect("run git in node-verifier fixture");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

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

fn run_node_verifier(
    directory: &Path,
    proof: &Path,
    depth: &str,
    left: &str,
    right: &str,
    criterion_path: &str,
    criterion_marker: &str,
    criterion_guard: &str,
) -> std::process::Output {
    Command::new(root().join(NODE_VERIFIER))
        .arg(directory)
        .arg(proof)
        .arg("1.1")
        .arg(depth)
        .arg(left)
        .arg(right)
        .arg(criterion_path)
        .arg(criterion_marker)
        .arg(criterion_guard)
        .output()
        .expect("run the committed node verifier")
}

#[test]
fn the_node_verifier_rejects_a_proof_without_a_repository_effect() {
    let (directory, proof) = node_verifier_fixture("proof-only");

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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
        "",
        "",
        "README.md",
        "pub extras: Vec<Self>",
        LEAF_ANCHOR,
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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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

    let output = run_node_verifier(&directory, &proof, "2", "1.1.1", "1.1.2", "", "", "");

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

    let output = run_node_verifier(&directory, &proof, "2", "1.1.1", "1.1.2", "", "", "");

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

    let output = run_node_verifier(&directory, &proof, "2", "1.1.1", "1.1.2", "", "", "");

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

    let output = run_node_verifier(&directory, &proof, "2", "1.1.1", "1.1.2", "", "", "");

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

    let output = run_node_verifier(&directory, &proof, "2", "1.1.1", "1.1.2", "", "", "");

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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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
        "",
        "",
        "README.md",
        LEAF_ANCHOR,
        LEAF_ANCHOR,
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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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

    let output = run_node_verifier(
        &directory,
        &proof,
        "5",
        "",
        "",
        "README.md",
        LEAF_MARKER,
        LEAF_ANCHOR,
    );

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
        "",
        "",
        "fixture.rs",
        LEAF_MARKER,
        "pub struct Node",
    );

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "unparsable_leaf_change"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn the_ladder_clears_prior_node_artifacts_before_a_replay() {
    let script = read(LADDER);
    let run_one = script
        .split_once("run_one() {\n")
        .expect("ladder defines run_one")
        .1
        .split_once("\n  setsid env ")
        .expect("ladder starts the Formal AI server after node setup")
        .0;

    for artifact in [
        "agent-stream.jsonl",
        "agent-stderr.log",
        "formal-ai.log",
        "proof.md",
        "effect.lino",
    ] {
        assert!(
            run_one.contains(&format!("rm -f \"$session_dir/{artifact}\"")),
            "replaying a node must remove stale {artifact} before collecting new evidence",
        );
    }
}

#[test]
fn the_ladder_keeps_server_memory_out_of_agent_authored_effects() {
    let script = read(LADDER);

    assert!(
        script.contains("FORMAL_AI_MEMORY_PATH=\"$work/.git/formal-ai-memory/memory.lino\""),
        "server-private .lino and binary .links state must stay below .git so Agent snapshots cannot mistake it for an authored repository effect",
    );
    assert!(
        !script.contains("FORMAL_AI_MEMORY_PATH=\"$work/.agent-ladder/memory.lino\""),
        "the Agent worktree must contain only fixture inputs and Agent-authored effects",
    );
}

#[test]
fn the_shared_agent_harness_keeps_server_memory_out_of_agent_authored_effects() {
    let script = read("experiments/agent_cli_e2e/run_agent_cli.sh");

    assert!(
        script.contains("FORMAL_AI_MEMORY_PATH=\"$SERVER_STATE/memory.lino\""),
        "server-private .lino and binary .links state must live outside the Agent workspace",
    );
    assert!(
        !script.contains("FORMAL_AI_MEMORY_PATH=\"$WORKDIR/memory.lino\""),
        "the Agent worktree must contain only fixture inputs and Agent-authored effects",
    );
}

#[test]
fn the_ladder_generates_a_complete_binary_tree_of_sixty_three_nodes() {
    if !python_available() {
        eprintln!("skipping: python3 is not installed on this host");
        return;
    }
    let directory = temporary_directory("ladder-tree");
    let nodes = generate_tree(&directory);

    assert_eq!(nodes.len(), NODE_COUNT);
    let mut per_depth = BTreeMap::new();
    for node in &nodes {
        *per_depth.entry(node.depth).or_insert(0_usize) += 1;
    }
    assert_eq!(
        per_depth,
        BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)]),
        "the ladder must double at every level",
    );

    let paths = nodes
        .iter()
        .map(|node| node.path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(paths.len(), NODE_COUNT, "every node path is unique");
    assert_eq!(nodes[0].path, "R", "the root is emitted first");

    for node in &nodes {
        if node.depth == 5 {
            assert_eq!(
                (node.left.as_str(), node.right.as_str()),
                ("", ""),
                "leaf {} must not claim children",
                node.path,
            );
            assert_eq!(node.criterion, "tracked_source_change");
            assert!(
                !node.criterion_path.is_empty(),
                "{} criterion path",
                node.path
            );
            assert!(
                !node.criterion_marker.is_empty(),
                "{} criterion marker",
                node.path
            );
            assert!(
                !node.criterion_guard.is_empty(),
                "{} criterion guard",
                node.path
            );
            // A change contract, not an observation contract: the marker is
            // what the node has to introduce, so finding it already committed
            // would make the leaf passable without changing anything.
            let criterion_source = read(&node.criterion_path);
            assert!(
                !criterion_source.contains(&node.criterion_marker),
                "{} change marker {:?} is already present in {}",
                node.path,
                node.criterion_marker,
                node.criterion_path,
            );
            assert!(
                criterion_source.contains(&node.criterion_guard),
                "{} anchor {:?} is absent from {}",
                node.path,
                node.criterion_guard,
                node.criterion_path,
            );
        } else {
            // Exactly two children, named by the binary 1/2 convention, and both
            // of them are nodes the ladder will actually select.
            let prefix = if node.path == "R" {
                String::new()
            } else {
                format!("{}.", node.path)
            };
            assert_eq!(
                node.left,
                format!("{prefix}1"),
                "left child of {}",
                node.path
            );
            assert_eq!(
                node.right,
                format!("{prefix}2"),
                "right child of {}",
                node.path
            );
            assert!(paths.contains(&node.left), "missing {}", node.left);
            assert!(paths.contains(&node.right), "missing {}", node.right);
            assert_eq!(node.criterion, "new_composite_effect");
            assert!(
                node.criterion_path.is_empty(),
                "{} criterion path",
                node.path
            );
            assert!(
                node.criterion_marker.is_empty(),
                "{} criterion marker",
                node.path
            );
            assert!(
                node.criterion_guard.is_empty(),
                "{} criterion guard",
                node.path
            );
        }
    }

    let leaves = nodes
        .iter()
        .filter(|node| node.depth == 5)
        .map(|node| node.text.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        leaves.len(),
        LEAF_COUNT,
        "the {LEAF_COUNT} leaves must be distinctly worded",
    );

    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn generated_tree_rows_have_no_trailing_whitespace() {
    if !python_available() {
        eprintln!("skipping: python3 is not installed on this host");
        return;
    }
    let directory = temporary_directory("ladder-tree-whitespace");
    let _ = generate_tree(&directory);
    let tree = fs::read_to_string(directory.join("tree.tsv")).expect("generated tree");

    for (index, line) in tree.lines().enumerate() {
        assert_eq!(
            line.trim_end(),
            line,
            "generated tree row {} has trailing whitespace",
            index + 1,
        );
    }

    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn every_ladder_node_can_be_selected_with_its_dependency_subtree() {
    if !python_available() {
        eprintln!("skipping: python3 is not installed on this host");
        return;
    }
    let directory = temporary_directory("ladder-select");
    let nodes = generate_tree(&directory);
    let script = read(LADDER);
    let selector = heredoc(
        &script,
        "python3 - \"$NODES\" \"$TREE_DEPTH\" \"$NODE_FILTER\" > \"$OUT/selected.tsv\" <<'PY'\n",
        "\nPY\n",
    );
    let selector_path = directory.join("select.py");
    let tree_path = directory.join("tree.tsv");
    fs::write(&selector_path, selector).expect("write selector");

    let select = |mode: &str, filter: &str| -> Vec<String> {
        let output = Command::new("python3")
            .arg(&selector_path)
            .arg(&tree_path)
            .arg(mode)
            .arg(filter)
            .output()
            .expect("run the ladder node selector");
        assert!(
            output.status.success(),
            "selecting {mode}/{filter} failed:\n{}",
            String::from_utf8_lossy(&output.stderr),
        );
        let selected = String::from_utf8(output.stdout).expect("selected nodes are UTF-8");
        assert!(
            selected.lines().all(|line| line.trim_end() == line),
            "selected TSV rows must not have trailing whitespace: {selected:?}",
        );
        selected
            .lines()
            .map(|line| line.split('\t').next().expect("node path").to_owned())
            .collect()
    };

    for (depth, expected) in [(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)] {
        assert_eq!(
            select(&depth.to_string(), "").len(),
            expected,
            "depth {depth} must select {expected} node(s)",
        );
    }
    // `all` is the full tree, smallest tasks first, so a capability gap is found
    // on an atomic leaf before an expensive composite node is attempted.
    let all = select("all", "");
    assert_eq!(all.len(), NODE_COUNT);
    assert_eq!(all.last().map(String::as_str), Some("R"));

    for node in &nodes {
        let prefix = format!("{}.", node.path);
        let expected = all
            .iter()
            .filter(|candidate| {
                node.path == "R" || *candidate == &node.path || candidate.starts_with(&prefix)
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            select("all", &node.path),
            expected,
            "node {} must be selected after all descendants whose verified effects it consumes",
            node.path,
        );
    }

    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn the_ladder_writes_tab_separated_outcomes_and_real_prompt_paragraphs() {
    let script = read(LADDER);
    // `echo "$id\tFAIL\t…"` emits a literal backslash-t in bash, which collapses
    // the whole outcome row into one field.
    assert!(
        !script.contains("echo \"$id\\t"),
        "the ladder must not write run.log rows with echo",
    );
    for row in [
        "printf '%s\\tFAIL\\tformal_ai_server_start\\n'",
        "printf '%s\\tFAIL\\tagent_exit_%s\\n'",
        "printf '%s\\tFAIL\\tmissing_proof\\n'",
        "printf '%s\\tFAIL\\tbad_proof_marker\\n'",
        "printf '%s\\tPASS\\tdepth=%s\\n'",
    ] {
        assert!(script.contains(row), "run.log row missing: {row}");
    }
    // A double-quoted "\n" is not a newline either, so the node instructions
    // have to be built rather than interpolated.
    assert!(
        !script.contains("--prompt \"$prompt\\n"),
        "the ladder must not interpolate node instructions into a double-quoted string",
    );
    assert!(
        script.contains("printf -v full_prompt"),
        "the ladder must build its node prompt with printf",
    );
    assert!(
        script.contains("--prompt \"$full_prompt\""),
        "the ladder must pass the built prompt to the Agent CLI",
    );
}

#[test]
fn the_ladder_keeps_every_tool_turn_on_the_formal_ai_session() {
    let script = read(LADDER);

    assert!(
        script.contains(concat!(
            "\"$AGENT\" --no-summarize-session --compaction-model same \\\n",
            "      --model formalai/formal-ai",
        )),
        "the real Agent run must not replace the task with an unrelated session summary",
    );
}

#[test]
fn the_composite_contract_extracts_raw_results_from_both_children() {
    let script = read(LADDER);

    assert!(
        script.contains("Inspect both files before writing anything"),
        "a composite must consume both immutable child effects",
    );
    assert!(
        script.contains(r#"sed -n "s/^result=//p""#),
        "the contract must extract the raw result field rather than a tool-rendered file view",
    );
    assert!(
        script.contains("Do not copy tool-rendered line numbers"),
        "line-number decorations must never become part of a composed child result",
    );
}

#[test]
fn the_ladder_reproduction_experiment_is_committed_and_executable() {
    for script in [LADDER, EXPERIMENT] {
        let path = root().join(script);
        assert!(path.is_file(), "{script} is missing");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(&path).expect("metadata").permissions().mode();
            assert!(mode & 0o111 != 0, "{script} must be committed executable");
        }
    }
}

#[test]
fn the_qualifying_pr_dry_run_selects_one_session_without_a_pipefail_sigpipe() {
    let script = read("experiments/issue_1066_qualifying_pr/dry-run.sh");

    assert!(
        script.contains("grep -h -m1 -o 'ses_[A-Za-z0-9]*'"),
        "grep must stop after its own first match so pipefail cannot turn head's early exit into status 141",
    );
    assert!(
        !script.contains("| head -1"),
        "the session selector must not use an early-closing head pipeline under pipefail",
    );
}
