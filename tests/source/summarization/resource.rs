//! Recursive repository-resource formalization and summarization.
//!
//! This layer generalizes the file summarizer in [`super::file`] from "one
//! file" to "any repository resource": a single file **or** a directory
//! (folder) tree of arbitrary depth. It exists because summarizing only files
//! is too specialized — a repository is a recursive tree of folders and files,
//! and the same summarization must apply at every level.
//!
//! A directory is summarized by the project's meta-algorithm loop applied
//! recursively:
//!
//! 1. **Decompose.** A directory is split into its child resources (files and
//!    subdirectories).
//! 2. **Solve each.** Every child is formalized and summarized on its own —
//!    files through [`super::file`], subdirectories by recursing into this same
//!    function.
//! 3. **Compose.** The child summaries are composed back into one directory
//!    summary, together with aggregate metadata (file/subdirectory counts, total
//!    lines and bytes).
//!
//! Recursion depth is bounded by the summarization *mode ladder*
//! ([`SummarizationMode::one_step_shorter`]): each level of nesting is rendered
//! one mode shorter than its parent, so a `Full` directory summary describes its
//! direct children in `Standard` detail, their children in `Short` detail, and
//! everything deeper as a `Topic` label. This keeps deep trees bounded while
//! still surfacing the most important structure first.
//!
//! Everything here is a pure function of its input tree plus the
//! [`SummarizationConfig`]: no filesystem access happens inside this module, so
//! it stays deterministic and testable. Callers that walk a real directory build
//! a [`RepositoryEntry`] tree first and pass it in.

use std::fmt::Write as _;

use crate::links_format::flatten_lino_value;

use super::file::{formalize_repository_file, RepositoryFileFormalization};
use super::{SummarizationConfig, SummarizationMode};

/// Input tree describing a repository resource to formalize.
///
/// This is the deterministic, filesystem-free representation a caller builds
/// before formalization. A real directory walk maps to nested
/// [`RepositoryEntry::Directory`] / [`RepositoryEntry::File`] nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryEntry {
    /// A single file with its repository-relative path and full content.
    File { path: String, content: String },
    /// A directory with its repository-relative path and ordered children.
    Directory { path: String, children: Vec<Self> },
}

impl RepositoryEntry {
    /// Build a file entry.
    #[must_use]
    pub fn file(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self::File {
            path: path.into(),
            content: content.into(),
        }
    }

    /// Build a directory entry from an ordered list of children.
    #[must_use]
    pub fn directory(path: impl Into<String>, children: Vec<Self>) -> Self {
        Self::Directory {
            path: path.into(),
            children,
        }
    }

    /// The repository-relative path of this entry, regardless of kind.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::File { path, .. } | Self::Directory { path, .. } => path,
        }
    }
}

/// Link-native representation of a formalized repository directory.
#[derive(Debug, Clone)]
pub struct RepositoryDirectoryFormalization {
    pub path: String,
    /// Number of files directly inside this directory (non-recursive).
    pub direct_file_count: usize,
    /// Number of subdirectories directly inside this directory (non-recursive).
    pub direct_directory_count: usize,
    /// Number of files anywhere in the subtree (recursive).
    pub total_file_count: usize,
    /// Number of directories anywhere in the subtree, excluding this one.
    pub total_directory_count: usize,
    /// Sum of line counts across every file in the subtree.
    pub total_line_count: usize,
    /// Sum of byte counts across every file in the subtree.
    pub total_byte_count: usize,
    /// Ordered formalized children (files and subdirectories).
    pub children: Vec<RepositoryResourceFormalization>,
}

/// Formalized repository resource: either a file or a directory subtree.
#[derive(Debug, Clone)]
pub enum RepositoryResourceFormalization {
    File(RepositoryFileFormalization),
    Directory(RepositoryDirectoryFormalization),
}

impl RepositoryResourceFormalization {
    /// The repository-relative path of this resource.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::File(file) => &file.path,
            Self::Directory(directory) => &directory.path,
        }
    }

    /// `true` when this resource is a directory.
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        matches!(self, Self::Directory(_))
    }

    /// Render a prose summary for this resource using the supplied config.
    ///
    /// Files defer to [`RepositoryFileFormalization::summary`]; directories use
    /// the recursive decompose → summarize → compose loop documented at the
    /// module level.
    #[must_use]
    pub fn summary(&self, config: &SummarizationConfig) -> String {
        match self {
            Self::File(file) => file.summary(config),
            Self::Directory(directory) => directory.summary(config),
        }
    }

    /// Render the formalized resource as compact indented Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        match self {
            Self::File(file) => file.links_notation(),
            Self::Directory(directory) => directory.links_notation(),
        }
    }
}

