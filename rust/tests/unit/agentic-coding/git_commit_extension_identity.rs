use super::{valid_branch_target, valid_workflow_target};
#[test]
fn git_lock_suffix_remains_case_sensitive() {
    assert!(!valid_branch_target("topic.lock"));
    assert!(valid_branch_target("topic.LOCK"));
    assert!(!valid_branch_target("topic.lock/nested"));
    assert!(valid_branch_target("topic.LOCK/nested"));
}
#[test]
fn workflow_extension_matches_original_lowercased_suffix_including_dot_files() {
    for path in [
        "ci/check.yml",
        "ci/check.YAML",
        ".yml",
        ".YAML",
        "nested/.yml",
    ] {
        assert!(valid_workflow_target(path));
    }
    for path in [
        "ci/check.yml/",
        "ci/check.yaml.extra",
        "../check.yml",
        "ci//check.yml",
    ] {
        assert!(!valid_workflow_target(path));
    }
}
