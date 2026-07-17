//! Regression coverage for the CI/CD production-readiness audit in issue #739.

use std::fs;

fn release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("release workflow should be readable")
}

fn job_block<'a>(workflow: &'a str, job: &str, next_job: &str) -> &'a str {
    workflow
        .split_once(&format!("  {job}:\n"))
        .unwrap_or_else(|| panic!("missing {job} job"))
        .1
        .split_once(&format!("\n  {next_job}:\n"))
        .unwrap_or_else(|| panic!("missing {next_job} job after {job}"))
        .0
}

#[test]
fn rustdoc_is_a_pre_release_pull_request_gate() {
    let workflow = release_workflow();
    let lint = job_block(&workflow, "lint", "test");

    assert!(lint.contains("RUSTDOCFLAGS: -D warnings"));
    assert!(lint.contains("cargo doc --no-deps --lib"));
    assert!(
        workflow.find("cargo doc --no-deps --lib").unwrap() < workflow.find("  build:\n").unwrap(),
        "the first fail-closed documentation build must run before packaging or release"
    );
}

#[test]
fn production_workflow_does_not_brand_the_web_app_as_a_demo() {
    let workflow = release_workflow();

    assert!(!workflow.to_ascii_lowercase().contains("demo"));
    assert!(workflow.contains("  deploy-pages:\n"));
    assert!(workflow.contains("name: Deploy Web App to GitHub Pages"));
    assert!(workflow.contains("name: E2E Tests (local web app)"));
    assert!(workflow.contains("needs: [deploy-pages]"));
}
