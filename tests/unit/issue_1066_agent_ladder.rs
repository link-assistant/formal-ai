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
const EXPERIMENT: &str = "experiments/issue_1066_self_development/reproduce-ladder-tree-generation.sh";

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
            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 6, "every node row has six fields: {line:?}");
            Node {
                path: fields[0].to_owned(),
                depth: fields[1]
                    .parse()
                    .unwrap_or_else(|_| panic!("node depth must be a number: {line:?}")),
                text: fields[2].to_owned(),
                criterion: fields[3].to_owned(),
                left: fields[4].to_owned(),
                right: fields[5].to_owned(),
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
            assert_eq!(node.criterion, "observable evidence exists");
        } else {
            // Exactly two children, named by the binary 1/2 convention, and both
            // of them are nodes the ladder will actually select.
            let prefix = if node.path == "R" {
                String::new()
            } else {
                format!("{}.", node.path)
            };
            assert_eq!(node.left, format!("{prefix}1"), "left child of {}", node.path);
            assert_eq!(node.right, format!("{prefix}2"), "right child of {}", node.path);
            assert!(paths.contains(&node.left), "missing {}", node.left);
            assert!(paths.contains(&node.right), "missing {}", node.right);
            assert_eq!(node.criterion, "all_children_pass");
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
fn every_ladder_node_can_be_selected_by_depth_and_by_path() {
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
        String::from_utf8(output.stdout)
            .expect("selected nodes are UTF-8")
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
        assert_eq!(
            select("all", &node.path),
            vec![node.path.clone()],
            "node {} must be selectable on its own",
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
