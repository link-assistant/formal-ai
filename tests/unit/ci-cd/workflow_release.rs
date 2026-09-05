//! Release-workflow structure + issue #479 Pages deploy / landing-page
//! assertions. The desktop-gating half lives in `workflow_release_desktop`,
//! split out when this file crossed the 1000-line cap. Shared helpers live in
//! `workflow_fixtures`.

use std::fs;

use super::workflow_fixtures::*;

#[test]
fn pages_deploy_waits_for_release_ref_before_pages_upload() {
    let workflow = release_workflow();
    let auto_release = job_block(&workflow, "auto-release");
    let manual_release = job_block(&workflow, "manual-release");
    let deploy_demo = job_block(&workflow, "deploy-pages");

    assert!(auto_release.contains("outputs:\n      pages_sha:"));
    assert!(auto_release.contains("Resolve Pages deploy ref"));
    assert!(manual_release.contains("outputs:\n      pages_sha:"));
    assert!(manual_release.contains("Resolve Pages deploy ref"));
    assert!(deploy_demo.contains("needs: [build, auto-release, manual-release]"));
    assert!(deploy_demo.contains("needs.build.result == 'success'"));
    assert!(deploy_demo.contains("github.ref == 'refs/heads/main'"));
    assert!(deploy_demo.contains("needs.auto-release.result == 'success'"));
    assert!(deploy_demo.contains("needs.manual-release.result == 'success'"));
    assert!(deploy_demo.contains("Select Pages deployment ref"));
    assert!(deploy_demo.contains(
        "PAGES_DEPLOY_SHA: ${{ needs.auto-release.outputs.pages_sha || needs.manual-release.outputs.pages_sha || github.sha }}"
    ));
    assert!(deploy_demo.contains("ref: ${{ steps.pages_ref.outputs.sha }}"));
}

#[test]
fn rust_script_install_steps_use_retry_wrapper() {
    let workflow = release_workflow();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let install_script =
        fs::read_to_string(format!("{manifest_dir}/scripts/install-rust-script.sh")).unwrap();

    assert!(
        !workflow.contains("run: cargo install rust-script"),
        "workflow should not call cargo install directly because crates.io HTTP failures are transient"
    );
    assert_eq!(
        workflow
            .matches("run: bash scripts/install-rust-script.sh")
            .count(),
        // +1 for the evidence-check job added for issue #808.
        9,
        "each rust-script install step should use the retry wrapper"
    );
    assert!(install_script.contains("RUST_SCRIPT_INSTALL_ATTEMPTS"));
    assert!(install_script.contains("cargo install rust-script --locked"));
    assert!(install_script.contains("sleep \"$delay\""));
}

#[test]
fn pages_deploy_uses_github_pages_workflow_artifact() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-pages");

    assert!(deploy_demo.contains("pages: write"));
    assert!(deploy_demo.contains("id-token: write"));
    assert!(deploy_demo.contains("environment:\n      name: github-pages"));
    assert!(deploy_demo.contains("url: ${{ steps.deployment.outputs.page_url }}"));
    assert!(deploy_demo.contains("actions/configure-pages@v6"));
    assert!(deploy_demo.contains("actions/upload-pages-artifact@v5"));
    assert!(deploy_demo.contains("path: src/web"));
    assert!(deploy_demo.contains("id: deployment"));
    assert!(deploy_demo.contains("actions/deploy-pages@v5"));
    assert!(!deploy_demo.contains("peaceiris/actions-gh-pages"));
    assert!(!deploy_demo.contains("publish_dir: src/web"));
    assert!(!deploy_demo.contains("publish_branch: gh-pages"));
}

