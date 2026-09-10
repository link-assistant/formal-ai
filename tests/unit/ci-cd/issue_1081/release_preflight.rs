//! Issue #1081, D15: the release proved it could publish only by publishing.
//!
//! Principle 16 of `docs/CI-CD-BEST-PRACTICES.md` -- "Prove You Can Publish
//! Before You Build" -- was the one principle this repository implemented at
//! neither end. Every release credential was exercised for the first time by
//! the step that used it: `CARGO_TOKEN` by `cargo publish`, `GITHUB_TOKEN` by
//! the GHCR push, `DOCKERHUB_TOKEN` by `docker/login-action`, all of them
//! inside `auto-release`, a job with a 90-minute cap whose Docker step alone
//! is budgeted 45 minutes. A revoked token therefore cost a full build before
//! anyone learned the release could not happen, and the failure arrived as a
//! registry error in the middle of a long log rather than as an answer.
//!
//! `scripts/preflight-credentials.sh` asks the same registries the same
//! question in seconds, from a job the release jobs `needs:`. The tests below
//! pin the wiring in the workflow and the behaviour of the script, the latter
//! against a stubbed `curl` so the verdicts can be exercised offline.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

use super::super::workflow_fixtures::{job_block, release_workflow, workflow_step_block};

fn script() -> String {
    format!(
        "{}/scripts/preflight-credentials.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// A sandbox whose `curl` is the offline stub in `experiments/`, so the probe
/// can be driven through every verdict without a network. The experiment script
/// is reused rather than copied: a second copy is a second thing to keep true.
fn sandbox() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-preflight-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("sandbox must be created");
    let stub = format!(
        "{}/experiments/issue_1081_preflight/fake-curl.sh",
        env!("CARGO_MANIFEST_DIR")
    );
    let curl = dir.join("curl");
    fs::copy(&stub, &curl).expect("the curl stub must be installed");
    fs::set_permissions(&curl, fs::Permissions::from_mode(0o755)).expect("the stub is executable");
    dir
}

/// Every environment variable `scripts/preflight-credentials.sh` reads.
///
/// A test that sets only the variables it cares about is not testing the
/// script -- it is testing the script plus whatever the machine happens to
/// export. Run 34136192028 failed exactly there: the runner sets
/// `CARGO_REGISTRY_TOKEN`, the script prefers it over the `CARGO_TOKEN` the
/// test had set, and the probe went out with the runner's real credential
/// while the assertion looked for the fixture's. The suite passed on every
/// developer machine, where that variable is absent. So clear the whole set
/// first and set back only what the case means to provide; the ambient
/// environment then cannot decide the verdict either way.
///
/// `the_preflight_env_list_covers_every_variable_the_script_reads` keeps this
/// list honest against the script.
const PREFLIGHT_INPUTS: &[&str] = &[
    "CARGO_REGISTRY_TOKEN",
    "CARGO_TOKEN",
    "CRATES_IO_API",
    "CRATE_NAME",
    "DOCKERHUB_IMAGE",
    "DOCKERHUB_REGISTRY",
    "DOCKERHUB_TOKEN",
    "DOCKERHUB_TOKEN_ENDPOINT",
    "DOCKERHUB_USERNAME",
    "GHCR_IMAGE",
    "GHCR_REGISTRY",
    "GHCR_TOKEN_ENDPOINT",
    "GITHUB_ACTOR",
    "GITHUB_REPOSITORY",
    "GITHUB_STEP_SUMMARY",
    "GITHUB_TOKEN",
    "PREFLIGHT_MODE",
    "PREFLIGHT_RETRIES",
    "PREFLIGHT_RETRY_DELAY",
    "PREFLIGHT_VERBOSE",
];

/// A command that runs the preflight against the stubbed `curl` in `dir`, with
/// every credential present and `routes` deciding what each registry answers.
/// The caller may still add or remove variables; it starts from a known state.
fn preflight(dir: &std::path::Path, mode: &str, routes: &str) -> Command {
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = Command::new("bash");
    command.arg(script()).args(["--mode", mode]);
    for name in PREFLIGHT_INPUTS {
        command.env_remove(name);
    }
    command
        .env("PATH", path)
        .env("FAKE_CURL_ROUTES", routes)
        .env("CARGO_TOKEN", "cargo-token")
        .env("CRATE_NAME", "formal-ai")
        .env("GHCR_IMAGE", "ghcr.io/link-assistant/formal-ai")
        .env("GITHUB_ACTOR", "konard")
        .env("GITHUB_TOKEN", "job-token")
        .env("DOCKERHUB_IMAGE", "linkassistant/formal-ai")
        .env("DOCKERHUB_USERNAME", "user")
        .env("DOCKERHUB_TOKEN", "hub-token")
        .env("PREFLIGHT_RETRIES", "1")
        .env("PREFLIGHT_RETRY_DELAY", "0");
    command
}

/// Run the preflight with every credential present and `routes` deciding what
/// each registry answers.
fn run(mode: &str, routes: &str) -> Output {
    let dir = sandbox();
    let output = preflight(&dir, mode, routes)
        .env("FAKE_CURL_LOG", dir.join("calls.log"))
        .output()
        .expect("the preflight must run");
    fs::remove_dir_all(&dir).ok();
    output
}

const EVERYTHING_PUBLISHABLE: &str = concat!(
    "/crates/ 200 {\"crate\":{\"name\":\"formal-ai\"}}\n",
    "/token 200 {\"token\":\"bearer\"}\n",
    "blobs/uploads 202\n"
);

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// crates.io cannot be verified read-only (see
/// `the_crates_io_token_is_never_judged_by_the_cookie_only_me_endpoint`), so a
/// fully publishable release reports it as the one honest `unknown` and is not
/// blocked by it.
#[test]
fn a_release_whose_credentials_all_work_is_not_blocked() {
    let output = run("release", EVERYTHING_PUBLISHABLE);
    let report = stdout(&output);
    assert!(
        output.status.success(),
        "a publishable release must not be blocked: {report}"
    );
    assert!(
        report.contains("2 verified, 0 failed, 1 unknown"),
        "{report}"
    );
    assert!(
        report.contains("GET /api/v1/me is cookie-only"),
        "the unknown verdict must say why crates.io cannot be probed: {report}"
    );
}

/// The point of the preflight is one report naming *every* broken credential,
/// so a probe that fails must not stop the ones after it. The crates.io failure
/// here is the only one the probe can still establish: an absent token.
#[test]
fn every_failure_is_reported_and_not_only_the_first() {
    let dir = sandbox();
    let output = preflight(
        &dir,
        "release",
        "/token 200 {\"token\":\"bearer\"}\nblobs/uploads 403\n",
    )
    .env_remove("CARGO_TOKEN")
    .output()
    .expect("the preflight must run");
    fs::remove_dir_all(&dir).ok();
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("crates.io publish token"), "{report}");
    assert!(report.contains("GHCR push"), "{report}");
    assert!(report.contains("Docker Hub push"), "{report}");
    assert!(report.contains("0 verified, 3 failed"), "{report}");
}

