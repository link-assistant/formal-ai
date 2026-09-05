//! Regression coverage for issue #1017: the second generation of timeouts that
//! hid as cancellations, plus the audit of every remaining CI diagnostic.
//!
//! Issue #977 established the rule: a job killed by `timeout-minutes` is
//! reported by GitHub as **cancelled**, not **failed**. Run 31937348472 hit the
//! same wall from the other side. `macOS Core Tests / Run macOS core slice
//! 10/12` had a 480s step budget under a 600s job cap, but 133s of that job was
//! spent on checkout, toolchain and artifact download *outside* the budgeted
//! step, so the job clock always won: the slice was killed at 600s with 467s of
//! testing done, reported `cancelled`, and the whole pipeline inherited that
//! conclusion -- no release was published for the merge of pull request #1016.
//!
//! The invariant these tests pin: **`timeout-minutes` is a backstop, never the
//! deadline.** The step budget plus the job's unbudgeted setup plus the SIGTERM
//! grace must expire first, so an overrun is a `failure` with an `::error`
//! annotation that names what ran long. The reconstruction is in
//! `dev/log/issues/1017/pulls/1018/README.md`.

use std::fs;
use std::process::Command;
use std::time::Instant;

use super::issue_796::{run_classifier, sandbox};
use super::workflow_fixtures::{job_block, workflow_job_names};

/// The share of a job's cap a single step budget may claim. The remainder pays
/// for checkout, toolchain install, cache restore, artifact transfer and the
/// wrapper's SIGTERM grace -- 133s of the 600s cap on the slice that failed.
const MAX_BUDGET_SHARE_PERCENT: u64 = 70;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

pub(crate) fn workflow_files() -> Vec<(String, String)> {
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
            (
                name,
                fs::read_to_string(&path).unwrap().replace("\r\n", "\n"),
            )
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no workflow files found");
    files
}

/// `timeout-minutes:` as written, which may be a `${{ ... }}` expression when a
/// matrix leg needs a different cap than its siblings.
pub(crate) fn job_timeout(job: &str) -> Option<&str> {
    job.lines()
        .find_map(|line| line.trim().strip_prefix("timeout-minutes:"))
        .map(str::trim)
}

fn run_budget_wrapper(env: &[(&str, &str)], arguments: &[&str]) -> (std::process::Output, u64) {
    let mut command = Command::new("bash");
    command
        .arg(format!(
            "{}/scripts/run-with-budget-warning.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .args(arguments);
    for (key, value) in env {
        command.env(key, value);
    }

    let started = Instant::now();
    let output = command.output().expect("run the budget wrapper");
    (output, started.elapsed().as_secs())
}

/// The core invariant. Every budgeted step must be able to blow its budget --
/// and be reported for it -- while the job clock still has room to spare.
#[test]
fn every_step_budget_expires_before_the_job_clock_it_guards() {
    let mut checked = 0;

    for (name, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = job_block(&body, job_name);

            for budget_seconds in job.lines().filter_map(|line| {
                line.trim()
                    .strip_prefix("TEST_BUDGET_SECONDS:")
                    .and_then(|value| value.trim().parse::<u64>().ok())
            }) {
                let cap_minutes: u64 = job_timeout(job)
                    .unwrap_or_else(|| {
                        panic!("{name}: job `{job_name}` budgets a step but declares no cap")
                    })
                    .parse()
                    .unwrap_or_else(|_| {
                        panic!(
                            "{name}: job `{job_name}` budgets a step under a cap this test \
                             cannot compare against; write the cap as a plain number of minutes"
                        )
                    });
                checked += 1;
                let cap_seconds = cap_minutes * 60;
                let share = budget_seconds * 100 / cap_seconds;
                assert!(
                    share <= MAX_BUDGET_SHARE_PERCENT,
                    "{name}: job `{job_name}` gives a step a {budget_seconds}s \
                     budget under a {cap_minutes}m cap ({share}% of it). Setup \
                     outside the budgeted step -- checkout, toolchain, cache, \
                     artifacts -- has to fit in the remainder, or the job clock \
                     expires first and the overrun is reported as `cancelled` \
                     instead of `failure` (issue #977, issue #1017). Keep the \
                     budget at or below {MAX_BUDGET_SHARE_PERCENT}% of the cap."
                );
            }
        }
    }

    assert!(
        checked >= 4,
        "expected to check at least the macOS archive build, the macOS slice \
         and both release test suites, checked {checked}"
    );
}

/// The budget sweep above can only judge a job that *declares* a budget, so a
/// job with none is invisible to it -- 44 of the 47 capped jobs, when this was
/// measured. `E2E (opencode-vscode)` was one of them: in run 32050028114 it sat
/// in an unbudgeted `apt-get update` until the 25-minute job cap killed it, and
/// GitHub reported that kill as `cancelled`, so a hung Ubuntu mirror surfaced as
/// a benign-looking cancellation instead of a failure. That is issue #1017's own
/// false negative, in a workflow the original sweep never reached.
///
/// Budgeting all 44 would be churn. The rule pinned here is narrower and is the
/// one the incident actually establishes: a step whose runtime is decided by a
/// *remote* host -- a package mirror, a registry -- must own a deadline, because
/// it is the class that hangs indefinitely and converts into a cancellation.
#[test]
fn network_installs_under_a_job_cap_own_a_deadline() {
    // Package-manager fetches whose duration depends on a remote mirror. The
    // wrapper is listed beside the raw commands because issue #1021 moved the
    // matrix's `apt-get` behind it: the rule follows the fetch, not its
    // spelling, or hiding a mirror call inside a script would exempt it.
    const UNBOUNDED_NETWORK_COMMANDS: &[&str] = &[
        "apt-get update",
        "apt-get install",
        "scripts/apt-install-with-retry.sh",
    ];

    let mut checked = 0_usize;
    for (name, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = job_block(&body, job_name);
            if job_timeout(job).is_none() {
                continue; // no cap: a different contract governs it
            }
            for command in UNBOUNDED_NETWORK_COMMANDS {
                if !job.contains(command) {
                    continue;
                }
                checked += 1;
                assert!(
                    job.contains("run-with-budget-warning.sh"),
                    "{name}: job `{job_name}` runs `{command}` under a job cap \
                     but budgets nothing. A hung mirror then burns the whole cap \
                     and GitHub reports the kill as `cancelled`, not `failure` \
                     -- the issue #977 false negative, one level down. Wrap it in \
                     scripts/run-with-budget-warning.sh so the deadline is owned \
                     by the step and an overrun says so (issue #1017)."
                );
            }
        }
    }

    assert!(
        checked >= 1,
        "expected at least the agentic matrix's Xvfb install, checked {checked}"
    );
}