impl RepositoryDirectoryFormalization {
    /// Compose a prose summary for this directory by recursively summarizing its
    /// children and joining the results behind an aggregate identity sentence.
    #[must_use]
    pub fn summary(&self, config: &SummarizationConfig) -> String {
        let mut parts = vec![self.identity_sentence()];

        if config.mode == SummarizationMode::Topic {
            // Topic mode is a label, not a body: the identity sentence already
            // carries the path and aggregate counts.
            return parts.remove(0);
        }

        let child_config = config.clone().with_mode(config.mode.one_step_shorter());
        let cap = child_summary_cap(config.mode);
        let mut child_summaries = Vec::new();
        for child in self.children.iter().take(cap) {
            let summary = child.summary(&child_config);
            if !summary.is_empty() {
                child_summaries.push(summary);
            }
        }
        let hidden = self.children.len().saturating_sub(cap);

        if !child_summaries.is_empty() {
            parts.push(format!("Contents: {}", child_summaries.join(" ")));
        }
        if hidden > 0 {
            parts.push(format!(
                "{hidden} more {} omitted for brevity.",
                pluralize(hidden, "entry", "entries")
            ));
        }
        parts.join(" ")
    }

    fn identity_sentence(&self) -> String {
        format!(
            "{} is a repository directory with {} {} and {} {} ({} {} total across {} {}).",
            self.path,
            self.direct_file_count,
            pluralize(self.direct_file_count, "file", "files"),
            self.direct_directory_count,
            pluralize(
                self.direct_directory_count,
                "subdirectory",
                "subdirectories"
            ),
            self.total_line_count,
            pluralize(self.total_line_count, "line", "lines"),
            self.total_file_count,
            pluralize(self.total_file_count, "file", "files"),
        )
    }

    /// Render the formalized directory as compact indented Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("repository_directory\n");
        push_field(&mut out, 1, "path", &self.path);
        push_field(
            &mut out,
            1,
            "direct_file_count",
            &self.direct_file_count.to_string(),
        );
        push_field(
            &mut out,
            1,
            "direct_directory_count",
            &self.direct_directory_count.to_string(),
        );
        push_field(
            &mut out,
            1,
            "total_file_count",
            &self.total_file_count.to_string(),
        );
        push_field(
            &mut out,
            1,
            "total_directory_count",
            &self.total_directory_count.to_string(),
        );
        push_field(
            &mut out,
            1,
            "total_line_count",
            &self.total_line_count.to_string(),
        );
        push_field(
            &mut out,
            1,
            "total_byte_count",
            &self.total_byte_count.to_string(),
        );
        for child in &self.children {
            match child {
                RepositoryResourceFormalization::File(file) => {
                    push_field(&mut out, 1, "file", &file.path);
                }
                RepositoryResourceFormalization::Directory(directory) => {
                    push_field(&mut out, 1, "directory", &directory.path);
                }
            }
        }
        out.trim_end().to_owned()
    }
}

/// Formalize an arbitrary repository resource (file or directory) recursively.
#[must_use]
pub fn formalize_repository_resource(entry: &RepositoryEntry) -> RepositoryResourceFormalization {
    match entry {
        RepositoryEntry::File { path, content } => {
            RepositoryResourceFormalization::File(formalize_repository_file(path, content))
        }
        RepositoryEntry::Directory { path, children } => {
            RepositoryResourceFormalization::Directory(formalize_repository_directory(
                path, children,
            ))
        }
    }
}

/// Formalize a repository directory from its path and ordered child entries.
#[must_use]
pub fn formalize_repository_directory(
    path: &str,
    children: &[RepositoryEntry],
) -> RepositoryDirectoryFormalization {
    let formalized_children: Vec<RepositoryResourceFormalization> =
        children.iter().map(formalize_repository_resource).collect();

    let mut direct_file_count = 0;
    let mut direct_directory_count = 0;
    let mut total_file_count = 0;
    let mut total_directory_count = 0;
    let mut total_line_count = 0;
    let mut total_byte_count = 0;

    for child in &formalized_children {
        match child {
            RepositoryResourceFormalization::File(file) => {
                direct_file_count += 1;
                total_file_count += 1;
                total_line_count += file.line_count;
                total_byte_count += file.byte_count;
            }
            RepositoryResourceFormalization::Directory(directory) => {
                direct_directory_count += 1;
                total_directory_count += 1 + directory.total_directory_count;
                total_file_count += directory.total_file_count;
                total_line_count += directory.total_line_count;
                total_byte_count += directory.total_byte_count;
            }
        }
    }

    RepositoryDirectoryFormalization {
        path: path.to_owned(),
        direct_file_count,
        direct_directory_count,
        total_file_count,
        total_directory_count,
        total_line_count,
        total_byte_count,
        children: formalized_children,
    }
}

/// Summarize any repository resource (file or directory) with the supplied
/// configuration. This is the general entry point that subsumes
/// [`super::file::summarize_repository_file`].
#[must_use]
pub fn summarize_repository_resource(
    entry: &RepositoryEntry,
    config: &SummarizationConfig,
) -> String {
    formalize_repository_resource(entry).summary(config)
}

/// How many direct children a directory summary lists in prose, per mode. Deeper
/// detail is reached by recursion at a shorter mode, not by listing more
/// children, so this stays small enough to keep summaries readable.
const fn child_summary_cap(mode: SummarizationMode) -> usize {
    match mode {
        SummarizationMode::Topic => 0,
        SummarizationMode::Short => 2,
        SummarizationMode::Standard => 4,
        SummarizationMode::Full | SummarizationMode::Expand => usize::MAX,
    }
}

const fn pluralize(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

fn push_field(out: &mut String, indent: usize, name: &str, value: &str) {
    for _ in 0..indent {
        out.push_str("  ");
    }
    let _ = writeln!(out, "{name} {}", flatten_lino_value(value));
}
