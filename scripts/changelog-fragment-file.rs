//! Exclusive fragment creation preserves an earlier release entry on collision.
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn create(path: &Path, content: &str) -> io::Result<()> {
    // Timestamp-based names can collide across processes or repeat in one second.
    // create_new performs the existence check and creation as one filesystem effect.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};

    struct Workspace(PathBuf);
    impl Workspace {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "fragment-preservation-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn fragment(&self) -> PathBuf {
            self.0.join("20261120000000-manual-patch.md")
        }
    }
    impl Drop for Workspace {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn repeated_creation_preserves_the_original_fragment_bytes() {
        let workspace = Workspace::new();
        let path = workspace.fragment();
        let original = "---\r\nbump: minor\r\n---\r\n\nKeep λ and literal \\n.\n";
        create(&path, original).unwrap();
        assert_eq!(
            create(&path, "replacement").unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(&path).unwrap(), original.as_bytes());
    }

    #[test]
    fn concurrent_creators_have_exactly_one_complete_winner() {
        let workspace = Workspace::new();
        let path = workspace.fragment();
        let barrier = Arc::new(Barrier::new(2));
        let workers: Vec<_> = ["first fragment", "second fragment"]
            .into_iter()
            .map(|content| {
                let path = path.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    (content, create(&path, content))
                })
            })
            .collect();
        let results: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(
            results.iter().filter(|(_, result)| result.is_ok()).count(),
            1
        );
        let winner = results.iter().find(|(_, result)| result.is_ok()).unwrap().0;
        assert_eq!(fs::read_to_string(&path).unwrap(), winner);
        for (_, result) in results.into_iter().filter(|(_, result)| result.is_err()) {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        }
    }

    #[test]
    fn missing_parent_is_an_error_without_a_fragment() {
        let workspace = Workspace::new();
        let path = workspace.0.join("missing/entry.md");
        assert_eq!(
            create(&path, "entry").unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn an_existing_symbolic_link_cannot_overwrite_its_target() {
        let workspace = Workspace::new();
        let target = workspace.0.join("original.md");
        fs::write(&target, "original").unwrap();
        let path = workspace.fragment();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert_eq!(
            create(&path, "replacement").unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read_to_string(target).unwrap(), "original");
    }
}
