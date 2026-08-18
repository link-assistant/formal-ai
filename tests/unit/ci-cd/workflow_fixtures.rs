//! Shared fixtures for the release-workflow CI/CD unit tests.
//!
//! These helpers read the real workflow YAML from the repo and slice out
//! individual jobs/steps so the assertions in `workflow_release.rs` and
//! `release_publishing.rs` can stay focused and small.

use std::fs;

/// Everything CI executes for a pull request, workflow and gate registry as one
/// text.
///
/// Issue #991 moved the `lint` job's checks into `data/meta/ci-gates/`, so a
/// test that asks "does CI run this check?" must read both files.
/// `crate::ci_gates` answers that for the whole suite; it is re-exported here
/// because these fixtures are where the workflow tests already look.
pub use crate::ci_gates::{ci_surface, release_workflow};

/// Issue #895: the two coverage denominators live in their own workflow. They
/// are a leaf of the release graph -- nothing `needs:` them -- so moving them
/// out of `release.yml` changed no ordering, and it keeps that file under the
/// 2000-line ceiling `scripts/check-file-size.rs` enforces.
pub fn coverage_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/coverage.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

pub fn desktop_release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/desktop-release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

pub fn job_block<'a>(workflow: &'a str, job_name: &str) -> &'a str {
    let marker = format!("  {job_name}:\n");
    let start = workflow.find(&marker).unwrap();
    let body_start = start + marker.len();
    let rest = &workflow[body_start..];

    let next_job = rest
        .lines()
        .scan(0usize, |offset, line| {
            let current_offset = *offset;
            *offset += line.len() + 1;
            Some((current_offset, line))
        })
        .find_map(|(offset, line)| {
            let starts_at_job_indent = line.starts_with("  ") && !line.starts_with("    ");
            (starts_at_job_indent && line.trim_end().ends_with(':')).then_some(offset)
        });

    next_job.map_or_else(
        || &workflow[start..],
        |end| &workflow[start..body_start + end],
    )
}

pub fn workflow_step_block<'a>(job: &'a str, step_name: &str) -> &'a str {
    let marker = format!("      - name: {step_name}\n");
    let start = job.find(&marker).unwrap();
    let body_start = start + marker.len();
    let rest = &job[body_start..];

    let next_step = rest
        .lines()
        .scan(0usize, |offset, line| {
            let current_offset = *offset;
            *offset += line.len() + 1;
            Some((current_offset, line))
        })
        .find_map(|(offset, line)| line.starts_with("      - ").then_some(offset));

    next_step.map_or_else(|| &job[start..], |end| &job[start..body_start + end])
}

pub fn workflow_job_names(workflow: &str) -> Vec<&str> {
    let marker = "jobs:\n";
    let start = workflow.find(marker).unwrap() + marker.len();

    workflow[start..]
        .lines()
        .filter_map(|line| {
            let starts_at_job_indent = line.starts_with("  ") && !line.starts_with("    ");
            // A comment at job indentation is not a job. Issue #1017: a prose
            // line that happened to end in a colon was parsed as one, which
            // made every job-sweeping contract fail against a phantom job whose
            // "name" was the sentence -- an error that points at the wrong
            // thing entirely and takes a CI round trip to see.
            let is_comment = line.trim_start().starts_with('#');
            (starts_at_job_indent && !is_comment && line.trim_end().ends_with(':'))
                .then(|| line.trim().trim_end_matches(':'))
        })
        .collect()
}