/// PR #965 review: "All CI/CD warnings, and errors must be also fixed.
/// Including all false positives and false negatives."
///
/// `actions/deploy-pages` accepts at most a 600 000 ms wait. On `main` run
/// 31090830031 the artifact uploaded successfully (15.7 MB, id 8966823763) and
/// the action then reported `Current status: deployment_queued` for that entire
/// default before `Timeout reached, aborting!` — a red pipeline caused by
/// GitHub's Pages deployment queue rather than by anything in the commit. The
/// next push deployed green with no code change, which is the signature of a
/// false positive. Pin the documented maximum rather than supplying the
/// unsupported 1 200 000 ms value that the action silently clamps, and keep
/// the job's own budget above it.
#[test]
fn pages_deploy_uses_the_actions_maximum_supported_wait() {
    const DEPLOY_ACTION_MAXIMUM_MS: u64 = 600_000;

    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-pages");

    let timeout_ms = deploy_demo
        .split("actions/deploy-pages@v5")
        .nth(1)
        .and_then(|after| after.split_once("timeout: "))
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("the deploy-pages step should pin an explicit `timeout:` in milliseconds");

    assert!(
        timeout_ms == DEPLOY_ACTION_MAXIMUM_MS,
        "deploy-pages accepts at most {DEPLOY_ACTION_MAXIMUM_MS} ms, got {timeout_ms} ms"
    );

    let job_budget_ms = job_block(&workflow, "deploy-pages")
        .split_once("timeout-minutes: ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|minutes| minutes * 60_000)
        .expect("deploy-pages should declare timeout-minutes");

    assert!(
        job_budget_ms > timeout_ms,
        "the deploy-pages job budget ({job_budget_ms} ms) must exceed the deployment wait \
         ({timeout_ms} ms), or `timeout-minutes` kills the job before the wait can help"
    );
}

#[test]
fn pages_e2e_uses_deployment_output_url() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-pages");
    let pages_e2e = job_block(&workflow, "test-e2e-pages");

    assert!(deploy_demo.contains("page_url: ${{ steps.deployment.outputs.page_url }}"));
    assert!(pages_e2e.contains("needs.deploy-pages.outputs.page_url"));
    assert!(!pages_e2e.contains("PAGES_URL=https://link-assistant.github.io/formal-ai"));
}

#[test]
fn pages_deploy_is_pinned_and_live_e2e_waits_for_matching_deployment() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-pages");
    let pages_e2e = job_block(&workflow, "test-e2e-pages");

    assert!(
        deploy_demo.contains("ref: ${{ steps.pages_ref.outputs.sha }}"),
        "Pages deployment should use the selected Pages SHA, which is the release child commit when auto-release creates one"
    );
    assert!(
        deploy_demo.contains("Stamp GitHub Pages artifact"),
        "Pages deployment should stamp a per-commit asset marker before upload"
    );
    assert!(
        deploy_demo.contains(
            "scripts/stamp-pages-artifact.sh src/web \"${{ steps.pages_ref.outputs.sha }}\""
        ),
        "Pages deployment should stamp src/web with the selected Pages deployment SHA"
    );
    assert!(
        pages_e2e.contains("scripts/wait-for-pages-deployment.sh"),
        "live Pages e2e should poll for the deployed commit before Playwright starts"
    );
    assert!(
        pages_e2e.contains("needs.deploy-pages.outputs.page_url"),
        "live Pages e2e should probe the resolved Pages URL"
    );
    assert!(
        pages_e2e.contains(
            "PAGES_DEPLOY_SHA: ${{ needs.deploy-pages.outputs.pages_sha || github.sha }}"
        ),
        "live Pages e2e should wait for the same selected SHA that deploy-pages stamped"
    );
    assert!(
        !pages_e2e.contains("run: sleep 30"),
        "a fixed sleep can still test stale GitHub Pages assets"
    );
}

#[test]
fn wait_for_pages_deployment_is_marker_authoritative() {
    // Issue #479 (root cause, take 2): the live-Pages freshness probe used to
    // require the raw deploy SHA to appear in the served index BODY
    // (`grep -Fq "$expected_sha" "$index_file"`). That coupled the probe to every
    // root page embedding the commit SHA verbatim. When the issue #479 landing
    // page shipped at `/` WITHOUT cache-busted asset refs, the stamped index
    // never contained the SHA, so the probe ran the full 300s and timed out --
    // failing the whole pipeline and (via the desktop-release gate) suppressing
    // every desktop build. The probe is now marker-authoritative: GitHub Pages
    // deploys atomically, so deployment.json advertising "sha":"<expected_sha>"
    // is sufficient proof the matching stamped index is live.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wait_script = fs::read_to_string(format!(
        "{manifest_dir}/scripts/wait-for-pages-deployment.sh"
    ))
    .unwrap();

    // The marker SHA is the authoritative freshness signal.
    assert!(
        wait_script.contains("${expected_sha}") && wait_script.contains("\"$marker_file\""),
        "wait script should require the deployment.json marker to advertise the expected SHA"
    );
    // The brittle "index body must contain the SHA" requirement must be gone.
    assert!(
        !wait_script.contains("-Fq \"$expected_sha\" \"$index_file\""),
        "issue #479 regression: the probe must NOT hard-require the raw SHA in the index body -- \
         a valid root page without cache-busted asset refs would hang the probe for the full timeout"
    );
    // But the defensive placeholder guards (catch a half-run stamp step) remain.
    assert!(
        wait_script.contains("__FORMAL_AI_ASSET_VERSION__")
            && wait_script.contains("__FORMAL_AI_VERSION__"),
        "wait script should still reject an index that retains un-stamped placeholders"
    );
}

