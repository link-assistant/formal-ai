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
