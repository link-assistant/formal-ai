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
//! Retrying, though, is only an improvement while it can finish. Run
//! 100948708530 reached a clean answer on attempt 2 in 4.35s -- after attempt 1
//! had hung for five minutes, which cancelled the 15-minute job two lockfiles
//! later. So the gate also has to be quick about failing.
//!
//! These tests hold both from the outside, through the script's own entry
//! point: an unanswered registry is retried, an answered one ends the gate on
//! the first attempt, a hung one is killed at its deadline, the whole gate
//! stops at its own budget, and an outage that outlasts every attempt still
//! fails rather than waving the lockfiles through.
//!
//! The last test holds the shipped numbers instead of injected ones, because a
//! deadline shorter than a healthy audit turns this gate from a fix into the
//! outage it was written to survive.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// What the gate saw in run 100948708530: a request that never returns at all.
const HANG: &str = "hang";

/// A registry that answers, just not quickly. Slowness is not an outage.
const SLOW: &str = "slow";

/// What the gate exists for: the registry answered, and the answer is a finding.
const ADVISORY: &str = "advisory";

/// Run the real gate with stand-in `bun` and `npm` on `PATH`.
///
/// `failures` is how many leading invocations misbehave in `mode`; every later
/// one reports a clean lockfile. Both stand-ins share one call log, so the
/// count the assertions read is the number of audits the gate actually ran.
fn run_audit_gate(mode: &str, failures: u32, attempts: u32) -> (Output, usize) {
    run_bounded_audit_gate(mode, failures, attempts, 180, 300)
}

/// The same, with the deadline one attempt gets and the budget they all share.
fn run_bounded_audit_gate(
    mode: &str,
    failures: u32,
    attempts: u32,
    attempt_seconds: u32,
    budget_seconds: u32,
) -> (Output, usize) {
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
         \x20   {HANG})\n\
         \x20     sleep 300 ;;\n\
         \x20   {SLOW})\n\
         \x20     sleep 1 ;;\n\
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
        .env(
            "FORMAL_AI_AUDIT_ATTEMPT_SECONDS",
            attempt_seconds.to_string(),
        )
        .env("FORMAL_AI_AUDIT_BUDGET_SECONDS", budget_seconds.to_string())
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

/// The failure from run 100948708530: an attempt that never returns is killed
/// at its own deadline, and the next one runs. Without this the gate spends the
/// job's budget waiting and the job is cancelled with no verdict at all.
#[test]
fn a_hung_request_is_killed_at_its_deadline_and_the_next_attempt_answers() {
    let started = SystemTime::now();
    let (output, audits) = run_bounded_audit_gate(HANG, 1, 2, 2, 240);
    let elapsed = started.elapsed().expect("read the clock");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "a hung request is not a finding about a lockfile: {stdout}{stderr}"
    );
    // 124 is the deadline's own status: the attempt was killed, not answered.
    assert!(
        stderr.contains("Attempt 1/2 exited 124 within its 2s deadline"),
        "the log must name the killed attempt and its deadline: {stderr}"
    );
    // The claim is that a 300s hang was cut at 2s, so check the clock rather
    // than the wording -- five lockfiles, one of which hung once.
    assert!(
        elapsed < Duration::from_secs(60),
        "a 2s deadline against a 300s hang must not take {elapsed:?}"
    );
    assert_eq!(
        audits, 6,
        "only the hung lockfile is audited twice: {stdout}"
    );
}

/// The attempts share one budget, so a registry that is down for everyone
/// cannot be paid for five times over. The gate stops at its own deadline,
/// which is far short of the job's, and says which one it hit.
#[test]
fn a_sustained_outage_stops_at_the_gate_budget_rather_than_the_job_timeout() {
    let started = SystemTime::now();
    let (output, _) = run_bounded_audit_gate(HANG, 99, 5, 2, 4);
    let elapsed = started.elapsed().expect("read the clock");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "an unaudited lockfile must not pass: {stdout}{stderr}"
    );
    assert!(
        stderr.contains("The gate's 4s budget ran out"),
        "the give-up must name the budget it hit, not just exit: {stderr}"
    );
    assert!(
        elapsed < Duration::from_secs(60),
        "five lockfiles x five 2s attempts must be cut short by a 4s budget, \
         not run to {elapsed:?}"
    );
}

