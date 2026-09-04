//! Regression coverage for issue #1014's complete CI diagnostic audit.

use std::fs;
use std::path::{Path, PathBuf};

use super::workflow_fixtures::{job_block, release_workflow};

fn repository_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(repository_path(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
        .replace("\r\n", "\n")
}

fn files_below(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("read {}: {error}", root.display()))
    {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            files_below(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn direct_files_with(root: &Path, prefix: &str, suffix: &str) -> Vec<PathBuf> {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("read {}: {error}", root.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
        })
        .collect()
}

#[test]
fn every_named_baseline_and_initial_pull_request_run_has_preserved_evidence() {
    let evidence = repository_path("dev/log/issues/1014/pulls/1015");
    let logs = evidence.join("ci-logs");
    let branch_logs = logs.join("branch-initial");
    let raw = evidence.join("raw-data");

    assert_eq!(direct_files_with(&logs, "run-", ".log").len(), 13);
    assert_eq!(direct_files_with(&branch_logs, "branch-", ".log").len(), 5);
    assert_eq!(direct_files_with(&raw, "run-", "-jobs.json").len(), 12);
    assert_eq!(direct_files_with(&raw, "run-", "-artifacts.json").len(), 12);
    assert_eq!(
        direct_files_with(&raw, "branch-run-", "-jobs.json").len(),
        5
    );
    assert_eq!(
        direct_files_with(&raw, "branch-run-", "-artifacts.json").len(),
        5
    );
    assert_eq!(
        direct_files_with(&raw.join("check-annotations"), "", ".json").len(),
        66
    );
}

#[test]
fn an_ineligible_cycle_blocks_the_release_without_weakening_the_manual_gate() {
    let workflow = release_workflow();
    let automatic = job_block(&workflow, "auto-release");
    let manual = job_block(&workflow, "manual-release");
    let policy = repository_file("scripts/self-development-loop.rs");
    let preflight = repository_file("scripts/check-self-development-release.rs");

    assert!(policy.contains("SelfDevelopmentReleaseStatus"));
    assert!(policy.contains("Blocked"));
    assert!(preflight.contains("set_output(\"should_release\", \"false\")"));
    // Issue #1066: work is not deferred in this repository, however hard it is.
    // An ineligible cycle fails the preflight from the first push, so there is
    // no state in which the pipeline reports success while publishing nothing.
    assert!(
        preflight.contains("SelfDevelopmentReleaseStatus::Blocked"),
        "the preflight must classify an ineligible cycle as blocked"
    );
    assert!(
        preflight.contains("return Err(reason)"),
        "a blocked cycle must fail the preflight rather than report success"
    );
    for forbidden in [
        "Deferred",
        "Overdue",
        "DEFERRAL_BUDGET_DAYS",
        "DEFERRAL_BUDGET_FRAGMENTS",
    ] {
        assert!(
            !policy.contains(forbidden) && !preflight.contains(forbidden),
            "the release path must carry no deferral machinery, found `{forbidden}`"
        );
    }
    assert!(
        !preflight.contains("::notice title=Release deferred::"),
        "an ineligible cycle must not be downgraded to a notice"
    );
    assert!(automatic.contains("id: release_gate"));
    assert!(automatic.contains("steps.release_gate.outputs.should_release == 'true'"));
    assert!(manual.contains("scripts/version-and-commit.rs"));
    assert!(
        repository_file("scripts/version-and-commit.rs")
            .contains("ensure_self_development_release")
    );
}

#[test]
fn macos_core_shards_reuse_one_nextest_archive() {
    let workflow = release_workflow();
    let regular_tests = job_block(&workflow, "test");
    let macos_call = job_block(&workflow, "macos-core-tests");
    let macos = repository_file(".github/workflows/macos-core-tests.yml");

    assert!(macos_call.contains("uses: ./.github/workflows/macos-core-tests.yml"));
    assert_eq!(regular_tests.matches("os: macos-15-intel").count(), 1);
    assert!(!regular_tests.contains("test-suite: core-"));
    // Issue #1059: one runner. What this test pins is that the lane reuses the
    // archive instead of compiling, which is unchanged.
    assert_eq!(macos.matches("- { partition:").count(), 1);
    assert_eq!(macos.matches("cargo nextest archive").count(), 1);
    assert!(macos.contains("actions/upload-artifact@v7"));
    // Issue #1039 moved the download to `scripts/download-artifact-with-retry.sh`
    // so a transient storage failure retries instead of reddening a slice that
    // never ran a test. What this test pins is that the slices *reuse the one
    // archive* rather than each building their own -- the mechanism that
    // fetches it is free to change, and the retry is covered by issue #1039.
    assert!(macos.contains("scripts/download-artifact-with-retry.sh"));
    assert!(macos.contains("cargo nextest run --archive-file"));
    assert!(macos.contains("--extract-to \"$GITHUB_WORKSPACE\""));
    assert!(macos.contains("--archive-file"));
    assert!(macos.contains("git rev-parse 'HEAD^{tree}'"));
    assert!(macos.contains("macos-core-tests/tree"));
    assert!(repository_path("experiments/issue_1014_nextest_archive/run.sh").is_file());
}

#[test]
fn unix_agent_runner_uses_command_streams_exact_argv_api() {
    let cargo = repository_file("Cargo.toml");
    let runner = repository_file("src/orchestration/runner.rs");

    assert!(cargo.contains("command-stream = \"=0.16.0\""));
    assert!(runner.contains("command_stream::StreamingRunner::from_argv("));
    assert!(!runner.contains("command_stream::quote(&part)"));
}

#[test]
fn gemini_e2e_declares_true_color_and_keeps_mutable_home_outside_the_scanned_project() {
    let harness = repository_file("experiments/agent_cli_e2e/run_issue_907.sh");

    assert!(harness.contains("export COLORTERM=truecolor"));
    assert!(harness.contains("PROJECT_DIR=\"$WORKDIR/project\""));
    assert!(harness.contains("HOME=\"$WORKDIR/home\""));
    assert!(harness.contains("cd \"$PROJECT_DIR\""));
}

#[test]
fn captured_dependency_manifests_cannot_be_discovered_as_live_projects() {
    let evidence_roots = [
        repository_path("dev/log"),
        repository_path("docs/case-studies"),
    ];
    let scanner_names = [
        "Cargo.toml",
        "Cargo.lock",
        "package.json",
        "package-lock.json",
        "pyproject.toml",
        "requirements.txt",
        "poetry.lock",
        "uv.lock",
    ];
    let mut live_manifests = Vec::new();
    for root in evidence_roots {
        let mut files = Vec::new();
        files_below(&root, &mut files);
        live_manifests.extend(files.iter().filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .filter(|name| scanner_names.contains(name))
                .map(|_| {
                    path.strip_prefix(repository_path(""))
                        .unwrap()
                        .display()
                        .to_string()
                })
        }));
    }

    assert!(
        live_manifests.is_empty(),
        "captured evidence must use a .snapshot suffix: {live_manifests:#?}"
    );
}

