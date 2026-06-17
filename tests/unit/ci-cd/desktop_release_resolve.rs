//! Behavioral tests for scripts/desktop-release-resolve.sh (issue #479).
//!
//! The Desktop Release workflow's `resolve` job decides which release tag to
//! build desktop assets for and whether a build is needed. The original logic
//! required a tag whose commit EQUALS the completed CI run's head SHA. But the
//! automated release tags a CHILD "chore: release vX.Y.Z" commit (its first
//! parent is the head SHA) and is pushed with `GITHUB_TOKEN`, so:
//!   * no tag ever points at the head SHA -> the match failed -> the build was
//!     skipped -> no assets were uploaded, and
//!   * every /download entry read "Not available in latest release".
//!
//! These tests run the extracted resolve script against a mocked `gh` CLI and
//! assert the resolved `tag`/`should_build` outputs for each event shape. The
//! `auto_release_child_commit_triggers_build` case is the direct reproduction
//! of issue #479: under the old logic it produced `should_build=false`; the fix
//! must produce `should_build=true`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn script_path() -> String {
    format!(
        "{}/scripts/desktop-release-resolve.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// A canned `gh` CLI. Each field maps to one shape of `gh` invocation the
/// resolve script makes; the mock dispatches on argv exactly like the real CLI.
#[derive(Default)]
struct GhMock<'a> {
    /// Newline-joined tag names returned for `gh api .../tags ... --jq ...`
    /// (the Tier 1 exact-SHA candidates). Empty = no tag points at the head SHA.
    tags_jq_output: &'static str,
    /// `gh release view --json tagName --jq .tagName` (no positional tag).
    latest_tag: &'static str,
    /// `gh api .../commits/<tag> --jq .parents[0].sha` (diagnostic parent check).
    parent_sha: &'static str,
    /// Whether `gh release view <tag> --json tagName` succeeds.
    release_exists: bool,
    /// Newline-joined `formal-ai-desktop-*` names returned for
    /// `gh release view <tag> --json assets --jq ...`.
    asset_names: &'a str,
}

struct ResolveOutput {
    tag: String,
    should_build: String,
    stdout: String,
    stderr: String,
    ok: bool,
}

/// Write a mock `gh` executable that mirrors the real CLI surface the resolve
/// script touches, driven entirely by `MOCK_*` environment variables.
// The mock is a raw shell string; its `${VAR}` expansions look like format
// arguments to clippy but are deliberately plain shell, not Rust formatting.
#[allow(clippy::literal_string_with_formatting_args)]
fn write_mock_gh(dir: &Path) {
    let mock = r#"#!/usr/bin/env bash
# Mock `gh` for desktop-release-resolve.sh tests. Canned responses via MOCK_*.
sub="$1"; shift || true
case "$sub" in
  api)
    path="$1"; shift || true
    case "$path" in
      *"/tags"*)     [ -n "${MOCK_TAGS_JQ_OUTPUT:-}" ] && printf '%s\n' "${MOCK_TAGS_JQ_OUTPUT}" ;;
      *"/commits/"*) printf '%s\n' "${MOCK_PARENT_SHA:-}" ;;
    esac ;;
  release)
    action="$1"; shift || true
    if [ "$action" = "view" ]; then
      first="${1:-}"
      if [ -n "$first" ] && [ "${first#--}" = "$first" ]; then
        # positional tag present -> existence or asset query
        case "$*" in
          *"--json assets"*)
            if [[ "$*" == *"length"* ]]; then
              [ -n "${MOCK_ASSET_NAMES:-}" ] && printf '%s\n' "${MOCK_ASSET_NAMES}" | sed '/^$/d' | wc -l || printf '0\n'
            else
              [ -n "${MOCK_ASSET_NAMES:-}" ] && printf '%s\n' "${MOCK_ASSET_NAMES}"
            fi ;;
          *"--json tagName"*) [ "${MOCK_RELEASE_EXISTS:-1}" = "1" ] && { printf '{"tagName":"%s"}\n' "$first"; exit 0; } || exit 1 ;;
        esac
      else
        # no positional tag -> latest release tagName
        [ -n "${MOCK_LATEST_TAG:-}" ] && { printf '%s\n' "$MOCK_LATEST_TAG"; exit 0; } || exit 1
      fi
    fi ;;
esac
exit 0
"#;
    let gh = dir.join("gh");
    fs::write(&gh, mock).expect("write mock gh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&gh).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&gh, perms).expect("chmod mock gh");
    }
}

fn unique_tmp(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "formal-ai-resolve-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ))
}

