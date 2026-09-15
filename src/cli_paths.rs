//! Stable path resolution for CLI subcommands that enter child workspaces.

use std::path::{Path, PathBuf};

/// Resolve a caller-supplied root once, before a child process changes its
/// working directory.
///
/// Keeping `.` relative made the benchmark grader create
/// `./target/formal-ai-benchmarks/run/<suite>/<case>.py`, enter the suite
/// directory, and then pass that same relative path to Python. The path was
/// consequently resolved a second time below the new working directory.
/// Every CLI that accepts a repository root uses this boundary so descendants
/// always receive one stable absolute path.
pub fn resolve_root(explicit: Option<PathBuf>, fallback: &Path) -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| fallback.to_path_buf());
    let root = explicit.unwrap_or_else(|| current.clone());
    let absolute = if root.is_absolute() {
        root
    } else {
        current.join(root)
    };
    absolute.canonicalize().unwrap_or(absolute)
}
