#[path = "../../../src/agentic_coding/progress/creation_path.rs"]
mod production;
use production::{
    creation_path_matches, projected_creation_path, workspace_relative_creation_path,
};

#[test]
fn host_relative_and_absolute_bindings_are_symmetric_and_exact() {
    let relative = "reports/δ candidate.txt";
    let absolute = "/workspace/project/reports/δ candidate.txt";
    let root = Some("/workspace/project");
    assert_eq!(
        projected_creation_path(relative, root).as_deref(),
        Some(absolute)
    );
    assert_eq!(
        workspace_relative_creation_path(absolute, root).as_deref(),
        Some(relative)
    );
    assert!(creation_path_matches(relative, absolute, root));
    assert!(creation_path_matches(absolute, relative, root));
    assert!(!creation_path_matches(
        relative,
        "/workspace/project/reports/δ Candidate.txt",
        root
    ));
}

#[test]
fn filesystem_root_and_absent_host_preserve_only_their_exact_contract() {
    assert_eq!(
        projected_creation_path("nested/α.txt", Some("/")).as_deref(),
        Some("/nested/α.txt")
    );
    assert_eq!(
        workspace_relative_creation_path("/nested/α.txt", Some("/")).as_deref(),
        Some("nested/α.txt")
    );
    assert!(creation_path_matches(
        "nested/α.txt",
        "/nested/α.txt",
        Some("/")
    ));
    assert!(creation_path_matches("same.txt", "same.txt", None));
    assert!(creation_path_matches(
        "/exact/file.txt",
        "/exact/file.txt",
        None
    ));
    assert!(!creation_path_matches(
        "same.txt",
        "/workspace/same.txt",
        None
    ));
    assert_eq!(projected_creation_path("same.txt", None), None);
    assert_eq!(
        workspace_relative_creation_path("/workspace/same.txt", None),
        None
    );
}

#[test]
fn foreign_roots_and_prefix_siblings_cannot_supply_creation_evidence() {
    let root = Some("/workspace/project");
    for observed in [
        "/foreign/project/a.txt",
        "/workspace/project-other/a.txt",
        "/workspace/project",
        "/workspace/project/",
    ] {
        assert!(
            !creation_path_matches("a.txt", observed, root),
            "{observed}"
        );
        assert_eq!(
            workspace_relative_creation_path(observed, root),
            None,
            "{observed}"
        );
    }
}

#[test]
fn lexical_aliases_and_invalid_host_roots_are_not_normalized() {
    let root = Some("/workspace/project");
    for relative in [
        "",
        ".",
        "..",
        "./a.txt",
        "../a.txt",
        "dir/../a.txt",
        "dir/./a.txt",
        "dir//a.txt",
        "a.txt/",
        "/a.txt",
        "C:/a.txt",
        "C:\\a.txt",
        "dir\\a.txt",
        "a\0.txt",
        "a\n.txt",
        "a\r.txt",
    ] {
        assert_eq!(
            projected_creation_path(relative, root),
            None,
            "{relative:?}"
        );
        let observed = format!("/workspace/project/{relative}");
        assert_eq!(
            workspace_relative_creation_path(&observed, root),
            None,
            "{relative:?}"
        );
        assert!(
            !creation_path_matches(relative, &observed, root),
            "{relative:?}"
        );
    }
    for invalid_root in [
        "workspace/project",
        "/workspace/project/",
        "/workspace//project",
        "/workspace/../project",
        "/C:foo",
        "C:/workspace",
        "C:\\workspace",
    ] {
        assert_eq!(
            projected_creation_path("a.txt", Some(invalid_root)),
            None,
            "{invalid_root}"
        );
        assert!(!creation_path_matches(
            "a.txt",
            "/workspace/project/a.txt",
            Some(invalid_root)
        ));
    }
}
