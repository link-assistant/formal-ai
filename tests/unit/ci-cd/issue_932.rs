//! Issue #932: the release pipeline must exercise each generated language
//! project inside the link-foundation/box image that matches the language.
//!
//! PR [#119](https://github.com/link-assistant/formal-ai/pull/119) asked for it
//! ("prefer test each software project of a specific language inside such
//! version of link foundation box docker image, that matches the language …
//! use traditional commands to init repositories for each language"), and no CI
//! leg did it. These assertions fail against the pre-fix workflow: the
//! `box-language-projects` job did not exist, so nothing pulled a box image and
//! nothing ran `cargo new` / `npm init` / `go mod init` against a Formal-AI
//! answer.
//!
//! The matrix is data, not YAML: `data/meta/box-language-projects.lino` names
//! the languages, and the checks below keep the workflow, the shell harness and
//! the contract from drifting apart.

use std::process::Command;

use formal_ai::box_language_projects::box_language_contract;

use super::workflow_fixtures::{job_block, release_workflow, workflow_step_block};

const JOB: &str = "box-language-projects";

#[test]
fn the_release_pipeline_runs_one_leg_per_language() {
    let contract = box_language_contract();
    assert_eq!(
        contract.workflow_job, JOB,
        "the contract must name the job that consumes it"
    );

    let workflow = release_workflow();
    let job = job_block(&workflow, JOB);

    let languages: Vec<&str> = job
        .lines()
        .find_map(|line| line.trim().strip_prefix("language: ["))
        .expect("the job must fan out over a language matrix")
        .trim_end_matches(']')
        .split(", ")
        .collect();
    let expected: Vec<&str> = contract
        .projects
        .iter()
        .map(|project| project.language.as_str())
        .collect();
    assert_eq!(
        languages, expected,
        "the workflow matrix must match data/meta/box-language-projects.lino"
    );

    for language in ["c", "cpp", "csharp"] {
        assert!(
            !languages.contains(&language),
            "{language} has no dedicated box image yet and must stay deferred"
        );
    }
    assert!(
        job.contains("fail-fast: false"),
        "one broken language must not hide the others"
    );
}

/// A leg that silently skipped — no Docker, no image — would report success
/// while verifying nothing, which is exactly the hole issue #932 is closing.
#[test]
fn the_leg_generates_the_project_and_refuses_to_skip_in_ci() {
    let contract = box_language_contract();
    let workflow = release_workflow();
    let job = job_block(&workflow, JOB);

    let generate = workflow_step_block(
        job,
        "Generate the ${{ matrix.language }} project from Formal-AI answers",
    );
    assert!(
        generate.contains(&format!(
            "{} ${{{{ matrix.language }}}}",
            contract.corpus_script
        )),
        "the project under test must come from solver answers: {generate}"
    );

    let verify = workflow_step_block(job, "Build and run it inside its link-foundation/box image");
    assert!(
        verify.contains("STRICT=1"),
        "a skip must be red in CI: {verify}"
    );
    assert!(
        verify.contains("SKIP_GENERATE=1"),
        "the leg must verify the project the previous step generated: {verify}"
    );
    assert!(
        verify.contains(&contract.verify_script),
        "the leg must run the same verifier a developer runs locally: {verify}"
    );
}

/// The leg is as expensive as the other container legs, so it is gated the same
/// way and cancels superseded pushes on branches.
#[test]
fn the_leg_is_gated_like_the_other_slow_legs() {
    let workflow = release_workflow();
    let job = job_block(&workflow, JOB);

    assert!(job.contains("needs: [detect-changes, build-artifacts]"));
    assert!(job.contains("timeout-minutes: 30"));
    assert!(job.contains("cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}"));
    for gate in [
        "github.event_name == 'workflow_dispatch'",
        "needs.detect-changes.outputs.any-code-changed == 'true'",
        "needs.detect-changes.outputs.rs-changed == 'true'",
        "needs.detect-changes.outputs.workflow-changed == 'true'",
    ] {
        assert!(job.contains(gate), "missing gate {gate}");
    }

    let status = job_block(&workflow, "pipeline-status");
    assert!(
        status.contains(JOB),
        "a failed or cancelled {JOB} leg must fail the run (issue #977)"
    );
}

#[test]
fn every_harness_script_is_tracked_executable_bash() {
    let contract = box_language_contract();
    for script in [
        &contract.corpus_script,
        &contract.verify_script,
        &contract.container_script,
    ] {
        let path = format!("{}/{script}", env!("CARGO_MANIFEST_DIR"));
        let parsed = Command::new("bash")
            .arg("-n")
            .arg(&path)
            .status()
            .expect("bash must run");
        assert!(parsed.success(), "{script} must parse");

        let mode = std::os::unix::fs::PermissionsExt::mode(
            &std::fs::metadata(&path)
                .unwrap_or_else(|error| panic!("{script}: {error}"))
                .permissions(),
        );
        assert_eq!(mode & 0o111, 0o111, "{script} must be executable");
    }
}