/// Run the resolve script with the given event environment and mocked `gh`,
/// returning the parsed `GITHUB_OUTPUT` plus captured streams.
fn run_resolve(label: &str, env: &[(&str, &str)], mock: &GhMock<'_>) -> ResolveOutput {
    let tmp = unique_tmp(label);
    let bin = tmp.join("bin");
    fs::create_dir_all(&bin).expect("create scratch bin dir");
    write_mock_gh(&bin);
    let output_file = tmp.join("github_output");
    fs::write(&output_file, "").expect("seed GITHUB_OUTPUT");

    let path_env = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut cmd = Command::new("/bin/bash");
    cmd.arg(script_path())
        .env("PATH", path_env)
        .env("GITHUB_OUTPUT", &output_file)
        .env("REPO", "link-assistant/formal-ai")
        .env("GH_TOKEN", "test-token")
        .env("MOCK_TAGS_JQ_OUTPUT", mock.tags_jq_output)
        .env("MOCK_LATEST_TAG", mock.latest_tag)
        .env("MOCK_PARENT_SHA", mock.parent_sha)
        .env(
            "MOCK_RELEASE_EXISTS",
            if mock.release_exists { "1" } else { "0" },
        )
        .env("MOCK_ASSET_NAMES", mock.asset_names);
    for (key, value) in env {
        cmd.env(key, value);
    }

    let output = cmd.output().expect("run resolve script");
    let rendered = fs::read_to_string(&output_file).unwrap_or_default();
    let _ = fs::remove_dir_all(&tmp);

    let mut tag = String::new();
    let mut should_build = String::new();
    for line in rendered.lines() {
        if let Some(rest) = line.strip_prefix("tag=") {
            tag = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("should_build=") {
            should_build = rest.to_string();
        }
    }

    ResolveOutput {
        tag,
        should_build,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        ok: output.status.success(),
    }
}

fn bash_available() -> bool {
    Path::new("/bin/bash").exists()
}

fn expected_asset_names(version: &str) -> String {
    [
        format!("formal-ai-desktop-macos-arm64-{version}.dmg"),
        format!("formal-ai-desktop-macos-arm64-{version}.zip"),
        format!("formal-ai-desktop-macos-x64-{version}.dmg"),
        format!("formal-ai-desktop-macos-x64-{version}.zip"),
        format!("formal-ai-desktop-windows-installer-x64-{version}.exe"),
        format!("formal-ai-desktop-windows-installer-arm64-{version}.exe"),
        format!("formal-ai-desktop-windows-portable-x64-{version}.exe"),
        format!("formal-ai-desktop-windows-portable-arm64-{version}.exe"),
        format!("formal-ai-desktop-linux-x64-{version}.AppImage"),
        format!("formal-ai-desktop-linux-arm64-{version}.AppImage"),
        format!("formal-ai-desktop-linux-x64-{version}.deb"),
        format!("formal-ai-desktop-linux-arm64-{version}.deb"),
        format!("formal-ai-desktop-linux-x64-{version}.tar.gz"),
        format!("formal-ai-desktop-linux-arm64-{version}.tar.gz"),
    ]
    .join("\n")
}

fn macos_and_windows_asset_names(version: &str) -> String {
    [
        format!("formal-ai-desktop-macos-arm64-{version}.dmg"),
        format!("formal-ai-desktop-macos-arm64-{version}.zip"),
        format!("formal-ai-desktop-macos-x64-{version}.dmg"),
        format!("formal-ai-desktop-macos-x64-{version}.zip"),
        format!("formal-ai-desktop-windows-installer-x64-{version}.exe"),
        format!("formal-ai-desktop-windows-installer-arm64-{version}.exe"),
        format!("formal-ai-desktop-windows-portable-x64-{version}.exe"),
        format!("formal-ai-desktop-windows-portable-arm64-{version}.exe"),
    ]
    .join("\n")
}

#[test]
fn auto_release_child_commit_triggers_build() {
    // The exact issue #479 reproduction: a successful CI run completes on the
    // PARENT commit (head SHA), while the release tag lives on the CHILD
    // "chore: release" commit, so NO tag points at the head SHA. The latest
    // release carries no desktop assets yet. The fix must resolve that latest
    // release and build (old logic skipped with should_build=false).
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "child-commit",
        &[
            ("EVENT", "workflow_run"),
            ("WORKFLOW_RUN_HEAD_SHA", "0abd3f45parenthead"),
        ],
        &GhMock {
            tags_jq_output: "", // no tag points at the head SHA (the bug condition)
            latest_tag: "v0.201.0",
            parent_sha: "0abd3f45parenthead", // child release descends from head SHA
            release_exists: true,
            asset_names: "",
        },
    );
    assert!(
        result.ok,
        "resolve script failed\nstdout:\n{}\nstderr:\n{}",
        result.stdout, result.stderr
    );
    assert_eq!(
        result.tag, "v0.201.0",
        "should resolve the latest release tag (the auto-release child commit)"
    );
    assert_eq!(
        result.should_build, "true",
        "issue #479: a freshly released version with no desktop assets must build them"
    );
}

