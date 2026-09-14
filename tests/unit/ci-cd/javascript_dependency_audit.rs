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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Distinguishes scratch directories that the clock cannot.
static SCRATCH_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

/// A scratch directory for one test's stand-in binaries and call logs.
///
/// The clock alone is not enough to tell two of these apart. Several tests
/// share a mode name, they run concurrently, and `SystemTime::now()` is only as
/// fine-grained as the host clock -- which on a container can be coarse enough
/// that two of them land on the same nanosecond. Sharing a directory means
/// sharing the call logs, and a test then counts another's attempts: this suite
/// first failed by reporting four attempts from a gate configured for two.
fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after 1970")
        .as_nanos();
    let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("javascript-audit-{name}-{nanos}-{sequence}"));
    fs::create_dir_all(&path).expect("create the scratch directory");
    path
}

/// What the gate saw in run 100928011479: a request that never got an answer.
/// This is `bun audit`'s wording.
const OUTAGE: &str = "outage";

/// The same fault in `npm audit`'s wording, which is not `bun`'s.
const NPM_OUTAGE: &str = "npm-outage";

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
fn run_audit_gate(mode: &str, failures: u32, attempts: u32) -> (Output, Vec<usize>) {
    run_bounded_audit_gate(mode, failures, attempts, 180, 300)
}

/// The same, with the deadline one attempt gets and the budget they all share.
fn run_bounded_audit_gate(
    mode: &str,
    failures: u32,
    attempts: u32,
    attempt_seconds: u32,
    budget_seconds: u32,
) -> (Output, Vec<usize>) {
    let directory = temp_dir(mode);
    let calls = directory.join("calls");

    let stand_in = format!(
        "#!/usr/bin/env bash\n\
         # One call log per workspace. The audits run concurrently now, so a\n\
         # single shared log would race and count somebody else's attempts.\n\
         log=\"$FAKE_AUDIT_CALLS.$(printf '%s' \"$PWD\" | tr -c 'A-Za-z0-9' '_')\"\n\
         printf '%s\\n' \"$0 $*\" >> \"$log\"\n\
         if [ \"$(wc -l < \"$log\")\" -le \"$FAKE_AUDIT_FAILURES\" ]; then\n\
         \x20 case \"$FAKE_AUDIT_MODE\" in\n\
         \x20   {ADVISORY})\n\
         \x20     echo '1 vulnerability found (1 moderate)'\n\
         \x20     exit 1 ;;\n\
         \x20   {HANG})\n\
         \x20     # `exec`, because `timeout` signals its direct child and\n\
         \x20     # nothing else: a forked sleep outlives the kill and keeps\n\
         \x20     # the pipe the gate is still reading from open.\n\
         \x20     exec sleep 300 ;;\n\
         \x20   {SLOW})\n\
         \x20     sleep 1 ;;\n\
         \x20   {NPM_OUTAGE})\n\
         \x20     echo 'npm warn audit 503 Service Unavailable - POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - Service Unavailable' >&2\n\
         \x20     echo 'npm warn audit network timeout at: https://registry.npmjs.org/-/npm/v1/security/advisories/bulk' >&2\n\
         \x20     echo 'npm error audit endpoint returned an error' >&2\n\
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
        .env(
            "FORMAL_AI_AUDIT_ATTEMPT_SECONDS",
            attempt_seconds.to_string(),
        )
        .env("FORMAL_AI_AUDIT_BUDGET_SECONDS", budget_seconds.to_string())
        .env("FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS", "0")
        .output()
        .expect("run the JavaScript dependency gate");

    // One log per lockfile, so "how many attempts" is answerable per lockfile
    // rather than only in total. Sorted because concurrent audits finish in no
    // fixed order, and it is the shape that carries the meaning.
    let mut attempts_per_lock: Vec<usize> = fs::read_dir(&directory)
        .expect("read the scratch directory")
        .filter_map(|entry| {
            let path = entry.expect("read a scratch entry").path();
            let name = path.file_name()?.to_str()?;
            name.starts_with("calls.").then(|| {
                fs::read_to_string(&path)
                    .expect("read a call log")
                    .lines()
                    .count()
            })
        })
        .collect();
    attempts_per_lock.sort_unstable();
    (output, attempts_per_lock)
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
        10,
        "each unanswered attempt must be visible in the job log: {stderr}"
    );
    // Every lockfile is turned away twice and answered on the third attempt.
    assert_eq!(
        audits,
        vec![3; 5],
        "the gate must retry rather than skip, on every lockfile: {stdout}"
    );
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
    assert_eq!(
        audits,
        vec![1; 5],
        "an answer is not retried, on any lockfile: {stdout}"
    );
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
        audits,
        vec![2; 5],
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
    // than the wording.
    assert!(
        elapsed < Duration::from_secs(60),
        "a 2s deadline against a 300s hang must not take {elapsed:?}"
    );
    assert_eq!(
        audits,
        vec![2; 5],
        "each lockfile hangs once and is answered once: {stdout}"
    );
}

