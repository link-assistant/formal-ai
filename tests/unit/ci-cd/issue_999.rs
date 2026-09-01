//! Regression gates for issue #999's CI/CD false-positive and false-negative audit.

use std::fs;

use super::workflow_fixtures::{desktop_release_workflow, job_block, release_workflow};

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
        .replace("\r\n", "\n")
}

#[test]
fn macos_tests_are_partitioned_without_raising_the_failed_budget() {
    let workflow = release_workflow();
    let test = job_block(&workflow, "test");
    let macos_call = job_block(&workflow, "macos-core-tests");
    let macos = repository_file(".github/workflows/macos-core-tests.yml");

    assert!(test.contains("name: Test (${{ matrix.os }} / ${{ matrix.test-suite }})"));
    assert!(test.contains("test-suite: full"));
    // Issue #1059: the lane runs 139 platform-sensitive tests on one runner
    // rather than the whole suite across eight.
    assert!(macos.contains("- { partition: 1 }"));
    assert!(test.contains("test-suite: specification"));
    assert_eq!(test.matches("os: macos-15-intel").count(), 1);
    assert!(macos_call.contains("uses: ./.github/workflows/macos-core-tests.yml"));
    // Issue #1055 raised the archive cap to 35m; see issue_1012 for the math.
    assert!(macos.contains("timeout-minutes: 35"));
    assert!(macos.contains("cargo nextest archive"));
    assert!(macos.contains("cargo nextest run --archive-file"));
    assert!(macos.contains("--macos-platform"));
    assert!(test.contains("cargo test --test unit --all-features --verbose specification::"));
    assert!(macos.contains("test(specification::)"));
    assert!(test.contains("matrix.test-suite == 'full'"));
    assert!(test.contains("matrix.test-suite == 'specification'"));
}

#[test]
fn write_jobs_queue_together_without_workflow_level_cancellation() {
    let release = release_workflow();
    let desktop = desktop_release_workflow();
    let external = repository_file(".github/workflows/external-benchmarks.yml");
    let shared = concat!(
        "    concurrency:\n",
        "      group: formal-ai-repository-writes\n",
        "      queue: max\n",
    );

    for workflow in [&release, &desktop, &external] {
        let header = workflow.split("\njobs:\n").next().unwrap();
        assert!(!header.contains("\nconcurrency:\n"));
    }
    for job_name in [
        "auto-release",
        "manual-release",
        "changelog-pr",
        "deploy-pages",
    ] {
        assert!(job_block(&release, job_name).contains(shared), "{job_name}");
    }
    let desktop_build = job_block(&desktop, "build");
    assert!(desktop_build.contains("group: desktop-release-${{ github.event.pull_request.number || needs.resolve.outputs.tag }}-${{ matrix.label }}"));
    assert!(desktop_build.contains("queue: max"));
    let vscode = job_block(&desktop, "vscode");
    assert!(vscode.contains("group: desktop-release-${{ github.event.pull_request.number || needs.resolve.outputs.tag }}-vscode"));
    assert!(vscode.contains("queue: max"));
    assert!(job_block(&desktop, "finalize").contains(shared));
    assert!(job_block(&external, "external-benchmarks").contains(shared));
}

#[test]
fn known_advisories_do_not_emit_false_warning_annotations() {
    let cache_budget = repository_file("scripts/check-cache-budget.rs");
    let self_hosting = repository_file("scripts/self-hosting-metric.rs");
    let deploy = job_block(&release_workflow(), "deploy-pages").to_owned();

    assert!(!cache_budget.contains("::warning file={}::Closure-driven cache bucket"));
    assert!(cache_budget.contains("::notice file={}::Closure-driven cache bucket"));
    assert!(!self_hosting.contains("::warning title=Self-hosting ratchet fell"));
    assert!(self_hosting.contains("::notice title=Self-hosting ratchet fell"));
    assert!(deploy.contains("timeout: 600000"));
    assert!(!deploy.contains("timeout: 1200000"));
}

