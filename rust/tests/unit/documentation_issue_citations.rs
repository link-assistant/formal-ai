//! Plan 11's consolidated issue-citation gate.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

#[test]
fn issue_citations_match_the_committed_issue_state_snapshot() {
    let output = Command::new("rust-script")
        .arg("scripts/check-issue-citations.rs")
        .current_dir(repo_root())
        .output()
        .expect("run the issue-citation checker");
    assert!(
        output.status.success(),
        "issue citations must match the committed snapshot\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