/// Issue #1085. `GET /api/v1/me` is `AuthCheck::only_cookie()` in crates.io
/// (`src/controllers/user/me.rs`): it answers 403 to every API token, valid or
/// not. The issue #1081 probe read that 403 as a revoked token and blocked the
/// release of run 34149311523 with the token that had published v0.347.0 two
/// days earlier. No crates.io read endpoint accepts a token, so the honest
/// verdict is `unknown`, the token is never sent anywhere, and `/me` is never
/// called.
#[test]
fn the_crates_io_token_is_never_judged_by_the_cookie_only_me_endpoint() {
    let dir = sandbox();
    let log = dir.join("calls.log");
    let config_log = dir.join("config.log");
    let output = preflight(
        &dir,
        "release",
        concat!(
            "/me 403 {\"errors\":[{\"detail\":\"authentication failed\"}]}\n",
            "/crates/ 200 {\"crate\":{\"name\":\"formal-ai\"}}\n",
            "/token 200 {\"token\":\"bearer\"}\n",
            "blobs/uploads 202\n"
        ),
    )
    .env("FAKE_CURL_LOG", &log)
    .env("FAKE_CURL_CONFIG_LOG", &config_log)
    .output()
    .expect("the preflight must run");
    let calls = fs::read_to_string(&log).expect("the stub must record its calls");
    let configs =
        fs::read_to_string(&config_log).expect("the stub must record what it read on stdin");
    fs::remove_dir_all(&dir).ok();
    let report = stdout(&output);
    assert!(
        output.status.success(),
        "a 403 from a cookie-only endpoint is not a rejected token: {report}"
    );
    assert!(report.contains("1 unknown"), "{report}");
    assert!(
        !calls.contains("/api/v1/me"),
        "the cookie-only endpoint must never be consulted: {calls}"
    );
    assert!(
        !calls.contains("cargo-token") && !configs.contains("cargo-token"),
        "the publish token has no read-only use and must not leave the job: {calls}\n{configs}"
    );
    assert!(
        !report.contains("revoked, expired or misscoped"),
        "the verdict the false positive produced must be gone: {report}"
    );
}