#[test]
fn every_javascript_lock_surface_has_one_explicit_advisory_gate() {
    let gate = repository_file("data/meta/ci-gates/check-javascript-dependencies.lino");
    let script = repository_file("scripts/check-javascript-dependencies.sh");

    assert!(gate.contains("stage web"));
    assert!(gate.contains("scripts/check-javascript-dependencies.sh"));
    assert!(script.contains("git ls-files"));
    assert!(script.contains("$NF == name"));
    assert!(script.contains("queue_locks \"bun.lock\" bun audit"));
    assert!(script.contains("queue_locks \"package-lock.json\" npm audit"));
    // Queueing an audit is not running one. Both surfaces are audited concurrently,
    // so the gate's verdict only exists once every queued audit has been waited on.
    assert!(script.contains("wait_for_audits"));
    assert!(script.contains("bun audit --audit-level=moderate"));
    assert!(script.contains("npm audit --package-lock-only --audit-level=moderate"));
    for lock in [
        "bun.lock",
        "experiments/agent_cli_e2e/issue_819_tui/bun.lock",
        "tests/e2e/package-lock.json",
        "desktop/package-lock.json",
        "vscode/package-lock.json",
    ] {
        assert!(repository_path(lock).is_file(), "missing live lock {lock}");
    }
    assert!(!repository_file("package.json").contains("\"dompurify\": \"3.4.12\""));
}