#[test]
fn pages_e2e_navigation_preserves_repository_subpath() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pages_config = fs::read_to_string(format!(
        "{manifest_dir}/tests/e2e/playwright.pages.config.js"
    ))
    .unwrap();
    let demo_spec =
        fs::read_to_string(format!("{manifest_dir}/tests/e2e/tests/demo.spec.js")).unwrap();
    let multilingual_spec = fs::read_to_string(format!(
        "{manifest_dir}/tests/e2e/tests/multilingual.spec.js"
    ))
    .unwrap();
    let connectivity_spec = fs::read_to_string(format!(
        "{manifest_dir}/tests/e2e/tests/connectivity.spec.js"
    ))
    .unwrap();

    assert!(
        pages_config.contains("normalizeBaseUrl"),
        "Pages e2e should normalize PAGES_URL with a trailing slash so ./ resolves inside /formal-ai/"
    );
    assert!(
        pages_config.contains("https://link-assistant.github.io/formal-ai/"),
        "default Pages URL should include the repository subpath and trailing slash"
    );

    // The app moved to /app/ (issue #479), so the Pages baseURL targets /app/.
    // The app specs navigate with a relative './' (→ /app/), while connectivity
    // reaches its sibling harness with a relative '../tests/' (→ /tests/). Both
    // are relative, so the /formal-ai/ repository subpath is always preserved;
    // an absolute '/…' would drop it.
    for (path, spec, expected_nav) in [
        (
            "tests/e2e/tests/demo.spec.js",
            demo_spec.as_str(),
            "page.goto('./')",
        ),
        (
            "tests/e2e/tests/multilingual.spec.js",
            multilingual_spec.as_str(),
            "page.goto('./')",
        ),
        (
            "tests/e2e/tests/connectivity.spec.js",
            connectivity_spec.as_str(),
            "page.goto('../tests/')",
        ),
    ] {
        assert!(
            !spec.contains("page.goto('/');"),
            "{path} should not navigate to / because URL resolution drops the /formal-ai/ subpath"
        );
        assert!(
            spec.contains(expected_nav),
            "{path} should navigate with a relative {expected_nav} so Pages tests stay under the repository subpath"
        );
    }
}

#[test]
fn test_job_skips_non_code_changes() {
    // Issue #442: the `test` job ran whenever the `changelog` job was *skipped*.
    // `changelog` is skipped precisely when there are no code changes (docs-only
    // commits, .gitkeep edits, changelog-fragment-only commits), so the
    // `needs.changelog.result == 'skipped'` clause turned "nothing relevant
    // changed" into "run the entire test suite". This regression guard pins the
    // corrected gating: `test` keys off the detect-changes outputs, exactly like
    // `lint` and `coverage`, and never resurrects the changelog-skip escape.
    let workflow = release_workflow();
    let test = job_block(&workflow, "test");

    assert!(
        !test.contains("needs.changelog.result == 'skipped'"),
        "test job must not run merely because the changelog check was skipped; \
         a skipped changelog means there were no code changes (issue #442)"
    );
    assert!(
        !test.contains("needs.changelog.result == 'success'"),
        "test job should be decoupled from the changelog check and gate on the \
         change detector instead (issue #442)"
    );
    // Issue #1017 added `base`, which resolves the base-branch commit once so
    // every gate merges the same one. Assert the dependency rather than the
    // exact list, so adding a dependency cannot break a contract that is about
    // `detect-changes`.
    assert!(
        test.contains("needs: [detect-changes")
            && test.lines().any(|line| {
                line.trim().starts_with("needs:") && line.contains("detect-changes")
            }),
        "test job should depend on detect-changes so it can gate on its outputs"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.any-code-changed == 'true'"),
        "test job should run when code files changed"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.rs-changed == 'true'"),
        "test job should run when Rust sources changed"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.toml-changed == 'true'"),
        "test job should run when Cargo manifests changed"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.workflow-changed == 'true'"),
        "test job should run when the CI workflow itself changed"
    );
    assert!(
        !test.contains("github.event_name == 'push'")
            && test.contains("github.event_name == 'workflow_dispatch'"),
        "issue #846 requires pushes to obey detect-changes while manual \
         dispatch remains unconditional"
    );
    assert!(
        // Issue #808 / CI-CD-BEST-PRACTICES.md section 10: `always()` also runs
        // the job when the *run* is cancelled, which is not what this gate wants.
        // `!cancelled()` is enough to stop a skipped `detect-changes` from
        // cascading -- any status-check function disables the auto-skip.
        test.contains("!cancelled()") && !test.contains("always()"),
        "test job needs !cancelled() so the skipped detect-changes dependency \
         does not cascade on workflow_dispatch"
    );
}