/// A registry token endpoint answers 200 for scopes it will not honour, so the
/// only probe that is not a guess is the one that tries to write.
#[test]
fn a_token_that_authenticates_but_cannot_push_is_a_failure() {
    let output = run(
        "release",
        concat!(
            "/crates/ 200 {\"crate\":{\"name\":\"formal-ai\"}}\n",
            "/token 200 {\"token\":\"bearer\"}\n",
            "blobs/uploads 403\n"
        ),
    );
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(
        report.contains("authenticates but cannot write"),
        "a 200 from the token endpoint must not be read as permission to push: {report}"
    );
}

/// "0 verified, 3 unknown" is actionable; "no failures" is not.
#[test]
fn a_run_that_could_verify_nothing_is_not_a_pass() {
    let output = run("release", "/ 000\n");
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(
        report.contains("0 verified, 0 failed, 3 unknown"),
        "{report}"
    );
    assert!(
        report.contains("a run that verified no credential is not a pass"),
        "{report}"
    );
    assert!(
        !report.contains("::error::3 release credential"),
        "an unreachable registry has not said the credential is broken: {report}"
    );
}

/// On a pull request the same absence means nothing: forks have no secrets and
/// the code can still be tested.
#[test]
fn report_mode_states_the_problem_without_failing_the_run() {
    let output = run(
        "report",
        "/token 200 {\"token\":\"bearer\"}\nblobs/uploads 403\n",
    );
    let report = stdout(&output);
    assert!(output.status.success(), "{report}");
    assert!(report.contains("would block a release"), "{report}");
    assert!(!report.contains("::error::"), "{report}");
}

/// The probe must leave nothing behind in the registry it probed -- and no
/// credential in the one place a secret cannot be taken back from. An argument
/// list is world-readable while the process lives (`/proc/<pid>/cmdline`, `ps`),
/// so the tokens have to travel to curl by another route; the stub records the
/// two channels separately, and this test reads both.
#[test]
fn every_opened_blob_upload_session_is_cancelled() {
    let dir = sandbox();
    let log = dir.join("calls.log");
    let config_log = dir.join("config.log");
    let output = preflight(&dir, "release", EVERYTHING_PUBLISHABLE)
        .env("FAKE_CURL_LOG", &log)
        .env("FAKE_CURL_CONFIG_LOG", &config_log)
        .output()
        .expect("the preflight must run");
    let calls = fs::read_to_string(&log).expect("the stub must record its calls");
    let configs =
        fs::read_to_string(&config_log).expect("the stub must record what it read on stdin");
    fs::remove_dir_all(&dir).ok();
    assert!(output.status.success(), "{}", stdout(&output));
    assert_eq!(
        calls.matches("-X POST").count(),
        2,
        "one upload session per registry: {calls}"
    );
    assert_eq!(
        calls.matches("-X DELETE").count(),
        2,
        "each opened session must be cancelled: {calls}"
    );
    for secret in ["cargo-token", "hub-token", "job-token"] {
        assert!(
            !calls.contains(secret),
            "no credential may reach the argument list of a logged call: {calls}"
        );
    }
    // The registry credential still has to be sent, or the probe proves
    // nothing: the same assertion passes trivially for a script that
    // authenticates with nothing at all. The crates.io token is the exception
    // (issue #1085): no read endpoint accepts it, so it must not be sent.
    assert!(
        !configs.contains("cargo-token"),
        "the crates.io token has no read-only use and must not be sent: {configs}"
    );
    assert!(
        configs.contains("Authorization: Bearer"),
        "the registry bearer token must still reach curl, by the private channel: {configs}"
    );
}

