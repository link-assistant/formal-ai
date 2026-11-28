//! Fallible, deterministic fragment discovery shared by release helpers.

// Keep rust-script from wrapping this module as an expression in test builds.
#[cfg(test)]
#[allow(dead_code)]
fn main() {}

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// List sorted `.md` fragment paths, excluding README.md.
///
/// A missing directory means no fragments. All other directory and entry
/// failures are errors, including a regular file in place of the directory.
pub fn list_fragments(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(path_error("read fragment directory", directory, error)),
    };
    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| path_error("read fragment directory entry", directory, error))?
            .path();
        if path.extension().is_some_and(|ext| ext == "md")
            && path.file_name().is_some_and(|name| name != "README.md")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Add operation and path context while preserving the I/O error kind.
pub fn path_error(operation: &str, path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("Failed to {operation} '{}': {error}", path.display()),
    )
}

/// Read a release file without dropping I/O errors or their path context.
pub fn read_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path).map_err(|error| path_error("read file", path, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_read_failures_are_not_empty_collections() {
        let root = std::env::temp_dir().join(format!(
            "changelog-files-{}-{}",
            std::process::id(),
            module_path!().replace("::", "-")
        ));
        fs::write(&root, "not a directory").unwrap();
        let error = list_fragments(&root).unwrap_err();
        fs::remove_file(&root).unwrap();
        assert!(error.to_string().contains(&root.display().to_string()));
    }

    #[test]
    fn missing_directory_has_no_fragments() {
        let root = std::env::temp_dir().join(format!("absent-fragments-{}", std::process::id()));
        assert!(list_fragments(&root).unwrap().is_empty());
    }
}

/// Keep the bytes read for publication bound to cleanup's exact file inventory.
#[derive(Debug)]
pub struct Fragment {
    pub path: PathBuf,
    pub content: String,
}

pub fn read_fragments(directory: &Path) -> io::Result<Vec<Fragment>> {
    list_fragments(directory)?
        .into_iter()
        .map(|path| read_file(&path).map(|content| Fragment { path, content }))
        .collect()
}

/// Never enumerate again for deletion or silently consume a changed fragment.
pub fn remove_fragments(fragments: &[Fragment]) -> io::Result<()> {
    for fragment in fragments {
        if read_file(&fragment.path)? != fragment.content {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Fragment changed after collection: {}",
                    fragment.path.display()
                ),
            ));
        }
    }
    for fragment in fragments {
        fs::remove_file(&fragment.path)
            .map_err(|error| path_error("remove fragment", &fragment.path, error))?;
    }
    Ok(())
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "changelog-snapshot-{}-{}-{id}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn unreadable_fragment_aborts_instead_of_dropping_an_entry() {
        let root = Directory::new();
        fs::write(root.0.join("a.md"), "valid").unwrap();
        let invalid = root.0.join("b.md");
        fs::write(&invalid, [0xff]).unwrap();
        let error = read_fragments(&root.0).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("b.md"));
        assert_eq!(fs::read(&invalid).unwrap(), [0xff]);
        assert!(root.0.join("a.md").exists());
    }

    #[test]
    fn cleanup_consumes_only_the_sorted_snapshot_and_keeps_new_fragments() {
        let root = Directory::new();
        for name in ["b.md", "a.md", "README.md", "notes.txt"] {
            fs::write(root.0.join(name), name).unwrap();
        }
        let fragments = read_fragments(&root.0).unwrap();
        assert_eq!(
            fragments
                .iter()
                .map(|f| f.path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            ["a.md", "b.md"]
        );
        fs::write(root.0.join("later.md"), "arrived after collection").unwrap();
        remove_fragments(&fragments).unwrap();
        for name in ["README.md", "notes.txt", "later.md"] {
            assert!(root.0.join(name).exists());
        }
        for name in ["a.md", "b.md"] {
            assert!(!root.0.join(name).exists());
        }
    }

    #[test]
    fn cleanup_refuses_changed_or_missing_snapshots_before_deleting_anything() {
        let root = Directory::new();
        for name in ["a.md", "b.md"] {
            fs::write(root.0.join(name), name).unwrap();
        }
        let fragments = read_fragments(&root.0).unwrap();
        fs::write(root.0.join("b.md"), "new content").unwrap();
        assert_eq!(
            remove_fragments(&fragments).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(root.0.join("a.md").exists());
        assert_eq!(
            fs::read_to_string(root.0.join("b.md")).unwrap(),
            "new content"
        );
        fs::remove_file(root.0.join("b.md")).unwrap();
        assert_eq!(
            remove_fragments(&fragments).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert!(root.0.join("a.md").exists());
    }
}
