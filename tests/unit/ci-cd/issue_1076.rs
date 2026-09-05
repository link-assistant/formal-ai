//! Regression coverage for issue #1076: the CI/CD false positives, false
//! negatives, warnings and errors found by auditing every run at `main` head
//! `701d6a45`.
//!
//! The reported failure was `Coverage / Code Coverage` reporting **cancelled**
//! rather than **failure** after burning 2,377s of a 40-minute cap. Splitting
//! that run's `Generate code coverage` step by test binary (the measurements
//! are in `dev/log/issues/1076/pulls/1077/analysis/`) shows the cause is not
//! the code and not the cargo cache:
//!
//! * the `integration` target ran the **same 358 tests** in 1572.8s that the
//!   previous day's run finished in 213.6s -- a 7.4x spread on identical work;
//! * the slowdown is global (all 18 heavy modules, 2.3x-21.6x) and progressive
//!   (2.2x in the first decile of the run, 14.5x in the last);
//! * it hits `issue_749_shell_routing`, which is pure in-process CPU work with
//!   no `Command::new`, no `spawn`, no `sleep` and no network, on a runner
//!   image identical to the successful run's;
//! * compilation took 3m13s and two runs that *also* missed the cargo cache
//!   finished successfully in 25.3 and 33.7 minutes.
//!
//! So the runner degraded, not the repository -- and the repository had no
//! defence: the step declared no budget, so `timeout-minutes` was the deadline
//! rather than a backstop, and no job records the CPU, memory or disk
//! telemetry that would let the next occurrence be attributed.
//!
//! These tests pin the three rules that follow from that, plus the cache and
//! audit defects the same sweep found. The full reconstruction is in
//! `dev/log/issues/1076/pulls/1077/README.md`.

use std::fs;

use super::issue_1017::{job_timeout, workflow_files};
use super::workflow_fixtures::{job_block, workflow_job_names};

/// Mirrors `issue_1017::MAX_BUDGET_SHARE_PERCENT`. A declared budget may claim
/// at most this share of its job's cap; the remainder pays for checkout,
/// toolchain, cache restore, artifact transfer and the SIGTERM grace.
const MAX_BUDGET_SHARE_PERCENT: u64 = 70;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// Steps whose runtime is decided by something the repository does not
/// control -- a shared runner's spare CPU, a registry, a remote builder. Issue
/// #1017 established the rule for network installs; run 33955786082 extends it
/// to the class that actually failed: a long compute step on a borrowed host.
///
/// Each entry is `(marker, human name)`, where `marker` is text that appears in
/// the step that runs it.
const UNBOUNDED_LONG_STEPS: &[(&str, &str)] = &[
    ("cargo llvm-cov --all-features", "instrumented coverage run"),
    ("docker/build-push-action", "container build and push"),
];

/// The rule the cancelled run establishes. A step that can run for tens of
/// minutes on a host whose speed the repository does not control must own a
/// deadline that expires before the job clock, so an overrun is a `failure`
/// with an annotation naming it -- never a `cancelled` that reads like a
/// superseded run (issue #977, issue #1017).
///
/// `run:` steps own it through `scripts/run-with-budget-warning.sh`. Steps that
/// are a third-party action cannot be wrapped, so they own a step-level
/// `timeout-minutes:`, which GitHub reports as a step **failure**.
#[test]
fn every_long_running_step_under_a_job_cap_owns_a_deadline() {
    let mut checked = 0_usize;

    for (name, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = job_block(&body, job_name);
            if job_timeout(job).is_none() {
                continue; // no cap: a different contract governs it
            }

            for (marker, description) in UNBOUNDED_LONG_STEPS {
                if !job.contains(marker) {
                    continue;
                }
                checked += 1;

                let wrapped = job.contains("run-with-budget-warning.sh");
                let step_capped = job
                    .lines()
                    .any(|line| line.trim().starts_with("timeout-minutes:"))
                    && job.matches("timeout-minutes:").count() > 1;

                assert!(
                    wrapped || step_capped,
                    "{name}: job `{job_name}` runs a {description} \
                     (`{marker}`) under a job cap but gives it no deadline of \
                     its own. When the runner is slow -- run 33955786082 was \
                     7.4x slower on identical tests -- the step consumes the \
                     whole cap and GitHub reports the kill as `cancelled`, not \
                     `failure`, so nothing turns red. Wrap a `run:` step in \
                     scripts/run-with-budget-warning.sh, or give an action step \
                     its own `timeout-minutes:`."
                );
            }
        }
    }

    assert!(
        checked >= 2,
        "expected to reach at least the coverage run and the container \
         publish, reached {checked}"
    );
}