#[test]
fn change_gated_jobs_never_depend_on_a_skipped_changelog() {
    // Generalises issue #442 across every change-gated job: none of them should
    // treat a *skipped* changelog/version check as a signal to run. A skipped
    // upstream check means "no code changed", which must never widen coverage.
    // The `coverage` job moved to `.github/workflows/coverage.yml` for issue
    // #895; `coverage_jobs_never_depend_on_a_skipped_upstream_check` in
    // `workflow_coverage.rs` pins this same property for it there.
    let workflow = release_workflow();
    for job_name in ["lint", "test", "test-e2e-local"] {
        let job = job_block(&workflow, job_name);
        // Inspect only effective YAML (skip `#` comment lines) so the rationale
        // comments that quote the old buggy clause don't trip the guard.
        let has_skip_dependency = job
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .any(|line| line.contains("result == 'skipped'"));
        assert!(
            !has_skip_dependency,
            "{job_name} job must not run because an upstream check was skipped (issue #442)"
        );
    }
}

/// Issue #812: both release jobs gated on `[lint, test, build]` alone, so a red
/// `Secrets Scan` or E2E suite on `main` did not stop the crate, the Docker
/// image and the GitHub Release from publishing.
#[test]
fn releases_do_not_publish_past_a_failing_secrets_scan_or_e2e_suite() {
    let workflow = release_workflow();

    for job_name in ["auto-release", "manual-release"] {
        let job = job_block(&workflow, job_name);
        let effective: String = job
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        for gate in ["secrets-scan", "test-e2e-local", "test-agent-cli-e2e"] {
            // The acceptable results must be enumerated, not excluded. A job
            // killed by its own `timeout-minutes` reports as 'cancelled', which
            // a `!= 'failure'` guard would wave through -- run 29767811026 is
            // the observed instance of exactly that result value.
            assert!(
                effective.contains(&format!(
                    "(needs.{gate}.result == 'success' || needs.{gate}.result == 'skipped')"
                )),
                "{job_name} must gate on {gate} being success-or-skipped, so a \
                 timed-out (cancelled) job cannot release (issue #812)"
            );
            assert!(
                !effective.contains(&format!("needs.{gate}.result != 'failure'")),
                "{job_name} must not use `!= 'failure'` for {gate}: a timeout \
                 reports as 'cancelled' and would pass that check (issue #812)"
            );
            assert!(
                effective.contains(gate),
                "{job_name} must declare {gate} in needs: (issue #812)"
            );
        }
    }
}

/// Issue #812: run 29767811026 reported `Test (ubuntu-latest)` as failed with
/// every test passing -- the suite finished 1.1 s before `timeout-minutes: 15`
/// killed the job. The budget must exceed the measured cost, and the step must
/// say so out loud before the margin is eaten again.
#[test]
fn test_job_budget_exceeds_the_measured_suite_cost_and_warns_before_it_is_eaten() {
    let workflow = release_workflow();
    let test_job = job_block(&workflow, "test");

    // Issue #1017: the cap has to clear the budget by more than the job's
    // unbudgeted setup -- checkout, disk cleanup, the data-file and self-AST
    // census gates and the doc tests measured 455s on run 31937348472 -- or the
    // job clock still wins and the overrun is reported as `cancelled`.
    assert!(
        test_job.contains("timeout-minutes: 35"),
        "every slice must retain a job budget above the measured 25min suite"
    );
    assert!(
        test_job.contains("TEST_BUDGET_SECONDS: 1200"),
        "the execution warning must leave setup and teardown headroom inside the job budget"
    );
    assert!(
        test_job.contains("scripts/run-with-budget-warning.sh"),
        "creeping back toward the cap must be visible in the run summary rather \
         than resurfacing as a mystery cancellation (issue #812)"
    );
}

/// Run 31428625331 passed all 2,629 tests on macOS, then exceeded the job
/// budget during cleanup. The focused data-integrity gates had already run the
/// same tests before the full suite, consuming more than 90 seconds twice.
#[test]
fn full_suite_does_not_repeat_focused_data_integrity_checks() {
    let workflow = release_workflow();
    let test_job = job_block(&workflow, "test");
    let full_suite = workflow_step_block(test_job, "Run tests");
    let macos = fs::read_to_string(format!(
        "{}/.github/workflows/macos-core-tests.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("macOS core workflow");
    let core_suite = workflow_step_block(&macos, "Run macOS platform tests");

    assert!(test_job.contains("cargo test --test unit data_files -- --nocapture"));
    assert!(test_job.contains("cargo test --test unit self_ast_census -- --nocapture"));
    // Issue #1055 moved the skip flags into the runner script the lane invokes.
    let runner = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/scripts/run-prebuilt-tests.sh"
    ))
    .expect("read the runner");
    assert!(full_suite.contains("run-prebuilt-tests.sh"));
    assert!(
        runner.contains("--skip data_files::") && runner.contains("--skip self_ast_census"),
        "the full suite must skip integrity groups already exercised by their focused gates"
    );
    assert!(
        core_suite.contains("test(data_files::)")
            && core_suite.contains("test(self_ast_census)")
            && core_suite.contains("test(specification::)"),
        "the macOS core shard must skip focused gates and the separately executed specification shard"
    );
}

