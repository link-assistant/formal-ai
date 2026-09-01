//! Docker-resource hygiene required by issue #1069's repeated server runs.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
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
