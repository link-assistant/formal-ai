//! Durable history storage and incremental watermarks (issue #1180).

use super::commits::run_git;
use super::github::import_issues_and_pulls_inner;
use super::{
    BTreeSet, CURSOR_ROOT, HistoryRules, MemoryEvent, Path, PathBuf, RepositoryHistoryImportError,
    effective_record, escape_value, fs, import_ci_runs, import_commits, parse_lino,
};

/// Append `events` to the store at `store_path`, skipping ids already
/// present, and return how many were appended.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when the store cannot be read
/// or written.
pub fn write_repository_history(
    events: &[MemoryEvent],
    store_path: &Path,
) -> Result<usize, RepositoryHistoryImportError> {
    let io = |error: std::io::Error| {
        RepositoryHistoryImportError::with_detail("memory_store_io", format!("{error}"))
    };
    let mut store = crate::memory::MemoryStore::load_from_file(store_path).map_err(io)?;
    let existing: BTreeSet<String> = store
        .events()
        .iter()
        .map(|event| event.id.clone())
        .collect();
    let mut appended = 0;
    for event in events {
        if existing.contains(&event.id) {
            continue;
        }
        store.append(event.clone());
        appended += 1;
    }
    store.save_to_file(store_path).map_err(io)?;
    Ok(appended)
}

/// The incremental watermark: the last commit sha, the newest issue/PR
/// `updatedAt`, and the highest run `databaseId` this importer has stored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryHistoryCursor {
    pub last_commit_sha: Option<String>,
    pub last_issue_pr_updated_at: Option<String>,
    pub last_ci_run_database_id: Option<u64>,
}

impl RepositoryHistoryCursor {
    /// Read the cursor document at `path`; a missing file is a fresh start.
    #[must_use]
    pub fn read(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let tree = parse_lino(&text);
        let Some(node) = tree.children.iter().find(|child| child.name == CURSOR_ROOT) else {
            return Self::default();
        };
        Self {
            last_commit_sha: optional_value(node.find_child_value("last_commit_sha")),
            last_issue_pr_updated_at: optional_value(
                node.find_child_value("last_issue_pr_updated_at"),
            ),
            last_ci_run_database_id: node
                .find_child_value("last_ci_run_database_id")
                .parse::<u64>()
                .ok(),
        }
    }

    /// Write the cursor document to `path`, creating parent directories.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryHistoryImportError`] on a filesystem failure.
    pub fn write(&self, path: &Path) -> Result<(), RepositoryHistoryImportError> {
        fn row(out: &mut String, name: &str, value: &str) {
            use std::fmt::Write as _;
            let _ = writeln!(out, "  {} \"{}\"", name, escape_value(value));
        }
        let mut out = format!("{CURSOR_ROOT}\n");
        if let Some(sha) = &self.last_commit_sha {
            row(&mut out, "last_commit_sha", sha);
        }
        if let Some(updated) = &self.last_issue_pr_updated_at {
            row(&mut out, "last_issue_pr_updated_at", updated);
        }
        if let Some(run) = self.last_ci_run_database_id {
            row(&mut out, "last_ci_run_database_id", &run.to_string());
        }
        let io = |error: std::io::Error| {
            RepositoryHistoryImportError::with_detail("cursor_write_failed", format!("{error}"))
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        fs::write(path, out).map_err(io)
    }

    /// Advance over chronologically imported events: the last commit wins,
    /// and run ids take the maximum.
    pub fn advance(&mut self, events: &[MemoryEvent], rules: &HistoryRules) {
        let defaults = HistoryRules::defaults();
        let commit = effective_record(rules, "commit", &defaults);
        let ci = effective_record(rules, "ci_run", &defaults);
        for event in events {
            let kind = event.kind.as_deref();
            if kind == Some(commit.kind.as_str()) {
                if let Some(sha) = event.id.strip_prefix(commit.id_prefix.as_str()) {
                    self.last_commit_sha = Some(sha.to_owned());
                }
            } else if kind == Some(ci.kind.as_str())
                && let Some(number) = event
                    .id
                    .strip_prefix(ci.id_prefix.as_str())
                    .and_then(|number| number.parse::<u64>().ok())
            {
                self.last_ci_run_database_id = Some(
                    self.last_ci_run_database_id
                        .map_or(number, |current| current.max(number)),
                );
            }
        }
    }
}

fn optional_value(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

/// The `owner-name` slug of the repository at `repo_root`, from its origin
/// remote; `local-repository` when there is none.
#[must_use]
pub fn repository_slug(repo_root: &Path) -> String {
    let Ok(url) = run_git(repo_root, &["config", "--get", "remote.origin.url"]) else {
        return String::from("local-repository");
    };
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::from("local-repository");
    }
    let without_git = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    let segments: Vec<&str> = without_git
        .split(['/', ':'])
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.len() >= 2 {
        return format!(
            "{}-{}",
            segments[segments.len() - 2],
            segments[segments.len() - 1]
        );
    }
    String::from("local-repository")
}

/// Where this importer persists a repository's history store and cursor.
#[must_use]
pub fn store_paths(memory_dir: &Path, repo_root: &Path) -> (PathBuf, PathBuf) {
    let base = memory_dir
        .join("repository-history")
        .join(repository_slug(repo_root));
    (base.join("events.lino"), base.join("cursor.lino"))
}

/// One incremental import pass.
///
/// Commits from the working repository, then
/// issues/pull requests/reviews and Actions runs from a github-logs
/// directory (when given), appended to the store under `memory_dir`.
/// Returns the number of events appended.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when a source or the store fails.
pub fn import_incremental(
    repo_root: &Path,
    logs_dir: Option<&Path>,
    memory_dir: &Path,
    rules: &HistoryRules,
) -> Result<usize, RepositoryHistoryImportError> {
    let (store_path, cursor_path) = store_paths(memory_dir, repo_root);
    let mut cursor = RepositoryHistoryCursor::read(&cursor_path);
    let mut events = import_commits(repo_root, cursor.last_commit_sha.as_deref(), rules)?;
    let mut issue_watermark = cursor.last_issue_pr_updated_at.clone();
    if let Some(dir) = logs_dir {
        let (imported, watermark) =
            import_issues_and_pulls_inner(dir, cursor.last_issue_pr_updated_at.as_deref(), rules)?;
        events.extend(imported);
        if watermark.is_some() {
            issue_watermark = watermark;
        }
        events.extend(import_ci_runs(dir, cursor.last_ci_run_database_id, rules)?);
    }
    let appended = write_repository_history(&events, &store_path)?;
    cursor.advance(&events, rules);
    cursor.last_issue_pr_updated_at = issue_watermark;
    cursor.write(&cursor_path)?;
    Ok(appended)
}
