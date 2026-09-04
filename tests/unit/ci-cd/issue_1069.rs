//! Docker-resource hygiene required by issue #1069's repeated server runs.

use std::fs;

use super::workflow_fixtures::{job_block, workflow_step_block};

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

/// The two computer-use E2E steps bound the whole run, not only each session.
///
/// Run 33880485514 killed `Run agent CLI E2E — verified computer-use
/// record/replay (issue #707)` at its 10-minute `timeout-minutes`, ten
/// scenarios into the twenty it drives, and the only diagnosis the log carried
/// was `##[error]The action ... has timed out after 10 minutes` -- which
/// scenario ran long had to be reconstructed from stdout timestamps. Nothing
/// had regressed: the same step on the same branch measured 131s, 136s and
/// 533s on green runs, because every session waits on a remote model.
///
/// The defect is the one issue #977 and issue #1017 named from the other side:
/// `timeout-minutes` was the deadline instead of the backstop. The script
/// bounded each session (`AGENT_TIMEOUT_SECONDS`, 120s) and nothing bounded the
/// run, so twenty sessions were entitled to 2400s under a 600s step -- a
/// budget that could only ever hold on a fast day. Three clocks now nest, each
/// strictly inside the next: the script clamps every session to what is left of
/// `TEST_BUDGET_SECONDS` and names the scenario that spent it,
/// `run-with-budget-warning.sh` terminates at the budget with an `::error`, and
/// `timeout-minutes` is left as the backstop it is supposed to be.
#[test]
fn computer_use_e2e_steps_bound_the_run_and_not_only_each_session() {
    let workflow = repository_file(".github/workflows/release.yml");
    let job = job_block(&workflow, "test-agent-cli-e2e");

    for (step_name, script_path) in [
        (
            "\"Run agent CLI E2E — verified computer-use record/replay (issue #707)\"",
            "experiments/agent_cli_e2e/run_issue_707.sh",
        ),
        (
            "\"Run agent CLI E2E — held-out computer-use generalization (issue #707)\"",
            "experiments/agent_cli_e2e/run_issue_707_generalization.sh",
        ),
    ] {
        let step = workflow_step_block(job, step_name);
        assert!(
            step.contains("scripts/run-with-budget-warning.sh"),
            "{step_name} must own its deadline instead of waiting for the runner"
        );
        let budget_seconds: u64 = step
            .lines()
            .find_map(|line| line.trim().strip_prefix("TEST_BUDGET_SECONDS:"))
            .unwrap_or_else(|| panic!("{step_name} declares no TEST_BUDGET_SECONDS"))
            .trim()
            .parse()
            .expect("the budget is a plain number of seconds");
        let backstop_minutes: u64 = step
            .lines()
            .find_map(|line| line.trim().strip_prefix("timeout-minutes:"))
            .unwrap_or_else(|| panic!("{step_name} declares no timeout-minutes"))
            .trim()
            .parse()
            .expect("the backstop is a plain number of minutes");
        // The wrapper terminates at the budget and then waits out a SIGTERM
        // grace before SIGKILL, so a backstop equal to the budget still lets
        // the runner win the race and report `cancelled` (issue #977).
        assert!(
            backstop_minutes * 60 >= budget_seconds + 60,
            "{step_name}: a {budget_seconds}s budget under a {backstop_minutes}m backstop \
             leaves the wrapper no room to terminate, warn and exit 124 first"
        );

        let script = repository_file(script_path);
        assert!(
            script.contains("TEST_BUDGET_SECONDS"),
            "{script_path} must read the budget of the step that runs it"
        );
        assert!(
            !script.contains("timeout \"$AGENT_TIMEOUT_SECONDS\""),
            "{script_path} must clamp each session to what is left of the run budget, \
             or the sessions it is entitled to run outlast the step that runs them"
        );
        assert!(
            script.contains("timeout \"$session_seconds\""),
            "{script_path} must pass the clamped per-session deadline"
        );
        assert!(
            script.contains("of ${LOOP_DEADLINE_SECONDS}s"),
            "{script_path} must print elapsed time beside each scenario, so a step that \
             runs long names the scenario instead of needing stdout timestamps"
        );
    }
}