#[test]
fn javascript_installs_apply_a_scoped_lifecycle_policy() {
    let workflow = release_workflow();
    for line in workflow.lines().filter(|line| line.contains("npm ci")) {
        assert!(
            line.contains("--no-audit") && line.contains("--no-fund"),
            "{line}"
        );
    }

    for path in [
        ".github/workflows/release.yml",
        ".github/workflows/proactive-failure-report-e2e.yml",
        "experiments/agentic_cli_matrix/install_client.sh",
    ] {
        let source = repository_file(path);
        for line in source.lines().filter(|line| {
            let command = line.trim_start();
            command.starts_with("bun add -g") || command.starts_with("run: bun add -g")
        }) {
            assert!(
                line.contains("--ignore-scripts") || line.contains("--trust"),
                "{path}: {line}"
            );
            if line.contains("--trust") {
                assert!(
                    (path == ".github/workflows/release.yml"
                        && line.contains("opencode-ai@1.18.25"))
                        || (path == "experiments/agentic_cli_matrix/install_client.sh"
                            && line.contains("\"$spec\"")),
                    "{path}: {line}"
                );
            }
        }
    }

    assert!(workflow.contains("bun add -g --trust opencode-ai@1.18.25"));
    let installer = repository_file("experiments/agentic_cli_matrix/install_client.sh");
    assert!(installer.contains("[ \"$CLIENT\" = opencode ]"));
    assert!(installer.contains("bun add -g --trust \"$spec\""));
    assert!(
        repository_file("experiments/agentic_cli_matrix/clients.lock")
            .contains("opencode-ai@1.18.25")
    );
}

#[test]
fn vscode_dependency_graph_test_runs_after_its_dependencies_are_installed() {
    let package = repository_file("vscode/package.json");
    let workflow = repository_file(".github/workflows/desktop-release.yml");
    let test_script = package
        .split("\"test\": \"")
        .nth(1)
        .and_then(|tail| tail.split('"').next())
        .expect("VS Code source-only test script");

    assert!(
        !test_script.contains("bundle-web-tools.test.mjs"),
        "the lint-stage test must remain runnable from committed source without npm install"
    );
    assert!(
        package.contains("\"test:package\": \"node --test scripts/bundle-web-tools.test.mjs\"")
    );
    let install = workflow
        .find("scripts/install-node-dependencies.sh vscode")
        .expect("locked VS Code dependency install");
    let dependency_test = workflow
        .find("npm run test:package")
        .expect("dependency-backed VS Code package test");
    let package_vsix = workflow
        .find("npm run package")
        .expect("VSIX package command");
    assert!(
        install < dependency_test && dependency_test < package_vsix,
        "the real dependency graph must be tested after install and before packaging"
    );
}

#[test]
fn diagnostic_ledger_classifies_every_observed_signal_without_weakening_failures() {
    let ledger = repository_file("dev/log/issues/1014/pulls/1015/README.md");
    for signal in [
        "Auto Release error",
        "Pipeline Status failure",
        "Core test slice took",
        "npm reports two high advisories",
        "Blocked 2 postinstalls",
        "Gemini true-color warnings",
        "projects.json.lock",
        "Dependency Graph workflow",
        "ca-certificates.crt",
        "CodeQL sources",
        "Cache restore misses",
        "npm 11 refuses",
    ] {
        assert!(ledger.contains(signal), "missing diagnostic {signal:?}");
    }
    assert!(ledger.contains("No suppression or aggregator weakening"));
}

