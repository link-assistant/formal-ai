//! Regression coverage for issue #1001: the released container image was private.
//!
//! `ghcr.io/link-assistant/formal-ai` was pushed with the workflow
//! `GITHUB_TOKEN`, which creates a *private* package -- container packages do
//! not inherit the visibility of the repository they are linked to, only its
//! access permissions. The pipeline never looked at the result, so every
//! release shipped an image that no anonymous `docker pull` could fetch, and no
//! job failed. Hive Mind found it months later as
//! `error from registry: unauthorized`.
//!
//! The gate is `scripts/verify-ghcr-visibility.sh`, probing GHCR's anonymous
//! token endpoint (200 public / 401 private / 403 absent). Each assertion below
//! fails against the pre-fix pipeline.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::workflow_fixtures::{job_block, release_workflow};

fn script() -> String {
    format!(
        "{}/scripts/verify-ghcr-visibility.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// A sandbox whose `curl` answers with `status` and records every invocation,
/// so the probe can be exercised without touching the network.
fn sandbox(status: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-ghcr-visibility-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("sandbox must be created");
    let curl = dir.join("curl");
    fs::write(
        &curl,
        format!(
            "#!/usr/bin/env bash\n\
             printf '%s\\n' \"$*\" >> \"$SANDBOX/calls.log\"\n\
             out=\"\"\n\
             while [ $# -gt 0 ]; do\n\
               if [ \"$1\" = '-o' ]; then out=\"$2\"; fi\n\
               shift\n\
             done\n\
             if [ -n \"$out\" ]; then echo '{{\"token\":\"super-secret-pull-token\"}}' > \"$out\"; fi\n\
             printf '{status}'\n"
        ),
    )
    .expect("fake curl must be written");
    fs::set_permissions(&curl, fs::Permissions::from_mode(0o755))
        .expect("fake curl must be executable");
    dir
}

fn run(dir: &Path, image: Option<&str>) -> Output {
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = Command::new("bash");
    command
        .arg(script())
        .env("PATH", path)
        .env("SANDBOX", dir)
        .env("VERIFY_GHCR_VISIBILITY_DELAY", "0")
        .env_remove("GHCR_IMAGE");
    if let Some(image) = image {
        command.env("GHCR_IMAGE", image);
    }
    command.output().expect("probe must run")
}

fn calls(dir: &Path) -> String {
    fs::read_to_string(dir.join("calls.log")).unwrap_or_default()
}

fn cleanup(dir: &Path) {
    fs::remove_dir_all(dir).expect("sandbox must be removed");
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn a_public_package_passes_and_never_prints_the_pull_token() {
    let dir = sandbox("200");
    let output = run(&dir, Some("ghcr.io/link-assistant/formal-ai"));
    let calls = calls(&dir);
    cleanup(&dir);

    assert!(output.status.success(), "a 200 means the image is public");
    let text = format!(
        "{}{}",
        stdout(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("is public"), "the pass must say why: {text}");
    assert!(
        !text.contains("super-secret-pull-token"),
        "the issued token must never be echoed into the build log: {text}"
    );
    assert_eq!(calls.lines().count(), 1, "a 200 needs no retry");
}

/// The exact production state: the package exists, but only to authenticated
/// callers. This is what must turn a green release red.
#[test]
fn a_private_package_fails_the_release_with_the_manual_remedy() {
    let dir = sandbox("401");
    let output = run(&dir, Some("ghcr.io/link-assistant/formal-ai"));
    cleanup(&dir);

    assert!(
        !output.status.success(),
        "a private image must fail the release instead of being found downstream"
    );
    let text = stdout(&output);
    assert!(text.contains("::error::"), "must annotate the run: {text}");
    assert!(text.contains("PRIVATE"), "{text}");
    assert!(
        text.contains("Change visibility"),
        "the failure must name the manual remedy: {text}"
    );
}

#[test]
fn an_absent_package_is_reported_as_a_missing_push_not_as_privacy() {
    let dir = sandbox("403");
    let output = run(&dir, Some("ghcr.io/link-assistant/formal-ai"));
    cleanup(&dir);

    assert!(!output.status.success());
    let text = stdout(&output);
    assert!(text.contains("DENIED"), "{text}");
    assert!(
        !text.contains("PRIVATE"),
        "403 is not the private case: {text}"
    );
}

/// A registry hiccup must not be reported as "private" -- that would send a
/// release engineer to the package settings for a problem that is not there.
#[test]
fn transient_failures_are_retried_and_then_reported_as_transient() {
    let dir = sandbox("503");
    let output = run(&dir, Some("ghcr.io/link-assistant/formal-ai"));
    let attempts = calls(&dir).lines().count();
    cleanup(&dir);

    assert!(!output.status.success());
    assert_eq!(attempts, 3, "the default retry budget must be spent");
    let text = stdout(&output);
    assert!(text.contains("kept failing"), "{text}");
    assert!(!text.contains("PRIVATE"), "{text}");
}

#[test]
fn an_unexpected_status_fails_loudly_rather_than_passing_by_default() {
    let dir = sandbox("418");
    let output = run(&dir, Some("ghcr.io/link-assistant/formal-ai"));
    cleanup(&dir);

    assert!(!output.status.success());
    assert!(stdout(&output).contains("Unexpected HTTP 418"));
}

#[test]
fn a_missing_image_reference_is_not_silently_treated_as_success() {
    let dir = sandbox("200");
    let output = run(&dir, None);
    cleanup(&dir);

    assert!(
        !output.status.success(),
        "an unset GHCR_IMAGE means the probe verified nothing"
    );
    assert!(stdout(&output).contains("GHCR_IMAGE is not set"));
}

/// Visibility is a property of the package, not of a tag, so the tag must be
/// stripped before it is spliced into the `repository:<name>:pull` scope --
/// otherwise every probe would ask about a repository that does not exist and
/// report a spurious 403.
#[test]
fn the_probe_scope_drops_the_registry_host_and_the_tag() {
    for image in [
        "ghcr.io/link-assistant/formal-ai:0.339.1",
        "ghcr.io/link-assistant/formal-ai@sha256:0123456789abcdef",
        "ghcr.io/link-assistant/formal-ai",
    ] {
        let dir = sandbox("200");
        run(&dir, Some(image));
        let calls = calls(&dir);
        cleanup(&dir);

        assert!(
            calls.contains("scope=repository:link-assistant/formal-ai:pull"),
            "{image} produced the wrong scope: {calls}"
        );
    }
}

/// Sending credentials would make a private package answer 200 and recreate
/// exactly the false negative this gate exists to remove.
#[test]
fn the_probe_stays_anonymous() {
    let body = fs::read_to_string(script()).expect("probe script");
    let executable: String = body
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    for credential in ["Authorization", "GITHUB_TOKEN", "-u ", "--user"] {
        assert!(
            !executable.contains(credential),
            "the probe must not authenticate; found {credential}"
        );
    }
}

#[test]
fn both_release_paths_verify_the_image_they_just_pushed() {
    let workflow = release_workflow();

    for job_name in ["auto-release", "manual-release"] {
        let job = job_block(&workflow, job_name);
        assert!(
            job.contains("run: bash scripts/verify-ghcr-visibility.sh"),
            "{job_name} must verify the GHCR image it publishes (issue #1001)"
        );

        let push = job
            .split("- name: Publish Docker image to GHCR\n")
            .nth(1)
            .unwrap_or_else(|| panic!("{job_name} must publish to GHCR"));
        let verify_offset = push
            .find("- name: Verify the published GHCR image is anonymously pullable")
            .unwrap_or_else(|| panic!("{job_name} must verify after the push"));
        let dockerhub_offset = push.find("- name: Configure Docker Hub publishing");
        if let Some(dockerhub_offset) = dockerhub_offset {
            assert!(
                verify_offset < dockerhub_offset,
                "{job_name}: the verification must run right after the GHCR push"
            );
        }

        // The gate has to be conditioned exactly like the push it verifies, or
        // it would fail every run that legitimately publishes nothing.
        let push_condition = step_condition(push);
        let verify_condition = step_condition(&push[verify_offset..]);
        assert_eq!(
            push_condition, verify_condition,
            "{job_name}: verification must run under the same condition as the push"
        );
    }
}

/// Extract the `if:` block of the first step in `step_body`.
fn step_condition(step_body: &str) -> String {
    let body = step_body
        .split_once("\n      - ")
        .map_or(step_body, |(head, _)| head);
    // `if: |` folds its condition over following lines indented deeper than the
    // step's own keys (8 spaces).
    let mut condition = Vec::new();
    for line in body
        .lines()
        .skip_while(|line| !line.trim_start().starts_with("if:"))
    {
        let indent = line.len() - line.trim_start().len();
        if condition.is_empty() || indent > 8 {
            condition.push(line.trim());
        } else {
            break;
        }
    }
    condition.join(" ")
}

#[test]
fn the_probe_script_is_valid_bash() {
    let status = Command::new("bash")
        .arg("-n")
        .arg(script())
        .status()
        .expect("bash must run");
    assert!(
        status.success(),
        "scripts/verify-ghcr-visibility.sh must parse"
    );
}