#[test]
fn current_template_security_and_link_gates_are_present() {
    let security = repository_file(".github/workflows/security.yml");
    assert!(security.contains("language: [rust, actions]"));
    assert!(security.contains("github/codeql-action/init@v4"));
    assert!(security.contains("github/codeql-action/analyze@v4"));
    assert!(security.contains("actions/dependency-review-action@v5"));
    assert!(security.contains("fail-on-severity: high"));

    let links = repository_file(".github/workflows/links.yml");
    assert!(links.contains("lycheeverse/lychee-action@v2"));
    assert!(links.contains("--host-concurrency 2"));
    assert!(links.contains("formal-ai-link-checker/1.0"));
    assert!(links.contains("sed 's|https://…|https://example.com/placeholder|g' CHANGELOG.md"));
    assert!(links.contains("--exclude-path '^CHANGELOG\\.md$'"));
    assert!(links.contains("> LYCHEE_CHANGELOG.md"));
    assert!(links.contains("--exclude-path docs/case-studies"));
    assert!(links.contains("--exclude-path dev/log"));
    assert!(links.contains("node scripts/check-web-archive.mjs"));
    // Issue #1017 narrowed this from `always()`: a cancelled link check has no
    // verdict to report, so it must not append a "broken links" error to a run
    // that never finished checking them.
    assert!(links.contains("if: ${{ !cancelled() && steps.lychee.outputs.exit_code != 0 }}"));
    assert!(!links.contains("steps.webarchive.outputs.all_archived != 'true'"));
    assert!(
        repository_file("scripts/check-web-archive.mjs").contains("archive.org/wayback/available")
    );
}

#[test]
fn actionlint_tracks_githubs_queue_schema_without_hiding_other_errors() {
    let lint = job_block(&release_workflow(), "lint").to_owned();
    assert!(lint.contains("ACTIONLINT_VERSION: 1.7.12"));
    assert!(lint.contains("rhysd/actionlint/issues/657"));
    assert!(lint.contains("unexpected key \"queue\" for \"concurrency\" section"));
    assert!(!lint.contains("actionlint -ignore '.*'"));
}

#[test]
fn warning_band_files_are_small_and_split_responses_cover_the_registry() {
    for (path, warning_limit) in [
        // Issue #921 added the Hive Mind full-circle gate (a two-step install
        // and run in `test-agent-cli-e2e`), which is this pull request's
        // deliverable and cannot live anywhere cheaper: extracting it to a
        // reusable workflow would duplicate the release build the E2E job
        // already produces, and `tests/unit/issue_921.rs` pins both step names
        // in this file. Main left three lines of headroom at 1497, so the band
        // moves by the gate's real cost and no further.
        (".github/workflows/release.yml", 1_510),
        ("src/intent_formalization.rs", 900),
        ("src/agentic_coding/general_planner.rs", 900),
        ("src/web/worker/formal_ai_worker_20.js", 1_400),
        ("data/seed/multilingual-responses-agentic.lino", 1_400),
        ("data/seed/multilingual-responses-agentic-tools.lino", 1_400),
    ] {
        let lines = repository_file(path).lines().count();
        assert!(
            lines <= warning_limit,
            "{path} has {lines} lines; warning threshold is {warning_limit}"
        );
    }

    for language in formal_ai::language::registered_languages() {
        for intent in [
            "coding_workspace_effect_observed",
            "coding_workspace_written_unverified",
            "coding_workspace_verification_failed",
            "coding_repository_source_observation",
            "coding_repository_result_observation",
            "tool_result_failed_exit_code",
            // Issues #824 and #944 added the verified-mutating-action pair to
            // the same split file, for the same reason the four above are
            // there: they are what the workspace tools observed, not what the
            // planner said.
            "mutating_action_completed",
            "mutating_action_blocked",
        ] {
            assert!(
                formal_ai::seed::response_for(intent, language.slug()).is_some(),
                "{intent} must remain directly available in {} after the seed split",
                language.slug()
            );
        }
    }

    // Issue #991 made `data/meta/seed-registry.lino` the one inventory and
    // generates the rest from it, so the split file is declared once and
    // reaches every production surface by construction.
    assert!(
        repository_file("data/meta/seed-registry.lino")
            .contains("seed multilingual-responses-agentic-tools\n"),
        "the seed registry must declare the split response file"
    );
    for generated in [
        "src/seed/embedded_registry.rs",
        "tests/source/seed/embedded.rs",
        "src/web/seed-files.js",
    ] {
        assert!(
            repository_file(generated).contains("multilingual-responses-agentic-tools.lino"),
            "{generated} must carry the split response file; regenerate it with \
             `rust-script scripts/generate-seed-registry.rs --write`"
        );
    }
}
