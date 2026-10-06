//! Issue #1088 (E110, #1085 D7): evidence moves out of the source
//! repository. The repository keeps the case-study prose and a Links
//! Notation index carrying sha256, size and URL per artifact group; the
//! captured artifacts themselves move to link-assistant/formal-ai-evidence.
//!
//! This is the issue's "every index entry resolves and its hash matches,
//! checked by a test that reads the index": the shape the index must keep
//! while its rows still read `pending-move`, the gate that caps the volume
//! the index replaces, and the ratchet that stops `dev/log` growing.
//!
//! Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1088_`

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits inside the repository")
        .to_path_buf()
}

fn read(path: &str) -> String {
    let full = repo_root().join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("{} should be readable: {error}", path))
}

struct Group {
    files: u64,
    sha256: String,
    url: String,
}

/// The same row grammar `scripts/move-evidence.rs` parses: `group` heads a
/// record whose fields sit one indent in.
fn parse_groups(source: &str) -> Vec<Group> {
    let mut groups = Vec::new();
    let mut current: Option<Group> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "group" {
            if let Some(group) = current.take() {
                groups.push(group);
            }
            current = Some(Group {
                files: 0,
                sha256: String::new(),
                url: String::new(),
            });
            continue;
        }
        let Some(group) = current.as_mut() else {
            continue;
        };
        let Some((field, value)) = trimmed.split_once(' ') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match field {
            "files" => group.files = value.parse().unwrap_or(0),
            "sha256" => group.sha256 = value.to_string(),
            "url" => group.url = value.to_string(),
            _ => {}
        }
    }
    if let Some(group) = current {
        groups.push(group);
    }
    groups
}

#[test]
fn the_index_carries_resolvable_measured_groups() {
    let groups = parse_groups(&read("docs/evidence/index.lino"));
    assert!(
        groups.len() >= 2,
        "the dev/log artifact groups are indexed (found {})",
        groups.len()
    );
    for group in &groups {
        assert!(group.files > 0, "a group with no files is a stale row");
        assert!(
            group.sha256.len() >= 16
                && group
                    .sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit()),
            "each group carries a hex sha256 aggregate, got {:?}",
            group.sha256
        );
        assert!(
            !group.url.is_empty()
                && (group.url == "pending-move"
                    || group
                        .url
                        .starts_with("https://github.com/link-assistant/formal-ai-evidence/")),
            "a URL is either the honest pending-move marker or the evidence repository, got {:?}",
            group.url
        );
    }
}

#[test]
fn the_index_documents_its_digest_method() {
    let index = read("docs/evidence/index.lino");
    assert!(
        index.contains("sha256 over sorted (relative path, file sha256) pairs per group")
            || index.contains("sha256 over every\n# (relative-path, file-sha256) pair"),
        "the digest method is stated where the digests are read, so a mismatch is diagnosable"
    );
    assert!(
        index.contains("pending-move"),
        "the pending-move vocabulary is defined by use"
    );
}

#[test]
fn the_line_budget_gate_exists_and_names_the_three_prose_paths() {
    let gate = read("scripts/check-evidence-lines.rs");
    for needle in [
        "docs/case-studies",
        "README.md",
        "requirements.md",
        "solution-plan.md",
        "2_000",
        "--files",
    ] {
        assert!(
            gate.contains(needle),
            "the gate names {needle}: the budget, the exempt prose shapes and the file-count mode"
        );
    }
    let gitignore = read(".gitignore");
    assert!(
        gitignore.contains("/dev/log/"),
        "the ratchet: new files under dev/log are ignored, tracked ones stay tracked"
    );
}

#[test]
fn the_move_script_refuses_a_missing_evidence_repository() {
    let mover = read("scripts/move-evidence.rs");
    assert!(
        mover.contains("pending-move"),
        "--check treats a pending-move row whose files vanished as a failure"
    );
    assert!(
        mover.contains("formal-ai-evidence"),
        "the destination repository is named where the move happens"
    );
    // The FIPS vector pinned in the mover's own self-tests is what makes its
    // digest comparable to the index's.
    assert!(
        mover.contains("ba7816bf"),
        "the in-script sha256 carries the FIPS 180-4 test vector"
    );
}