/// Issue #1017 pinned "declared budget <= 70% of cap" for the budgets that
/// existed. The coverage step had none, so it was invisible to that gate; this
/// re-asserts the share for the budget added here, and additionally requires
/// the budget to leave room for the *measured* worst case. The last twenty-one
/// `Coverage` runs took 14.4 to 33.8 minutes when they succeeded
/// (`analysis/coverage-job-durations.tsv`), so a budget below that worst case
/// would turn a merely slow runner into a red build -- the false *positive*
/// half of what issue #1076 asks for.
#[test]
fn the_coverage_budget_covers_the_measured_worst_case_and_fits_its_cap() {
    /// Worst case across the twenty-one measured `Coverage` runs that
    /// succeeded, in seconds: 33.8 minutes on 2026-08-23.
    const MEASURED_WORST_CASE_SECONDS: u64 = 33 * 60 + 48;

    let body = repository_file(".github/workflows/coverage.yml");
    let job = job_block(&body, "coverage");

    let budget_seconds: u64 = job
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("TEST_BUDGET_SECONDS:")
                .and_then(|value| value.trim().parse::<u64>().ok())
        })
        .expect(
            "coverage.yml: the `coverage` job must declare TEST_BUDGET_SECONDS \
             for the instrumented run; without it `timeout-minutes` is the \
             deadline and an overrun reports `cancelled` (issue #977)",
        );

    let cap_minutes: u64 = job_timeout(job)
        .expect("coverage.yml: the `coverage` job must declare a cap")
        .parse()
        .expect("coverage.yml: write the `coverage` cap as a plain number");
    let cap_seconds = cap_minutes * 60;

    let share = budget_seconds * 100 / cap_seconds;
    assert!(
        share <= MAX_BUDGET_SHARE_PERCENT,
        "coverage.yml: a {budget_seconds}s budget under a {cap_minutes}m cap is \
         {share}% of it; checkout, the toolchain, cargo-llvm-cov's install, the \
         ratchet and four artifact uploads have to fit in the remainder, or the \
         job clock expires first and the overrun is a `cancelled` again. Keep \
         the budget at or below {MAX_BUDGET_SHARE_PERCENT}%."
    );

    assert!(
        budget_seconds > MEASURED_WORST_CASE_SECONDS,
        "coverage.yml: a {budget_seconds}s budget is at or below the \
         {MEASURED_WORST_CASE_SECONDS}s worst case measured across the last \
         twenty-one green `Coverage` runs, so an ordinarily slow runner would \
         fail the build for a reason that has nothing to do with the code under \
         test -- the false positive issue #1076 also asks to remove."
    );
}

/// The evidence gap the cancelled run exposed. Grepping all five collected
/// coverage logs for `no space left`, `Cannot allocate`, `out of memory` and
/// `oom-kill` returns nothing, and no job records `nproc`, load average,
/// `/proc/stat` steal or `df` -- so a 7.4x slowdown on identical tests cannot
/// be attributed to CPU steal, memory pressure or disk exhaustion.
///
/// Issue #1076 asks for the debug output that closes that gap, defaulting off.
/// `FORMAL_AI_CI_VERBOSE` is the switch the repository already uses for exactly
/// this (`.github/actions/setup-sccache`, `scripts/run-with-budget-warning.sh`).
#[test]
fn runner_telemetry_exists_and_defaults_to_off() {
    let script = repository_file("scripts/report-runner-capacity.sh");

    for probe in [
        "nproc",
        "/proc/loadavg",
        "/proc/stat",
        "MemAvailable",
        "df -h",
    ] {
        assert!(
            script.contains(probe),
            "scripts/report-runner-capacity.sh must sample `{probe}`: without \
             it a run that is 7.4x slower on identical tests cannot be \
             attributed to the host"
        );
    }

    let body = repository_file(".github/workflows/coverage.yml");
    let job = job_block(&body, "coverage");
    assert!(
        job.contains("report-runner-capacity.sh"),
        "coverage.yml: the `coverage` job is the one that failed for a reason \
         no log can explain; it must sample runner capacity"
    );
    assert!(
        job.contains("FORMAL_AI_CI_VERBOSE"),
        "coverage.yml: runner telemetry must be gated on FORMAL_AI_CI_VERBOSE, \
         the switch this repository already uses for opt-in CI diagnostics"
    );

    // Default off: nothing may set the switch to `true` in a checked-in
    // workflow, or the diagnostics stop being opt-in.
    for (name, workflow) in workflow_files() {
        for line in workflow.lines() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("FORMAL_AI_CI_VERBOSE:") {
                let value = value.trim().trim_matches(['\'', '"']);
                assert!(
                    value != "true",
                    "{name}: FORMAL_AI_CI_VERBOSE is pinned to `true`; issue \
                     #1076 requires the verbose mode to default to off so it \
                     is switched on deliberately, not permanently"
                );
            }
        }
    }
}