/// A job with no cap inherits GitHub's 360-minute default: six billable hours,
/// and a `cancelled` conclusion at the end of them.
#[test]
fn every_job_declares_a_timeout_or_delegates_to_one_that_does() {
    for (name, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = job_block(&body, job_name);
            // `timeout-minutes` is not a valid key on a reusable-workflow call;
            // the caps live on the jobs of the called workflow.
            if job
                .lines()
                .any(|line| line.trim().starts_with("uses: ./.github/workflows/"))
            {
                continue;
            }
            assert!(
                job_timeout(job).is_some(),
                "{name}: job `{job_name}` declares no timeout-minutes, so it \
                 inherits the 360-minute default (issue #1017)"
            );
        }
    }
}

/// Every read-only job must belong to a concurrency group, so a superseded
/// push releases its runners instead of holding them to completion. The two
/// jobs listed here are deliberate exceptions with a stated reason; a third
/// exception has to be argued in this list, not left implicit in the YAML.
#[test]
fn superseded_read_only_work_releases_its_runners() {
    // `pipeline-status` is the run's verdict, and it is what converts a hidden
    // `cancelled` into a red failure (issue #977, `scripts/check-pipeline-status.sh`).
    // Cancelling the reporter would restore exactly the blind spot it exists to
    // close. `cycle` is the scheduled learning writer: its workflow-level group
    // already sets `cancel-in-progress: false` so a running writer finishes.
    const EXEMPT: &[(&str, &str)] = &[
        ("release.yml", "pipeline-status"),
        ("learning-cycle.yml", "cycle"),
    ];

    let mut checked = 0_usize;
    for (name, body) in workflow_files() {
        let header = body.split("\njobs:\n").next().unwrap_or_default();
        // A reusable workflow has no trigger of its own; it is cancelled with
        // the caller's job, which carries the group.
        if header.contains("workflow_call:") {
            continue;
        }
        let workflow_level = header.lines().any(|line| line == "concurrency:");
        for job_name in workflow_job_names(&body) {
            if EXEMPT.contains(&(name.as_str(), job_name)) {
                continue;
            }
            let job = job_block(&body, job_name);
            let job_level = job
                .lines()
                .any(|line| line.trim_end() == "    concurrency:");
            checked += 1;
            assert!(
                workflow_level || job_level,
                "{name}: job `{job_name}` belongs to no concurrency group, so a \
                 superseded push runs it twice (issue #1017)"
            );
        }
    }
    assert!(checked >= 40, "expected every workflow job, saw {checked}");
}

