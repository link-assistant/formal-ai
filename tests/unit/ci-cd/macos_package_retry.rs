//! Behavioral regression coverage for transient macOS DMG creation failures.
//!
//! GitHub-hosted macOS runners can intermittently fail in `hdiutil` after
//! electron-builder has successfully signed the application and created its
//! ZIP. The package wrapper must retry that narrow host-device failure without
//! turning unrelated packaging errors into false successes.

#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn wrapper() -> String {
    format!(
        "{}/desktop/scripts/package-macos-with-retry.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn sandbox(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-macos-package-{label}-{}-{nonce}",
        std::process::id()
    ));
    let bin = root.join("bin");
    fs::create_dir_all(root.join("release")).expect("release scratch directory must be created");
    fs::create_dir_all(&bin).expect("mock bin directory must be created");

    let npx = bin.join("npx");
    fs::write(
        &npx,
        r#"#!/usr/bin/env bash
set -euo pipefail
attempt=0
if [ -f "$FORMAL_AI_TEST_ATTEMPT_FILE" ]; then
  attempt="$(cat "$FORMAL_AI_TEST_ATTEMPT_FILE")"
fi
attempt=$((attempt + 1))
printf '%s\n' "$attempt" > "$FORMAL_AI_TEST_ATTEMPT_FILE"

case "$FORMAL_AI_TEST_MODE" in
  transient-then-success)
    if [ "$attempt" -eq 1 ]; then
      printf 'partial\n' > release/formal-ai-desktop-macos-arm64-0.0.0.dmg
      echo 'hdiutil: create failed - Device not configured' >&2
      exit 37
    fi
    if [ -e release/formal-ai-desktop-macos-arm64-0.0.0.dmg ]; then
      echo 'incomplete DMG was not cleaned before retry' >&2
      exit 99
    fi
    ;;
  transient-attach-then-success)
    if [ "$attempt" -eq 1 ]; then
      echo 'hdiutil: attach failed - Device not configured' >&2
      exit 38
    fi
    ;;
  persistent-transient)
    echo 'hdiutil: create failed - Resource busy' >&2
    exit 37
    ;;
  unrelated)
    echo 'codesign: errSecInternalComponent' >&2
    exit 42
    ;;
esac
"#,
    )
    .expect("mock npx must be written");
    fs::set_permissions(&npx, fs::Permissions::from_mode(0o755))
        .expect("mock npx must be executable");
    root
}

fn run_wrapper(root: &Path, mode: &str) -> Output {
    let attempts = root.join("attempts");
    let path = format!(
        "{}:{}",
        root.join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new("bash")
        .arg(wrapper())
        .args(["--mac", "--publish", "never"])
        .current_dir(root)
        .env("PATH", path)
        .env("FORMAL_AI_TEST_ATTEMPT_FILE", attempts)
        .env("FORMAL_AI_TEST_MODE", mode)
        .env("FORMAL_AI_MACOS_PACKAGE_RETRY_DELAY_SECONDS", "0")
        .output()
        .expect("macOS package wrapper must run")
}

fn attempt_count(root: &Path) -> String {
    fs::read_to_string(root.join("attempts"))
        .expect("mock attempt counter must exist")
        .trim()
        .to_owned()
}

#[test]
fn retries_known_hdiutil_device_failures_and_cleans_incomplete_dmg() {
    let root = sandbox("retry");
    let output = run_wrapper(&root, "transient-then-success");
    let count = attempt_count(&root);
    fs::remove_dir_all(&root).expect("sandbox must be removed");

    assert!(
        output.status.success(),
        "a known transient should recover; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count, "2", "the successful retry should be attempt two");
}

#[test]
fn retries_known_hdiutil_attach_device_failure() {
    let root = sandbox("attach-retry");
    let output = run_wrapper(&root, "transient-attach-then-success");
    let count = attempt_count(&root);
    fs::remove_dir_all(&root).expect("sandbox must be removed");

    assert!(
        output.status.success(),
        "the observed transient attach failure should recover; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count, "2", "the successful retry should be attempt two");
}

#[test]
fn unrelated_packaging_failure_is_not_retried() {
    let root = sandbox("unrelated");
    let output = run_wrapper(&root, "unrelated");
    let count = attempt_count(&root);
    fs::remove_dir_all(&root).expect("sandbox must be removed");

    assert_eq!(output.status.code(), Some(42));
    assert_eq!(count, "1", "unrelated failures must fail immediately");
}

#[test]
fn persistent_hdiutil_failure_stops_after_three_attempts() {
    let root = sandbox("bounded");
    let output = run_wrapper(&root, "persistent-transient");
    let count = attempt_count(&root);
    fs::remove_dir_all(&root).expect("sandbox must be removed");

    assert_eq!(
        output.status.code(),
        Some(37),
        "the final electron-builder status must be preserved"
    );
    assert_eq!(count, "3", "the retry must remain bounded");
}
