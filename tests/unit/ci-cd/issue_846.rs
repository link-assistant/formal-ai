//! Regression coverage for excluded-only CI changes on issue #846.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn job<'a>(workflow: &'a str, name: &str, next: &str) -> &'a str {
    workflow
        .split(&format!("  {name}:\n"))
        .nth(1)
        .and_then(|tail| tail.split(&format!("\n  {next}:\n")).next())
        .unwrap_or_else(|| panic!("missing {name} job"))
}

#[test]
fn change_gated_jobs_do_not_bypass_detection_on_pushes() {
    let workflow = repository_file(".github/workflows/release.yml");

    // The coverage job moved to .github/workflows/coverage.yml for issue #895;
    // `coverage_workflow_keeps_the_timeout_and_change_gating_contract` in
    // workflow_coverage.rs asserts this same property for it there.
    for (name, next) in [
        ("secrets-scan", "version-check"),
        ("lint", "test"),
        ("test", "build"),
        ("test-e2e-local", "test-agent-cli-e2e"),
        ("test-agent-cli-e2e", "deploy-pages"),
    ] {
        let condition = job(&workflow, name, next)
            .split("    steps:\n")
            .next()
            .expect("job preamble");
        assert!(
            !condition.contains("github.event_name == 'push'"),
            "{name} must let detect-changes govern pushes"
        );
        assert!(
            condition.contains("github.event_name == 'workflow_dispatch'"),
            "{name} must remain manually runnable"
        );
    }
}

#[test]
fn ignored_directories_are_authoritative_for_every_change_output() {
    let detector = repository_file("scripts/detect-code-changes.rs");

    assert!(detector.contains(
        "const CI_IGNORED_PATH_PREFIXES: &[&str] = \
         &[\"experiments/\", \"dev/log/\", \"docs/case-studies/\"];"
    ));
    assert!(detector.contains(
        "let relevant_files: Vec<&String> = changed_files\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}.iter()\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}.filter(|file| !is_ignored_by_ci(file))"
    ));
    assert!(
        !detector.contains("set_output(\"mjs-changed\""),
        "the unused mjs-changed output must be removed"
    );
}

#[test]
fn excluded_only_change_matrix_is_covered_for_pushes_and_pull_requests() {
    let detector = repository_file("scripts/detect-code-changes.rs");

    for path in [
        "experiments/repro.rs",
        "experiments/README.md",
        "experiments/repro.mjs",
        "dev/log/run.rs",
        "docs/case-studies/issue-846/repro.rs",
    ] {
        assert!(
            detector.contains(&format!("\"{path}\"")),
            "missing ignored-path regression fixture for {path}"
        );
    }
    assert!(detector.contains("for event_name in [\"push\", \"pull_request\"]"));
}

#[test]
fn honest_manual_contributions_are_not_rejected_by_the_self_hosting_job() {
    let workflow = repository_file(".github/workflows/release.yml");
    let evidence_job = job(&workflow, "evidence-check", "docker-build");

    assert!(
        evidence_job.contains("self-hosting-metric.rs --since"),
        "the job must still validate every claimed Formal-AI session and evidence file"
    );
    assert!(
        !evidence_job.contains("--check-ratchet"),
        "CONTRIBUTING.md permits an honest 0% release and forbids adding Formal-AI trailers \
         to manual work, so the evidence job must not reject unattributed contributions"
    );
}

#[test]
fn case_study_preserves_the_incident_and_complete_template_audit() {
    let readme = repository_file("docs/case-studies/issue-846/README.md");
    for required in [
        "30118611467",
        "ff38e2ab221ef27df7ab4ecc779b9c7293cd7a11",
        "js-ai-driven-development-pipeline-template/issues/113",
        "rust-ai-driven-development-pipeline-template/issues/109",
        "python-ai-driven-development-pipeline-template/issues/40",
        "csharp-ai-driven-development-pipeline-template/issues/40",
    ] {
        assert!(
            readme.contains(required),
            "case study must record {required}"
        );
    }

    let root = env!("CARGO_MANIFEST_DIR");
    for path in [
        "docs/case-studies/issue-846/requirements.md",
        "docs/case-studies/issue-846/solution-plans.md",
        "docs/case-studies/issue-846/raw-data/online-research.md",
        "docs/case-studies/issue-846/raw-data/run-30118611467.json",
        "docs/case-studies/issue-846/raw-data/ci-logs/run-30118611467.log",
    ] {
        assert!(
            fs::metadata(format!("{root}/{path}")).is_ok(),
            "missing preserved evidence {path}"
        );
    }

    for template in [
        "js-ai-driven-development-pipeline-template",
        "rust-ai-driven-development-pipeline-template",
        "python-ai-driven-development-pipeline-template",
        "csharp-ai-driven-development-pipeline-template",
    ] {
        for artifact in [
            "revision.txt",
            "ci-file-tree.txt",
            "relevant-patterns.txt",
            "reported-issue.json",
        ] {
            let path =
                format!("docs/case-studies/issue-846/raw-data/templates/{template}/{artifact}");
            assert!(
                fs::metadata(format!("{root}/{path}")).is_ok(),
                "missing template audit artifact {path}"
            );
        }
    }
}
