//! Seeded input is context, not evidence that the model authored a change.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn executable(path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Sandbox(PathBuf);

impl Sandbox {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // The path is the exact private directory returned by mktemp below.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_case(mode: &str) -> (Sandbox, Output) {
    let temporary = Command::new("mktemp").arg("-d").output().unwrap();
    assert!(temporary.status.success());
    let root = Sandbox(PathBuf::from(
        String::from_utf8(temporary.stdout).unwrap().trim(),
    ));
    for dir in ["bin", "scripts", "seed", "published", "scratch"] {
        fs::create_dir(root.path().join(dir)).unwrap();
    }
    fs::write(root.path().join("seed/leaf.txt"), "original\n").unwrap();
    fs::write(root.path().join("seed/support.txt"), "support\n").unwrap();
    fs::write(root.path().join("published/leaf.txt"), "keep destination\n").unwrap();
    if mode == "already_landed" {
        fs::write(root.path().join("published/leaf.txt"), "modified\n").unwrap();
        fs::write(root.path().join("published/support.txt"), "support\n").unwrap();
    }
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::copy(
        repository.join("scripts/classify-agent-cli-stderr.sh"),
        root.path().join("scripts/classify-agent-cli-stderr.sh"),
    )
    .unwrap();
    executable(&root.path().join("bin/curl"), "#!/bin/sh\nexit 0\n");
    executable(
        &root.path().join("bin/formal-ai"),
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'formal-ai test'; else exec sleep 30; fi\n",
    );
    executable(
        &root.path().join("bin/agent"),
        r#"#!/bin/sh
case "$AUTHORING_FIXTURE_MODE" in
  unchanged) ;;
  identical) printf 'original\n' > leaf.txt ;;
  modified|already_landed) printf 'modified\n' > leaf.txt ;;
  new) printf 'new artifact\n' > created.txt ;;
  *) exit 9 ;;
esac
printf '{"session_id":"ses_authoringfixture"}\n'
"#,
    );
    let produced = if mode == "new" {
        "created.txt"
    } else {
        "leaf.txt"
    };
    let output = Command::new("bash")
        .arg(repository.join("scripts/author-change-with-formal-ai.sh"))
        .args([
            "--task",
            "author an isolated fixture",
            "--produces",
            produced,
            "--into",
            "published/leaf.txt",
            "--produces",
            "support.txt",
            "--into",
            "published/support.txt",
            "--seed",
            "seed",
            "--evidence",
            "evidence",
            "--pull-request",
            "https://github.com/link-assistant/formal-ai/pull/888",
            "--message",
            "fixture",
            "--no-commit",
        ])
        .env_clear()
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.path().join("bin").display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("TMPDIR", root.path().join("scratch"))
        .env("FORMAL_AI_REPO_ROOT", root.path())
        .env("BIN", root.path().join("bin/formal-ai"))
        .env("AGENT", root.path().join("bin/agent"))
        .env("AUTHORING_FIXTURE_MODE", mode)
        .output()
        .unwrap();
    (root, output)
}

#[test]
fn read_only_or_identical_seed_replays_are_not_authorship() {
    for mode in ["unchanged", "identical"] {
        let (root, output) = run_case(mode);
        assert!(
            !output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("no produced artifact differs"));
        assert_eq!(
            fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
            "keep destination\n"
        );
        assert!(!root.path().join("published/support.txt").exists());
    }
}

#[test]
fn a_changed_artifact_can_be_published_with_unchanged_support() {
    let (root, output) = run_case("modified");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "modified\n"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("published/support.txt")).unwrap(),
        "support\n"
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("unstaged for review"));
}

#[test]
fn a_new_artifact_is_an_authored_effect() {
    let (root, output) = run_case("new");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "new artifact\n"
    );
}

#[test]
fn already_published_bytes_do_not_become_new_authorship_through_fresh_logs() {
    let (root, output) = run_case("already_landed");
    assert!(
        !output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("no produced artifact differs"));
    assert_eq!(
        fs::read_to_string(root.path().join("published/leaf.txt")).unwrap(),
        "modified\n"
    );
}
