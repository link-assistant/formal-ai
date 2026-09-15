//! Stable path resolution for CLI subcommands that enter child workspaces.

use std::path::PathBuf;

/// Resolve a caller-supplied root once, before a child process changes its
/// working directory.
///
/// Keeping `.` relative made the benchmark grader create
/// `./target/formal-ai-benchmarks/run/<suite>/<case>.py`, enter the suite
/// directory, and then pass that same relative path to Python. The path was
/// consequently resolved a second time below the new working directory.
/// Every CLI that accepts a repository root uses this boundary so descendants
/// always receive one stable absolute path.
pub fn resolve_root(explicit: Option<PathBuf>, fallback: PathBuf) -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| fallback.clone());
    let root = explicit.unwrap_or_else(|| current.clone());
    let absolute = if root.is_absolute() {
        root
    } else {
        current.join(root)
    };
    absolute.canonicalize().unwrap_or(absolute)
}

#[cfg(test)]
mod tests {
    use super::resolve_root;
    use std::path::PathBuf;

    #[test]
    fn a_relative_root_is_resolved_before_a_child_changes_directory() {
        let root = resolve_root(Some(PathBuf::from(".")), PathBuf::from("."));
        assert!(root.is_absolute());
        assert_eq!(root, std::env::current_dir().expect("current directory"));
    }

    #[test]
    fn an_absolute_root_keeps_its_identity() {
        let current = std::env::current_dir().expect("current directory");
        assert_eq!(
            resolve_root(Some(current.clone()), PathBuf::from("ignored")),
            current
        );
    }
}
