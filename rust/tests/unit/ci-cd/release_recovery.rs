//! Release convergence and package-verification workflow contracts.

use super::*;

#[test]
fn prepares_one_version_verifies_its_archive_and_can_resume_it() {
    let workflow = release_workflow();
    let auto_release = job_block(&workflow, "auto-release");

    let auto_version = workflow_step_block(auto_release, "Collect changelog and bump version");
    assert!(
        auto_version.contains("--resume-prepared") && auto_version.contains("--branch main"),
        "a rerun of the original merge event must sync and resume its already-prepared main version instead of bumping again"
    );
    assert!(
        auto_release.contains("id: prepared")
            && auto_release.contains("HAS_FRAGMENTS: 'false'")
            && auto_release.contains("rust-script scripts/check-release-needed.rs"),
        "publication state must be re-read after the version step, because that step may sync to a prepared release commit"
    );

    let auto_build = auto_release
        .find("- name: Build release")
        .expect("auto release should build the selected version");
    let auto_package = auto_release
        .find("- name: Verify packaged crate archive")
        .expect("auto release should verify the exact crate archive");
    let auto_publish = auto_release
        .find("- name: Publish to Crates.io")
        .expect("auto release should publish the verified archive");
    assert!(auto_build < auto_package && auto_package < auto_publish);
    assert!(
        workflow_step_block(auto_release, "Verify packaged crate archive")
            .contains("cargo package --locked -p formal-ai"),
        "the release must compile the packaged archive before cargo publish"
    );

    for step_name in [
        "Wait for Crate availability on Crates.io",
        "Log in to GitHub Container Registry",
        "Set up Docker Buildx",
        "Extract GHCR Docker metadata",
        "Publish Docker image to GHCR",
        "Configure Docker Hub publishing",
        "Create GitHub Release",
    ] {
        let step = workflow_step_block(auto_release, step_name);
        assert!(
            step.contains("steps.prepared.outputs.crate_published == 'true'"),
            "auto-release {step_name} should use the prepared version's publication state"
        );
        assert!(
            step.contains("steps.publish-crate.outputs.publish_result == 'success'"),
            "auto-release {step_name} should wait for a successful crates.io publish before creating downstream artifacts"
        );
        assert!(
            !step.contains("steps.check.outputs.should_release == 'true'\n"),
            "auto-release {step_name} should not run solely because a release is needed"
        );
    }

    let manual_release = job_block(&workflow, "manual-release");
    let manual_build = manual_release
        .find("- name: Build release")
        .expect("manual release should build the selected version");
    let manual_package = manual_release
        .find("- name: Verify packaged crate archive")
        .expect("manual release should verify the exact crate archive");
    let manual_publish = manual_release
        .find("- name: Publish to Crates.io")
        .expect("manual release should publish the verified archive");
    assert!(manual_build < manual_package && manual_package < manual_publish);
    assert!(
        !workflow_step_block(manual_release, "Version and commit").contains("--resume-prepared"),
        "an explicitly requested manual release remains a new bump, not an automatic recovery"
    );
    for step_name in [
        "Wait for Crate availability on Crates.io",
        "Log in to GitHub Container Registry",
        "Set up Docker Buildx",
        "Extract GHCR Docker metadata",
        "Publish Docker image to GHCR",
        "Configure Docker Hub publishing",
        "Create GitHub Release",
    ] {
        let step = workflow_step_block(manual_release, step_name);
        assert!(
            step.contains("steps.publish-crate.outputs.publish_result == 'success'"),
            "manual-release {step_name} should wait for a successful crates.io publish before creating downstream artifacts"
        );
        assert!(
            !step.contains(
                "steps.version.outputs.version_committed == 'true' || steps.version.outputs.already_released == 'true'\n"
            ),
            "manual-release {step_name} should not run solely because a version step completed"
        );
    }
}
