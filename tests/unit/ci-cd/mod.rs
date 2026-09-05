mod changelog_parsing;
#[path = "../../../scripts/check-associative-terminology.rs"]
mod check_associative_terminology;
#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/check-crate-package-size.rs"]
mod check_crate_package_size;
#[path = "../../../scripts/check-file-size.rs"]
mod check_file_size;
mod codeql_sink_heuristics;
#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/create-github-release.rs"]
mod create_github_release;
mod desktop_release_resolve;
#[allow(dead_code)]
#[path = "../../../scripts/detect-code-changes.rs"]
mod detect_code_changes;
mod issue_1001;
mod issue_1012;
mod issue_1014;
mod issue_1017;
mod issue_1021;
mod issue_1031;
mod issue_1037;
mod issue_1039;
mod issue_1041;
mod issue_1043;
mod issue_1045;
mod issue_1047;
mod issue_1049;
mod issue_1051;
mod issue_1053;
mod issue_1055;
mod issue_1057;
mod issue_1059;
mod issue_1064;
mod issue_1069;
mod issue_1076;
mod issue_717;
mod issue_730;
mod issue_739;
mod issue_742;
mod issue_796;
mod issue_798;
mod issue_846;
mod issue_932;
mod issue_977;
mod issue_980;
mod issue_999;
mod javascript_dependency_audit;
mod macos_package_retry;
mod release_publishing;
mod release_site_layout;
#[path = "../../../scripts/rust-paths.rs"]
mod rust_paths;
mod source_test_placement;
mod workflow_coverage;
mod workflow_fixtures;
mod workflow_release;
mod workflow_release_desktop;
mod workflow_task_ladder;
mod workspace_manifest_resolution;
