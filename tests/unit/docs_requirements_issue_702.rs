//! Issue #702 documentation traceability.
//!
//! The issue's final deliverable is that the collected data and the analysis
//! live under `docs/case-studies/issue-702/` and that every requirement is
//! traceable from the repository-level matrix to the code and the test that
//! covers it. This test keeps those documents from drifting away from the
//! implementation they describe.

use std::fs;
use std::path::Path;

#[test]
fn issue_702_world_model_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #702 Dialogue World Model",
            "| R702-1 ",
            "| R702-2 ",
            "| R702-3 ",
            "| R702-4 ",
            "| R702-5 ",
            "| R702-6 ",
            "| R702-7 ",
            "| R702-8 ",
            "| R702-9 ",
            "| R702-10 ",
            "DialogueWorldModel",
            "world_model_mode",
            "docs/case-studies/issue-702",
        ],
    );

    let readme = read(root.join("docs/case-studies/issue-702/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-702/README.md",
        &readme,
        &[
            "# Issue 702 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "R702-1",
            "R702-8",
            "src/world_model_dialog.rs",
            "src/solver_handlers/world_state.rs",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-702/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-702/requirements.md",
        &issue_requirements,
        &[
            "R702-1",
            "R702-8",
            "R702-D",
            "provenance",
            "synchronization loop",
            "relative-meta-logic",
            "trace-only",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-702/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-702/solution-plans.md",
        &solution_plans,
        &[
            "Current state from the dialogue, with provenance",
            "Target state from",
            "askable from chat",
            "Synchronization loop with append-only events",
            "Merge and split as first-class operations",
            "Dependent statements via relative-meta-logic",
            "Action-consequence prediction",
            "bAbI-style tracking slice with a ratchet",
        ],
    );

    // The benchmark slice the acceptance criteria ask for must stay catalogued
    // with its ratchet test, and its license provenance must stay recorded.
    let catalog = read(root.join("docs/benchmarks.md"));
    assert_contains_all(
        "docs/benchmarks.md",
        &catalog,
        &[
            "world-state-tracking-suite.lino",
            "issue_702_world_state_suite_tracks_each_case",
            "bAbI-style world-state tracking",
        ],
    );
    let licenses = read(root.join("data/benchmarks/LICENSES.md"));
    assert_contains_all(
        "data/benchmarks/LICENSES.md",
        &licenses,
        &["Issue #702 World-State Tracking Slice", "bAbI"],
    );

    for relative in [
        "docs/case-studies/issue-702/raw-data/issue-702.json",
        "docs/case-studies/issue-702/raw-data/issue-702-comments.json",
        "docs/case-studies/issue-702/raw-data/issue-651-parent.json",
        "docs/case-studies/issue-702/raw-data/pr-675.json",
        "data/benchmarks/world-state-tracking-suite.lino",
        "src/world_model_atoms.rs",
        "src/world_model_dialog.rs",
        "src/solver_handlers/world_state.rs",
        "tests/unit/issue_702_world_model_dialog.rs",
        "tests/unit/issue_702_world_state_chat.rs",
        "tests/unit/specification/world_state_benchmarks.rs",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #702 traceability",
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}",
        );
    }
}
