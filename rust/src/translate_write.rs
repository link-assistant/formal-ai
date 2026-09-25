//! The `--write` half of plan 16 L2g: turning a rendered translation into
//! the committed sibling tree (issue #1138).
//!
//! The pure mapping ([`crate::meta_translate::write_target`]) lives beside
//! the dispatcher so the wasm surface keeps it; this module is the std half
//! the CLI and the agent driver share — reading the source root, refusing
//! transactionally, and writing only a complete target tree. A half-written
//! `./ts` is exactly the corrupt state the L3 mismatch check exists to
//! refuse, so a tree write that cannot finish writes nothing at all.

use std::path::{Path, PathBuf};

use crate::es_meta::Refusal;
use crate::meta_translate::{self, SourceRoot, TranslationOutcome, WriteTargetError};

/// The per-file refusal line every report carries — the file and its named
/// constructs, grounded in the seed (`translate_write_refused_item`) so the
/// item text is not a source literal (R379).
#[must_use]
pub fn refusal_items(repo_relative: &str, refusals: &[Refusal]) -> Vec<String> {
    let constructs = refusals
        .iter()
        .map(|refusal| refusal.construct.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        crate::seed::render_response(
            "translate_write_refused_item",
            "en",
            &[("source", repo_relative), ("constructs", &constructs)],
        )
        .unwrap_or_else(|| "translate_write_refused_item".to_owned()),
    ]
}

/// One file a write run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenFile {
    /// The repo-relative source path.
    pub source: String,
    /// The repo-relative target path that was written.
    pub target: String,
    /// How many tokens the pivot carried for this file.
    pub carried: usize,
}

/// What a write run produced, rendered from the seed by [`WriteReport::intent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteReport {
    /// Every file translated and written into the `to` root.
    Wrote {
        files: Vec<WrittenFile>,
        to: SourceRoot,
    },
    /// One mapped file (the `--input` form).
    WroteOne(WrittenFile),
    /// The source path is not a file under the source root the mapping owns.
    WrongRoot { path: String },
    /// The leg's target is not a committed tree `--write` maps into.
    UnsupportedLeg { from: SourceRoot, to: SourceRoot },
    /// Named constructs the projection seed refuses; nothing was written.
    Refused { items: Vec<String> },
    /// A source file could not be normalized into the pivot.
    Invalid { path: String, reason: String },
    /// A source file named by the request is not there to read.
    Missing { path: String },
    /// The source root holds none of the files the leg owns.
    Empty { root: SourceRoot },
}

impl WriteReport {
    /// The seed intent id and its placeholder values, so every surface
    /// renders the same report through the same seed rows.
    #[must_use]
    pub fn intent(&self) -> (&'static str, Vec<(&'static str, String)>) {
        match self {
            Self::Wrote { files, to } => {
                let tokens = files.iter().map(|file| file.carried).sum::<usize>();
                (
                    "translate_wrote",
                    vec![
                        ("files", files.len().to_string()),
                        ("tokens", tokens.to_string()),
                        ("to", to.name().to_owned()),
                    ],
                )
            }
            Self::WroteOne(file) => (
                "translate_wrote_file",
                vec![
                    ("source", file.source.clone()),
                    ("target", file.target.clone()),
                    ("carried", file.carried.to_string()),
                ],
            ),
            Self::WrongRoot { path } => {
                ("translate_write_wrong_root", vec![("path", path.clone())])
            }
            Self::UnsupportedLeg { from, to } => (
                "translate_write_unsupported",
                vec![
                    ("from", from.name().to_owned()),
                    ("to", to.name().to_owned()),
                ],
            ),
            Self::Refused { items } => {
                ("translate_write_refused", vec![("items", items.join("; "))])
            }
            Self::Invalid { path, reason } => (
                "translate_source_invalid",
                vec![("path", path.clone()), ("reason", reason.clone())],
            ),
            Self::Missing { path } => ("translate_source_missing", vec![("path", path.clone())]),
            Self::Empty { root } => (
                "translate_write_empty",
                vec![("root", root.name().to_owned())],
            ),
        }
    }

    /// Whether the run failed to produce its output.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        !matches!(self, Self::Wrote { .. } | Self::WroteOne(_))
    }
}

