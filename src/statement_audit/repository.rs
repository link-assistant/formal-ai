use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::model::{RepositoryCorpus, RepositoryDocument};

impl RepositoryCorpus {
    /// Read every UTF-8 Git-tracked file, falling back to a filtered tree walk.
    pub fn from_repository(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref();
        // An empty result can mean an initialized repository whose first files
        // have not been added yet. Those files are still the complete snapshot
        // the caller explicitly asked us to audit, so use the tree fallback.
        let paths = tracked_paths(root)
            .filter(|paths| !paths.is_empty())
            .unwrap_or_else(|| walked_paths(root));
        let mut corpus = Self::default();
        for path in paths {
            let normalized = path.to_string_lossy().replace('\\', "/");
            corpus.tracked_paths.insert(normalized.clone());
            match fs::read(root.join(&path)) {
                Ok(bytes) if !bytes.contains(&0) => match String::from_utf8(bytes) {
                    Ok(content) => corpus
                        .documents
                        .push(RepositoryDocument::new(normalized, content)),
                    Err(_) => corpus.skipped_paths.push(normalized),
                },
                Ok(_) | Err(_) => corpus.skipped_paths.push(normalized),
            }
        }
        corpus
            .documents
            .sort_by(|left, right| left.path.cmp(&right.path));
        corpus.skipped_paths.sort();
        Ok(corpus)
    }
}

fn tracked_paths(root: &Path) -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| PathBuf::from(String::from_utf8_lossy(path).into_owned()))
            .collect(),
    )
}

fn walked_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    walk_directory(root, root, &mut paths);
    paths.sort();
    paths
}

fn walk_directory(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if !matches!(
                entry.file_name().to_str(),
                Some(".git" | "target" | "node_modules")
            ) {
                walk_directory(root, &path, paths);
            }
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(root) {
                paths.push(relative.to_path_buf());
            }
        }
    }
}
