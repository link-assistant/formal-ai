//! Issue #1138 B7 (plan 03, L3): the repository allowlist is reviewable data.
//!
//! The default-deny arm survives verbatim; what changes is that widening the set
//! is an edit to `data/seed/repository-command-allowlist.lino` rather than an
//! edit to a hard-coded list in Rust. A program with no row is refused, and so
//! is a listed program used with an unlisted subcommand.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::repository_workspace::{allows, command_allowlist};

const SEED: &str = "data/seed/repository-command-allowlist.lino";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The programs the seed document names, read by the test rather than by the
/// code under test.
fn seed_programs() -> BTreeSet<String> {
    let text = fs::read_to_string(repo_root().join(SEED))
        .unwrap_or_else(|error| panic!("{SEED} should be readable: {error}"));
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("program ").map(|value| {
                value
                    .trim()
                    .trim_matches('"')
                    .to_owned()
            })
        })
        .collect()
}

/// A program with no row is refused, exactly as it is today.
#[test]
fn an_unlisted_program_is_still_refused() {
    assert!(
        !allows("curl", &["https://example.org"]),
        "the default-deny arm must survive: curl has no allowlist row"
    );
    assert!(
        !seed_programs().contains("curl"),
        "the seed table must not quietly gain a network fetcher"
    );
}

/// `git clone` is allowed and `git push` is not: a row is a program *and* a
/// subcommand, never a bare program.
#[test]
fn an_unlisted_git_subcommand_is_refused() {
    assert!(
        allows("git", &["clone", "--no-checkout", "owner/name", "/tmp/root"]),
        "cloning is how a repository task starts"
    );
    assert!(
        !allows("git", &["push", "origin", "main"]),
        "a repository task may never push; the allowlist is per subcommand"
    );
}

/// The set of allowed programs comes from the seed table, not from Rust.
#[test]
fn allowlist_rows_come_from_seed_not_from_rust() {
    let from_code: BTreeSet<String> = command_allowlist()
        .into_iter()
        .map(|row| row.program)
        .collect();
    assert_eq!(
        from_code,
        seed_programs(),
        "the allowlist the code enforces must be exactly the seed table's programs"
    );
    assert!(
        !from_code.is_empty(),
        "an empty allowlist would make the protocol unable to clone anything"
    );
    for row in command_allowlist() {
        assert!(
            row.subcommand.is_some(),
            "row `{}` must name a subcommand so the grant is not a whole program",
            row.id
        );
    }
}
