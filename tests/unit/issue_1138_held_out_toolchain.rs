//! Issue #1138 B6 (plan 06, L10): the held-out toolchains are genuinely held out.
//!
//! `kotlinc` and `scalac` already appear in the language catalogue and would
//! leak into any recovery test written around them. The held-out programs are
//! therefore `zig` (family 1) and `gleam` (family 2), and this test is the guard
//! that keeps them held out: the moment either name is written into `src`,
//! `data`, `scripts` or `.github`, the recovery result stops being evidence of
//! generalization and this test says so.

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// The two programs plan 06's families are written around.
const HELD_OUT: &[&str] = &["zig", "gleam"];

/// Where a leak would matter. Test files and case-study prose are where these
/// names are allowed to live.
const SCANNED: &[&str] = &["src", "data", "scripts", ".github"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn the_held_out_program_is_absent_from_the_repository() {
    let root = repo_root();
    let mut leaks: Vec<String> = Vec::new();

    for directory in SCANNED {
        let scan_root = root.join(directory);
        if !scan_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&scan_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let Ok(text) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let lowered = text.to_lowercase();
            for program in HELD_OUT {
                if lowered.contains(program) {
                    leaks.push(format!(
                        "{} names the held-out program `{program}`",
                        entry
                            .path()
                            .strip_prefix(&root)
                            .unwrap_or(entry.path())
                            .display()
                    ));
                }
            }
        }
    }

    leaks.sort_unstable();
    leaks.dedup();
    assert!(
        leaks.is_empty(),
        "a held-out toolchain that the repository already names proves nothing: {leaks:?}"
    );
}