#[test]
fn workflow_run_builds_when_release_is_missing_linux_assets() {
    // Maintainer follow-up on 2026-06-15: Linux was still unavailable after the
    // first issue #479 fixes. The live v0.204.0 release had macOS and Windows
    // desktop assets but no `formal-ai-desktop-linux-*` assets. The old
    // idempotency guard counted "any desktop assets" and skipped the automatic
    // build, making the partial release permanent. A partial release must build
    // so the next Desktop Release run can self-heal the missing matrix.
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let partial_assets = macos_and_windows_asset_names("0.204.0");
    let result = run_resolve(
        "partial-linux-missing",
        &[
            ("EVENT", "workflow_run"),
            ("WORKFLOW_RUN_HEAD_SHA", "0abd3f45parenthead"),
        ],
        &GhMock {
            tags_jq_output: "",
            latest_tag: "v0.204.0",
            parent_sha: "0abd3f45parenthead",
            release_exists: true,
            asset_names: &partial_assets,
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(result.tag, "v0.204.0");
    assert_eq!(
        result.should_build, "true",
        "a partial latest release that lacks Linux assets must be rebuilt"
    );
}

#[test]
fn workflow_run_skips_when_release_has_all_required_assets() {
    // Idempotency: re-running the pipeline (or a run that did not cut a new
    // release and falls back to the latest one) must not rebuild assets that
    // already exist.
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let complete_assets = expected_asset_names("0.201.0");
    let result = run_resolve(
        "has-assets",
        &[
            ("EVENT", "workflow_run"),
            ("WORKFLOW_RUN_HEAD_SHA", "0abd3f45parenthead"),
        ],
        &GhMock {
            tags_jq_output: "",
            latest_tag: "v0.201.0",
            parent_sha: "0abd3f45parenthead",
            release_exists: true,
            asset_names: &complete_assets,
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(result.tag, "v0.201.0");
    assert_eq!(
        result.should_build, "false",
        "a release that already has all required desktop assets must not rebuild on workflow_run"
    );
}

#[test]
fn workflow_run_uses_exact_tag_when_one_points_at_head_sha() {
    // Defensive Tier 1: if the release flow ever stops creating a child commit
    // and a tag DOES point at the head SHA, use it directly.
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "exact-sha",
        &[
            ("EVENT", "workflow_run"),
            ("WORKFLOW_RUN_HEAD_SHA", "deadbeefheadsha"),
        ],
        &GhMock {
            tags_jq_output: "v0.201.0", // a tag points directly at the head SHA
            latest_tag: "v0.201.0",
            parent_sha: "",
            release_exists: true,
            asset_names: "",
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(result.tag, "v0.201.0");
    assert_eq!(result.should_build, "true");
}

#[test]
fn workflow_run_skips_when_no_release_exists() {
    // A successful CI run that produced no release at all (and none exists yet)
    // must not attempt a desktop build.
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "no-release",
        &[
            ("EVENT", "workflow_run"),
            ("WORKFLOW_RUN_HEAD_SHA", "0abd3f45parenthead"),
        ],
        &GhMock {
            tags_jq_output: "",
            latest_tag: "",
            parent_sha: "",
            release_exists: false,
            asset_names: "",
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(result.should_build, "false");
}

#[test]
fn workflow_run_skips_when_head_sha_missing() {
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "no-head-sha",
        &[("EVENT", "workflow_run"), ("WORKFLOW_RUN_HEAD_SHA", "")],
        &GhMock {
            latest_tag: "v0.201.0",
            release_exists: true,
            ..GhMock::default()
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(
        result.should_build, "false",
        "a workflow_run payload without a head SHA cannot be matched to a release"
    );
}

#[test]
fn release_event_builds_resolved_tag_even_with_existing_assets() {
    // A manual `release` publish (PAT/UI) always (re)builds: the idempotency
    // guard is scoped to automatic workflow_run builds only.
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "release-event",
        &[("EVENT", "release"), ("RELEASE_TAG", "v0.5.0")],
        &GhMock {
            release_exists: true,
            asset_names: "formal-ai-desktop-linux-x64-0.5.0.AppImage",
            ..GhMock::default()
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(result.tag, "v0.5.0");
    assert_eq!(
        result.should_build, "true",
        "release events should always (re)build the published tag"
    );
}

#[test]
fn workflow_dispatch_rebuilds_requested_tag() {
    // Manual rebuild for a specific tag must proceed even if assets already
    // exist (maintainer-forced refresh / clobber).
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "dispatch",
        &[("EVENT", "workflow_dispatch"), ("INPUT_TAG", "v0.4.0")],
        &GhMock {
            release_exists: true,
            asset_names: "formal-ai-desktop-linux-x64-0.4.0.AppImage",
            ..GhMock::default()
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(result.tag, "v0.4.0");
    assert_eq!(result.should_build, "true");
}

#[test]
fn workflow_dispatch_without_tag_falls_back_to_latest() {
    if !bash_available() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let result = run_resolve(
        "dispatch-latest",
        &[("EVENT", "workflow_dispatch"), ("INPUT_TAG", "")],
        &GhMock {
            latest_tag: "v0.201.0",
            release_exists: true,
            asset_names: "",
            ..GhMock::default()
        },
    );
    assert!(result.ok, "resolve script failed: {}", result.stderr);
    assert_eq!(
        result.tag, "v0.201.0",
        "an empty dispatch tag should fall back to the latest release"
    );
    assert_eq!(result.should_build, "true");
}
