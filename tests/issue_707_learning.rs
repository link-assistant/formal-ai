//! Issue #707 acceptance: what the corpus taught, and what it honestly did not.
//!
//! The generalization slice proves that held-out requests get verified plans.
//! These tests pin the *learning* that makes that possible: the induced schemas
//! are committed as evidence and must not drift silently, every primitive the
//! corpus demonstrates must be explained, the residue must be reported rather
//! than papered over, and no synthesized plan may inherit a path from the
//! example that taught its operation.

use std::collections::BTreeSet;
use std::fs;

use formal_ai::computer_use::{learned, synthesize, ComputerUsePrimitive};

const SNAPSHOT: &str = "docs/case-studies/issue-707/learned-schemas.lino";

/// Directories a synthesized plan is allowed to create for its own artifacts.
const PRODUCED_PREFIXES: [&str; 4] = ["reports/", "out/", "processed/", "restored"];

#[test]
fn the_committed_schema_snapshot_matches_a_fresh_induction() {
    let committed = fs::read_to_string(SNAPSHOT).expect("committed schema snapshot");
    assert_eq!(
        committed,
        learned().links_notation(),
        "the learned schemas drifted from the committed evidence; \
         regenerate with `cargo run --bin formal-ai -- computer-use --learn > {SNAPSHOT}`"
    );
}

#[test]
fn every_operation_the_corpus_demonstrates_is_explained_and_none_are_rejected() {
    let schemas = learned();
    assert_eq!(
        schemas.operations.len(),
        12,
        "learned operations: {:?}",
        schemas.operations.keys().collect::<Vec<_>>()
    );
    assert!(
        schemas.rejected.is_empty(),
        "rejected schemas: {:?}",
        schemas.rejected
    );
    let primitives = schemas
        .operations
        .values()
        .map(|operation| operation.step.signature.primitive)
        .chain(
            schemas
                .resources
                .values()
                .flat_map(|binding| binding.steps.iter().map(|step| step.primitive)),
        )
        .collect::<BTreeSet<_>>();
    for required in [
        ComputerUsePrimitive::FsWrite,
        ComputerUsePrimitive::FsList,
        ComputerUsePrimitive::FsMove,
        ComputerUsePrimitive::ShellRun,
        ComputerUsePrimitive::HttpFetch,
        ComputerUsePrimitive::HttpPost,
        ComputerUsePrimitive::DomQuery,
        ComputerUsePrimitive::DomExtract,
        ComputerUsePrimitive::ArchivePack,
        ComputerUsePrimitive::ArchiveUnpack,
        ComputerUsePrimitive::ProcessStatus,
    ] {
        assert!(
            primitives.contains(&required),
            "no schema explains {}",
            required.name()
        );
    }
}

#[test]
fn every_resource_binding_is_supported_by_a_recorded_task() {
    for (slug, binding) in &learned().resources {
        assert!(!binding.steps.is_empty(), "{slug} materialises nothing");
        assert!(!binding.support.is_empty(), "{slug} cites no evidence");
    }
}

#[test]
fn unexplained_steps_are_reported_rather_than_invented() {
    // The corpus contains one step whose operation no prompt names in all four
    // languages. Induction must say so out loud instead of guessing a schema.
    let residue = &learned().unexplained;
    assert!(
        residue.iter().all(|entry| entry.contains(':')),
        "residue entries must name task and primitive: {residue:?}"
    );
    let snapshot = fs::read_to_string(SNAPSHOT).expect("snapshot");
    for entry in residue {
        assert!(
            snapshot.contains(entry),
            "residue {entry} is missing from the committed evidence"
        );
    }
}

#[test]
fn no_synthesized_plan_inherits_a_path_from_another_resource() {
    let schemas = learned();
    for prompt in [
        "Count the lines in the notes and archive the result",
        "Show the distinct values in the customers file",
        "Enumerate the documents and bundle them",
        "Report the process status for the inventory",
        "Move the inbox note and archive it",
    ] {
        let synthesis = synthesize(prompt).expect("plan");
        let binding = &schemas.resources[&synthesis.resource];
        let own = binding
            .steps
            .iter()
            .flat_map(|step| step.arguments.as_object().into_iter().flatten())
            .filter_map(|(_, value)| value.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        for step in &synthesis.plan.steps {
            for (field, value) in step.arguments.as_object().expect("object") {
                let candidates = match value.as_str() {
                    Some(text) => vec![text.to_owned()],
                    None => value
                        .as_array()
                        .map(|entries| {
                            entries
                                .iter()
                                .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
                                .collect()
                        })
                        .unwrap_or_default(),
                };
                for candidate in candidates {
                    if !candidate.contains('/') || candidate.starts_with("fixture://") {
                        continue;
                    }
                    assert!(
                        own.contains(&candidate)
                            || PRODUCED_PREFIXES
                                .iter()
                                .any(|prefix| candidate.starts_with(prefix)),
                        "{prompt}: step field {field} = {candidate} came from another example"
                    );
                }
            }
        }
    }
}