#[test]
fn template_comparison_and_upstream_reports_cover_each_ecosystem() {
    let raw = repository_path("dev/log/issues/1014/pulls/1015/raw-data");
    for source in [
        "references/rust-ai-driven-development-pipeline-template-tree.json",
        "references/js-ai-driven-development-pipeline-template-tree.json",
        "references/python-ai-driven-development-pipeline-template-tree.json",
        "references/CI-CD-BEST-PRACTICES.md",
        "upstream-rust-template-132.json",
        "upstream-js-template-134.json",
        "upstream-python-template-58.json",
        "upstream-gemini-28826.json",
        "upstream-web-capture-153.json",
        "upstream-html-to-markdown-459.json",
        "related/playwright-33031.json",
        "related/web-capture-154.json",
    ] {
        assert!(
            raw.join(source).is_file(),
            "missing research evidence {source}"
        );
    }

    let reports = repository_path("dev/log/issues/1014/pulls/1015/upstream-reports");
    let report_files = direct_files_with(&reports, "", ".md");
    assert_eq!(report_files.len(), 8);
    for report in report_files {
        let body = fs::read_to_string(&report).expect("upstream report body");
        if report
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("-follow-up.md"))
        {
            assert!(body.contains("validated downstream workaround"));
            assert!(body.contains("issues/154"));
            continue;
        }
        assert!(
            body.contains("## Reproduction") || body.contains("## Minimal reproduction"),
            "{} lacks a reproduction section",
            report.display()
        );
        assert!(
            body.contains("## Workaround"),
            "{} lacks ## Workaround",
            report.display()
        );
        assert!(
            body.contains("## Suggested code fix") || body.contains("## Suggested source fix"),
            "{} lacks a suggested code fix",
            report.display()
        );
    }
}

#[test]
fn tests_first_record_contains_the_failing_and_passing_composed_regression() {
    let red = repository_file("dev/log/issues/1014/pulls/1015/local-tests/regression-red.log");
    let green = repository_file("dev/log/issues/1014/pulls/1015/local-tests/regression-green.log");
    assert!(red.contains("test result: FAILED. 0 passed; 7 failed"));
    assert!(green.contains("test result: ok. 7 passed; 0 failed"));
}

#[test]
fn issue_and_pull_request_delivery_documents_target_the_prepared_pull_request() {
    for path in [
        "dev/log/issues/1014/pulls/1015/README.md",
        "docs/case-studies/issue-1014/README.md",
        "docs/case-studies/pull-request-1015/README.md",
    ] {
        let document = repository_file(path);
        for marker in ["#1014", "#1015", "test", "evidence"] {
            assert!(document.contains(marker), "{path} is missing {marker:?}");
        }
    }

    // The changelog entry is the fourth delivery document, but a fragment does
    // not survive the release that ships it: v0.346.0 (c11b23d34) consumed
    // `changelog.d/20260815_160000_issue_1014_ci_diagnostic_audit.md` and this
    // test failed on every run afterwards. Follow the entry across its
    // lifecycle the way `docs_requirements_issue_656` does -- a fragment before
    // release, a CHANGELOG.md section after -- so the markers stay pinned
    // either way.
    let fragment = repository_path("changelog.d/20260815_160000_issue_1014_ci_diagnostic_audit.md");
    let (source, changelog) = if fragment.is_file() {
        (
            "changelog.d/20260815_160000_issue_1014_ci_diagnostic_audit.md",
            repository_file("changelog.d/20260815_160000_issue_1014_ci_diagnostic_audit.md"),
        )
    } else {
        ("CHANGELOG.md", repository_file("CHANGELOG.md"))
    };
    for marker in ["#1014", "#1015", "test", "evidence"] {
        assert!(
            changelog.contains(marker),
            "the issue #1014 changelog entry in {source} is missing {marker:?}"
        );
    }
}

#[test]
fn whole_issue_contract_composes_all_nine_requirements() {
    let requirements = repository_file("REQUIREMENTS.md");
    for requirement in 1..=9 {
        assert!(requirements.contains(&format!("R1014-{requirement}")));
    }
    for implementation in [
        "scripts/check-self-development-release.rs",
        ".github/workflows/macos-core-tests.yml",
        "scripts/check-javascript-dependencies.sh",
        "experiments/agent_cli_e2e/run_issue_907.sh",
    ] {
        assert!(
            repository_path(implementation).is_file(),
            "missing {implementation}"
        );
    }
}
