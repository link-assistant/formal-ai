//! Docker-resource hygiene required by issue #1069's repeated server runs.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

#[test]
fn web_bundle_generation_skips_nondeterministic_identifier_minification() {
    let package: serde_json::Value =
        serde_json::from_str(&repository_file("package.json")).expect("package.json is valid JSON");
    let build = package["scripts"]["build:web"]
        .as_str()
        .expect("build:web is a string");

    // Bun #40657: 1.4.0 can assign different minified identifiers to an
    // unchanged graph under load. Keep the deterministic size reductions until
    // the fix after oven-sh/bun#40664 reaches a stable release.
    assert_eq!(build.matches("--minify-whitespace").count(), 4);
    assert_eq!(build.matches("--minify-syntax").count(), 4);
    assert!(
        !build.split_ascii_whitespace().any(|arg| arg == "--minify"),
        "bare --minify re-enables nondeterministic identifier renaming"
    );
}

#[test]
fn pre_commit_prunes_docker_without_blocking_commits() {
    let hook = repository_file(".githooks/pre-commit");
    assert!(hook.contains("scripts/prune-docker.sh"));
    assert!(
        hook.contains("scripts/prune-docker.sh\" || true"),
        "Docker being absent or unhealthy must never block a commit"
    );
}

#[test]
fn docker_pruner_checks_leaks_and_respects_a_ceiling() {
    let pruner = repository_file("scripts/prune-docker.sh");
    assert!(pruner.contains("docker ps -a"));
    assert!(pruner.contains("docker images -f dangling=true"));
    assert!(pruner.contains("docker container prune --force"));
    assert!(pruner.contains("docker image prune --force"));
    assert!(pruner.contains("DOCKER_MAX_SIZE_GB"));
    assert!(pruner.contains("docker system df"));
    assert!(pruner.contains("docker system prune --force"));
    assert!(pruner.contains("DOCKER_NO_PRUNE"));
}

#[test]
fn docker_jobs_prune_on_every_non_cancelled_exit() {
    let workflow = repository_file(".github/workflows/release.yml");
    let cleanup_steps = workflow
        .matches("if: ${{ !cancelled() }}\n        run: scripts/prune-docker.sh")
        .count();
    assert!(
        cleanup_steps >= 2,
        "the image-build and box-language Docker batches both need cleanup"
    );
}

#[test]
fn detached_memory_upgrade_container_is_automatically_removed() {
    let harness = repository_file("experiments/issue_982_memory_upgrade/run_container_upgrade.sh");
    assert!(harness.contains("docker run --rm -d --privileged --name \"$server\""));
    assert!(harness.contains("trap cleanup EXIT"));
    assert!(harness.contains("docker rm -f \"$server\""));
}

/// The authorship route is what lets Formal AI's work ride inside an ordinary
/// pull request instead of needing one of its own, so its contract is pinned
/// where a rewrite has to notice it: the three trailers the self-hosting metric
/// reads, an evidence directory carrying both markers that metric looks for, and
/// a workspace the Agent CLI cannot see its own logs through.
#[test]
fn the_authorship_route_commits_the_trailers_the_release_gate_reads() {
    let script = repository_file("scripts/author-change-with-formal-ai.sh");
    assert!(script.contains("Formal-AI-Session: %s"));
    assert!(script.contains("Formal-AI-Evidence: %s"));
    assert!(script.contains("Formal-AI-Pull-Request: %s"));
    assert!(
        script.contains("printf 'formal-ai session %s\\n' \"$session_id\""),
        "one evidence file must carry the producer marker and the session id together"
    );
    assert!(
        script.contains(
            r#"[[ "$pull_request" =~ ^https://github\.com/[^/]+/[^/]+/pull/[1-9][0-9]*$ ]]"#
        ),
        "a trailer the metric cannot parse must be rejected here, not at release time"
    );
}

#[test]
fn the_authorship_route_keeps_the_agent_cli_out_of_its_own_logs() {
    let script = repository_file("scripts/author-change-with-formal-ai.sh");
    assert!(script.contains("work=\"$(mktemp -d)\""));
    assert!(script.contains("state=\"$(mktemp -d)\""));
    assert!(
        script.contains(">\"$state/agent-stream.raw.log\""),
        "the live stream must land outside both the workspace and the evidence directory"
    );
    assert!(script.contains("FORMAL_AI_MEMORY_PATH=\"$state/memory.lino\""));
    assert!(script.contains("FORMAL_AI_DREAMING=0"));
}

/// Producing a change must not imply producing a pull request (issue #1069).
#[test]
fn the_authorship_route_neither_opens_a_pull_request_nor_pushes() {
    let script = repository_file("scripts/author-change-with-formal-ai.sh");
    assert!(!script.contains("gh pr create"));
    assert!(!script.contains("git -C \"$ROOT\" push"));
    assert!(
        script.contains("git -C \"$ROOT\" diff --cached --quiet && die"),
        "a run that reproduced the committed bytes has authored nothing"
    );
}
