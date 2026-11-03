//! Exact lexical workspace bindings for creation evidence; no filesystem authority is inferred.

fn clean_segments(path: &str) -> bool {
    !path.is_empty()
        && !path.chars().any(|c| matches!(c, '\\' | '\0' | '\r' | '\n'))
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

fn safe_relative(path: &str) -> bool {
    clean_segments(path)
        && !path.starts_with('/')
        && !(path.as_bytes().get(1) == Some(&b':') && path.as_bytes()[0].is_ascii_alphabetic())
}

fn valid_host_root(root: &str) -> bool {
    root == "/" || root.strip_prefix('/').is_some_and(safe_relative)
}

pub(super) fn projected_creation_path(relative: &str, host_root: Option<&str>) -> Option<String> {
    let root = host_root.filter(|root| valid_host_root(root))?;
    if !safe_relative(relative) {
        return None;
    }
    Some(if root == "/" {
        format!("/{relative}")
    } else {
        format!("{root}/{relative}")
    })
}

pub(super) fn workspace_relative_creation_path(
    observed: &str,
    host_root: Option<&str>,
) -> Option<String> {
    let root = host_root.filter(|root| valid_host_root(root))?;
    let prefix = if root == "/" {
        "/".to_owned()
    } else {
        format!("{root}/")
    };
    let relative = observed.strip_prefix(&prefix)?;
    (projected_creation_path(relative, host_root).as_deref() == Some(observed))
        .then(|| relative.to_owned())
}

pub(super) fn creation_path_matches(
    logical: &str,
    observed: &str,
    host_root: Option<&str>,
) -> bool {
    logical == observed
        || projected_creation_path(logical, host_root).as_deref() == Some(observed)
        || projected_creation_path(observed, host_root).as_deref() == Some(logical)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