/// D9. Issue #1055 introduced `./.github/actions/cache-cargo-registry` so every
/// job restores the same registry through the same key ladder, and converted
/// three call sites. Eight inline copies stayed behind, with six distinct key
/// prefixes -- so the same registry is stored six times over against a 10 GB
/// repository-wide quota that `analysis/cache-usage-fresh.json` measured at
/// 11.44 GB, i.e. already in eviction. The cancelled run's cache miss was one
/// consequence: its restore keys `Linux-cargo-coverage-<hash>` and
/// `Linux-cargo-coverage-` both missed, because the composite action's generic
/// `Linux-cargo-` fallback was not there to catch it.
#[test]
fn every_cargo_registry_cache_goes_through_the_shared_action() {
    for (name, body) in workflow_files() {
        for (index, block) in body.split("uses: actions/cache@").enumerate() {
            if index == 0 {
                continue; // text before the first cache step
            }
            // The step body ends at the next step boundary.
            let step: String = block
                .split("\n      - ")
                .next()
                .unwrap_or(block)
                .to_string();
            if !step.contains("~/.cargo/registry") {
                continue;
            }
            panic!(
                "{name}: an inline `actions/cache` step caches \
                 `~/.cargo/registry`. Use `./.github/actions/cache-cargo-registry` \
                 instead (issue #1055): it is the only spelling that carries the \
                 generic `${{{{ runner.os }}}}-cargo-` fallback, whose absence is \
                 why run 33955786082 missed the cache outright, and it stops the \
                 same registry being stored under six different prefixes against \
                 one 10 GB quota."
            );
        }
    }

    // Reaching here means every registry cache is consolidated; assert the
    // shared action is actually in use so the sweep cannot pass vacuously.
    let users = workflow_files()
        .into_iter()
        .filter(|(_, body)| body.contains("./.github/actions/cache-cargo-registry"))
        .count();
    assert!(
        users >= 3,
        "expected the shared cargo-registry action to be used by at least the \
         three workflows that already adopted it, found {users}"
    );
}

/// D2. `cache-to: type=gha` writes build layers into the same 10 GB repository
/// quota the cargo and sccache caches share. Measured at 2026-09-05T09:54Z: 48
/// `buildkit-blob-*` entries -- 0.9% of the entries -- held 4.91 GB, **42.9%**
/// of the quota, against 5,439 `sccache/*` entries holding 4.43 GB and just six
/// surviving `*-cargo-*` entries.
///
/// The first draft of this test demanded `mode=min`, and that was wrong.
/// `mode=min` exports only the *final* stage's layers; this image is
/// multi-stage, so the final stage is a thin runtime layer and the expensive
/// compile layers would stop being cached at all -- reintroducing exactly the
/// uncached release build that issue #977 was filed for (run 31065367736), and
/// contradicting `issue_977::every_docker_build_push_step_uses_the_gha_layer_cache`
/// and `issue_1057::a_from_source_publish_still_exports_layers`. `mode=max` is
/// correct and stays.
///
/// The two levers that actually bound the footprint, and that this test pins:
///
/// 1. An explicit `scope=`. Without it buildx uses the default scope
///    `buildkit`, and the upstream documentation is explicit about the
///    consequence: "each build will overwrite the cache of the previous,
///    leaving only the final cache." One scope per image, not one per job, so
///    two jobs building the same image still share.
/// 2. A bounded number of *writers*. Issue #1057 established that only a
///    from-source publish exports; a step that copies a prebuilt binary, or a
///    second registry push of layers a sibling step just wrote, reads and sets
///    `cache-to: type=inline`. Two exporters is the whole budget -- the GHCR
///    publish in `auto-release` and the one in `manual-release` -- so a third
///    appearing is a regression this catches.
#[test]
fn container_build_caches_are_bounded_and_scoped() {
    let mut checked = 0_usize;

    for (name, body) in workflow_files() {
        for line in body.lines() {
            let trimmed = line.trim();
            let Some(spec) = trimmed.strip_prefix("cache-to:") else {
                continue;
            };
            let spec = spec.trim();
            if !spec.contains("type=gha") {
                continue;
            }
            checked += 1;

            assert!(
                spec.contains("scope="),
                "{name}: `{spec}` writes into buildx's default cache scope \
                 `buildkit`, where -- in Docker's own words -- \"each build \
                 will overwrite the cache of the previous, leaving only the \
                 final cache\". Give it an explicit `scope=`."
            );
        }
    }

    assert!(
        checked >= 1,
        "expected at least one `cache-to: type=gha` site, found {checked}"
    );
    assert!(
        checked <= 2,
        "{checked} steps export layers to the shared 10 GB quota. Issue #1057 \
         allows exactly two writers, the from-source GHCR publish in \
         `auto-release` and the one in `manual-release`; every other publish \
         step reads the cache and sets `cache-to: type=inline`, so the pool \
         holds one copy of the layers instead of one per registry."
    );
}