/// The wrapper owns the deadline: it must kill what it wraps and exit non-zero,
/// or the job clock gets there first and the failure is mislabelled.
#[test]
fn budget_wrapper_terminates_the_overrun_and_reports_it_as_an_error() {
    let (output, elapsed) = run_budget_wrapper(
        &[("TEST_WARN_RATIO_PERCENT", "50")],
        &["2", "Runaway slice", "bash", "-c", "sleep 120"],
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(124),
        "an overrun must exit 124 like `timeout(1)`, not {:?}: {stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("::error title=Runaway slice exceeded its execution budget::"),
        "the overrun must leave an ::error annotation naming the step: {stderr}"
    );
    assert!(
        elapsed < 60,
        "the wrapper waited {elapsed}s to enforce a 2s budget; it must not \
         depend on the job clock to stop a runaway command"
    );
}

/// The warning has to arrive while the command is still running -- a
/// post-mortem warning on a killed job is exactly the diagnostic that was
/// missing from run 31937348472.
#[test]
fn budget_wrapper_warns_while_the_command_is_still_alive() {
    let (output, _) = run_budget_wrapper(
        &[("TEST_WARN_RATIO_PERCENT", "30")],
        &["4", "Slow regression test", "bash", "-c", "sleep 2"],
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "a command that finishes inside its budget must still succeed: {stderr}"
    );
    assert!(
        stderr.contains("::warning title=Slow regression test is approaching its timeout::"),
        "warning should be emitted while the wrapped command is still alive: {stderr}"
    );
}

/// Enforcement is what CI needs and what a laptop does not: running the same
/// command locally must not be killed for being slower than a runner.
#[test]
fn budget_enforcement_has_a_documented_escape_hatch() {
    let (output, _) = run_budget_wrapper(
        &[
            ("TEST_BUDGET_ENFORCE", "false"),
            ("TEST_WARN_RATIO_PERCENT", "10"),
        ],
        &["1", "Local run", "bash", "-c", "sleep 2"],
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "TEST_BUDGET_ENFORCE=false must warn without killing: {stderr}"
    );
    assert!(
        stderr.contains("::warning title=Local run is approaching its timeout::"),
        "the warning must survive the escape hatch: {stderr}"
    );
}

/// The heartbeat that would have shown *which* test was running when the slice
/// was killed. Default off, per the repository's `FORMAL_AI_CI_VERBOSE`
/// convention, so an ordinary green run stays quiet.
#[test]
fn budget_wrapper_heartbeat_is_available_but_off_by_default() {
    let quiet = run_budget_wrapper(
        &[("TEST_BUDGET_HEARTBEAT_SECONDS", "1")],
        &["30", "Quiet run", "bash", "-c", "sleep 2"],
    )
    .0;
    let quiet_stderr = String::from_utf8_lossy(&quiet.stderr);
    assert!(quiet.status.success(), "{quiet_stderr}");
    assert!(
        !quiet_stderr.contains("[budget]"),
        "the heartbeat must stay off unless FORMAL_AI_CI_VERBOSE=true: {quiet_stderr}"
    );

    let verbose = run_budget_wrapper(
        &[
            ("FORMAL_AI_CI_VERBOSE", "true"),
            ("TEST_BUDGET_HEARTBEAT_SECONDS", "1"),
        ],
        &["30", "Verbose run", "bash", "-c", "sleep 2"],
    )
    .0;
    let verbose_stderr = String::from_utf8_lossy(&verbose.stderr);
    assert!(verbose.status.success(), "{verbose_stderr}");
    assert!(
        verbose_stderr.contains("[budget]"),
        "FORMAL_AI_CI_VERBOSE=true must trace elapsed-versus-budget so the next \
         overrun says which command was still running: {verbose_stderr}"
    );
}

/// The macOS lane must actually select tests.
///
/// Issue #1017 guarded a `slice:` denominator that disagreed with the matrix,
/// because that silently *drops* tests and leaves CI green anyway. Issue #1059
/// removed the sharding -- the lane now runs the modules named in
/// `data/meta/macos-platform-tests.lino` on one runner -- but the failure mode
/// it guarded against survives in a new shape: an empty or unreadable list
/// makes the filter match nothing, and a lane that runs no tests passes.
#[test]
fn the_macos_lane_selects_a_non_empty_set_of_tests() {
    let listed = repository_file("data/meta/macos-platform-tests.lino");

    let modules: Vec<&str> = listed
        .lines()
        .filter_map(|line| line.trim().strip_prefix("module "))
        .collect();

    assert!(
        modules.len() >= 5,
        "`data/meta/macos-platform-tests.lino` names {} modules. A macOS lane \
         that runs nothing is worse than no lane: it reports success without \
         testing the platform it exists for.",
        modules.len()
    );

    let macos = repository_file(".github/workflows/macos-core-tests.yml");
    assert!(
        macos.contains("platform.filter"),
        "the lane must consume the planned filter; without it the `-E` \
         expression is empty and nextest runs the whole archive"
    );
}