/// Run 30087447926 exhausted `/` twice while the test job rebuilt the package
/// on top of its restored, target-heavy cache. The first attempt killed the
/// runner worker; the second could not create a rustc temp directory and made
/// the linker crash with SIGBUS.
#[test]
fn test_job_reclaims_runner_disk_before_restoring_the_target_cache() {
    let workflow = release_workflow();
    let test_job = job_block(&workflow, "test");
    let cleanup_name = "- name: Free up runner disk space";
    // Issue #1076 moved this to the shared action, so the ordering anchors on
    // its path. Same invariant: reclaim the disk before unpacking onto it.
    let cache_name = "- uses: ./.github/actions/cache-cargo-registry";

    assert!(
        test_job.contains(cleanup_name),
        "the test job must reclaim disposable hosted-runner SDKs before a \
         target-heavy build can exhaust the root filesystem"
    );

    let cleanup = workflow_step_block(test_job, "Free up runner disk space");
    assert!(
        cleanup.contains("run: bash scripts/free-runner-disk.sh"),
        "the test job must use the repository's established, observable disk cleanup"
    );
    assert!(
        test_job.find(cleanup_name) < test_job.find(cache_name),
        "runner cleanup must happen before restoring the multi-gigabyte target cache"
    );
}

/// Issue #812: nothing validated the pipeline definitions themselves, and
/// `cargo clippy` ran without `-D warnings` while every lint in `[lints.clippy]`
/// is set to `warn` -- so clippy printed findings and exited 0.
#[test]
fn lint_job_gates_on_workflow_shell_and_clippy_findings() {
    // Clippy and the shell lint are registered gates since issue #991.
    // actionlint left this job in issue #1076: as a bare binary it skips every
    // `run:` check and exits 0 when ShellCheck is absent, so it runs as the
    // Docker image in `workflows.yml` now. `issue_999` owns the assertion.
    let workflow = ci_surface();
    let lint = job_block(&workflow, "lint");

    assert!(
        lint.contains("cargo clippy --lib --bins --tests --all-features -- -D warnings"),
        "clippy must lint executable test targets and fail the job on findings"
    );
    assert!(
        lint.contains("cargo check --examples --all-features"),
        "examples must be compile-checked without linking every standalone binary (issue #534)"
    );
    assert!(
        !lint.contains("actionlint"),
        "workflow linting belongs to .github/workflows/workflows.yml since issue \
         #1076; a second copy here would drift from the Docker-image form that \
         guarantees ShellCheck is present"
    );
    assert!(
        lint.contains("scripts/lint-shell-scripts.sh"),
        "standalone shell scripts must be linted (issue #812)"
    );

    // Issue #977 moved the selector into a script; the empty-list guard is the
    // part that actually keeps the step from passing vacuously.
    let shell_lint = std::fs::read_to_string(format!(
        "{}/scripts/lint-shell-scripts.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("lint-shell-scripts.sh");
    assert!(shell_lint.contains("shellcheck --severity=warning"));
    assert!(
        shell_lint.contains("the lint selector is broken"),
        "an empty selector must fail rather than lint nothing (issue #812)"
    );
}

/// The real Agent CLI ships a network-backed `websearch` tool. A temporary
/// outage of that provider used to abort both meaning-detail scenarios before
/// Formal AI could observe the search result, making this repository's gate
/// depend on an unrelated hosted service. Keep the complete research recipe,
/// but execute its search and fetch through the repository-owned MCP fixture.
#[test]
fn meaning_detail_e2e_uses_the_local_research_fixture() {
    let workflow = release_workflow();
    let agent_e2e = job_block(&workflow, "test-agent-cli-e2e");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let harness = fs::read_to_string(format!(
        "{manifest_dir}/experiments/agent_cli_e2e/run_agent_cli.sh"
    ))
    .expect("Agent CLI E2E harness");
    let fixture = fs::read_to_string(format!(
        "{manifest_dir}/experiments/agent_cli_e2e/mock-meaning-mcp.mjs"
    ))
    .expect("meaning research MCP fixture");

    for step_name in [
        "Run agent CLI E2E — tomato meaning (search → fetch → write → verify)",
        "Run agent CLI E2E — potato meaning (different wording, same recipe)",
    ] {
        let step = workflow_step_block(agent_e2e, step_name);
        assert!(
            step.contains("RESEARCH_MCP_FIXTURE: experiments/agent_cli_e2e/mock-meaning-mcp.mjs"),
            "{step_name} must not depend on Agent's hosted websearch provider"
        );
    }
    assert!(harness.contains("config.tools = { websearch: false, webfetch: false }"));
    assert!(harness.contains("command: [\"node\", process.argv[2]]"));
    for lexeme in ["L170542.json", "L3784.json"] {
        assert!(
            fixture.contains(&format!("sourcePath(\"{lexeme}\")")),
            "fixture must serve the committed {lexeme} evidence"
        );
    }
}

/// A research harness that declares no MCP tool-call timeout makes the Agent
/// CLI compute its per-tool deadline as `NaN`, so a call the mock answers in
/// milliseconds aborts with `timed out after NaN seconds`. Run 32107664418
/// failed exactly that way: `run_issue_781.sh` ended after one fetch and
/// tripped its own `[ "$fetches" -ge 3 ]` assertion.
///
/// Pinned for `run_issue_781.sh` only, which is where the fault was observed.
/// `run_issue_687.sh` and `run_issue_771.sh` look similarly exposed but have
/// never produced a `NaN` deadline, and adding the same keys to them broke
/// them -- so they are left as they are rather than "fixed" against a fault
/// they do not have.
///
/// `mcp_defaults` is added for the Agent CLI only. `OpenCode` reads the same
/// file and validates it against its own schema, which rejects that key
/// outright ("Configuration is invalid ... Unrecognized key: `mcp_defaults`"),
/// so writing it unconditionally trades one red client for another.
#[test]
fn the_research_harness_declares_an_mcp_tool_call_timeout() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let harness = fs::read_to_string(format!(
        "{manifest_dir}/experiments/agent_cli_e2e/run_issue_781.sh"
    ))
    .expect("research E2E harness should be readable");

    assert!(
        harness.contains(r#""tool_call_timeout": 120000"#),
        "run_issue_781.sh must give its MCP server an explicit tool-call \
         timeout, or the Agent CLI computes it as NaN and aborts the call"
    );
    assert!(
        harness.contains("max_tool_call_timeout: 600000"),
        "run_issue_781.sh must declare mcp_defaults.max_tool_call_timeout"
    );
    assert!(
        harness.contains(r#"if [ "$for_client" = agent ]"#),
        "mcp_defaults must be written only for the Agent CLI: OpenCode reads \
         the same file and its schema rejects that key"
    );
}

/// Issue #534 forbids caching `target/` in CI, and the reason is disk rather
/// than taste: a hosted ubuntu runner ships ~14 GB free, seven jobs already run
/// `scripts/free-runner-disk.sh` to survive, and a job that exhausts `/` takes
/// the runner down with no failed step and no log to download. A cached target
/// tree measured 930 MB per job on top of that.
///
/// The guard read three hand-listed files, so `agentic-cli-matrix.yml` and
/// `external-benchmarks.yml` cached it unnoticed. It now reads every workflow;
/// this test pins that it does, because a policy enforced over part of the tree
/// reports compliance it has not checked.
#[test]
fn the_disk_policy_reads_every_workflow_not_a_hand_listed_few() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let guard = fs::read_to_string(format!("{manifest_dir}/scripts/check-disk-usage-policy.rs"))
        .expect("disk usage policy guard should be readable");

    assert!(
        guard.contains(r#"fs::read_dir(".github/workflows")"#),
        "the disk policy must sweep the workflow directory, or a workflow it \
         does not list can cache the target tree unnoticed"
    );

    let dir = format!("{manifest_dir}/.github/workflows");
    let mut checked = 0;
    for entry in fs::read_dir(&dir).expect("workflow directory should be readable") {
        let path = entry.expect("directory entry").path();
        if path.extension().is_none_or(|ext| ext != "yml") {
            continue;
        }
        let workflow = fs::read_to_string(&path).expect("workflow should be readable");
        assert!(
            !workflow.lines().any(|line| line.trim() == "target"),
            "{} caches the target tree, which issue #534 forbids",
            path.display()
        );
        checked += 1;
    }
    assert!(checked > 0, "the sweep must actually find workflows");
}

/// Agent can otherwise launch its hosted `opencode/big-pickle` summarizer
/// between tool turns. If that unrelated provider is unavailable, the client
/// exits before returning the tool result to Formal AI.
#[test]
fn agent_cli_e2e_disables_hosted_session_summarization() {
    let workflow = release_workflow();
    let agent_e2e = job_block(&workflow, "test-agent-cli-e2e");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let harness = fs::read_to_string(format!(
        "{manifest_dir}/experiments/agent_cli_e2e/run_agent_cli.sh"
    ))
    .expect("Agent CLI E2E harness");

    assert!(harness.contains(
        "--no-summarize-session \\\n    --compaction-model same \\\n    --model \"formal-ai/formal-ai\""
    ));
    assert!(agent_e2e.contains("LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION: \"false\""));

    for script in ["run_issue_687.sh", "run_issue_771.sh", "run_issue_781.sh"] {
        let research_harness =
            fs::read_to_string(format!("{manifest_dir}/experiments/agent_cli_e2e/{script}"))
                .expect("research E2E harness should be readable");

        assert!(
            research_harness.contains("mock-research-mcp.mjs")
                && research_harness.contains("\"websearch\": false")
                && research_harness.contains("\"webfetch\": false"),
            "{script} must disable Agent's hosted research tools"
        );
    }
}

#[test]
fn agent_cli_e2e_does_not_call_an_unrelated_summary_provider() {
    // Run 29911330673 completed the self-AST recipe and wrote its validated
    // artifact, then the external Agent CLI tried to summarize the session
    // through `opencode/big-pickle`. That unrelated provider was unavailable,
    // turning a successful formal-ai round-trip into exit code 1. Keep the
    // harness focused on the provider under test and preserve its strict
    // zero-exit assertion by disabling the optional post-run summary.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let harness = fs::read_to_string(format!(
        "{manifest_dir}/experiments/agent_cli_e2e/run_agent_cli.sh"
    ))
    .unwrap();

    assert_eq!(
        harness
            .matches("\n        --no-summarize-session \\\n")
            .count()
            + harness.matches("\n    --no-summarize-session \\\n").count(),
        2,
        "both the initial and resumed Agent CLI turns must disable summarization"
    );
}

#[test]
fn release_workflow_jobs_have_explicit_timeouts() {
    let workflow = release_workflow();
    let expected_timeouts = [
        ("detect-changes", 5),
        ("changelog", 10),
        // Issue #808: pull-request gates for the trailer invariant, the Docker
        // image and committed credentials.
        ("evidence-check", 10),
        ("docker-build", 60),
        ("secrets-scan", 10),
        ("version-check", 5),
        // Issue #1017: resolves the base-branch commit once so `lint`, `test`
        // and the macOS lane all merge the same one instead of each resolving
        // the tip at its own start time. A reusable workflow, so it owns its
        // own cap -- and so `release.yml` does not pay lines for it against the
        // 1500-line band this same file pins below.
        ("base", 0),
        // Issue #812: raised from 10; the job grew from ~3.3 to ~7.8 minutes.
        // Issue #1076 (D5): raised from 15. Over 400 `main` runs the worst
        // case was 12.7 min, 84.4% of that cap and trending up (6.0 -> 12.7
        // across the window) -- the deadline, not the backstop, and a cap kills
        // as `cancelled`. No single step dominates here, so the cap is the only
        // mechanism available; 25 puts the same worst case at 50.7%.
        ("lint", 25),
        // Issue #812: raised from 15 after run 29767811026 was killed 1.1 s
        // after the suite passed. See
        // `test_job_budget_exceeds_the_measured_suite_cost_and_warns_before_it_is_eaten`.
        // Issue #1012 partitions the macOS core suite so every slice retains
        // this baseline rather than extending a monolithic timeout.
        // Issue #1017 raised this from 25: 455s of the job runs outside the
        // budgeted step, so a 20-minute budget under a 25-minute cap could
        // never expire first.
        ("test", 35),
        // Issue #1014 compiles one nextest archive and fans it out to five
        // macOS runners. The reusable workflow owns both internal timeouts.
        ("macos-core-tests", 0),
        // Issue #896: raised from 10; the published web-search/web-capture
        // graphs moved the job from ~4-5 to 7.2 minutes, and a cold release
        // build after a Cargo.lock change hit the former cap.
        // Issue #1076 (D5): raised from 15. The run that missed its cargo
        // cache took 11.6 min, 77.0% of the cap; 20 puts it at 57.8%.
        ("build", 20),
        // Issue #1076 (D5): raised from 60. Worst case 50.6 min over 400
        // `main` runs, 84.4% of that cap -- and the 45-minute GHCR budget that
        // owns the real deadline could never have fired inside it, since the
        // rest of the job costs ~18 min. A budget the cap pre-empts is not a
        // budget: the run still ends `cancelled`.
        ("auto-release", 90),
        // Same image, same steps, same budgets, same backstop (issue #1076).
        ("manual-release", 90),
        ("changelog-pr", 10),
        ("test-e2e-local", 40),
        // Issue #538: real Agent CLI ↔ formal-ai OpenAI-compatible round-trip.
        // Boots `formal-ai serve`, drives it with `@link-assistant/agent`, and
        // asserts the CLI writes the enriched meaning file. The extra headroom
        // over test-e2e-local absorbs a cold cargo build of the release binary.
        // Raised from 20 (PR #965 review): 16m16s and 17m30s green left ~2
        // minutes of headroom, and run 31097339962 tipped over into a
        // *cancelled* job that looked like a regression but was only variance.
        // Raised from 32 (issue #1069): 19m54s green, of which the two
        // computer-use steps cost 5m32s, is 39m24s at their 900s+600s budgets.
        ("test-agent-cli-e2e", 45),
        // Issue #1012: the shared release binary is built once before the seven
        // Box image legs, avoiding seven identical cache restores and builds.
        ("build-artifacts", 20),
        // Issue #932: per-language matrix leg that pulls one link-foundation/box
        // image, generates the project from solver answers and runs the
        // language's traditional init commands inside it. The budget covers a
        // cold release build plus the image pull.
        ("box-language-projects", 30),
        // deploy-pages also runs `cargo doc` for the /docs/api reference (issue
        // #479), which compiles the dependency tree on a cold cargo cache.
        // Raised from 20 (PR #965 review): the budget also has to cover the
        // GitHub Pages deployment queue, which is not part of the build and can
        // stall for many minutes — see
        // `pages_deploy_uses_the_actions_maximum_supported_wait`.
        ("deploy-pages", 35),
        ("test-e2e-pages", 15),
        // Issue #977: the terminal gate that turns a silently-`cancelled` run
        // (the shape a `timeout-minutes` kill takes) into a red failure.
        ("pipeline-status", 5),
    ];

    let actual_jobs = workflow_job_names(&workflow);
    let expected_jobs = expected_timeouts
        .iter()
        .map(|(job_name, _)| *job_name)
        .collect::<Vec<_>>();
    assert_eq!(actual_jobs, expected_jobs);

    for (job_name, timeout_minutes) in expected_timeouts {
        let job = job_block(&workflow, job_name);
        if timeout_minutes == 0 {
            // A reusable call declares no cap of its own; the called workflow's
            // jobs own them. Read whichever workflow this job actually calls,
            // so a second delegating job cannot be checked against the first
            // one's file (issue #1017 added `base`).
            let called = job
                .lines()
                .find_map(|line| line.trim().strip_prefix("uses: ./"))
                .unwrap_or_else(|| {
                    panic!("{job_name} is listed as delegating but calls no reusable workflow")
                })
                .trim();
            let reusable = fs::read_to_string(format!("{}/{called}", env!("CARGO_MANIFEST_DIR")))
                .unwrap_or_else(|error| panic!("read {called}: {error}"));
            for inner_job in workflow_job_names(&reusable) {
                assert!(
                    job_block(&reusable, inner_job).contains("    timeout-minutes:"),
                    "{inner_job} should declare an explicit timeout"
                );
            }
            continue;
        }
        let expected = format!("    timeout-minutes: {timeout_minutes}\n");
        assert!(
            job.contains(&expected),
            "{job_name} should declare {expected:?}"
        );
    }
}

/// Issue #479: the docs hub links to a Rust API reference at `/docs/api/`. The
/// deploy-pages job generates it with `cargo doc` and copies it into the Pages
/// artifact — and the copy must run *after* stamping (rustdoc HTML carries no
/// version placeholders, so copying post-stamp keeps the large generated tree
/// out of the placeholder scan).
#[test]
fn pages_deploy_generates_api_docs_and_copies_them_after_stamping() {
    let workflow = release_workflow();
    let deploy = job_block(&workflow, "deploy-pages");

    assert!(
        deploy.contains("bash scripts/build-rust-api-docs.sh"),
        "deploy-pages should invoke the API-docs builder"
    );
    assert!(
        deploy.contains("cp -R target/doc/. src/web/docs/api/"),
        "deploy-pages should copy the generated docs into src/web/docs/api/"
    );

    let stamp_pos = deploy
        .find("Stamp GitHub Pages artifact")
        .expect("deploy-pages should stamp the Pages artifact");
    let copy_pos = deploy
        .find("Copy Rust API docs into the Pages artifact")
        .expect("deploy-pages should copy the API docs");
    assert!(
        stamp_pos < copy_pos,
        "the API-docs copy must run after the stamp step so rustdoc HTML is not scanned for placeholders"
    );

    // cargo doc emits the crate root under target/doc/formal_ai/ (lib name
    // formal_ai); a redirect at the doc root keeps /docs/api/ from 404ing.
    let docs_script = std::fs::read_to_string(format!(
        "{}/scripts/build-rust-api-docs.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("build-rust-api-docs.sh");
    assert!(
        docs_script.contains("cargo doc --no-deps --lib"),
        "the API-docs builder should run cargo doc"
    );
    assert!(
        docs_script.contains("url=formal_ai/index.html"),
        "a redirect should point /docs/api/ at the crate root"
    );
}