/// D10. All four `link-foundation` pipeline templates -- rust, js, python and
/// php -- ship `.github/zizmor.yml` and a `workflows.yml` that runs both
/// `actionlint` and `zizmor`. This repository ran `actionlint` only, and ran it
/// as a bare binary inside the pipeline's `lint` job, so two distinct gaps
/// existed at once.
///
/// The security gap was not theoretical. The first zizmor run against these
/// workflows returned four high-severity `template-injection` findings, all in
/// `release.yml`: `github.event.inputs.bump_type` and
/// `github.event.inputs.description` interpolated directly into the `run:` line
/// of two steps holding `secrets.GITHUB_TOKEN`. Both steps already bound those
/// values in `env:` and used the raw expression anyway.
///
/// The false-negative gap was worse, because it made a green check meaningless:
/// actionlint does not lint `run:` blocks itself, it shells out to
/// `ShellCheck`, and when `ShellCheck` is not on PATH it skips every shell
/// check and still exits 0. Measured on this repository -- the binary exits 0
/// without `ShellCheck` installed, and reports SC1073 with it. The Docker image
/// bundles `ShellCheck` and pyflakes, which is why the hive-mind
/// best-practices document (principle 14) requires that form.
#[test]
fn workflows_are_audited_for_security_not_only_syntax() {
    let config = repository_file(".github/zizmor.yml");
    assert!(
        config.contains("rules:"),
        ".github/zizmor.yml must declare a `rules:` section, matching the four \
         link-foundation templates"
    );

    let mut runners = workflow_files()
        .into_iter()
        .filter(|(_, body)| body.contains("zizmor"))
        .map(|(name, _)| name)
        .peekable();
    assert!(
        runners.peek().is_some(),
        "no workflow runs zizmor. actionlint checks syntax; zizmor checks \
         security -- unpinned actions, template injection into `run:`, \
         over-broad token permissions. All four link-foundation templates run \
         both (issue #1076)."
    );

    let audit = workflow_files()
        .into_iter()
        .find(|(name, _)| name == "workflows.yml")
        .map(|(_, body)| body)
        .expect(".github/workflows/workflows.yml must exist (issue #1076)");

    // Principle 14: a confidence floor, not a severity floor. `min-severity`
    // would drop findings by how bad they are, which is how a real high-severity
    // finding gets hidden behind a threshold nobody revisits. `min-confidence`
    // drops them by how sure the tool is, which is a statement about noise.
    assert!(
        audit.contains("min-confidence: medium"),
        "the zizmor job must set a confidence floor (issue #1076)"
    );
    assert!(
        !audit.contains("min-severity"),
        "the zizmor job must not filter by severity -- that hides real findings \
         rather than noisy ones (hive-mind CI/CD best practices, principle 14)"
    );

    // Principle 14: annotations, not SARIF. A SARIF upload needs code scanning
    // enabled and fails quietly on forks; an annotation is a red check anywhere.
    assert!(
        audit.contains("advanced-security: false") && audit.contains("annotations: true"),
        "zizmor findings must surface as annotations, not a SARIF upload that \
         fails silently where code scanning is off (issue #1076)"
    );

    // The audit must name what it audits. `zizmor-action`'s `inputs:` defaults
    // to `.`, and at that scope it walks `docs/case-studies/`, where this
    // repository archives other projects' workflows as evidence. Measured on
    // this tree: default scope exits 14 with 1739 findings, 140 of them high,
    // every displayed one in an archived copy of a workflow no one here can
    // change; scoped to the live pipeline the same command exits 0. A red check
    // nobody can act on is a false positive, and issue #1076 is about removing
    // those -- not adding one.
    assert!(
        audit.contains("inputs: .github/workflows .github/actions"),
        "the zizmor job must state its scope explicitly. `inputs:` defaults to \
         `.`, which audits the archived workflows under docs/case-studies/ and \
         fails the job on findings that belong to other repositories \
         (issue #1076)"
    );

    // Principle 14: actionlint runs as the image, because the bare binary is a
    // silent false negative whenever ShellCheck is missing from PATH.
    assert!(
        audit.contains("docker://rhysd/actionlint:"),
        "actionlint must run as the Docker image, which bundles ShellCheck; the \
         bare binary skips every `run:` block check and exits 0 when ShellCheck \
         is absent (issue #1076)"
    );

    // Principle 14: "A blanket `ignore` is indistinguishable from no gate at
    // all." Every suppression in either config names what it suppresses.
    for (file, body) in [
        (".github/zizmor.yml", &config),
        (
            ".github/actionlint.yaml",
            &repository_file(".github/actionlint.yaml"),
        ),
    ] {
        for blanket in ["- '*'\n", "- '*.yml'", "- \"*\"", "- '.*'"] {
            assert!(
                !body.contains(blanket),
                "{file} suppresses everything with {blanket:?}; scope it to the \
                 files and rules it is actually about (issue #1076)"
            );
        }
    }
}

