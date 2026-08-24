//! Regression coverage for issue #977: timeouts that hid as cancellations.
//!
//! A job killed by `timeout-minutes` is reported by GitHub as **cancelled**,
//! not **failed**, and a run whose only non-success job is cancelled inherits
//! the benign-looking `cancelled` conclusion. Eighteen consecutive `main` runs
//! went untriaged that way while `Auto Release` timed out mid-Docker-build, so
//! eleven versions (0.326.2 .. 0.333.0) reached crates.io and git tags with no
//! GitHub Release at all. The reconstruction is in
//! `dev/log/issues/977/pulls/978/ANALYSIS.md`.
//!
//! Each assertion below fails against the pre-fix workflow.

use std::fs;

use super::workflow_fixtures::{ci_surface, job_block, release_workflow, workflow_job_names};

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

fn workflow_files() -> Vec<(String, String)> {
    let dir = format!("{}/.github/workflows", env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<(String, String)> = fs::read_dir(&dir)
        .expect("workflows directory")
        .map(|entry| entry.expect("workflow entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
        })
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let body = fs::read_to_string(&path)
                .expect("workflow body")
                .replace("\r\n", "\n");
            (name, body)
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no workflow files found");
    files
}

/// Parse `timeout-minutes: N` out of a job preamble.
fn job_timeout_minutes(workflow: &str, job_name: &str) -> u32 {
    job_block(workflow, job_name)
        .lines()
        .find_map(|line| line.trim().strip_prefix("timeout-minutes:"))
        .unwrap_or_else(|| panic!("{job_name} must declare timeout-minutes"))
        .trim()
        .parse()
        .expect("numeric timeout-minutes")
}

/// The 10m36s font download in run 31073507682 came from `--with-deps`, which
/// bundles an unbounded `apt-get install` into the same step as the browser
/// download. Splitting them is what makes the apt half boundable and non-fatal.
#[test]
fn playwright_setup_never_uses_the_unbounded_with_deps_shortcut() {
    for (name, body) in workflow_files() {
        // Comments are allowed to name the flag -- they explain why it is gone.
        let executable: String = body
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            !executable.contains("playwright install --with-deps"),
            "{name} still runs `playwright install --with-deps`; split the \
             system-deps install into its own bounded, non-fatal step so a slow \
             Ubuntu mirror cannot consume the whole job budget (issue #977)"
        );
    }
}

#[test]
fn both_playwright_jobs_cache_browsers_and_bound_the_apt_step() {
    let workflow = release_workflow();

    for job_name in ["test-e2e-local", "test-e2e-pages"] {
        let job = job_block(&workflow, job_name);

        assert!(
            job.contains("path: ~/.cache/ms-playwright"),
            "{job_name} must cache the Playwright browser binaries"
        );
        assert!(
            job.contains("npx playwright install chromium"),
            "{job_name} must install the browser without --with-deps"
        );

        let deps_step = job
            .split("- name: Install Playwright system dependencies (best effort)\n")
            .nth(1)
            .unwrap_or_else(|| panic!("{job_name} must have a system-deps step"))
            .split("\n      - ")
            .next()
            .expect("step body");

        assert!(
            deps_step.contains("timeout-minutes:"),
            "{job_name}: the apt step must be bounded so a slow mirror cannot \
             turn a green suite into a cancelled job"
        );
        assert!(
            deps_step.contains("continue-on-error: true"),
            "{job_name}: headless Chromium runs without the font packages apt \
             is fetching, so a mirror stall must not fail the job"
        );
    }
}

/// The core false negative: `globalTimeout` was set to exactly the job cap, so
/// the job clock always won, Playwright never exited non-zero, `if: failure()`
/// never fired, and no report artifact was uploaded.
#[test]
fn playwright_aborts_before_the_job_clock_does() {
    let config = repository_file("tests/e2e/playwright.local.config.js");

    let global_timeout_minutes: u32 = config
        .lines()
        .find_map(|line| line.trim().strip_prefix("globalTimeout:"))
        .expect("globalTimeout must be configured")
        .trim()
        .trim_end_matches(',')
        .split('*')
        .next()
        .expect("globalTimeout must be written as `N * 60_000`")
        .trim()
        .parse()
        .expect("numeric globalTimeout minutes");

    let job_timeout_minutes = job_timeout_minutes(&release_workflow(), "test-e2e-local");

    assert!(
        global_timeout_minutes < job_timeout_minutes,
        "globalTimeout ({global_timeout_minutes}m) must stay below the \
         test-e2e-local cap ({job_timeout_minutes}m) so Playwright aborts first \
         and the job reports `failure` with a report, instead of being killed \
         by timeout-minutes and reported as `cancelled` (issue #977)"
    );

    assert!(
        config.contains("workers: process.env.CI ? 4 : undefined"),
        "the suite is 468 tests; Playwright's CI default of half the cores (2 \
         on a 4-vCPU runner) could not finish within any sane budget"
    );
}

