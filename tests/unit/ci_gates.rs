//! What CI actually executes, for any test that needs to ask.
//!
//! Issue #991 moved the `lint` job's checks out of `.github/workflows/release.yml`
//! into `data/meta/ci-gates/`, one shard per gate, because an append-only step
//! list is what made that workflow the third most conflicted path in the
//! repository (`data/meta/merge-conflict-ledger.lino`).
//!
//! That split changed where the answer lives, not the question: "does CI run this
//! check?" is still exactly the right thing for a test to assert. So the answer
//! moves here, as one text spanning both files, and the tests keep asking.
//!
//! Tests across the suite -- `ci_cd`, `docs_requirements`, and the issue-scoped
//! documentation pins -- read it through this module rather than each learning
//! the registry's layout.

use std::fs;
use std::path::Path;

/// The gate runner itself, compiled into the suite.
///
/// The registry is read with the same parser CI runs, so a shard the runner
/// would reject can never look registered to a test, and the script's embedded
/// unit tests run as part of this suite.
#[path = "../../scripts/run-ci-gates.rs"]
pub mod run_ci_gates;

/// The release workflow as committed.
pub fn release_workflow() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(".github/workflows/release.yml"))
        .expect("the release workflow should be readable")
        .replace("\r\n", "\n")
}

/// Everything CI executes for a pull request: the workflow with each stage's
/// registered gates spliced in at the step that runs them.
///
/// Gates land where CI reaches them, so position in this text still means
/// position in the run -- which is what the ordering assertions depend on.
pub fn ci_surface() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gates = run_ci_gates::load_registry(root).expect("the gate registry should load");
    run_ci_gates::workflow_surface(&release_workflow(), &gates)
}

/// Everything CI executes for a pull request, with each reusable workflow this
/// repository calls spliced in at the job that calls it.
///
/// Issue #1081 moved the `test-agent-cli-e2e` job's 324 steps into
/// `.github/workflows/agent-cli-e2e.yml` and left a `uses:` behind. That is a
/// change of file, not of behaviour -- CI runs the same steps in the same
/// place -- but every test that had asked "does the pipeline run this?" by
/// reading `release.yml` alone would have started answering no. A pipeline
/// that quietly stops being covered by its own contracts is exactly the false
/// negative issue #1081 is about, so the tests must not be able to lose sight
/// of a job by having it moved.
///
/// The splice is the same device [`ci_surface`] already uses for the gate
/// registry, for the same reason: the called steps land where CI reaches them,
/// so position in this text still means position in the run, and
/// `job_block(.., "test-agent-cli-e2e")` still returns that job's steps.
///
/// A test that cares *which file* a step lives in should still read that file
/// directly -- [`release_workflow`] and the fixtures in
/// `ci-cd/workflow_fixtures.rs` are unchanged for exactly that case.
pub fn pipeline_workflows() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut surface = ci_surface();
    for called in called_workflows(&surface) {
        let body = fs::read_to_string(root.join(&called))
            .unwrap_or_else(|err| {
                panic!("{called} is called by the pipeline but unreadable: {err}")
            })
            .replace("\r\n", "\n");
        let steps = called_workflow_job_body(&called, &body);
        let call = format!("    uses: ./{called}\n");
        assert!(
            surface.contains(&call),
            "{called} is called at an indentation this splice does not recognise"
        );
        surface = surface.replace(&call, &steps);
    }
    surface
}

/// The bodies of the jobs in a called workflow, without their job keys.
///
/// A reusable workflow called from a job *is* that job: GitHub runs its steps
/// under the caller's `needs:`, `if:` and `concurrency:`. Splicing the bodies
/// in without the keys keeps that true of the text as well -- the caller stays
/// one job, and `workflow_job_names` does not grow a phantom.
///
/// A called workflow may declare more than one job:
/// `.github/workflows/macos-core-tests.yml` builds an archive in one and runs
/// it in another. Their bodies are concatenated in file order, which is the
/// order CI reaches them, and each job's own `needs:` is dropped -- it names a
/// job inside the called file, and leaving it in the text would make the
/// *caller* look like it depends on a job no other workflow can see. That is
/// the false negative issue #1081 is about, pointed the other way: a test
/// asking what `auto-release` waits for must not be answered with an edge from
/// somebody else's file.
fn called_workflow_job_body(path: &str, workflow: &str) -> String {
    let marker = "\njobs:\n";
    let start = workflow
        .find(marker)
        .unwrap_or_else(|| panic!("{path} is called as a workflow but declares no `jobs:`"))
        + marker.len();
    let rest = &workflow[start..];

    let is_job_key = |line: &str| {
        line.starts_with("  ")
            && !line.starts_with("   ")
            && !line.trim_start().starts_with('#')
            && line.trim_end().ends_with(':')
    };
    let key_lines: Vec<usize> = rest
        .lines()
        .enumerate()
        .filter(|(_, line)| is_job_key(line))
        .map(|(index, _)| index)
        .collect();
    assert!(
        !key_lines.is_empty(),
        "{path} is called as a workflow but declares no job under `jobs:`"
    );

    let lines: Vec<&str> = rest.lines().collect();
    let mut body = String::new();
    for (nth, &key) in key_lines.iter().enumerate() {
        let end = key_lines.get(nth + 1).copied().unwrap_or(lines.len());
        let mut index = key + 1;
        while index < end {
            let line = lines[index];
            let job_level = line.starts_with("    ") && !line.starts_with("     ");
            if job_level && line.trim_start().starts_with("needs:") {
                index += 1;
                // A flow sequence may be written over several lines, and a
                // block sequence always is; skip the value, not only the key.
                while index < end && lines[index].starts_with("     ") {
                    index += 1;
                }
                continue;
            }
            body.push_str(line);
            body.push('\n');
            index += 1;
        }
    }
    format!("{}\n", body.trim_end())
}