/// A lockfile that is getting no answer stops at its own budget rather than at
/// the job's `timeout-minutes`, which is far later and cancels the job before
/// it can say why. Because the lockfiles are audited concurrently, that budget
/// bounds the whole gate and not just one fifth of it.
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
        "five 2s attempts per lockfile must be cut short by a 4s budget, \
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
    // Five committed lockfiles, each answering on its first attempt, each
    // taking as long as the whole outage budget.
    assert_eq!(
        audits,
        vec![1; 5],
        "each lockfile is audited exactly once: {stdout}"
    );
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

/// `bun` and `npm` do not describe an outage the same way, and the first draft
/// of the transport-fault list was written from `bun` alone. Replayed against a
/// degraded registry, `npm audit` printed none of the strings it recognised --
/// no trailing `- 503`, no `code E*` -- so a genuine npm outage fell through to
/// the failing branch and would have turned the branch red for something
/// npmjs.org never said, which is the exact failure this gate was fixed for.
/// The three lines below are that run's output, verbatim.
#[test]
fn an_outage_worded_the_way_npm_words_it_is_also_retried() {
    let (output, audits) = run_audit_gate(NPM_OUTAGE, 2, 3);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "npm's wording for an unreachable registry must be read as an outage, \
         not as a finding: {stdout}{stderr}"
    );
    assert_eq!(
        stderr
            .matches("::warning title=advisory registry unreachable")
            .count(),
        10,
        "both unanswered attempts are announced as outages: {stderr}"
    );
    assert_eq!(
        audits,
        vec![3; 5],
        "the gate retries to an answer: {stdout}"
    );
}

/// The failure this gate was cancelled for in run 100973301529, which neither
/// limit above would have caught: every audit answered, and the job ran out of
/// time anyway. The five lockfiles took 92s, 97s, 155s and 203s-and-counting,
/// one after another, against a registry that was slow rather than down. A
/// deadline does not help -- nothing was killed -- and neither does the outage
/// budget, which charges only attempts that never answered.
///
/// What helps is not paying for the waits one at a time. Five lockfiles that
/// each take three seconds cost fifteen seconds in sequence and three
/// concurrently, so the clock is the assertion: anything at or above the sum
/// means the gate went back to auditing them in turn.
#[test]
fn the_lockfiles_are_audited_concurrently_rather_than_one_wait_after_another() {
    let started = SystemTime::now();
    // One attempt each, killed at three seconds, with a budget far too large to
    // be what stopped it -- so the only thing the clock can be measuring is
    // whether the five waits overlapped.
    let (output, audits) = run_bounded_audit_gate(HANG, 99, 1, 3, 300);
    let elapsed = started.elapsed().expect("read the clock");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "five unaudited lockfiles must not pass: {stdout}{stderr}"
    );
    assert_eq!(
        audits,
        vec![1; 5],
        "every lockfile is audited, not just the ones before the first \
         failure: {stdout}"
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "five 3s waits must overlap, not add up: {elapsed:?} is the sequential \
         cost, which is what cancelled run 100973301529"
    );
}
