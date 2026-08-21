use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceChange {
    pub path: String,
    pub kind: WorkspaceChangeKind,
    pub before_sha256: Option<String>,
    pub after_sha256: Option<String>,
    pub bytes_changed: u64,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    files: BTreeMap<String, FileState>,
}

#[derive(Debug, Clone)]
struct FileState {
    sha256: String,
    bytes: u64,
}

pub fn snapshot(root: &Path) -> io::Result<Snapshot> {
    let mut files = BTreeMap::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !ignored(entry.path(), root))
    {
        let entry = entry.map_err(io::Error::other)?;
        if entry.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("workspace_symlink:{}", entry.path().display()),
            ));
        }
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("workspace_special_file:{}", entry.path().display()),
            ));
        }
        let bytes = fs::read(entry.path())?;
        let relative = relative_path(root, entry.path())?;
        files.insert(
            relative,
            FileState {
                sha256: sha256(&bytes),
                bytes: bytes.len() as u64,
            },
        );
    }
    Ok(Snapshot { files })
}

pub fn changes(before: &Snapshot, after: &Snapshot) -> Vec<WorkspaceChange> {
    let paths = before
        .files
        .keys()
        .chain(after.files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    paths
        .into_iter()
        .filter_map(|path| {
            let old = before.files.get(&path);
            let new = after.files.get(&path);
            if old.map(|state| &state.sha256) == new.map(|state| &state.sha256) {
                return None;
            }
            let kind = match (old, new) {
                (None, Some(_)) => WorkspaceChangeKind::Added,
                (Some(_), None) => WorkspaceChangeKind::Removed,
                (Some(_), Some(_)) => WorkspaceChangeKind::Modified,
                (None, None) => return None,
            };
            Some(WorkspaceChange {
                path,
                kind,
                before_sha256: old.map(|state| state.sha256.clone()),
                after_sha256: new.map(|state| state.sha256.clone()),
                bytes_changed: old.map_or(0, |state| state.bytes)
                    + new.map_or(0, |state| state.bytes),
            })
        })
        .collect()
}

pub(super) fn copy_workspace(source: &Path, destination: &Path, excluded: &Path) -> io::Result<()> {
    if destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("candidate_workspace_exists:{}", destination.display()),
        ));
    }
    fs::create_dir_all(destination)?;
    for entry in WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !ignored(entry.path(), source) && !entry.path().starts_with(excluded))
    {
        let entry = entry.map_err(io::Error::other)?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(io::Error::other)?;
        if relative.as_os_str().is_empty() {
            continue;
        }
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), target)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("workspace_symlink:{}", entry.path().display()),
            ));
        }
    }
    Ok(())
}

pub(super) fn apply_changes(
    original: &Path,
    candidate: &Path,
    selected: &[WorkspaceChange],
) -> io::Result<()> {
    validate_changes(original, candidate, selected)?;
    for change in selected {
        let relative = safe_relative(&change.path)?;
        let target = original.join(&relative);
        match change.kind {
            WorkspaceChangeKind::Removed => {
                if target.exists() {
                    fs::remove_file(target)?;
                }
            }
            WorkspaceChangeKind::Added | WorkspaceChangeKind::Modified => {
                let source = candidate.join(relative);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(source, target)?;
            }
        }
    }
    Ok(())
}

pub(super) fn validate_changes(
    original: &Path,
    candidate: &Path,
    selected: &[WorkspaceChange],
) -> io::Result<()> {
    for change in selected {
        validate_change_shape(change)?;
        let relative = safe_relative(&change.path)?;
        require_hash(
            &original.join(&relative),
            change.before_sha256.as_deref(),
            "workspace_drift",
            &change.path,
        )?;
        require_hash(
            &candidate.join(relative),
            change.after_sha256.as_deref(),
            "candidate_drift",
            &change.path,
        )?;
    }
    Ok(())
}

fn validate_change_shape(change: &WorkspaceChange) -> io::Result<()> {
    let valid = matches!(
        (
            &change.kind,
            change.before_sha256.is_some(),
            change.after_sha256.is_some(),
        ),
        (WorkspaceChangeKind::Added, false, true)
            | (WorkspaceChangeKind::Modified, true, true)
            | (WorkspaceChangeKind::Removed, true, false)
    );
    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid_workspace_change:{}", change.path),
        ))
    }
}

fn require_hash(
    path: &Path,
    expected: Option<&str>,
    error_kind: &str,
    relative: &str,
) -> io::Result<()> {
    let actual = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Some(sha256(&fs::read(path)?)),
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{error_kind}:{relative}"),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if actual.as_deref() == expected {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{error_kind}:{relative}"),
        ))
    }
}

fn ignored(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        relative.components().next().is_some_and(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some(".git" | "target" | ".formal-ai" | ".formal-ai-orchestration")
            )
        })
    })
}

fn relative_path(root: &Path, path: &Path) -> io::Result<String> {
    let relative = path.strip_prefix(root).map_err(io::Error::other)?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn safe_relative(path: &str) -> io::Result<PathBuf> {
    let value = Path::new(path);
    if value.is_absolute()
        || value
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("workspace_escape:{path}"),
        ));
    }
    Ok(value.to_path_buf())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
