//! Replay complete native workflow, release and Pages contracts.
//! Existing modules keep their ordinary fixture readers and assertions.

pub mod ci_gates;
#[path = "ci-cd/workflow_fixtures.rs"]
pub mod workflow_fixtures;
#[path = "ci-cd/workflow_release.rs"]
pub mod workflow_release;

use workflow_fixtures::{job_block, repository_file};

#[path = "ci-cd/step_budgets_within_job_clocks/codeql.rs"]
mod codeql;

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

#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../../../scripts/version-changelog.rs"]
mod version_changelog;

// Exercise the native detector even when PR admission uses its JavaScript twin.
#[allow(dead_code)]
#[path = "../../../scripts/detect-code-changes.rs"]
pub mod detect_code_changes;

// Share the complete budget and pinned-image regressions with PR qualification.
#[path = "ci-cd/job_budget_fit.rs"]
mod job_budget_fit;
#[path = "ci-cd/pinned_tool_images.rs"]
mod pinned_tool_images;

// Qualify complete artifact, Pages and release policy assertions on pull requests.
#[path = "ci-cd/release_publishing.rs"]
mod release_publishing;

// Compile the canonical generator and its bundle/include contracts before merge.
#[allow(dead_code)]
#[path = "../../../scripts/generate-seed-registry.rs"]
mod generate_seed_registry;