/// A preflight nothing needs is a report nobody reads.
#[test]
fn the_release_jobs_need_the_preflight() {
    let workflow = release_workflow();
    let preflight = job_block(&workflow, "release-preflight");
    assert!(
        preflight.contains("run: bash scripts/preflight-credentials.sh"),
        "the preflight job must run the probe"
    );
    assert!(
        preflight.contains("packages: write"),
        "the GHCR probe opens a blob upload session, which pull permission cannot do"
    );
    let step = workflow_step_block(preflight, "Probe every release credential");
    assert!(
        step.contains("&& 'release' || 'report' }}"),
        "the mode must be an expression, so a pull request reports and a release blocks"
    );
    assert!(
        step.contains("PREFLIGHT_VERBOSE: ${{ env.FORMAL_AI_CI_VERBOSE == 'true' && '1' || '0' }}"),
        "the tracing stays opt-in and default-off"
    );

    for job_name in ["auto-release", "manual-release"] {
        let job = job_block(&workflow, job_name);
        let needs = job
            .split("steps:")
            .next()
            .expect("a job has a header before its steps");
        assert!(
            needs.contains("release-preflight,"),
            "{job_name} must not start before the preflight has answered"
        );
        assert!(
            job.contains("needs.release-preflight.result == 'success' &&"),
            "{job_name} must refuse to run when the preflight found a credential that cannot publish"
        );
    }
}

/// Every credential the release actually uses has to be one the preflight
/// probes, or the preflight is a green tick over an untested subset.
#[test]
fn the_preflight_probes_every_credential_the_release_publishes_with() {
    let workflow = release_workflow();
    let probe = fs::read_to_string(script()).expect("the preflight script is readable");
    // These are the *names* of the credentials, never their values -- the
    // whole point is to compare the workflow's list against the probe's. They
    // are spelled `credential_name` rather than `secret` because CodeQL's
    // `rust/cleartext-logging` heuristic reads a variable called `secret` in a
    // panic message as a leaked credential (alerts #104, #105) and nothing in
    // the assertion text tells it otherwise. The name that is accurate is also
    // the name that does not raise a false alarm.
    for (credential_name, probed_as) in [
        ("CARGO_TOKEN", "CARGO_TOKEN"),
        ("DOCKERHUB_TOKEN", "DOCKERHUB_TOKEN"),
        ("DOCKERHUB_USERNAME", "DOCKERHUB_USERNAME"),
    ] {
        assert!(
            workflow.contains(credential_name),
            "{credential_name} is expected to be one of the release credentials"
        );
        assert!(
            probe.contains(probed_as),
            "{credential_name} is used by the release but never probed before it"
        );
    }
    assert!(
        probe.contains("GITHUB_TOKEN"),
        "the job token is what pushes to ghcr.io, so it is a release credential too"
    );
}

/// The hermetic-environment fix is only as good as its list. If the script
/// grows a variable and `PREFLIGHT_INPUTS` does not, the suite silently goes
/// back to reading the machine for that one input -- which is how run
/// 34136192028 failed in the first place, and the kind of regression that
/// shows up months later on a runner rather than here.
#[test]
fn the_preflight_env_list_covers_every_variable_the_script_reads() {
    let source = fs::read_to_string(script()).expect("the preflight script must be readable");

    // `${NAME:-default}`, `${NAME}` and `$NAME` in the forms this script uses.
    // Local variables are lower case by convention here, so upper case is the
    // environment; `FAKE_CURL_*` belongs to the stub, not to the script.
    let mut missing: Vec<String> = Vec::new();
    let bytes: Vec<char> = source.chars().collect();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != '$' {
            index += 1;
            continue;
        }
        let mut cursor = index + 1;
        if cursor < bytes.len() && bytes[cursor] == '{' {
            cursor += 1;
        }
        let start = cursor;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_uppercase()
                || bytes[cursor].is_ascii_digit()
                || bytes[cursor] == '_')
        {
            cursor += 1;
        }
        let name: String = bytes[start..cursor].iter().collect();
        index = cursor.max(index + 1);
        if name.len() < 2
            || name
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_digit())
        {
            continue;
        }
        if name.starts_with("FAKE_CURL") || name == "PATH" || name == "HOME" {
            continue;
        }
        if !PREFLIGHT_INPUTS.contains(&name.as_str()) && !missing.contains(&name) {
            missing.push(name);
        }
    }

    assert!(
        missing.is_empty(),
        "scripts/preflight-credentials.sh reads {missing:?}, which the tests do not clear; \
         add them to PREFLIGHT_INPUTS so no ambient value can decide a verdict"
    );
}
