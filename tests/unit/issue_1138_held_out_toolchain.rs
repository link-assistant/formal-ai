//! Issue #1138 B6 (plan 06, L10): the held-out toolchains are genuinely held out.
//!
//! `kotlinc` and `scalac` already appear in the language catalogue and would
//! leak into any recovery test written around them. The held-out programs are
//! therefore `zig` (family 1) and `gleam` (family 2), and this test is the guard
//! that keeps them held out: the moment either name is written into `src`,
//! `data/seed`, `data/meta`, `scripts` or `.github`, the recovery result stops
//! being evidence of generalization and this test says so.
//!
//! **Why the scan names four data directories rather than `data` whole.** As
//! written in wave T the guard scanned all of `data/`, and wave F then committed
//! `data/benchmarks/self-use-prerequisite.lino`, whose held-out corpus names
//! both programs *by design* and which
//! `tests/unit/issue_1138_self_use_toolchain.rs` requires verbatim. Two
//! committed specifications contradicted each other. Plan 14's wave T settles it
//! in the memorization tests' own words: held-out prompts live in
//! `data/benchmarks/`, and the memorization scans exclude that directory —
//! a corpus the engine never reads is the one place a held-out name belongs.
//! The guard is therefore narrowed to what the runtime actually reads and what
//! CI actually runs, and to nothing else. Nothing else about it is weakened: the
//! same two names, the same substring match, the same empty-leaks assertion
//! (plan 06 L10).

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// The two programs plan 06's families are written around.
const HELD_OUT: &[&str] = &["zig", "gleam"];

/// Where a leak would matter: what the runtime reads (`src`, `data/seed`,
/// `data/meta`) and what CI runs (`scripts`, `.github`). Test files, case-study
/// prose and the held-out corpora under `data/benchmarks/` are where these names
/// are allowed to live.
const SCANNED: &[&str] = &["src", "data/seed", "data/meta", "scripts", ".github"];

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