/// Translate `repo_relative` and write it at its mapped sibling path under
/// `root` (the repository root the path is relative to).
pub fn write_one(
    from: SourceRoot,
    to: SourceRoot,
    root: &Path,
    repo_relative: &str,
) -> WriteReport {
    let target = match meta_translate::write_target(from, to, repo_relative) {
        Ok(target) => target,
        Err(WriteTargetError::WrongRoot { path }) => return WriteReport::WrongRoot { path },
        Err(WriteTargetError::UnsupportedLeg { from, to }) => {
            return WriteReport::UnsupportedLeg { from, to };
        }
    };
    let report = render_one(root, repo_relative, from, to).and_then(|(text, carried)| {
        write_target_file(root, &target, &text).map(|()| WrittenFile {
            source: repo_relative.to_owned(),
            target: target.to_owned(),
            carried,
        })
    });
    match report {
        Ok(file) => WriteReport::WroteOne(file),
        Err(report) => report,
    }
}

/// Translate every owned file under `root/<from>/` and write the whole
/// sibling tree — or, on any refusal or invalid file, nothing at all.
pub fn write_tree(from: SourceRoot, to: SourceRoot, root: &Path) -> WriteReport {
    let extension = match from.owned_extension() {
        Some(extension) => extension,
        None => {
            return WriteReport::UnsupportedLeg { from, to };
        }
    };
    let source_root = root.join(from.directory());
    let mut sources = Vec::new();
    collect_owned_files(&source_root, extension, &mut sources);
    if sources.is_empty() {
        return WriteReport::Empty { root: from };
    }
    let mut rendered = Vec::new();
    for path in &sources {
        let repo_relative = path
            .strip_prefix(root)
            .map(|relative| relative.display().to_string())
            .unwrap_or_default();
        let target = match meta_translate::write_target(from, to, &repo_relative) {
            Ok(target) => target,
            Err(WriteTargetError::WrongRoot { path }) => {
                return WriteReport::WrongRoot { path };
            }
            Err(WriteTargetError::UnsupportedLeg { from, to }) => {
                return WriteReport::UnsupportedLeg { from, to };
            }
        };
        match render_one(root, &repo_relative, from, to) {
            Ok((text, carried)) => rendered.push((repo_relative, target, text, carried)),
            Err(report) => return report,
        }
    }
    // Every file rendered: only now does the first target byte hit the tree,
    // so a refusal anywhere above leaves no half-generated sibling behind.
    let mut written = Vec::new();
    for (repo_relative, target, text, carried) in rendered {
        if let Err(report) = write_target_file(root, &target, &text) {
            return report;
        }
        written.push(WrittenFile {
            source: repo_relative,
            target,
            carried,
        });
    }
    WriteReport::Wrote { files: written, to }
}

/// Read and translate one file without touching the tree, so a tree write
/// can render every file before committing the first byte.
fn render_one(
    root: &Path,
    repo_relative: &str,
    from: SourceRoot,
    to: SourceRoot,
) -> Result<(String, usize), WriteReport> {
    let Ok(source) = std::fs::read_to_string(root.join(repo_relative)) else {
        return Err(WriteReport::Missing {
            path: repo_relative.to_owned(),
        });
    };
    match meta_translate::translate(from, to, repo_relative, &source) {
        TranslationOutcome::Rendered { target, carried } => Ok((target, carried)),
        TranslationOutcome::Refused { refusals } => Err(WriteReport::Refused {
            items: refusal_items(repo_relative, &refusals),
        }),
        TranslationOutcome::Pending { .. } => Err(WriteReport::UnsupportedLeg { from, to }),
        TranslationOutcome::Invalid { reason } => Err(WriteReport::Invalid {
            path: repo_relative.to_owned(),
            reason,
        }),
    }
}

/// Create the target's parent directories and write the rendered text.
fn write_target_file(root: &Path, target: &str, text: &str) -> Result<(), WriteReport> {
    let target_path = root.join(target);
    if let Some(parent) = target_path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        return Err(WriteReport::Invalid {
            path: target.to_owned(),
            reason: error.to_string(),
        });
    }
    match std::fs::write(&target_path, text) {
        Ok(()) => Ok(()),
        Err(error) => Err(WriteReport::Invalid {
            path: target.to_owned(),
            reason: error.to_string(),
        }),
    }
}

fn collect_owned_files(directory: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_owned_files(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    out.sort();
}
