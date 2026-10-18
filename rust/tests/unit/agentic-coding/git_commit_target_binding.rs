use super::*;

#[test]
fn positive_owned_branch_bindings_preserve_the_five_original_languages() {
    for request in [
        "Keep the solution on branch feature/held-out.",
        "Держи решение в ветке feature/held-out.",
        "समाधान को शाखा feature/held-out पर रखें।",
        "将解决方案保留在分支 feature/held-out 上。",
        "Mantén la solución en la rama feature/held-out.",
    ] {
        assert_eq!(named_branch(request).as_deref(), Some("feature/held-out"));
    }
    assert_eq!(
        named_branch(
            "A branch so observers can inspect it.\nYour prepared branch: feature/held-out"
        )
        .as_deref(),
        Some("feature/held-out")
    );
    assert_eq!(named_branch("Use branch so").as_deref(), Some("so"));
}

#[test]
fn quoted_ambiguous_negative_and_invalid_targets_never_become_a_push_target() {
    assert!(matches!(
        request_target_binding(
            "Use branch one\nUse branch two",
            "branch-binding-patterns",
            valid_branch_target
        ),
        RequestTargetBinding::Refused
    ));
    assert!(matches!(
        request_target_binding(
            "Do not use branch one",
            "branch-binding-patterns",
            valid_branch_target
        ),
        RequestTargetBinding::Refused
    ));
    assert!(matches!(
        request_target_binding(
            "Use branch one\nDo not use branch one",
            "branch-binding-patterns",
            valid_branch_target
        ),
        RequestTargetBinding::Refused
    ));
    assert!(matches!(
        request_target_binding(
            "\"Use branch foreign\"",
            "branch-binding-patterns",
            valid_branch_target
        ),
        RequestTargetBinding::Absent
    ));
    for target in [
        "bad..ref",
        "bad.lock",
        "a/.hidden",
        "a//b",
        "bad@{x",
        "-option",
        "bad ref",
        "a\\b",
        "a/",
        "a.",
    ] {
        assert!(!valid_branch_target(target), "{target}");
    }
}

#[test]
fn requested_yaml_destinations_are_bound_without_replacing_them_with_a_default() {
    for target in [
        "ci/test-hello-world.yml",
        "checks/nested/held-out.yaml",
        ".github/workflows/check.yml",
    ] {
        let request = format!("Save the workflow as {target}");
        assert!(
            matches!(request_target_binding(&request, "workflow-binding-patterns", valid_workflow_target), RequestTargetBinding::Bound(value) if value == target)
        );
    }
    assert!(matches!(
        request_target_binding(
            "Do not save the workflow as ci/foreign.yml",
            "workflow-binding-patterns",
            valid_workflow_target
        ),
        RequestTargetBinding::Refused
    ));
    for target in [
        "../escape.yml",
        "/absolute.yml",
        "ci//file.yaml",
        "ci/./file.yaml",
        "ci/a;touch.yml",
    ] {
        assert!(!valid_workflow_target(target), "{target}");
    }
}

#[test]
fn recipe_scaffold_cleanup_keeps_exact_target_and_declared_commit_scope() {
    let recipe = ExecutionRecipe {
        language: "text".to_owned(),
        source: "held-out source".to_owned(),
        path: "nested/owned.txt".to_owned(),
        supporting_files: Vec::new(),
        commands: Vec::new(),
    };
    let mut target = CommitTarget {
        binding_refused: false,
        branch: Some("feature/held-out".to_owned()),
        reference: "https://github.com/example/repository/issues/73".to_owned(),
    };
    let command = recipe_commit_command(&recipe, &target);
    assert!(command.contains("set -- 'nested/owned.txt'"));
    assert!(command.contains("git commit --only"));
    assert!(command.contains("git diff --cached --quiet -- .gitkeep"));
    assert!(command.contains("https://github.com/example/repository/issues/73"));
    assert!(command.contains("git symbolic-ref --quiet --short HEAD"));
    target.binding_refused = true;
    let refused = recipe_commit_command(&recipe, &target);
    assert!(!refused.contains("rm --"));
    assert!(!refused.contains("git commit"));
}