/// The archive and the slices each ran `simulate-fresh-merge.sh`, and each
/// resolved `origin/$BASE_REF` *at its own start time*. The macOS runner pool
/// serializes sixteen slices across roughly forty minutes, so a single push to
/// the base branch mid-run gave the archive one merged tree and the later slices
/// another: run 31993872931 built its archive at 04:54Z, `main` gained a commit
/// at 05:23:29Z, and every slice that started after that failed `Verify archive
/// source tree` -- slice 9 started 05:20Z and passed, slice 3 started 05:24Z and
/// failed. Fifteen red slices, no defect in the pull request. That is a false
/// result of exactly the class issue #1017 is about, so the base commit is
/// recorded once by the archive and merged by every slice.
/// Every job that merges the base branch must merge the *same* commit. Any job
/// resolving `origin/$BASE_REF` for itself reintroduces the race, and outside
/// the macOS lane -- which compares trees across jobs -- the divergence would be
/// silent: run 31993872684 packaged `linux-x64` and `macos-arm64` from `main` =
/// `1858b3386` and `windows-arm64` from `d1439e557`, and shipped six installers
/// built from two source trees as one release set with nothing to catch it.
#[test]
fn every_base_branch_merge_uses_one_pinned_commit_per_workflow() {
    let script = repository_file("scripts/simulate-fresh-merge.sh");
    assert!(
        script.contains("BASE_COMMIT"),
        "simulate-fresh-merge.sh must accept a pinned base commit"
    );
    assert!(
        script.contains("git merge \"$BASE_SHA\""),
        "the merge must use the resolved base commit so the pinned and unpinned \
         paths cannot diverge"
    );

    for (name, body) in workflow_files() {
        // Count real invocations only: the script is also named in prose, and a
        // comment that mentions it does not merge anything.
        let invocations = body
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with('#') && trimmed.contains("simulate-fresh-merge.sh")
            })
            .count();
        if invocations == 0 {
            continue;
        }
        let pinned = body.matches("BASE_COMMIT:").count();
        assert_eq!(
            pinned, invocations,
            "{name} runs the fresh-merge simulation {invocations} time(s) but \
             pins the base commit {pinned} time(s); an unpinned invocation \
             resolves the base branch tip at its own start time, so two jobs \
             minutes apart merge different commits (issue #1017)"
        );

        // The pinned value has to come from one resolver shared by the whole
        // workflow -- either a `base` job's output, or the caller's input.
        let from_one_source =
            body.contains("needs.base.outputs.commit") || body.contains("inputs.base-commit");
        assert!(
            from_one_source,
            "{name} must take its pinned commit from a single per-workflow \
             resolver (a `base` job output, or a `base-commit` input), not from \
             a per-job lookup"
        );
    }

    // The reusable macOS lane must accept the pin rather than resolve its own,
    // because its archive job and its slices are separate jobs.
    let macos = repository_file(".github/workflows/macos-core-tests.yml");
    assert!(
        macos.contains("base-commit:") && macos.contains("inputs.base-commit"),
        "the macOS lane must take the base commit from its caller so the archive \
         and every slice merge the same one"
    );
    let release = repository_file(".github/workflows/release.yml");
    assert!(
        release.contains("base-commit: ${{ needs.base.outputs.commit }}"),
        "release.yml must pass its pinned commit into the macOS lane"
    );
}

/// The `CodeQL` run emitted parse diagnostics on every single invocation because
/// the Rust extractor, in `build-mode: none`, parses every `.rs` file on disk --
/// including a deliberately truncated documentation excerpt.
#[test]
fn codeql_skips_archived_evidence_but_still_analyses_compiled_code() {
    let security = repository_file(".github/workflows/security.yml");
    assert!(
        security.contains("config-file: ./.github/codeql/codeql-config.yml"),
        "the CodeQL init step must load the shared config"
    );

    let config = repository_file(".github/codeql/codeql-config.yml");
    let excluded_paths: Vec<&str> = config
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.trim().strip_prefix("- "))
        .collect();

    for excluded in ["docs/**", "dev/**", "experiments/**"] {
        assert!(
            excluded_paths.contains(&excluded),
            "{excluded} holds archived evidence and scratch reproductions; \
             parsing it only produces diagnostics (issue #1017)"
        );
    }
    assert!(
        !excluded_paths
            .iter()
            .any(|path| path.starts_with("examples")),
        "`examples/` are real Cargo targets the workspace compiles, so they \
         must stay in scope: {excluded_paths:?}"
    );

    // The file that produced the diagnostics is still archived, so the
    // exclusion above is load-bearing rather than historical.
    let excerpt = "docs/case-studies/issue-96/raw-data/link-calculator-lib-excerpt.rs";
    assert!(
        fs::metadata(format!("{}/{excerpt}", env!("CARGO_MANIFEST_DIR"))).is_ok(),
        "{excerpt} is the truncated excerpt the exclusion exists for"
    );
}

