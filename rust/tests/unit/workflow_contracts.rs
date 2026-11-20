//! Replay complete native workflow, release and Pages contracts.
//! Existing modules keep their ordinary fixture readers and assertions.

pub mod ci_gates;
#[path = "ci-cd/workflow_fixtures.rs"]
pub mod workflow_fixtures;
#[path = "ci-cd/workflow_release.rs"]
pub mod workflow_release;

#[path = "../../../scripts/changelog-fragment-file.rs"]
pub mod changelog_fragment_file;

#[path = "../../../scripts/github-output.rs"]
pub mod github_output;

#[path = "ci-cd/cargo_package_identity.rs"]
mod cargo_package_identity;
#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/rust-paths.rs"]
pub mod rust_paths;
#[path = "ci-cd/workspace_manifest_resolution.rs"]
mod workspace_manifest_resolution;

#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../../../scripts/bump-version.rs"]
mod bump_version;

#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../../../scripts/collect-changelog.rs"]
mod collect_changelog;

#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../../../scripts/get-bump-type.rs"]
mod get_bump_type;
