//! Replay complete native workflow, release and Pages contracts.
//! Existing modules keep their ordinary fixture readers and assertions.

pub mod ci_gates;
#[path = "ci-cd/workflow_fixtures.rs"]
pub mod workflow_fixtures;
#[path = "ci-cd/workflow_release.rs"]
pub mod workflow_release;

#[path = "../../../scripts/changelog-fragment-file.rs"]
pub mod changelog_fragment_file;
