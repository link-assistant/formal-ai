//! Issue #991 R991-8: no file is a shared append point that two branches must
//! both edit. The policy is a reviewed registry (`data/meta/merge-conflict-policy.lino`)
//! measured from the merge history (`data/meta/merge-conflict-ledger.lino`), and
//! `scripts/check-merge-conflict-policy.rs` fails while the registry and
//! `.gitattributes` disagree. These tests pin the parts that make the probability
//! of a conflict structural: CI runs the checker and its own tests, every
//! union-merged path is declared in the registry, and the formerly shared lists
//! are directories of one file per entry.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

#[test]
fn ci_runs_the_merge_conflict_policy_checker_and_its_own_tests() {
    let surface = crate::ci_gates::ci_surface();
    for command in [
        "rust-script --test scripts/check-merge-conflict-policy.rs",
        "rust-script scripts/check-merge-conflict-policy.rs",
    ] {
        assert!(
            surface.contains(command),
            "CI must run `{command}` or the policy is a convention again"
        );
    }
    let policy = read("data/meta/merge-conflict-policy.lino");
    assert!(policy.contains("verify \"rust-script scripts/check-merge-conflict-policy.rs\""));
    assert!(policy.contains("regenerate_all \"bash scripts/regenerate-derived-artifacts.sh\""));
    for relative in [
        "data/meta/merge-conflict-ledger.lino",
        "scripts/analyze-merge-conflicts.py",
        "scripts/regenerate-derived-artifacts.sh",
    ] {
        assert!(root().join(relative).is_file(), "{relative} must exist");
    }
}

#[test]
fn every_union_merged_path_is_declared_in_the_reviewed_registry() {
    let policy = read("data/meta/merge-conflict-policy.lino");
    let attributes = read(".gitattributes");
    let union_paths: Vec<&str> = attributes
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter(|line| line.split_whitespace().any(|field| field == "merge=union"))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert!(
        !union_paths.is_empty(),
        ".gitattributes must declare the union-merged paths"
    );
    for path in union_paths {
        assert!(
            policy.contains(&format!("path \"{path}\"")),
            "`{path}` is union-merged in .gitattributes but has no reviewed entry in \
             data/meta/merge-conflict-policy.lino"
        );
    }
}

#[test]
fn shared_lists_are_one_file_per_entry() {
    for directory in [
        "docs/requirements",
        "data/meta/ci-gates",
        "data/meta/worker-line-budget",
    ] {
        let entries = fs::read_dir(root().join(directory))
            .unwrap_or_else(|error| panic!("{directory} should be a directory: {error}"))
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .count();
        assert!(
            entries > 1,
            "{directory} must hold one file per entry, found {entries}"
        );
    }
}
