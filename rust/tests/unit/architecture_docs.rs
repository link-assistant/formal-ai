//! `ARCHITECTURE.md` is a concise overview plus table of contents; the detailed
//! sections live in `docs/architecture/<topic>.md`. Documentation contracts that
//! pin architecture prose read the overview and every topic file together, so a
//! claim stays covered wherever in the architecture tree it is written.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
}

/// `ARCHITECTURE.md` first, then each `docs/architecture/*.md` topic file in
/// name order (repository-relative).
pub fn architecture_paths() -> Vec<PathBuf> {
    let topics = repo_root().join("docs/architecture");
    let mut files: Vec<PathBuf> = fs::read_dir(&topics)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", topics.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .map(|path| {
            path.strip_prefix(repo_root())
                .expect("topic files sit under the repository root")
                .to_path_buf()
        })
        .collect();
    files.sort();
    files.insert(0, PathBuf::from("ARCHITECTURE.md"));
    files
}

/// The overview and every topic file, concatenated.
pub fn read_all() -> String {
    architecture_paths()
        .iter()
        .map(|relative| {
            let path = repo_root().join(relative);
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