/// Run 31065367736 died at "Compiling fs2 v0.4.3" because the release Docker
/// builds recompiled the crate from scratch -- the PR-check build had used the
/// GHA layer cache all along, the release builds never did.
#[test]
fn every_docker_build_push_step_uses_the_gha_layer_cache() {
    let workflow = release_workflow();

    let build_steps: Vec<&str> = workflow
        .split("docker/build-push-action@")
        .skip(1)
        .collect();

    assert!(
        build_steps.len() >= 4,
        "expected the GHCR and Docker Hub publish steps in both auto-release \
         and manual-release, found {}",
        build_steps.len()
    );

    for (index, step) in build_steps.iter().enumerate() {
        let body = step.split("\n      - ").next().expect("step body");
        assert!(
            body.contains("cache-from: type=gha"),
            "docker build-push step #{index} has no `cache-from: type=gha`; \
             an uncached release build recompiles the whole crate and blew the \
             job budget in run 31065367736 (issue #977)"
        );
        // Issue #1057 narrowed this from "every step exports" to "every step
        // that builds from source exports". The cache is one 10GB pool shared
        // with sccache; it reached 10.01GB with buildkit holding 5.26GB against
        // sccache's 2.44GB, and the compile cache was being evicted to store
        // layers -- the macOS specification lane fell from a 48% to a 27% hit
        // rate and died at its budget with no test run.
        //
        // A step that copies a prebuilt binary compiles nothing worth keeping,
        // and a second publish step exports layers the first already wrote. Both
        // still *read* the cache, which is what keeps the uncached release build
        // this test was written for from coming back.
        // `body` starts after the action name, so the step's own name is not in
        // it; `cache-to: type=inline` is the marker a non-exporting step
        // carries, and it is only ever set deliberately.
        let exports_deliberately_skipped = body.contains("cache-to: type=inline");
        assert!(
            exports_deliberately_skipped || body.contains("cache-to: type=gha,mode=max"),
            "docker build-push step #{index} builds from source but has no \
             `cache-to: type=gha,mode=max`"
        );
    }
}

/// `Create GitHub Release` runs after the Docker publish steps, so the release
/// jobs need budget for build + publish + smoke test + one or two image builds.
#[test]
fn release_jobs_have_budget_to_reach_the_github_release_step() {
    let workflow = release_workflow();

    for job_name in ["auto-release", "manual-release"] {
        let minutes = job_timeout_minutes(&workflow, job_name);
        assert!(
            minutes >= 60,
            "{job_name} has timeout-minutes: {minutes}; at 30 the job was killed \
             mid-Docker-build and never reached `Create GitHub Release`, which \
             is how 0.326.2 .. 0.333.0 shipped without one (issue #977)"
        );
    }
}

/// The structural fix: nothing may time out unobserved.
#[test]
fn pipeline_status_gate_covers_every_other_job() {
    let workflow = release_workflow();
    let job_names = workflow_job_names(&workflow);

    assert!(
        job_names.contains(&"pipeline-status"),
        "release.yml must end in a pipeline-status gate so a job killed by \
         timeout-minutes cannot hide behind a `cancelled` run conclusion"
    );

    let gate = job_block(&workflow, "pipeline-status");

    assert!(
        gate.contains("if: always()"),
        "the gate must run even when an upstream job failed or was cancelled"
    );
    assert!(
        gate.contains("bash scripts/check-pipeline-status.sh"),
        "the gate logic lives in scripts/check-pipeline-status.sh"
    );

    let needs = gate
        .split("needs:")
        .nth(1)
        .expect("pipeline-status must declare needs")
        .split("steps:")
        .next()
        .expect("needs block");

    for job_name in &job_names {
        if *job_name == "pipeline-status" {
            continue;
        }
        assert!(
            needs.contains(job_name),
            "pipeline-status does not `needs: {job_name}`, so a timeout in that \
             job would still produce a silently-cancelled run (issue #977)"
        );
    }
}

