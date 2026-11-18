//! Original moved-operation assertions, with checked current source provenance.
use super::*;

pub(super) fn pages_deploy_waits_for_release_reference_before_pages_upload() {
    let workflow = crate::ci_gates::staged_release_operations::release_operation_workflow();
    let auto_release = job_block(&workflow, "auto-release");
    let manual_release = job_block(&workflow, "manual-release");
    let build_pages = job_block(&workflow, "build-pages");
    let pages_artifact = pages_artifact_workflow();
    let pages_build = job_block(&pages_artifact, "build");

    assert!(auto_release.contains("outputs:\n      pages_sha:"));
    assert!(auto_release.contains("Resolve Pages deploy ref"));
    assert!(manual_release.contains("outputs:\n      pages_sha:"));
    assert!(manual_release.contains("Resolve Pages deploy ref"));
    assert!(build_pages.contains("needs: [build, auto-release, manual-release]"));
    assert!(build_pages.contains("needs.build.result == 'success'"));
    assert!(build_pages.contains("github.ref == 'refs/heads/main'"));
    assert!(build_pages.contains("needs.auto-release.result == 'success'"));
    assert!(build_pages.contains("needs.manual-release.result == 'success'"));
    assert!(build_pages.contains("uses: ./.github/workflows/pages-artifact.yml"));
    assert!(build_pages.contains(
        "sha: ${{ needs.auto-release.outputs.pages_sha || needs.manual-release.outputs.pages_sha || github.sha }}"
    ));
    assert!(job_block(&workflow, "deploy-pages").contains("needs: [build-pages]"));
    assert!(pages_build.contains("Select Pages deployment ref"));
    assert!(pages_build.contains("PAGES_DEPLOY_SHA: ${{ inputs.sha }}"));
    assert!(pages_build.contains("ref: ${{ github.ref }}"));
    assert!(pages_build.contains("git rev-parse HEAD"));
    assert!(!pages_build.contains("ref: ${{ steps.pages_ref.outputs.sha }}"));
}

pub(super) fn rust_script_install_steps_use_retry_wrapper() {
    let workflow = crate::ci_gates::staged_release_operations::release_operation_workflow();
    let manifest_directory = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let install_script = fs::read_to_string(format!(
        "{manifest_directory}/scripts/install-rust-script.sh"
    ))
    .unwrap();

    assert!(
        !workflow.contains("run: cargo install rust-script"),
        "workflow should not call cargo install directly because crates.io HTTP failures are transient"
    );
    assert_eq!(
        workflow
            .matches("run: bash scripts/install-rust-script.sh")
            .count(),
        workflow
            .lines()
            .filter(|line| line.trim() == "- name: Install rust-script")
            .count(),
        "each rust-script install step should use the retry wrapper"
    );
    assert!(workflow.contains("run: bash scripts/install-rust-script.sh"));
    assert!(install_script.contains("RUST_SCRIPT_INSTALL_ATTEMPTS"));
    assert!(install_script.contains("cargo install rust-script --locked"));
    assert!(install_script.contains("sleep \"$delay\""));
}
