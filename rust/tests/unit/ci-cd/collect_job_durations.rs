//! Issue #1580: the duration collector behind the weekly Job Headroom audit
//! must not end its sample in silence when one GitHub API request fails.
//!
//! Run 40307862657 ended 190 runs into a 400-run sample with `exit code 1` and
//! no other line, and the two headroom audits after it read an empty file.
//! The tests here drive `scripts/collect-job-durations.sh` against a fake `gh`.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;

/// A `gh` stand-in that prints already-filtered rows, as `gh api --jq` would.
/// Run 101 fails once with a 502 and then answers, run 102 never answers, and
/// run 103 answers at once.
const FAKE_GH: &str = r#"#!/usr/bin/env bash
for argument in "$@"; do
  case "${argument}" in
    *"actions/runs?"*"page=1") printf '101\tCI\n102\tCI\n103\tCI\n'; exit 0 ;;
    *"actions/runs?"*) exit 0 ;;
    *"runs/101/jobs"*)
      if [ ! -e "${FAKE_STATE}/101" ]; then
        touch "${FAKE_STATE}/101"
        echo "HTTP 502: Bad Gateway" >&2
        exit 1
      fi
      printf 'build\tsuccess\t2026-01-01T00:00:00Z\t2026-01-01T00:01:00Z\n'; exit 0 ;;
    *"runs/102/jobs"*) echo "HTTP 403: API rate limit exceeded" >&2; exit 1 ;;
    *"runs/103/jobs"*)
      printf 'test\tsuccess\t2026-01-01T00:00:00Z\t2026-01-01T00:02:00Z\n'; exit 0 ;;
  esac
done
"#;

/// Runs the collector against [`FAKE_GH`] and returns its status, rows and log.
fn collect_with_fake_gh(case: &str, max_unread_percent: &str) -> (Option<i32>, String, String) {
    let work = std::env::temp_dir().join(format!(
        "formal-ai-issue-1580-collector-{}-{case}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(work.join("state")).expect("create the fake gh state directory");
    let gh = work.join("gh");
    fs::write(&gh, FAKE_GH).expect("write the fake gh");
    fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).expect("make the fake gh runnable");
    let path = format!(
        "{}:{}",
        work.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = std::process::Command::new("bash")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../scripts/collect-job-durations.sh"),
        )
        .args(["main", "3"])
        .env("PATH", path)
        .env("FAKE_STATE", work.join("state"))
        .env("GITHUB_REPOSITORY", "o/r")
        .env("COLLECT_RETRY_DELAY", "0")
        .env("COLLECT_MAX_UNREAD_PERCENT", max_unread_percent)
        .output()
        .expect("run the collector");
    let _ = fs::remove_dir_all(&work);
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Job Headroom run 40307862657 ended 190 runs into its sample with `exit
/// code 1` and no other line: one `gh api` call failed, `2>/dev/null` threw
/// its error away, and `pipefail` ended the script. A transient failure is
/// now retried, a run that stays unreadable is skipped with the error `gh`
/// gave, and the rows of every readable run are still written.
#[test]
fn a_failed_request_is_retried_and_reported_rather_than_ending_the_sample() {
    let (status, rows, log) = collect_with_fake_gh("tolerated", "50");
    assert_eq!(status, Some(0), "{log}");
    assert_eq!(
        rows,
        "101\tCI\tbuild\tsuccess\t2026-01-01T00:00:00Z\t2026-01-01T00:01:00Z\n\
         103\tCI\ttest\tsuccess\t2026-01-01T00:00:00Z\t2026-01-01T00:02:00Z\n",
        "run 101 is read on its retry and run 103 after the skipped run 102"
    );
    assert!(log.contains("HTTP 502: Bad Gateway"), "{log}");
    assert!(log.contains("HTTP 403: API rate limit exceeded"), "{log}");
    assert!(log.contains("::warning::Skipping run 102"), "{log}");
    assert!(
        log.contains("Collected jobs from 2 of 3 runs (1 unread)."),
        "{log}"
    );
}

/// The other half of the same rule: a sample with too many unread runs no
/// longer describes the branch, so the collection fails -- and says why.
#[test]
fn a_sample_with_too_many_unread_runs_fails_with_the_reason() {
    let (status, _, log) = collect_with_fake_gh("too-thin", "5");
    assert_eq!(status, Some(1), "{log}");
    assert!(
        log.contains("::error::1 of 3 sampled runs could not be read"),
        "{log}"
    );
}
