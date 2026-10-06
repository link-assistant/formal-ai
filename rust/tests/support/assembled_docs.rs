//! Readers for the two accumulating documents the repository keeps as several
//! size-capped files.
//!
//! No maintained file may exceed 1500 lines, so:
//!
//! - `REQUIREMENTS.md` is an index; the assembled requirement register is
//!   `docs/requirements/assembled/part-NN.md`, written in order by
//!   `rust-script scripts/assemble-requirements.rs --write`;
//! - `CHANGELOG.md` keeps the newest releases, and older ones roll into
//!   `docs/changelog/archive-NN.md`, written by
//!   `node experiments/issue_711_rebuild_changelog.mjs --write`.
//!
//! A test that asks "does the requirements document (or the changelog) say X"
//! reads the whole set through these helpers, never one file of it.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// Where the assembled requirement register's parts live.
pub const REQUIREMENT_PARTS: &str = "docs/requirements/assembled";
/// Where releases rolled out of `CHANGELOG.md` live.
pub const CHANGELOG_ARCHIVE: &str = "docs/changelog";

/// The repository root: the crate lives in `rust/`.
#[must_use]
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

/// The `<prefix>NN.md` files of `directory`, in file-name order. A missing
/// directory has none.
fn numbered_files(directory: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(directory)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with(prefix) && name.ends_with(".md"))
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

/// The whole assembled requirement register under `root`: every part, in order.
#[must_use]
pub fn requirements_at<P: AsRef<Path> + ?Sized>(root: &P) -> String {
    let parts = numbered_files(&root.as_ref().join(REQUIREMENT_PARTS), "part-");
    assert!(
        !parts.is_empty(),
        "{REQUIREMENT_PARTS}/ holds no part-NN.md files; run \
         `rust-script scripts/assemble-requirements.rs --write`"
    );
    parts
        .iter()
        .map(|part| read(part))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The whole assembled requirement register of this repository.
#[must_use]
pub fn requirements() -> String {
    requirements_at(&repository_root())
}

/// The whole changelog under `root`: `CHANGELOG.md`, then its archive files,
/// newest first.
#[must_use]
pub fn changelog_at<P: AsRef<Path> + ?Sized>(root: &P) -> String {
    let root = root.as_ref();
    let mut text = read(&root.join("CHANGELOG.md"));
    for archive in numbered_files(&root.join(CHANGELOG_ARCHIVE), "archive-")
        .iter()
        .rev()
    {
        text.push('\n');
        text.push_str(&read(archive));
    }
    text
}

/// The whole changelog of this repository.
#[must_use]
pub fn changelog() -> String {
    changelog_at(&repository_root())
}
