//! Issue #1028 regression for the binary recursive task-decomposition contract.

use formal_ai::task_decomposition::task_decomposition_contract;

#[test]
fn the_reviewed_decomposition_contract_requires_binary_recursion() {
    let contract = task_decomposition_contract().expect("the decomposition contract must parse");
    assert!(
        contract.binary.contains("exactly two children"),
        "the reviewed contract must make binary recursion explicit: {}",
        contract.binary
    );
    assert!(
        contract.binary.contains("1, 2, 4, 8, 16, 32"),
        "the reviewed contract must describe power-of-two layers: {}",
        contract.binary
    );
}
