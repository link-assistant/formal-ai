//! Source-placement tests for `src/repository_workspace/` (issue #1138, plan 03).
//!
//! The mirror exists to reach what `src/` keeps private, so a new module tree
//! has to obey the same two rules every other module obeys: the files live where
//! the convention says they live, and none of them crosses the file-size
//! ceiling `scripts/check-file-size.rs` enforces.

use std::fs;
use std::path::{Path, PathBuf};

/// The hard limit `scripts/check-file-size.rs` applies to Rust files.
const MAX_RUST_LINES: usize = 1_000;

/// Every file the module tree is declared to consist of (plan 03 "New module
/// tree"), so a file added outside the declared set is noticed.
const DECLARED: &[&str] = &[
    "src/repository_workspace/mod.rs",
    "src/repository_workspace/clone.rs",
    "src/repository_workspace/locate.rs",
    "src/repository_workspace/edit.rs",
    "src/repository_workspace/verify.rs",
    "src/repository_workspace/diff.rs",
    "src/cli_solve.rs",
];

fn repo_root() -> PathBuf {
    // `tests/source/` mirrors `src/`, so the manifest directory is the root.
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn module_files_are_where_the_convention_says() {
    for relative in DECLARED {
        assert!(
            repo_root().join(relative).is_file(),
            "the repository-workspace protocol declares {relative}, which does not exist"
        );
    }

    let directory = repo_root().join("src/repository_workspace");
    let mut present: Vec<String> = fs::read_dir(&directory)
        .expect("the module directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| {
            format!(
                "src/repository_workspace/{}",
                entry.file_name().to_string_lossy()
            )
        })
        .collect();
    present.sort();

    let mut declared: Vec<String> = DECLARED
        .iter()
        .filter(|path| path.starts_with("src/repository_workspace/"))
        .map(|path| (*path).to_owned())
        .collect();
    declared.sort();

    assert_eq!(
        present, declared,
        "the module tree must hold exactly the files the plan declares"
    );
}

#[test]
fn module_files_stay_under_the_size_ceiling() {
    for relative in DECLARED {
        let text = fs::read_to_string(repo_root().join(relative))
            .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"));
        let lines = text.lines().count();
        assert!(
            lines <= MAX_RUST_LINES,
            "{relative} has {lines} lines, over the {MAX_RUST_LINES}-line ceiling"
        );
    }
}