/// An audit is only worth having if it cannot be silenced. Every ignored
/// advisory carries a proof line that `scripts/check-rust-dependencies.sh`
/// re-derives from the dependency graph on every run.
#[test]
fn every_ignored_advisory_carries_a_proof_that_ci_rechecks() {
    let config = repository_file(".cargo/audit.toml");
    let security = repository_file(".github/workflows/security.yml");

    assert!(
        security.contains("bash scripts/check-rust-dependencies.sh"),
        "the Cargo audit job must go through the script that checks the \
         ignore proofs, not call `cargo audit` directly"
    );
    assert!(
        security.contains("schedule:"),
        "a lockfile that stops changing must still be re-audited: an advisory \
         published against an unchanged dependency has to surface on its own"
    );

    let ignored: Vec<&str> = config
        .lines()
        .find(|line| line.trim_start().starts_with("ignore ="))
        .expect("the audit config must declare an ignore list")
        .split('"')
        .skip(1)
        .step_by(2)
        .collect();

    for advisory in ignored {
        assert!(
            config.contains(&format!("# {advisory} unreachable = \"")),
            "{advisory} is ignored without a `# {advisory} unreachable = \
             \"<crate>@<version>\"` proof line; \
             scripts/check-rust-dependencies.sh re-derives that proof with \
             `cargo tree --invert`, so an ignore expires the moment the crate \
             enters the build graph (issue #1017)"
        );
    }
}

