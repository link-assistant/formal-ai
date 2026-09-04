//! `check_javascript_dependencies` must fail on advisories, not on outages.
//!
//! In run 100928011479 `Lint and Format Check` went red on a branch that
//! touched no lockfile at all. `bun audit` had spent five minutes inside one
//! request and exited with
//!
//! ```text
//! error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503
//! ```
//!
//! npmjs.org had said nothing whatsoever about `bun.lock`, and the branch wore
//! the result. A gate that is red when the registry is down and red when a
//! dependency is vulnerable reports the same colour for two different facts,
//! so the next reader learns to discount both.
//!
//! These tests hold the distinction from the outside, through the script's own
//! entry point: an unanswered registry is retried, an answered one ends the
//! gate on the first attempt, and an outage that outlasts every attempt still
//! fails rather than waving the lockfiles through.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

/// A scratch directory for one test's stand-in binaries.
fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after 1970")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("javascript-audit-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("create the scratch directory");
    path
}

/// What the gate saw in run 100928011479: a request that never got an answer.
const OUTAGE: &str = "outage";

/// What the gate exists for: the registry answered, and the answer is a finding.
const ADVISORY: &str = "advisory";

/// Run the real gate with stand-in `bun` and `npm` on `PATH`.
///
/// `failures` is how many leading invocations misbehave in `mode`; every later
/// one reports a clean lockfile. Both stand-ins share one call log, so the
/// count the assertions read is the number of audits the gate actually ran.
fn run_audit_gate(mode: &str, failures: u32, attempts: u32) -> (Output, usize) {
    let directory = temp_dir(mode);
    let calls = directory.join("calls");
    fs::write(&calls, "").expect("create the call log");

    let stand_in = format!(
        "#!/usr/bin/env bash\n\
         printf '%s\\n' \"$0 $*\" >> \"$FAKE_AUDIT_CALLS\"\n\
         if [ \"$(wc -l < \"$FAKE_AUDIT_CALLS\")\" -le \"$FAKE_AUDIT_FAILURES\" ]; then\n\
         \x20 case \"$FAKE_AUDIT_MODE\" in\n\
         \x20   {ADVISORY})\n\
         \x20     echo '1 vulnerability found (1 moderate)'\n\
         \x20     exit 1 ;;\n\
         \x20   *)\n\
         \x20     echo 'error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503' >&2\n\
         \x20     exit 1 ;;\n\
         \x20 esac\n\
         fi\n\
         echo 'No vulnerabilities found'\n\
         exit 0\n"
    );
    for name in ["bun", "npm"] {
        let path = directory.join(name);
        fs::write(&path, &stand_in)
            .unwrap_or_else(|error| panic!("write stand-in {name}: {error}"));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .expect("make the stand-in executable");
    }

    let repository = env!("CARGO_MANIFEST_DIR");
    let output = Command::new("bash")
        .arg(format!(
            "{repository}/scripts/check-javascript-dependencies.sh"
        ))
        .current_dir(repository)
        .env(
            "PATH",
            format!(
                "{}:{}",
                directory.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("FAKE_AUDIT_CALLS", &calls)
        .env("FAKE_AUDIT_FAILURES", failures.to_string())
        .env("FAKE_AUDIT_MODE", mode)
        .env("FORMAL_AI_AUDIT_ATTEMPTS", attempts.to_string())
        .env("FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS", "0")
        .output()
        .expect("run the JavaScript dependency gate");

    let audits = fs::read_to_string(&calls)
        .expect("read the call log")
        .lines()
        .count();
    (output, audits)
}

/// The failure from run 100928011479, replayed: two unanswered requests, then
/// an answer. The gate has to reach the answer.
#[test]
fn an_unanswered_registry_is_retried_until_it_answers() {
    let (output, audits) = run_audit_gate(OUTAGE, 2, 3);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "a 503 from the advisory registry is not a finding about a lockfile: {stdout}{stderr}"
    );
    assert_eq!(
        stderr
            .matches("::warning title=advisory registry unreachable")
            .count(),
        2,
        "each unanswered attempt must be visible in the job log: {stderr}"
    );
    // Five committed lockfiles, the first of which needed three attempts.
    assert_eq!(audits, 7, "the gate must retry rather than skip: {stdout}");
}

/// The registry answering with a finding is the case the gate exists for, so
/// it ends there. Retrying an answer would spend three times as long arriving
/// at the same red, and would teach a reader that red is negotiable.
#[test]
fn an_advisory_fails_the_gate_on_the_first_attempt() {
    let (output, audits) = run_audit_gate(ADVISORY, 99, 3);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "a moderate advisory must fail the gate: {stdout}{stderr}"
    );
    assert!(
        stdout.contains("1 vulnerability found (1 moderate)"),
        "the finding itself must reach the job log: {stdout}"
    );
    assert!(
        !stderr.contains("advisory registry unreachable"),
        "an answered registry must not be reported as an outage: {stderr}"
    );
    assert_eq!(audits, 1, "an answer is not retried: {stdout}");
}

/// Retrying is not passing. An outage that outlasts every attempt leaves the
/// lockfiles unaudited, and an unaudited lockfile is exactly what this gate
/// refuses to wave through -- so it stays closed and says why.
#[test]
fn an_outage_that_outlasts_every_attempt_still_closes_the_gate() {
    let (output, audits) = run_audit_gate(OUTAGE, 99, 2);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "an unaudited lockfile must not pass: {stdout}{stderr}"
    );
    assert!(
        stderr.contains("::error title=advisory registry unreachable"),
        "the give-up must annotate the job, not just exit: {stderr}"
    );
    assert_eq!(
        audits, 2,
        "the gate must stop at FORMAL_AI_AUDIT_ATTEMPTS: {stdout}"
    );
}
