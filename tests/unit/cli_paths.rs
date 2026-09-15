use std::path::PathBuf;

#[path = "../../src/cli_paths.rs"]
mod implementation;

use implementation::resolve_root;

#[test]
fn a_relative_root_is_resolved_before_a_child_changes_directory() {
    let root = resolve_root(Some(PathBuf::from(".")), &PathBuf::from("."));
    assert!(root.is_absolute());
    assert_eq!(root, std::env::current_dir().expect("current directory"));
}

#[test]
fn an_absolute_root_keeps_its_identity() {
    let current = std::env::current_dir().expect("current directory");
    assert_eq!(
        resolve_root(Some(current.clone()), &PathBuf::from("ignored")),
        current
    );
}