/// The repository-local workflow files referenced by `uses: ./…`, in the order
/// they appear. Remote reusable workflows are somebody else's file and are not
/// something a test in this repository can read.
fn called_workflows(surface: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in surface.lines() {
        let Some((_, rest)) = line.split_once("uses: ./") else {
            continue;
        };
        let path = rest.split_whitespace().next().unwrap_or_default();
        if !path.starts_with(".github/workflows/") {
            continue;
        }
        if !found.iter().any(|seen| seen == path) {
            found.push(path.to_string());
        }
    }
    found
}

#[cfg(test)]
mod pipeline_workflow_tests {
    use super::*;

    #[test]
    fn the_pipeline_surface_reaches_into_the_workflows_it_calls() {
        let called = called_workflows(&ci_surface());
        assert!(
            !called.is_empty(),
            "release.yml delegates to at least one reusable workflow in this repository; \
             if that stopped being true this helper is dead weight and should go"
        );
        let surface = pipeline_workflows();
        for path in &called {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"));
            let body = fs::read_to_string(root.join(path)).expect("a called workflow is readable");
            let steps = called_workflow_job_body(path, &body.replace("\r\n", "\n"));
            let first_step = steps
                .lines()
                .find(|line| line.trim_start().starts_with("- "))
                .expect("a called workflow runs at least one step");
            assert!(
                surface.contains(first_step.trim_end()),
                "{path} is called by the pipeline but its steps are missing from the surface"
            );
            assert!(
                !surface.contains(&format!("uses: ./{path}")),
                "{path} was spliced, so the call that named it should be gone"
            );
        }
    }

    /// A called workflow may hold several jobs, and the edges between them are
    /// its own business. Both halves matter: every job has to arrive (or a
    /// test asking "does CI run this step?" starts answering no for the ones
    /// that did not), and none of their `needs:` may arrive with them (or a
    /// test asking what the *caller* waits for reads an edge from a file the
    /// caller cannot see).
    #[test]
    fn every_job_of_a_called_workflow_arrives_and_its_internal_edges_do_not() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let job_level = |line: &str| line.starts_with("    ") && !line.starts_with("     ");

        for path in called_workflows(&ci_surface()) {
            let body = fs::read_to_string(root.join(&path))
                .expect("a called workflow is readable")
                .replace("\r\n", "\n");
            let spliced = called_workflow_job_body(&path, &body);

            let jobs = body
                .split("\njobs:\n")
                .nth(1)
                .expect("a called workflow declares jobs")
                .lines()
                .filter(|line| {
                    line.starts_with("  ")
                        && !line.starts_with("   ")
                        && !line.trim_start().starts_with('#')
                        && line.trim_end().ends_with(':')
                })
                .count();
            assert_eq!(
                spliced.lines().filter(|line| *line == "    steps:").count(),
                jobs,
                "{path} declares {jobs} job(s); every one of them has to be spliced in, \
                 or the pipeline surface loses steps CI still runs"
            );

            for line in spliced.lines() {
                assert!(
                    !(job_level(line) && line.trim_start().starts_with("needs:")),
                    "{path} left a job-level `needs:` in the spliced body ({line:?}); \
                     it names a job inside {path}, so the caller must not appear to need it"
                );
            }
        }
    }

    #[test]
    fn a_spliced_call_leaves_the_calling_job_a_single_job() {
        // The splice is only safe if the text still describes the same graph.
        // A called workflow's job key must not survive the move, or a job that
        // `needs:` the caller would appear to depend on something else.
        let before = ci_surface();
        let after = pipeline_workflows();
        let count = |text: &str| {
            text.lines()
                .filter(|line| {
                    line.starts_with("  ")
                        && !line.starts_with("   ")
                        && !line.trim_start().starts_with('#')
                        && line.trim_end().ends_with(':')
                })
                .count()
        };
        assert_eq!(
            count(&before),
            count(&after),
            "splicing a called workflow must not add or remove a job"
        );
    }
}
