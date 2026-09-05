mod changelog_parsing;
#[path = "../../../scripts/check-cargo-lock.rs"]
mod check_cargo_lock;
#[path = "../../../scripts/check-crate-size.rs"]
mod check_crate_size;
#[path = "../../../scripts/check-file-size.rs"]
mod check_file_size;
#[path = "../../../scripts/create-github-release.rs"]
mod create_github_release;
mod desktop_release_resolve;
#[allow(clippy::all, clippy::nursery, clippy::pedantic, dead_code)]
#[path = "../../../scripts/detect-code-changes.rs"]
mod detect_code_changes;
mod issue_119;
mod issue_127;
mod issue_135;
mod issue_141;
mod issue_143;
mod issue_147;
mod issue_149;
#[path = "../../../scripts/release-naming.rs"]
mod release_naming;
mod release_naming_tests;
#[path = "../../../scripts/rust-paths.rs"]
mod rust_paths;
#[allow(clippy::all, clippy::nursery, clippy::pedantic, dead_code)]
#[path = "../../../scripts/smoke-test-published-crate.rs"]
mod smoke_test_published_crate;
#[allow(clippy::all, clippy::nursery, clippy::pedantic, dead_code)]
#[path = "../../../scripts/version-and-commit.rs"]
mod version_and_commit;
mod version_and_commit_behind_check;
mod version_and_commit_tag_order;
#[allow(
    clippy::all,
    clippy::nursery,
    clippy::pedantic,
    dead_code,
    unused_imports
)]
#[path = "../../../scripts/wait-for-crate.rs"]
mod wait_for_crate;
mod workflow_desktop_release;
mod workflow_release;
mod workflow_security;
mod workspace_manifest_resolution;
