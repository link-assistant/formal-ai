mod changelog_parsing;
#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/check-crate-package-size.rs"]
mod check_crate_package_size;
#[path = "../../../scripts/check-file-size.rs"]
mod check_file_size;
#[path = "../../../scripts/create-github-release.rs"]
mod create_github_release;
mod desktop_release_resolve;
mod release_publishing;
#[path = "../../../scripts/rust-paths.rs"]
mod rust_paths;
mod source_test_placement;
mod workflow_fixtures;
mod workflow_release;
mod workspace_manifest_resolution;
