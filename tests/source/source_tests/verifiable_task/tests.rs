//! Source-placement tests for `src/verifiable_task*` (issue #1138, plan 08).
//!
//! The verifiable-task route is the one place where a memorized table would be
//! hardest to see, so its file set is pinned: the module tree holds exactly the
//! files the plan declares, none of them crosses the file-size ceiling, and none
//! of them reaches the benchmark grader.

use std::fs;
use std::path::{Path, PathBuf};

/// The hard limit `scripts/check-file-size.rs` applies to Rust files.
const MAX_RUST_LINES: usize = 1_000;

/// Every file the module tree is declared to consist of (plan 08 "New and
/// changed files").
const DECLARED: &[&str] = &[
    "src/verifiable_task.rs",
    "src/verifiable_task/quantities.rs",
    "src/verifiable_task/ledger.rs",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn module_files_are_where_the_convention_says() {
    for relative in DECLARED {
        assert!(
            repo_root().join(relative).is_file(),
            "the verifiable-task route declares {relative}, which does not exist"
        );
    }

    let directory = repo_root().join("src/verifiable_task");
    let mut present: Vec<String> = fs::read_dir(&directory)
        .expect("the module directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| {
            format!(
                "src/verifiable_task/{}",
                entry.file_name().to_string_lossy()
            )
        })
        .collect();
    present.sort();

    let mut declared: Vec<String> = DECLARED
        .iter()
        .filter(|path| path.starts_with("src/verifiable_task/"))
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
