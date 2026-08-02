use std::path::{Path, PathBuf};

/// An unforgeable-by-default capability bound to one workspace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentRunPermission {
    workspace: Option<PathBuf>,
}

impl AgentRunPermission {
    /// Explicitly grants external-process access to `workspace`.
    #[must_use]
    pub fn grant_for(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: Some(workspace.into()),
        }
    }

    pub(crate) fn permits(&self, workspace: &Path) -> bool {
        self.workspace
            .as_deref()
            .is_some_and(|granted| same_path(granted, workspace))
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
