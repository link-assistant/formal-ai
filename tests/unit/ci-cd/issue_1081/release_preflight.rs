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

/// Run the preflight with every credential present and `routes` deciding what
/// each registry answers.
fn run(mode: &str, routes: &str) -> Output {
    let dir = sandbox();
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new("bash")
        .arg(script())
        .args(["--mode", mode])
        .env("PATH", path)
        .env("FAKE_CURL_ROUTES", routes)
        .env("FAKE_CURL_LOG", dir.join("calls.log"))
        .env("CARGO_TOKEN", "cargo-token")
        .env("CARGO_REGISTRY_TOKEN", "cargo-token")
        .env("CRATE_NAME", "formal-ai")
        .env("GHCR_IMAGE", "ghcr.io/link-assistant/formal-ai")
        .env("GITHUB_TOKEN", "job-token")
        .env("DOCKERHUB_IMAGE", "linkassistant/formal-ai")
        .env("DOCKERHUB_USERNAME", "user")
        .env("DOCKERHUB_TOKEN", "hub-token")
        .env("PREFLIGHT_RETRIES", "1")
        .env("PREFLIGHT_RETRY_DELAY", "0")
        .env_remove("GITHUB_STEP_SUMMARY")
        .output()
        .expect("the preflight must run");
    fs::remove_dir_all(&dir).ok();
    output
}

const EVERYTHING_PUBLISHABLE: &str = concat!(
    "/me 200 {\"user\":{\"login\":\"konard\"}}\n",
    "/owners 200 {\"users\":[{\"login\":\"konard\"}]}\n",
    "/token 200 {\"token\":\"bearer\"}\n",
    "blobs/uploads 202\n"
);

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn a_release_whose_credentials_all_work_is_not_blocked() {
    let output = run("release", EVERYTHING_PUBLISHABLE);
    let report = stdout(&output);
    assert!(
        output.status.success(),
        "a publishable release must not be blocked: {report}"
    );
    assert!(
        report.contains("4 verified, 0 failed, 0 unknown"),
        "{report}"
    );
}

/// The point of the preflight is one report naming *every* broken credential,
/// so a probe that fails must not stop the ones after it.
#[test]
fn every_failure_is_reported_and_not_only_the_first() {
    let output = run(
        "release",
        "/me 403\n/token 200 {\"token\":\"bearer\"}\nblobs/uploads 403\n",
    );
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("crates.io publish token"), "{report}");
    assert!(report.contains("GHCR push"), "{report}");
    assert!(report.contains("Docker Hub push"), "{report}");
    assert!(report.contains("0 verified, 3 failed"), "{report}");
}

/// A token crates.io accepts still publishes nothing if it does not own the
/// crate, which a login-shaped check cannot see.
#[test]
fn a_valid_token_that_does_not_own_the_crate_is_a_failure() {
    let output = run(
        "release",
        concat!(
            "/me 200 {\"user\":{\"login\":\"someone-else\"}}\n",
            "/owners 200 {\"users\":[{\"login\":\"konard\"}]}\n",
            "/token 200 {\"token\":\"bearer\"}\n",
            "blobs/uploads 202\n"
        ),
    );
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(
        report.contains("is not an owner of formal-ai"),
        "the ownership check must name what `cargo publish` would reject: {report}"
    );
}

/// A registry token endpoint answers 200 for scopes it will not honour, so the
/// only probe that is not a guess is the one that tries to write.
#[test]
fn a_token_that_authenticates_but_cannot_push_is_a_failure() {
    let output = run(
        "release",
        concat!(
            "/me 200 {\"user\":{\"login\":\"konard\"}}\n",
            "/owners 200 {\"users\":[{\"login\":\"konard\"}]}\n",
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
        "/me 403\n/token 200 {\"token\":\"bearer\"}\nblobs/uploads 202\n",
    );
    let report = stdout(&output);
    assert!(output.status.success(), "{report}");
    assert!(report.contains("would block a release"), "{report}");
    assert!(!report.contains("::error::"), "{report}");
}

/// The probe must leave nothing behind in the registry it probed.
#[test]
fn every_opened_blob_upload_session_is_cancelled() {
    let dir = sandbox();
    let log = dir.join("calls.log");
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new("bash")
        .arg(script())
        .args(["--mode", "release"])
        .env("PATH", path)
        .env("FAKE_CURL_ROUTES", EVERYTHING_PUBLISHABLE)
        .env("FAKE_CURL_LOG", &log)
        .env("CARGO_TOKEN", "cargo-token")
        .env("GHCR_IMAGE", "ghcr.io/link-assistant/formal-ai")
        .env("GITHUB_TOKEN", "job-token")
        .env("DOCKERHUB_IMAGE", "linkassistant/formal-ai")
        .env("DOCKERHUB_USERNAME", "user")
        .env("DOCKERHUB_TOKEN", "hub-token")
        .env("PREFLIGHT_RETRIES", "1")
        .env("PREFLIGHT_RETRY_DELAY", "0")
        .env_remove("GITHUB_STEP_SUMMARY")
        .output()
        .expect("the preflight must run");
    let calls = fs::read_to_string(&log).expect("the stub must record its calls");
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
    assert!(
        !calls.contains("cargo-token"),
        "no credential may reach the argument list of a logged call: {calls}"
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
    for (secret, probed_as) in [
        ("CARGO_TOKEN", "CARGO_TOKEN"),
        ("DOCKERHUB_TOKEN", "DOCKERHUB_TOKEN"),
        ("DOCKERHUB_USERNAME", "DOCKERHUB_USERNAME"),
    ] {
        assert!(
            workflow.contains(secret),
            "{secret} is expected to be one of the release credentials"
        );
        assert!(
            probe.contains(probed_as),
            "{secret} is used by the release but never probed before it"
        );
    }
    assert!(
        probe.contains("GITHUB_TOKEN"),
        "the job token is what pushes to ghcr.io, so it is a release credential too"
    );
}
