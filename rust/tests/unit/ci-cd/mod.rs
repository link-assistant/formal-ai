mod api_docs_publishing;
mod artifact_download_retry;
mod attestation_and_coverage_audit;
mod authoring_effects;
mod box_language_release_legs;
mod changelog_parsing;
#[path = "../../../../scripts/check-associative-terminology.rs"]
mod check_associative_terminology;
#[allow(clippy::duplicate_mod)]
#[path = "../../../../scripts/check-crate-package-size.rs"]
mod check_crate_package_size;
#[path = "../../../../scripts/check-file-size.rs"]
mod check_file_size;
mod ci_diagnostic_audit;
mod ci_warning_audit;
mod codeql_sink_heuristics;
mod container_image_visibility;
#[allow(clippy::duplicate_mod)]
#[path = "../../../../scripts/create-github-release.rs"]
mod create_github_release;
mod default_branch_ci_failures;
mod desktop_jobs_and_permissions;
mod desktop_release_resolve;
#[allow(dead_code)]
#[path = "../../../../scripts/detect-code-changes.rs"]
mod detect_code_changes;
mod docker_layer_cache_budget;
mod docker_resource_hygiene;
mod example_binary_pruning;
mod excluded_only_changes;
mod executable_workflow_scripts;
mod fresh_merge_fetch_retry;
mod harness_release_profile;
mod issue_1017_evidence;
mod issue_1107_green_ledger;
mod issue_1111_non_linux_switch;
mod issue_1113_pull_request_status;
mod issue_1137_agentic_routing_replay;
mod issue_1138_dependency_patches;
mod issue_1138_layered_ci;
mod javascript_dependency_audit;
mod job_budget_fit;
mod job_timeouts_as_failures;
mod link_checker_retries;
mod longest_tests_first;
mod macos_package_retry;
mod macos_test_partitioning;
mod macos_test_selection;
mod network_download_retry;
mod npm_deprecation_review;
mod one_build_per_platform;
mod optimized_test_profile;
mod pinned_tool_images;
mod pr_1188_sharding;
mod production_readiness_gates;
mod protocol_callers;
mod release_publishing;
mod release_site_layout;
#[path = "../../../../scripts/rust-paths.rs"]
mod rust_paths;
mod rust_warning_policy;
mod sccache_statistics;
mod single_release_build;
mod source_test_placement;
mod step_budgets_within_job_clocks;
mod step_deadlines_and_coverage_budget;
mod test_lane_deduplication;
mod third_party_cli_version_pins;
mod tracked_git_hooks;
mod workflow_coverage;
mod workflow_fixtures;
mod workflow_release;
mod workflow_release_desktop;
mod workflow_task_ladder;
mod workspace_manifest_resolution;
