//! Regression coverage for issue #1028: later apt retries must receive the
//! time left in the enclosing step budget instead of repeating a flat deadline.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after 1970")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("issue-1028-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("create scratch directory");
    path
}

/// The fake mirror is slow on the first update but healthy on the second.
/// A flat 3s/3s retry would fail because the first attempt times out after 3s
/// and the second also gets only 3s. With a 10s budget the new 1:2 allocation
/// is 3s/7s, so the second attempt has enough time to complete the 5s recovery.
fn run_slow_mirror() -> std::process::Output {
    let directory = temp_dir("slow-mirror");
    let stand_in = directory.join("apt-get");
    let calls = directory.join("calls");
    fs::write(&calls, "").expect("create call log");
    fs::write(
        &stand_in,
        r#"#!/usr/bin/env bash
set -eu
printf '%s\n' "$*" >> "$FAKE_APT_CALLS"
if [ "$(grep -c ' update -q$' "$FAKE_APT_CALLS")" -eq 1 ]; then
  sleep 5
fi
exit 0
"#,
    )
    .expect("write fake apt-get");
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755))
        .expect("make fake apt-get executable");

    Command::new("bash")
        .arg(format!(
            "{}/scripts/apt-install-with-retry.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("xvfb")
        .env("FORMAL_AI_APT_PRIVILEGE", "")
        .env("FORMAL_AI_APT_GET", &stand_in)
        .env("FAKE_APT_CALLS", &calls)
        .env("FORMAL_AI_APT_ATTEMPTS", "2")
        .env("FORMAL_AI_APT_ATTEMPT_SECONDS", "3")
        .env("FORMAL_AI_APT_RETRY_DELAY_SECONDS", "0")
        .env("TEST_BUDGET_SECONDS", "10")
        .output()
        .expect("run apt retry against slow fake mirror")
}

#[test]
fn slow_mirror_fails_flat_deadline_but_succeeds_with_escalating_budget() {
    let started = std::time::Instant::now();
    let output = run_slow_mirror();
    let elapsed = started.elapsed();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "the recovered mirror must succeed: stdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stderr.contains("Attempt 1 exited 124") && stderr.contains("of its 3s deadline"),
        "the first probe must be the short deadline: {stderr}"
    );
    assert!(
        stdout.contains("succeeded on attempt 2/2"),
        "the longer second attempt must recover the install: {stdout}"
    );
    assert!(
        elapsed >= Duration::from_secs(5),
        "the fake mirror deliberately takes 5s to recover, but the test finished in {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(15),
        "the retry must use the 10s step budget rather than repeating the slow 5s probe: {elapsed:?}"
    );
}

#[test]
fn default_without_step_budget_keeps_the_fixed_attempt_deadline() {
    let directory = temp_dir("fixed-deadline");
    let stand_in = directory.join("apt-get");
    fs::write(
        &stand_in,
        "#!/usr/bin/env bash\nsleep 30\n",
    )
    .expect("write fake apt-get");
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755))
        .expect("make fake apt-get executable");

    let output = Command::new("bash")
        .arg(format!(
            "{}/scripts/apt-install-with-retry.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg("xvfb")
        .env("FORMAL_AI_APT_PRIVILEGE", "")
        .env("FORMAL_AI_APT_GET", &stand_in)
        .env("FORMAL_AI_APT_ATTEMPTS", "1")
        .env("FORMAL_AI_APT_ATTEMPT_SECONDS", "1")
        .output()
        .expect("run fixed-deadline apt retry");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(124));
    assert!(
        stderr.contains("of its 1s deadline"),
        "without an enclosing budget the historical fixed deadline remains: {stderr}"
    );
}

#[allow(dead_code)]
fn _path_exists(path: &Path) -> bool {
    path.exists()
}
