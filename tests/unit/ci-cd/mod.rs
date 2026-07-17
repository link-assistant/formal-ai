mod changelog_parsing;
#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/check-crate-package-size.rs"]
mod check_crate_package_size;
#[path = "../../../scripts/check-file-size.rs"]
mod check_file_size;
#[path = "../../../scripts/create-github-release.rs"]
mod create_github_release;
mod desktop_release_resolve;
#[allow(dead_code)]
#[path = "../../../scripts/detect-code-changes.rs"]
mod detect_code_changes;
mod issue_717;
mod issue_730;
mod issue_739;
mod issue_742;
mod release_publishing;
#[path = "../../../scripts/rust-paths.rs"]
mod rust_paths;
mod source_test_placement;
mod workflow_fixtures;
mod workflow_release;
mod workspace_manifest_resolution;