/// A diagnostic must never be able to fail the job it is diagnosing.
///
/// The verbose telemetry added for D11 starts a background sampler and stops it
/// in an `if: always()` step. The first draft of that step read
/// `kill "${CAPACITY_SAMPLER_PID:-0}"`, and `kill 0` is not a no-op: POSIX
/// defines PID `0` as *every process in the sender's own process group*, so
/// whenever the sampler step was skipped -- which is exactly what happens when
/// an earlier step failed -- the cleanup step would have signalled its own
/// shell. That converts an unrelated failure into a second, confusing one, and
/// it is a false positive of precisely the kind issue #1076 is about.
///
/// The rule this pins is deliberately wider than the one site: no `run:` block
/// in any workflow may pass an unguarded default to `kill`.
#[test]
fn no_cleanup_step_can_signal_its_own_process_group() {
    for (name, body) in workflow_files() {
        for (lineno, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.contains("kill ") || trimmed.starts_with('#') {
                continue;
            }
            // `kill 0`, `kill "0"`, `kill "${VAR:-0}"` and `kill -TERM 0` all
            // signal the caller's own process group.
            let signals_own_group = trimmed.contains(":-0}")
                || trimmed.contains("kill 0")
                || trimmed.contains("kill \"0\"")
                || trimmed.contains("kill -- 0");
            assert!(
                !signals_own_group,
                "{name}:{}: `{trimmed}` signals the sender's own process group \
                 when the variable is unset (`kill 0` means \"my process \
                 group\"). Guard it with `[ -n \"${{VAR:-}}\" ]` instead, so a \
                 diagnostic that never started cannot fail the job (issue #1076)",
                lineno + 1
            );
        }
    }
}

