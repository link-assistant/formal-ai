//! Small committed source input for actual workspace effects, not a clone of the evidence archive.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub(super) fn source_root() -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository root");
        let root = std::env::temp_dir().join(format!(
            "formal-ai-workspace-source-fixture-{}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("fresh small source fixture");
        for relative in ["rust/Cargo.toml", "rust/src/web_search_core.rs"] {
            let destination = root.join(relative);
            std::fs::create_dir_all(destination.parent().expect("parent"))
                .expect("fixture source directory");
            std::fs::copy(source.join(relative), destination).expect("actual source input");
        }
        for arguments in [
            vec!["init", "-q"],
            vec!["add", "rust/Cargo.toml", "rust/src/web_search_core.rs"],
            vec![
                "-c",
                "user.name=Workspace Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Exact source fixture",
            ],
        ] {
            let mut command = Command::new("git");
            command.arg("-C").arg(&root).args(arguments);
            let output = formal_ai::bounded_process::command_output(
                command,
                std::time::Duration::from_secs(20),
                1024 * 1024,
                1024 * 1024,
            )
            .expect("bounded fixture git command");
            assert!(
                output.status.success(),
                "fixture Git failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        root
    })
    .clone()
}
