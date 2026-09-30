//! Issue #1182 (E147, R8): the duplication gate fails on a fixture with
//! two identical helper bodies in two files and passes on the baseline.
//!
//! The gate is a rust-script (`scripts/check-duplicate-functions.rs`), so
//! the test exercises it end to end the way CI does — through the command
//! — on a fixture tree with its own baseline, never touching the real
//! `data/meta/duplicate-functions-baseline.lino`.
//!
//! Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1182_`

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits inside the repository")
        .to_path_buf()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("issue-1182-gate-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("fixture directory");
    dir
}

const IDENTICAL_BODY: &str = r#"fn helper(value: u32) -> u32 {
    let doubled = value * 2;
    let shifted = doubled + 1;
    let folded = shifted ^ doubled;
    folded.wrapping_add(shifted)
}
"#;

const DIFFERENT_BODY: &str = r#"fn helper(value: u32) -> u32 {
    let doubled = value * 3;
    let shifted = doubled + 7;
    let folded = shifted ^ doubled;
    folded.wrapping_add(shifted)
}
"#;

fn write_source(dir: &Path, file: &str, body: &str) {
    fs::write(dir.join("src").join(file), body).expect("fixture source");
}

fn run_gate(dir: &Path, baseline: &Path) -> std::process::Output {
    Command::new("rust-script")
        .env("RUSTUP_TOOLCHAIN", "1.98.1")
        .args([
            repo_root()
                .join("scripts/check-duplicate-functions.rs")
                .display()
                .to_string()
                .as_str(),
            "--roots",
            dir.join("src").display().to_string().as_str(),
            "--baseline",
            baseline.display().to_string().as_str(),
        ])
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| panic!("rust-script runs the gate (CI depends on it): {error}"))
}

#[test]
fn two_identical_bodies_across_files_fail_without_a_baseline() {
    let dir = scratch("new");
    write_source(&dir, "a.rs", IDENTICAL_BODY);
    write_source(&dir, "b.rs", IDENTICAL_BODY);
    let baseline = dir.join("baseline.lino");
    fs::write(&baseline, "duplicate_functions_baseline\n").expect("empty baseline");
    let output = run_gate(&dir, &baseline);
    assert!(
        !output.status.success(),
        "the gate fails on a new cross-file duplicate group"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("NEW"),
        "the failure names the new group: {stderr}"
    );
}

#[test]
fn the_write_baseline_mode_makes_the_same_tree_pass() {
    let dir = scratch("baselined");
    write_source(&dir, "a.rs", IDENTICAL_BODY);
    write_source(&dir, "b.rs", IDENTICAL_BODY);
    let baseline = dir.join("baseline.lino");
    let write = Command::new("rust-script")
        .env("RUSTUP_TOOLCHAIN", "1.98.1")
        .args([
            repo_root()
                .join("scripts/check-duplicate-functions.rs")
                .display()
                .to_string()
                .as_str(),
            "--write-baseline",
            "--roots",
            dir.join("src").display().to_string().as_str(),
            "--baseline",
            baseline.display().to_string().as_str(),
        ])
        .current_dir(repo_root())
        .output()
        .expect("rust-script writes the baseline");
    assert!(
        write.status.success(),
        "--write-baseline succeeds: {}",
        String::from_utf8_lossy(&write.stderr)
    );
    let written = fs::read_to_string(&baseline).expect("baseline written");
    assert!(
        written.contains("duplicate_functions_baseline") && written.contains("digest "),
        "the baseline carries the group digests"
    );
    let check = run_gate(&dir, &baseline);
    assert!(
        check.status.success(),
        "the baselined tree passes: {}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn a_changed_body_is_not_a_duplicate_and_a_shrunk_baseline_fails() {
    let dir = scratch("diverged");
    write_source(&dir, "a.rs", IDENTICAL_BODY);
    write_source(&dir, "b.rs", DIFFERENT_BODY);
    let baseline = dir.join("baseline.lino");
    // A stale baseline: it still lists the group the divergence removed.
    fs::write(
        &baseline,
        "duplicate_functions_baseline\n  group\n    digest \"0123456789abcdef\"\n    site \"src/a.rs:1\" fn helper\n",
    )
    .expect("stale baseline");
    let output = run_gate(&dir, &baseline);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("shrink"),
        "a stale entry demands the shrink, not silence: {stderr}"
    );
    assert!(
        !stderr.contains("NEW"),
        "the diverged bodies are not a new group: {stderr}"
    );
}