/// D14. A job name is data the pipeline is read by, and YAML eats it.
///
/// `.github/workflows/task-ladder.yml` declared
/// `name: Task Ladder (issue #840 dataset)` without quotes. In YAML an
/// unquoted `#` preceded by a space opens a comment, so the value the parser
/// keeps is `Task Ladder (issue` -- and that truncated string is what GitHub
/// stores, what the checks list shows, and what the Actions API returns. It
/// was found by the D5 headroom audit, which could not match the measured name
/// to any job the workflows declare, and it had been that way since the
/// workflow was written.
///
/// Four names were affected: the job names in `task-ladder.yml`,
/// `write-effect-ladder.yml` and `summarization-ratchet.yml`, and a step name
/// in `release.yml`. The evidence is in the collected measurements:
/// `dev/log/issues/1076/pulls/1077/analysis/job-durations-main.tsv` holds rows
/// reading `Write-Effect Ladder (issue` and
/// `Summarization quality ratchet (issue`.
///
/// Neither actionlint nor zizmor reports this: the file is valid YAML and the
/// workflow runs. Only the name is wrong, on every run, forever.
#[test]
fn no_declared_name_is_truncated_by_an_unquoted_comment() {
    for (name, body) in workflow_files() {
        for (lineno, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            let Some(value) = trimmed
                .strip_prefix("- name:")
                .or_else(|| trimmed.strip_prefix("name:"))
            else {
                continue;
            };
            let value = value.trim();
            // A quoted scalar carries `#` safely; an expression never contains
            // one at the top level.
            if value.starts_with('\'') || value.starts_with('"') || value.is_empty() {
                continue;
            }
            assert!(
                !value.contains(" #"),
                "{name}:{}: `{value}` is an unquoted YAML scalar, so everything \
                 from ` #` onward is a comment and the name GitHub records is \
                 `{}`. Wrap it in single quotes (issue #1076)",
                lineno + 1,
                value.split(" #").next().unwrap_or(value)
            );
        }
    }
}

/// D5. A job cap is a constant; the duration it bounds is only ever observed.
///
/// Issue #1017 gave long *steps* a deadline that expires before the job clock,
/// so an overrun reports `failure` instead of the `cancelled` that
/// `timeout-minutes` produces (issue #977). Nothing did the same for whole
/// jobs, and nothing measured whether the caps still fit: sampling 142 `main`
/// runs for this issue found `Coverage / Code Coverage` at **100.7%** of its
/// 40-minute cap -- already past it, silently -- and `Lint and Format Check`
/// at 84.4% and rising. The measurements are in
/// `dev/log/issues/1076/pulls/1077/analysis/job-durations-main.tsv`.
///
/// Raising those two caps fixes today's numbers and nothing else; the next
/// drift would be just as invisible. So the ratio is re-derived from the
/// Actions API on a schedule, and this test pins the parts of that arrangement
/// a commit can break.
#[test]
fn job_caps_are_audited_against_what_the_jobs_really_cost() {
    let audit = repository_file(".github/workflows/job-headroom.yml");

    // Scheduled, not per-pull-request: headroom is a property of a trend, and a
    // pull request has no measurements of its own to be judged against.
    assert!(
        audit.contains("schedule:"),
        "the audit must run on a schedule"
    );
    assert!(
        audit.contains("workflow_dispatch:"),
        "and be runnable on demand"
    );
    assert!(
        !audit.contains("pull_request:"),
        "a per-pull-request run would judge a commit on other commits' measurements"
    );
    // Reading run and job records is all it does. The scan is over the
    // declarations only: the file's own prose says "Nothing here writes", and a
    // raw substring search over the whole text would have read that as a
    // granted permission -- the exact shape of false positive this pull request
    // exists to remove.
    assert!(audit.contains("actions: read"));
    let declarations: String = audit
        .lines()
        .map(|line| line.split_once(" #").map_or(line, |(code, _)| code))
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !declarations.contains("write"),
        "the audit only reads; nothing in it needs a write permission"
    );
    assert!(audit.contains("scripts/collect-job-durations.sh"));
    assert!(audit.contains("rust-script scripts/check-job-headroom.rs --durations"));
    // The verdict has to be checkable: publish the rows it was computed from.
    assert!(audit.contains("actions/upload-artifact@v7"));

    // The two caps the audit was built to explain, each raised to put its
    // measured worst case back inside the share issue #1017 enforces.
    let release = repository_file(".github/workflows/release.yml");
    assert_eq!(
        job_timeout(job_block(&release, "lint")),
        Some("25"),
        "12.7 minutes measured against a 15-minute cap is 84.4%; the cap had \
         become the deadline (issue #1076)"
    );
    assert_eq!(
        job_timeout(job_block(&release, "build")),
        Some("20"),
        "11.6 minutes measured against a 15-minute cap is 77.0% (issue #1076)"
    );

    // A dispatch input reaching a `run:` block through `${{ }}` is the
    // injection shape zizmor reports; this one goes through `env:`.
    assert!(
        audit.contains("SAMPLE_BRANCH: ${{ inputs.branch }}")
            && audit.contains("\"${SAMPLE_BRANCH}\""),
        "workflow_dispatch inputs must reach the shell as environment variables"
    );

    // The half of the audit a commit *can* break -- reading the workflows -- is
    // a registered gate, so a renamed job or an unreadable cap fails on the
    // pull request rather than dropping out of the weekly run unnoticed.
    let gate = repository_file("data/meta/ci-gates/check-job-headroom.lino");
    assert!(gate.contains("rust-script --test scripts/check-job-headroom.rs"));
    assert!(gate.contains("stage rust"));
}

