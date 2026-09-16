//! `formal-ai solve` — one repository task, end to end, refusing to commit by
//! default (#1138 B7, plan 03 L12–L13).
//!
//! Mutation is opt-in: without `--commit` the diff is printed and the tree is
//! left alone. When a commit is made it carries the same four self-hosting
//! trailers `scripts/author-change-with-formal-ai.sh` already writes and
//! `scripts/self-hosting-metric.rs` already reads, and only `formal-ai` is an
//! authoring path — a hosted model is not.
//!
//! Wave T lands the shapes only; wave I7 leaves 03-L12 and 03-L13 fill the
//! bodies in.

use std::error::Error;
use std::path::PathBuf;

/// Arguments for `formal-ai solve`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveArgs {
    /// A GitHub issue URL, or `-` to read the requirement from stdin.
    pub issue: Option<String>,
    /// Literal requirement text, when no issue is given.
    pub task: Option<String>,
    /// Repository to work in (default: the current checkout).
    pub repository: String,
    /// Base commit (default: `HEAD` of `--repository`).
    pub base_commit: Option<String>,
    /// Which model authors the change. Only `formal-ai` is an authoring path.
    pub model: String,
    /// Where the run's raw traces are committed.
    pub evidence: PathBuf,
    /// The pull request the commit belongs to.
    pub pull_request: Option<String>,
    /// Refuse to commit; print the diff instead. Default-deny for mutation.
    pub commit: bool,
}

impl Default for SolveArgs {
    fn default() -> Self {
        Self {
            issue: None,
            task: None,
            repository: String::from("."),
            base_commit: None,
            model: String::from("formal-ai"),
            evidence: PathBuf::new(),
            pull_request: None,
            commit: false,
        }
    }
}

/// What one `solve` run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveOutcome {
    /// The unified diff, whatever it is.
    pub diff: String,
    /// Whether a commit was actually made.
    pub committed: bool,
    /// The commit message, including its trailers, when one was written.
    pub commit_message: String,
    /// Files written under `--evidence`.
    pub evidence_files: Vec<PathBuf>,
    /// Protocol steps whose postcondition was not observed.
    pub open: Vec<String>,
}

/// Run one repository task end to end and, when `--commit` is given, land it
/// with the four self-hosting trailers.
///
/// # Errors
/// Any protocol step whose postcondition was not observed aborts before the
/// commit; a partial tree is never committed. A `--model` that is not an
/// authoring path is refused before anything is written.
pub fn run_solve(_args: &SolveArgs) -> Result<SolveOutcome, Box<dyn Error>> {
    todo!("plan 03 leaf L12")
}
