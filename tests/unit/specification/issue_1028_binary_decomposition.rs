//! Issue #1028 regression for the recursive binary task-decomposition contract.

use formal_ai::task_decomposition::{SubTask, decompose_task, task_decomposition_contract};

#[test]
fn the_reviewed_decomposition_contract_requires_binary_recursion() {
    let contract = task_decomposition_contract().expect("the decomposition contract must parse");
    assert!(contract.binary.contains("exactly two children"));
    assert!(contract.binary.contains("1, 2, 4, 8, 16, 32"));
}

#[test]
fn actual_formal_ai_decomposition_is_binary_at_every_supported_depth() {
    let task = "Solve issue #1028: improve the retry scheduling behavior, add regression coverage, document the result, and verify the complete implementation.";

    for depth in 1..=5 {
        let decomposition = decompose_task(task, depth);
        assert!(decomposition.depth_bound_reached(), "depth {depth} must exercise the requested boundary");
        assert_binary_node(&decomposition.root, depth);

        let expected_leaves = 1usize << depth;
        assert_eq!(decomposition.leaves().len(), expected_leaves, "depth {depth} must expose {expected_leaves} leaves");
        assert_eq!(decomposition.root.node_count(), (1usize << (depth + 1)) - 1, "depth {depth} must form a complete binary tree");
    }
}

fn assert_binary_node(node: &SubTask, max_depth: u8) {
    assert!(!node.id.is_empty(), "every node needs an id");
    assert!(!node.text.trim().is_empty(), "every node needs a task formulation");
    assert!(!node.completion_criterion.trim().is_empty(), "every node needs a completion criterion");

    match node.children.as_slice() {
        [] => {
            assert_eq!(node.depth, max_depth, "a non-empty supported decomposition must end at the requested depth");
            assert!(node.is_independently_checkable(), "every leaf must be independently checkable");
        }
        [left, right] => {
            assert_eq!(left.path, format!("{}.1", node.path));
            assert_eq!(right.path, format!("{}.2", node.path));
            assert_eq!(left.depth, node.depth + 1);
            assert_eq!(right.depth, node.depth + 1);
            assert_binary_node(left, max_depth);
            assert_binary_node(right, max_depth);
        }
        children => panic!("node {} has {} children; binary decomposition requires exactly two", node.path, children.len()),
    }
}
