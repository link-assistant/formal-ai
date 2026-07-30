#[cfg(unix)]
mod unix {
    use formal_ai::orchestration::replay_session;
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);

    struct Case {
        language: &'static str,
        task: &'static str,
    }

    #[test]
    fn vendor_orchestration_preserves_tasks_in_every_supported_language() {
        let fake_bin = TestWorkspace::new("bin");
        let codex = fake_bin.path().join("codex");
        fs::write(&codex, "#!/bin/sh\nprintf '%s\\n' \"$@\"\n").unwrap();
        fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(std::iter::once(fake_bin.path().to_path_buf()).chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();

        for Case { language, task } in [
            Case {
                language: "en",
                task: "add a README badge",
            },
            Case {
                language: "ru",
                task: "добавь значок в README",
            },
            Case {
                language: "hi",
                task: "README में बैज जोड़ें",
            },
            Case {
                language: "zh",
                task: "在 README 中添加徽章",
            },
        ] {
            let workspace = TestWorkspace::new(language);
            let session_path = workspace.path().join("session.json");
            let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
                .args([
                    "agent",
                    "run",
                    "--cli",
                    "codex",
                    "--target",
                    "vendor",
                    "--task",
                    task,
                    "--workspace",
                ])
                .arg(workspace.path())
                .args(["--session"])
                .arg(&session_path)
                .env("PATH", &path)
                .output()
                .unwrap();

            assert!(
                output.status.success(),
                "{language}: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let session = replay_session(&fs::read(session_path).unwrap()).unwrap();
            assert_eq!(session.task, task, "{language}: recorded task");
            assert_eq!(
                session.args.last().map(String::as_str),
                Some(task),
                "{language}: task forwarded to client"
            );
            assert_eq!(
                session.stdout.lines().last(),
                Some(task),
                "{language}: client received task byte-for-byte"
            );
        }
    }

    struct TestWorkspace(PathBuf);

    impl TestWorkspace {
        fn new(label: &str) -> Self {
            let sequence = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "formal-ai-issue-703-languages-{}-{label}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