/// The budget is for an outage, so only attempts that never answered are
/// charged to it. Were every second charged, a run of slow-but-healthy audits
/// would exhaust the budget and fail the gate for the one thing it has no
/// complaint about: five lockfiles that each reported no vulnerabilities.
#[test]
fn a_slow_but_answering_registry_is_never_charged_to_the_outage_budget() {
    let (output, audits) = run_bounded_audit_gate(SLOW, 99, 3, 10, 1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "audits that answer must not exhaust a budget meant for ones that do \
         not: {stdout}{stderr}"
    );
    assert!(
        !stderr.contains("budget ran out"),
        "a verdict, however slow, spends none of the outage budget: {stderr}"
    );
    // Five committed lockfiles, each answering on its first attempt, together
    // taking five times the one-second budget.
    assert_eq!(audits, 5, "each lockfile is audited once: {stdout}");
}

/// A healthy `npm audit --package-lock-only` over `desktop/package-lock.json`
/// -- the largest lockfile in the repository -- was timed at 2m01s while
/// reporting `found 0 vulnerabilities`. The first draft of this gate shipped a
/// 120s deadline and killed exactly that audit twice before giving up on its
/// budget, which is the outage it was meant to survive, self-inflicted. So the
/// shipped defaults are pinned against that measurement rather than left to a
/// later guess, and the worst case is pinned against the job that hosts them.
#[test]
fn the_shipped_deadline_outlasts_a_healthy_audit_yet_fails_inside_the_job() {
    /// The measured healthy audit, rounded up to whole seconds.
    const HEALTHY_AUDIT_SECONDS: u32 = 121;

    let gate = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/check-javascript-dependencies.sh"),
    )
    .expect("read the audit gate");

    let default = |name: &str| -> u32 {
        gate.lines()
            .find_map(|line| {
                let (_, rest) = line.split_once(&format!("${{{name}:-"))?;
                let (value, _) = rest.split_once('}')?;
                value.parse().ok()
            })
            .unwrap_or_else(|| panic!("{name} must have a default in the gate:\n{gate}"))
    };

    let attempts = default("FORMAL_AI_AUDIT_ATTEMPTS");
    let attempt_seconds = default("FORMAL_AI_AUDIT_ATTEMPT_SECONDS");
    let retry_delay_seconds = default("FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS");
    let budget_seconds = default("FORMAL_AI_AUDIT_BUDGET_SECONDS");

    assert!(
        attempt_seconds > HEALTHY_AUDIT_SECONDS,
        "an attempt deadline of {attempt_seconds}s kills an audit measured at \
         {HEALTHY_AUDIT_SECONDS}s, so the gate would report an outage it caused"
    );
    assert!(
        budget_seconds > attempt_seconds,
        "a budget of {budget_seconds}s cannot fit even one {attempt_seconds}s \
         attempt, so the gate would give up before trying once"
    );

    // The budget is checked before an attempt, never during one, so the gate
    // can overshoot by a whole attempt. Counting that overshoot is what makes
    // the ceiling a fact about the gate rather than about a good day.
    let charged_attempts = budget_seconds.div_ceil(attempt_seconds);
    let worst_case = (charged_attempts * attempt_seconds)
        + (charged_attempts.min(attempts).saturating_sub(1) * retry_delay_seconds);

    // `Lint and Format Check` allows itself fifteen minutes and spends about
    // nine on a healthy run (run 100940478841). The gate has to fail well
    // inside what is left, or `timeout-minutes` kills the job before anyone
    // can read why -- which is what happened in run 100948708530, and what
    // issue #1017 means by a backstop rather than a deadline.
    assert!(
        worst_case < 8 * 60,
        "a worst case of {worst_case}s risks the 15-minute job being cancelled \
         before the gate can say the registry never answered"
    );
}
