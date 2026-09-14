//! Regression coverage for issue #1076 (D22): a DNS blip must not fail a build.
//!
//! `macOS Core Tests / Build macOS test archive` (run 33973154494, job
//! 101325331000) failed 30 seconds into its first real step:
//!
//! ```text
//! Fetching latest main...
//! fatal: unable to access 'https://github.com/link-assistant/formal-ai/': Could not resolve host: github.com
//! ##[error]Process completed with exit code 128
//! ```
//!
//! Name resolution failed on the runner. Nothing about the change under test
//! caused it, and every later step was skipped -- the same false positive as
//! D19, one layer down: an unretried network call in a required step. The
//! rules here hold `scripts/simulate-fresh-merge.sh` to retrying that fetch,
//! and to still failing when the fetch is genuinely broken.

#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn script(name: &str) -> String {
    format!("{}/scripts/{name}", env!("CARGO_MANIFEST_DIR"))
}

/// A sandbox whose `bin/git` counts its `fetch` calls and fails them according
/// to `FORMAL_AI_TEST_MODE`, delegating every other subcommand to a canned
/// answer -- the script only asks for a SHA and a commit count.
fn sandbox(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-fresh-merge-{label}-{}-{nonce}",
        std::process::id()
    ));
    let bin = root.join("bin");
    fs::create_dir_all(&bin).expect("mock bin directory must be created");

    let git = bin.join("git");
    fs::write(
        &git,
        r#"#!/usr/bin/env bash
set -uo pipefail

case "$1" in
  fetch)
    attempt=0
    if [ -f "$FORMAL_AI_TEST_ATTEMPT_FILE" ]; then
      attempt="$(cat "$FORMAL_AI_TEST_ATTEMPT_FILE")"
    fi
    attempt=$((attempt + 1))
    printf '%s\n' "$attempt" > "$FORMAL_AI_TEST_ATTEMPT_FILE"

    case "$FORMAL_AI_TEST_MODE" in
      resolve-fails-once)
        if [ "$attempt" -eq 1 ]; then
          # The exact line run 33973154494 failed on.
          echo "fatal: unable to access 'https://github.com/link-assistant/formal-ai/': Could not resolve host: github.com" >&2
          exit 128
        fi
        ;;
      resolve-always-fails)
        echo "fatal: unable to access 'https://github.com/link-assistant/formal-ai/': Could not resolve host: github.com" >&2
        exit 128
        ;;
    esac
    exit 0
    ;;
  config) exit 0 ;;
  rev-parse) echo "0000000000000000000000000000000000000000" ;;
  rev-list) echo "0" ;;
  *) exit 0 ;;
esac
"#,
    )
    .expect("mock git must be written");
    fs::set_permissions(&git, fs::Permissions::from_mode(0o755))
        .expect("mock git must be executable");
    root
}

fn run(root: &Path, mode: &str) -> Output {
    run_script(root, "simulate-fresh-merge.sh", mode)
}

fn run_script(root: &Path, name: &str, mode: &str) -> Output {
    let path = format!(
        "{}:{}",
        root.join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new("bash")
        .arg(script(name))
        .env("PATH", path)
        .env("BASE_REF", "main")
        .env("FORMAL_AI_TEST_MODE", mode)
        .env(
            "FORMAL_AI_TEST_ATTEMPT_FILE",
            root.join("attempts").display().to_string(),
        )
        // Keep the test fast; the wait exists for the runner, not for us.
        .env("FRESH_MERGE_RETRY_DELAY_SECONDS", "0")
        .current_dir(root)
        .env(
            "GITHUB_OUTPUT",
            root.join("github-output").display().to_string(),
        )
        .output()
        .unwrap_or_else(|error| panic!("{name} must run: {error}"))
}

fn attempts(root: &Path) -> u32 {
    fs::read_to_string(root.join("attempts"))
        .expect("the mock git must have recorded its fetch attempts")
        .trim()
        .parse()
        .expect("attempt count must be a number")
}

#[test]
fn a_name_resolution_failure_is_retried_rather_than_failing_the_job() {
    let root = sandbox("transient");
    let output = run(&root, "resolve-fails-once");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "a fetch that fails once and then succeeds must not fail the job.\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        attempts(&root),
        2,
        "the fetch must be retried exactly once here, not abandoned and not repeated needlessly"
    );
    assert!(
        stdout.contains("retrying"),
        "the retry must be visible in the log, so a run that only succeeded on the second \
         attempt is not read as a clean one.\nstdout:\n{stdout}"
    );
}

#[test]
fn a_fetch_that_never_succeeds_still_fails_the_job() {
    let root = sandbox("persistent");
    let output = run(&root, "resolve-always-fails");

    assert!(
        !output.status.success(),
        "a base branch that cannot be fetched at all must fail: the merge simulation is a \
         required check, and skipping it silently is the false negative this repository is \
         auditing"
    );
    assert!(
        attempts(&root) >= 3,
        "a persistent failure must be retried a few times before it is believed, attempts: {}",
        attempts(&root)
    );
}

/// `pin-base-commit.sh` resolves the commit every other job in the run merges,
/// so the same blip costs the whole workflow rather than one job.
#[test]
fn the_pinned_base_commit_survives_the_same_blip() {
    let root = sandbox("pin-transient");
    let output = run_script(&root, "pin-base-commit.sh", "resolve-fails-once");

    assert!(
        output.status.success(),
        "resolving the base tip must survive a transient fetch failure.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        attempts(&root),
        2,
        "the fetch must be retried exactly once here"
    );
}

#[test]
fn a_base_tip_that_cannot_be_resolved_still_fails() {
    let root = sandbox("pin-persistent");
    let output = run_script(&root, "pin-base-commit.sh", "resolve-always-fails");

    assert!(
        !output.status.success(),
        "an unresolvable base tip must fail rather than pin an empty commit"
    );
    assert!(
        attempts(&root) >= 3,
        "a persistent failure must be retried before it is believed, attempts: {}",
        attempts(&root)
    );
}