/// `hashFiles('**/Cargo.lock')` keys five caches. If the lockfile ever stopped
/// being committed, `hashFiles` would return the same empty-set hash on every
/// run: a permanently warm-looking, permanently wrong cache key, and a build
/// that no longer resolves the versions the tests ran against.
#[test]
fn cargo_lock_is_committed_so_cache_keys_stay_meaningful() {
    let tracked = Command::new("git")
        .args(["ls-files", "--error-unmatch", "Cargo.lock"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run git ls-files");

    assert!(
        tracked.status.success(),
        "Cargo.lock must be committed: it is the input to every \
         `hashFiles('**/Cargo.lock')` cache key, and an untracked lockfile \
         degrades all of them to one constant hash (issue #1017)"
    );

    // Issue #1076 collapsed the last inline registry-cache blocks into
    // `.github/actions/cache-cargo-registry`, so the key this test protects now
    // lives in the composite action rather than being repeated in release.yml.
    // Accept either home: what matters is that a `hashFiles('**/Cargo.lock')`
    // key still exists somewhere on the path release.yml takes.
    let action = repository_file(".github/actions/cache-cargo-registry/action.yml");
    let release = repository_file(".github/workflows/release.yml");
    assert!(
        action.contains("hashFiles('**/Cargo.lock')")
            || release.contains("hashFiles('**/Cargo.lock')"),
        "the cache keys this protects must still exist"
    );
    assert!(
        release.contains("./.github/actions/cache-cargo-registry")
            || release.contains("hashFiles('**/Cargo.lock')"),
        "release.yml must still reach a Cargo.lock-keyed registry cache"
    );
}

/// The link check parses lychee's Markdown report with a hand-written parser.
/// A regression there either invents broken links or swallows real ones, and
/// neither shows up as a test failure unless the parser itself is tested.
#[test]
fn link_report_parser_is_unit_tested_before_it_is_trusted() {
    let links = repository_file(".github/workflows/links.yml");

    assert!(
        links.contains("node --test scripts/check-web-archive.test.mjs"),
        "links.yml must run the parser's unit tests before lychee (issue #1017)"
    );
    assert_eq!(
        links.matches("scripts/check-web-archive.test.mjs").count(),
        3,
        "the test file must be in both `paths:` filters as well as the step, \
         or editing it would not re-run the check it guards"
    );
    assert!(
        links.contains("if: ${{ !cancelled() && steps.lychee.outputs.exit_code != 0 }}"),
        "a cancelled link check must not append a `broken links` error to a run \
         that never finished checking them"
    );
    assert!(
        !repository_file("scripts/check-web-archive.test.mjs").is_empty(),
        "the parser test file must exist"
    );
}

/// A `CodeQL` run that reports success while 1,023 files were "extracted with
/// errors" is the worst kind of false negative: the dashboard is green because
/// nothing was analysed. Run 31937348308 emitted 20,725 `macro expansion
/// failed` diagnostics, every one of them for a macro defined in `std`/`core`
/// or in a dependency and none defined here, because the extractor had no
/// sysroot override and the ambient `std` outran the rust-analyzer the `CodeQL`
/// bundle vendors (github/codeql#19982).
///
/// Both variables are load-bearing. Setting only `_SYSROOT_SRC` takes
/// rust-analyzer's `discover_with_src_override` path, which keeps the
/// discovered binary sysroot and leaves the failures in place — so a future
/// edit that drops one of them would silently restore the false negative.
#[test]
fn codeql_rust_lane_pins_the_extractor_sysroot() {
    let security = repository_file(".github/workflows/security.yml");
    let job = job_block(&security, "codeql");

    for variable in [
        "CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT=",
        "CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC=",
    ] {
        assert!(
            job.contains(variable),
            "the CodeQL job must export {variable} into $GITHUB_ENV, or `std` \
             macros stay unexpanded and their call sites go unanalysed \
             (issue #1017, github/codeql#19982)"
        );
    }

    assert!(
        job.contains("CODEQL_RUST_SYSROOT_TOOLCHAIN:"),
        "the pinned toolchain must be declared once, as a named variable, so \
         raising it as CodeQL vendors a newer rust-analyzer is a one-line edit"
    );
    assert!(
        job.contains("if: matrix.language == 'rust'"),
        "the sysroot pin costs a toolchain install and only affects the Rust \
         extractor, so it must not run in the `actions` lane"
    );
    assert!(
        job.contains("--component rust-src"),
        "`_SYSROOT_SRC` points into `rust-src`; without the component the \
         directory does not exist and the pin silently does nothing"
    );
    assert!(
        job.contains("::warning title=CodeQL sysroot pin unavailable::"),
        "losing the pin must be visible: it is a mitigation for an upstream \
         defect, so it warns instead of failing, and a silent fallback would \
         let the 20,725 diagnostics return unnoticed"
    );
}

/// The same false negative from a third direction. In run 95255998673 the
/// `macos-x64` leg wrote a complete DMG, ZIP and both blockmaps and then failed,
/// because electron-builder's download of `dmgbuild-bundle-x86_64-75c8a6c.tar.gz`
/// stalled for the whole 600 000 ms `got` request timeout: its own retry
/// recovered two seconds later, but `AsyncTaskManager.awaitTasks()` rethrew the
/// timeout it had already recorded. Ten of the job's 43.7 minutes were that one
/// stalled socket, against a 50-minute cap.
///
/// The prefetch seeds the checksum-validated archive cache `downloadAndExtract`
/// reads before it touches the network, so it only helps if it runs before
/// **every** packaging step, on every platform -- the toolset is not
/// macOS-specific.
#[test]
fn desktop_packaging_seeds_the_builder_toolset_cache_first() {
    let workflow = repository_file(".github/workflows/desktop-release.yml");
    let build = job_block(&workflow, "build");

    let prefetch = build
        .find("- name: Prefetch electron-builder toolsets")
        .expect(
            "the packaging job must prefetch electron-builder's toolsets; without it a stalled \
             toolset download fails a build that produced every artifact (issue #1017)",
        );

    for invocation in [
        "npx --no-install electron-builder",
        "bash scripts/package-macos-with-retry.sh",
    ] {
        let mut searched = 0;
        while let Some(offset) = build[searched..].find(invocation) {
            let at = searched + offset;
            assert!(
                at > prefetch,
                "`{invocation}` at byte {at} runs before the toolset prefetch at byte {prefetch}; \
                 an unseeded cache leaves the ten-minute stall in place"
            );
            searched = at + invocation.len();
        }
        // Issue #1055: the Linux and Windows legs now call the retry wrapper,
        // which invokes electron-builder itself. What this test guards is the
        // *order* -- packaging must follow the toolset prefetch -- so a leg
        // that packages through the wrapper satisfies it the same way.
        assert!(
            searched > 0 || build.contains("bash scripts/package-macos-with-retry.sh"),
            "`{invocation}` must still package the app"
        );
    }

    let step = &build[prefetch..];
    let step_body = &step[..step.find("\n      - name:").unwrap_or(step.len())];
    assert!(
        !step_body.contains("if:"),
        "7-Zip is downloaded on every platform, so the prefetch must not be \
         restricted to one leg (issue #1017 requirement R1017-11)"
    );
    assert!(
        !repository_file("desktop/scripts/prefetch-builder-toolsets.mjs").is_empty(),
        "the prefetch script must exist"
    );
}

/// A retry is only an improvement while it can finish. Packaging is the last
/// expensive step of the job, so an attempt started too late is killed by
/// `timeout-minutes` -- and GitHub reports that as **cancelled**, which is the
/// exact false negative this issue exists to remove.
#[test]
fn macos_packaging_retry_is_bounded_by_a_budget() {
    let workflow = repository_file(".github/workflows/desktop-release.yml");
    let build = job_block(&workflow, "build");

    let deadlines = build
        .matches("FORMAL_AI_MACOS_PACKAGE_DEADLINE_EPOCH: ${{ env.FORMAL_AI_JOB_DEADLINE_EPOCH }}")
        .count();
    let wrappers = build
        .matches("bash scripts/package-macos-with-retry.sh")
        .count();
    assert_eq!(
        deadlines, wrappers,
        "every macOS packaging step must pass the job deadline, or a retry can \
         outlive the job clock and report `cancelled` instead of `failure`"
    );

    // The deadline is only trustworthy while it is computed from the *same* cap
    // that kills the job. A literal here and a literal in `timeout-minutes`
    // would drift, and a guard derived from a stale cap is worse than none.
    assert!(
        build.contains("timeout-minutes: ${{ matrix.capmin }}"),
        "the job cap must come from the matrix so the deadline can reuse it"
    );
    assert!(
        build.contains("FORMAL_AI_JOB_DEADLINE_EPOCH=") && build.contains("$GITHUB_ENV"),
        "the build job must publish its deadline through $GITHUB_ENV"
    );
    let deadline_step = build
        .find("- name: Record the job deadline")
        .expect("the build job must record its own deadline");
    let deadline_body = &build[deadline_step..];
    let deadline_body = &deadline_body[..deadline_body
        .find("\n      - ")
        .unwrap_or(deadline_body.len())];
    assert!(
        deadline_body.contains("${{ matrix.capmin }}"),
        "the deadline must be derived from the matrix cap, not from a second \
         copy of the same number"
    );
    assert!(
        deadline_body.contains("POST_PACKAGING_RESERVE_SECONDS"),
        "the deadline must reserve time for the steps that follow packaging \
         (smoke test, checksums, uploads), not just for packaging itself"
    );
    let packaging_step = build
        .find("bash scripts/package-macos-with-retry.sh")
        .expect("a macOS packaging step must exist");
    assert!(
        deadline_step < packaging_step,
        "the deadline must be recorded before packaging consumes it"
    );

    let wrapper = repository_file("desktop/scripts/package-macos-with-retry.sh");
    assert!(
        wrapper.contains("Timeout awaiting 'request' for [0-9]+ms"),
        "the wrapper must recognise the toolset download timeout observed in \
         run 95255998673 as transient; matching only `hdiutil` signatures is \
         why that run was never retried"
    );
    assert!(
        wrapper.contains("FORMAL_AI_MACOS_PACKAGE_BUDGET_SECONDS:-}")
            && wrapper.contains("FORMAL_AI_MACOS_PACKAGE_DEADLINE_EPOCH:-}"),
        "both inputs must be optional so the wrapper keeps working outside this \
         workflow, where neither a budget nor a deadline exists"
    );
}

/// The npm 11.17.0 advisory as emitted verbatim by `npm install` in `vscode/`
/// on 2026-08-17, kept whole (banner, entries, blank line, closing advice) so
/// the classifier is exercised against the shape npm actually prints.
const NPM_ALLOW_SCRIPTS_WARNING: &str = "\
npm warn allow-scripts 3 packages have install scripts not yet covered by allowScripts:
npm warn allow-scripts   electron-winstaller@5.4.0 (postinstall: node ./lib/postinstall.js)
npm warn allow-scripts   node-pty@1.2.0-beta.15 (install: node scripts/install.js)
npm warn allow-scripts   puppeteer@25.7.0 (postinstall: node install.mjs)
npm warn allow-scripts
npm warn allow-scripts Run `npm approve-scripts --allow-scripts-pending` to review, or `npm approve-scripts <pkg>` to allow.";

/// A latent copy of issue #796, found by running the gate locally rather than
/// by waiting for it to fire. npm 11.17.0 prints an `allow-scripts` advisory
/// that today's runner image (npm 10.9.x -- run 95255998673 installed 495
/// packages with a clean stderr) does not, and
/// `scripts/install-node-dependencies.sh` classified every one of those lines
/// as an unexpected diagnostic: the next runner-image bump would have failed
/// both install steps of the Desktop Release workflow over a warning about
/// scripts that had already run successfully.
///
/// The gate itself is right to fail -- a dependency that gained an install
/// script is a supply-chain change worth a human -- so this pins the part that
/// was wrong: the report has to name the packages and the command that clears
/// them, instead of the bare "Unexpected npm stderr".
#[test]
fn unreviewed_install_scripts_are_reported_with_the_command_that_clears_them() {
    let dir = sandbox(NPM_ALLOW_SCRIPTS_WARNING);
    let output = run_classifier(&dir);
    fs::remove_dir_all(&dir).expect("sandbox must be removed");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    assert!(
        !output.status.success(),
        "an install script nobody has reviewed must still stop the build; \
         stderr: {stderr}"
    );
    for entry in [
        "electron-winstaller@5.4.0 (postinstall: node ./lib/postinstall.js)",
        "node-pty@1.2.0-beta.15 (install: node scripts/install.js)",
        "puppeteer@25.7.0 (postinstall: node install.mjs)",
    ] {
        assert!(
            stderr.contains(entry),
            "the report must name what runs, not just that something does: \
             {entry} missing from {stderr}"
        );
    }
    assert!(
        stderr.contains(
            "approve-scripts --no-allow-scripts-pin electron-winstaller node-pty puppeteer"
        ),
        "the report must hand over a runnable command whose package list is \
         name-only -- a pinned version unreviews itself on the next float, \
         which is exactly how issue #796 broke CI; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("Unexpected npm stderr"),
        "an advisory the script understands must not also be reported as one \
         it does not; stderr: {stderr}"
    );
}

/// npm's advisory is a preview of an enforced check: "A future release will
/// block unreviewed install scripts." When that lands, an undeclared
/// `allowScripts` stops `node-pty`, `keytar` and `esbuild` from building their
/// native halves and stops `puppeteer`/`@playwright/browser-chromium` from
/// fetching their browsers -- a much quieter failure than a warning. Declaring
/// the field now is what makes the two install steps forward-compatible, and
/// it removes the advisory today: both projects reinstall with an empty stderr
/// once the field is present.
#[test]
fn every_installed_node_project_records_its_install_scripts_by_name() {
    let mut projects: Vec<String> = Vec::new();
    for (_, workflow) in workflow_files() {
        for line in workflow.lines() {
            if let Some(tail) = line.split("scripts/install-node-dependencies.sh ").nth(1) {
                let directory = tail.trim().trim_end_matches('"');
                if !directory.is_empty() && !projects.iter().any(|seen| seen == directory) {
                    projects.push(directory.to_string());
                }
            }
        }
    }
    projects.sort();
    assert_eq!(
        projects,
        vec!["desktop".to_string(), "vscode".to_string()],
        "the projects installed through the classifier changed; the review \
         record below has to follow them"
    );
    // The third npm project, `tests/e2e`, installs with plain `npm ci` under
    // node 24 (npm 11) and reported no pending install scripts at all
    // (`npm approve-scripts --allow-scripts-pending --json` -> `[]`), so it has
    // nothing to record and is intentionally absent from this list.

    for project in projects {
        let manifest: serde_json::Value =
            serde_json::from_str(&repository_file(&format!("{project}/package.json")))
                .unwrap_or_else(|error| panic!("{project}/package.json must parse: {error}"));
        let allow_scripts = manifest
            .get("allowScripts")
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| {
                panic!(
                    "{project}/package.json must declare an `allowScripts` object recording \
                     which dependencies may run install scripts (issue #1017); write it with \
                     `npm --prefix {project} approve-scripts --no-allow-scripts-pin <package>`"
                )
            });
        assert!(
            !allow_scripts.is_empty(),
            "{project} installs packages with install scripts, so the record cannot be empty"
        );
        for (package, decision) in allow_scripts {
            assert_eq!(
                decision,
                &serde_json::Value::Bool(true),
                "{project}/package.json pins `{package}` to {decision}; a version range here \
                 unreviews itself the moment the dependency floats, which is issue #796 \
                 rewritten as a supply-chain gate -- record the name only"
            );
            assert!(
                !package.trim_start_matches('@').contains('@'),
                "{package} carries a version; `allowScripts` keys are package names"
            );
        }
    }

    // The packages behind the advisory measured on 2026-08-17. They are all
    // build-essential: native compilation (node-pty, keytar), prebuilt binary
    // extraction (esbuild, @vscode/vsce-sign), browser downloads (puppeteer,
    // @playwright/browser-chromium) and the Windows installer builder.
    for (project, package) in [
        ("desktop", "electron-winstaller"),
        ("desktop", "node-pty"),
        ("desktop", "puppeteer"),
        ("vscode", "@playwright/browser-chromium"),
        ("vscode", "@vscode/vsce-sign"),
        ("vscode", "esbuild"),
        ("vscode", "keytar"),
        ("vscode", "puppeteer"),
    ] {
        assert!(
            repository_file(&format!("{project}/package.json"))
                .contains(&format!("\"{package}\": true")),
            "{package} runs an install script during `{project}`'s install and must stay \
             recorded, or the next npm release blocks it"
        );
    }
}