/// `main` never gets cancelled by concurrency, so a cancellation there can only
/// be a timeout or a manual cancel -- both must be red. Elsewhere a supersede is
/// legitimate and must stay a warning.
#[test]
fn gate_script_treats_cancellation_on_main_as_a_failure() {
    let script = repository_file("scripts/check-pipeline-status.sh");

    assert!(script.contains("set -euo pipefail"));
    assert!(
        script.contains(r#"IS_MAIN" = "true""#),
        "the script must branch on whether this is the default branch"
    );
    assert!(
        script.contains("::error::Pipeline has cancelled jobs on main"),
        "a cancellation on main must be an error annotation"
    );
    assert!(
        script.contains("::warning::Cancelled jobs:"),
        "a cancellation on another ref is an ordinary supersede: warn only"
    );

    let workflow = release_workflow();
    let gate = job_block(&workflow, "pipeline-status");
    assert!(
        gate.contains("NEEDS_JSON: ${{ toJSON(needs) }}"),
        "the gate must pass the full needs map to the script"
    );
    assert!(
        gate.contains("github.ref == 'refs/heads/main' && github.event_name == 'push'"),
        "IS_MAIN must be computed from the ref and the event"
    );
}

/// Assert on the gate's actual behaviour, not just its text: run it against a
/// success, a timeout-shaped cancellation on and off `main`, and a failure.
#[test]
fn gate_script_behaves_correctly_for_every_conclusion() {
    let root = env!("CARGO_MANIFEST_DIR");

    // jq is preinstalled on GitHub-hosted runners; the script relies on it.
    // Off CI a missing `jq` means "this machine cannot run the case", not "the
    // gate is broken" -- but on CI its absence must be loud, or this test would
    // silently stop checking anything.
    let has_jq = std::process::Command::new("jq")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success());
    if !has_jq {
        assert!(
            std::env::var_os("CI").is_none(),
            "jq is required by scripts/check-pipeline-status.sh and must be \
             present on CI"
        );
        eprintln!(
            "skipping: jq is not installed; install it to run this test locally \
             (it is preinstalled on GitHub-hosted runners)"
        );
        return;
    }

    let all_ok = r#"{"lint":{"result":"success"},"test":{"result":"skipped"}}"#;
    let timed_out = r#"{"lint":{"result":"success"},"auto-release":{"result":"cancelled"}}"#;
    let failed = r#"{"lint":{"result":"failure"}}"#;

    let cases = [
        ("success and skipped", all_ok, "true", true),
        // The case this whole issue is about.
        ("cancelled on main is a timeout", timed_out, "true", false),
        (
            "cancelled elsewhere is a supersede",
            timed_out,
            "false",
            true,
        ),
        ("outright failure", failed, "false", false),
    ];

    for (label, needs_json, is_main, should_pass) in cases {
        let output = std::process::Command::new("bash")
            .arg("scripts/check-pipeline-status.sh")
            .current_dir(root)
            .env("NEEDS_JSON", needs_json)
            .env("IS_MAIN", is_main)
            .env("GITHUB_OUTPUT", "/dev/null")
            .output()
            .expect("run check-pipeline-status.sh");

        assert_eq!(
            output.status.success(),
            should_pass,
            "{label}: expected the gate to {}\nstdout: {}",
            if should_pass { "pass" } else { "fail" },
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

/// The Node 20 runtime deprecation warnings in `ci-logs/annotations.txt`.
#[test]
fn no_workflow_pins_an_action_on_the_deprecated_node_20_runtime() {
    let deprecated = [
        "actions/checkout@v4",
        "actions/setup-node@v4",
        "actions/setup-python@v5",
        "actions/upload-artifact@v4",
        "actions/download-artifact@v4",
        "actions/cache@v3",
    ];

    for (name, body) in workflow_files() {
        for action in deprecated {
            assert!(
                !body.contains(action),
                "{name} still pins {action}, which runs on the deprecated \
                 Node 20 runtime and emits a warning annotation on every run"
            );
        }
    }
}

/// The duplicated inline blocks that both grew release.yml past its ceiling and
/// let the two release paths drift apart.
#[test]
fn shared_release_logic_lives_in_scripts_not_duplicated_inline() {
    let workflow = release_workflow();
    // Some of these scripts are release-job steps and some became registered
    // gates in issue #991; both are CI calling the script rather than inlining
    // it, which is what this case is about.
    let surface = ci_surface();

    for script in [
        "scripts/check-pipeline-status.sh",
        "scripts/configure-dockerhub-publishing.sh",
        "scripts/invoke-create-github-release.sh",
        "scripts/lint-shell-scripts.sh",
        "scripts/run-desktop-lib-tests.sh",
        "scripts/build-rust-api-docs.sh",
    ] {
        assert!(
            !repository_file(script).is_empty(),
            "{script} must exist and be non-empty"
        );
        assert!(
            surface.contains(script),
            "CI must call {script} -- from a job step or a registered gate -- \
             rather than inlining it"
        );
    }

    // `configure-dockerhub-publishing.sh` and `invoke-create-github-release.sh`
    // replaced byte-identical inline copies in auto-release and manual-release;
    // both release paths must keep using the shared version.
    for script in [
        "scripts/configure-dockerhub-publishing.sh",
        "scripts/invoke-create-github-release.sh",
    ] {
        assert_eq!(
            workflow.matches(script).count(),
            2,
            "{script} must be called from both auto-release and manual-release"
        );
    }

    // `scripts/check-file-size.rs` hard-caps workflow files at 2000 lines.
    let lines = workflow.lines().count();
    assert!(
        lines <= 2_000,
        "release.yml is {lines} lines, over the 2000-line ceiling enforced by \
         scripts/check-file-size.rs"
    );
}