/// `TEST_BUDGET_SECONDS` is not the only way this repository budgets a step: 30
/// steps across the workflows carry a step-level `timeout-minutes:`, which is
/// the *other* mechanism that turns an overrun into a `failure` instead of a
/// `cancelled` job. `issue_1017::every_step_budget_expires_before_the_job_clock_it_guards`
/// sweeps only the first form, so the second was unaudited -- and this pull
/// request walked straight into the gap. Its first draft budgeted the GHCR
/// publish step at 25 minutes when the only two measured builds took 25.5 and
/// 32.5, which would have failed both: a false positive introduced by the fix
/// for a false negative. The number was corrected from the measurements; the
/// invariant that outlives the number is pinned here.
#[test]
fn every_step_level_timeout_can_fire_before_its_job_cap() {
    let mut checked = 0;

    for (name, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = job_block(&body, job_name);
            // Step keys sit two levels deeper than job keys, so the indent
            // distinguishes a step's own cap from the job's without a YAML
            // parser -- and `job_timeout` reads the first (job-level) one.
            let step_budgets = job.lines().filter_map(|line| {
                line.strip_prefix("        timeout-minutes:")
                    .and_then(|value| value.trim().parse::<u64>().ok())
            });

            for budget_minutes in step_budgets {
                let cap_minutes: u64 = job_timeout(job)
                    .unwrap_or_else(|| {
                        panic!("{name}: job `{job_name}` caps a step but declares no cap itself")
                    })
                    .parse()
                    .unwrap_or_else(|_| {
                        panic!(
                            "{name}: job `{job_name}` caps a step under a job cap this test \
                             cannot compare against; write the cap as a plain number of minutes"
                        )
                    });
                checked += 1;
                let share = budget_minutes * 100 / cap_minutes;
                assert!(
                    share <= MAX_BUDGET_SHARE_PERCENT,
                    "{name}: job `{job_name}` gives a step a {budget_minutes}m cap under a \
                     {cap_minutes}m job cap ({share}% of it). The job clock would expire \
                     first, so the overrun is reported as `cancelled` rather than as the \
                     step failure the step cap exists to produce (issues #977, #1017, \
                     #1076). Keep it at or below {MAX_BUDGET_SHARE_PERCENT}% of the cap."
                );
            }
        }
    }

    assert!(
        checked >= 20,
        "expected to sweep every step-level cap in the workflows, swept {checked}"
    );
}

/// The two Docker publish budgets, pinned to the measurements that chose them
/// rather than to a round number. `Publish Docker image to GHCR` is the step
/// that took 24.6 minutes of a 60-minute job while nothing turned red (D2b),
/// and it appears twice -- `auto-release` and `manual-release` publish the same
/// image -- so a fix applied to one of them is not a fix.
#[test]
fn both_release_jobs_budget_the_docker_publish_above_its_measured_worst_case() {
    let release = repository_file(".github/workflows/release.yml");

    for job_name in ["auto-release", "manual-release"] {
        let job = job_block(&release, job_name);
        assert_eq!(
            job_timeout(job),
            Some("90"),
            "{job_name}: 50.6 minutes measured over 400 `main` runs is 84.4% of a \
             60-minute cap, and a 45-minute step budget cannot fire underneath one"
        );

        let step_caps: Vec<u64> = job
            .lines()
            .filter_map(|line| {
                line.strip_prefix("        timeout-minutes:")
                    .and_then(|value| value.trim().parse::<u64>().ok())
            })
            .collect();
        assert_eq!(
            step_caps,
            vec![45, 20],
            "{job_name} must budget the GHCR publish at 45 minutes -- 1.4x the worst \
             measured build (32.5 min, run 33955786226) -- and the Docker Hub publish \
             that reuses its layers at 20"
        );
    }
}
