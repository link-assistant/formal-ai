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
