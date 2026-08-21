//! Issue #946 (E94): versioned recoverable memory with an immutable baseline.
//!
//! Issue #873 asks that "each state of memory should be recoverable, so if
//! compilation of next version of itself fails for formal AI debugging continues
//! from previous stable and tested version". [`crate::promotion`] already decides
//! *whether* a self-authored change may land; what it had no layer for is what
//! happens when the change lands and then does not build. Without one, a failed
//! self-compile leaves the workspace holding a version that neither compiles nor
//! has a recorded predecessor to go back to.
//!
//! This module is that layer. A [`RevisionLedger`] captures the bytes of every
//! tracked file *before* a candidate version is written, and restores those exact
//! bytes when the candidate fails:
//!
//! ```text
//! capture   the stable revision: tracked file bytes + the baseline pin
//!   mutate  write the candidate version over them
//!   check   does it compile, and does the whole baseline still pass?
//!     yes   adopt: the candidate becomes the new stable revision
//!     no    restore every tracked file to the captured bytes
//! ```
//!
//! Three things make the rollback trustworthy rather than merely present:
//!
//! 1. **The snapshot precedes the mutation.** A ledger that snapshots afterwards
//!    records the broken state, which is the state nobody wants back.
//! 2. **The baseline is pinned by digest, not by trust.** A candidate that edits
//!    a baseline test file is rolled back *before* its verdict is consulted, so a
//!    version can never pass by weakening the tests that judge it.
//! 3. **Adoption needs a full pass, not the absence of a failure.** A verdict
//!    reporting zero baseline tests is treated as a rollback, exactly as
//!    [`crate::promotion::PromotionProposal::passes_all_gates`] refuses a
//!    proposal with no gates.
//!
//! The module holds no prose: a caller that needs to explain a rollback looks the
//! wording up by [`RollbackReason::slug`] (R379).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest as _, Sha256};

use crate::engine::stable_id;
use crate::memory::MemoryEvent;

/// The Rust edition this crate is written in, read from `Cargo.toml` by
/// `build.rs` so the manifest stays the only place it is written down.
const CRATE_EDITION: &str = env!("FORMAL_AI_CRATE_EDITION");

/// Content digest of `bytes`, in the hexadecimal form the baseline pin records.
fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// The digests of the test files a version is not allowed to change.
///
/// Issue #946 asks for "most tests immutable as a baseline", never weakened to
/// make a failing version pass. A pin is the cheapest honest form of that: the
/// bytes of each baseline file at the moment the stable revision was captured. A
/// candidate that rewrites one of them has changed the judge, and
/// [`RevisionLedger::attempt`] rolls it back without asking how it scored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BaselinePin {
    entries: BTreeMap<String, String>,
}

impl BaselinePin {
    /// Pin the current bytes of every path in `paths`, relative to `root`.
    ///
    /// A path that does not exist is pinned as absent, so a candidate that
    /// *deletes* a baseline file drifts just as loudly as one that edits it.
    #[must_use]
    pub fn record(root: &Path, paths: &[String]) -> Self {
        let entries = paths
            .iter()
            .map(|path| {
                let bytes = fs::read(root.join(path)).unwrap_or_default();
                (path.clone(), digest(&bytes))
            })
            .collect();
        Self { entries }
    }

    /// The pinned paths, in a stable order.
    #[must_use]
    pub fn paths(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// How many paths this pin covers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether this pin covers nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The pinned paths whose bytes under `root` no longer match the pin.
    #[must_use]
    pub fn drift(&self, root: &Path) -> Vec<String> {
        self.entries
            .iter()
            .filter(|(path, pinned)| {
                let bytes = fs::read(root.join(path)).unwrap_or_default();
                digest(&bytes) != **pinned
            })
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Whether every pinned file is byte-identical to the pin.
    #[must_use]
    pub fn holds(&self, root: &Path) -> bool {
        self.drift(root).is_empty()
    }
}

/// One tracked file's exact bytes at the moment a revision was captured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionFile {
    /// Path relative to the ledger's root.
    pub path: String,
    /// The file's bytes, or `None` when the file did not exist.
    pub bytes: Option<Vec<u8>>,
}

/// A recoverable state of memory: the bytes of every tracked file, plus the
/// baseline pin that was in force when they were captured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRevision {
    /// Content-addressed revision id.
    pub id: String,
    /// The revision this one was captured from, if any.
    pub parent: Option<String>,
    /// Every tracked file, in the order the ledger tracks them.
    pub files: Vec<RevisionFile>,
    /// The baseline digests in force at capture time.
    pub baseline: BaselinePin,
}

impl MemoryRevision {
    /// Capture the current bytes of `tracked` and pin `baseline_paths`.
    #[must_use]
    pub fn capture(
        root: &Path,
        tracked: &[String],
        baseline_paths: &[String],
        parent: Option<String>,
    ) -> Self {
        let files: Vec<RevisionFile> = tracked
            .iter()
            .map(|path| RevisionFile {
                path: path.clone(),
                bytes: fs::read(root.join(path)).ok(),
            })
            .collect();
        let baseline = BaselinePin::record(root, baseline_paths);
        let fingerprint = files
            .iter()
            .map(|file| {
                format!(
                    "{}:{}",
                    file.path,
                    file.bytes
                        .as_deref()
                        .map_or_else(|| String::from("absent"), digest)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            id: stable_id("memory_revision", &fingerprint),
            parent,
            files,
            baseline,
        }
    }

    /// Write every tracked file back to the bytes this revision holds.
    ///
    /// A file the revision recorded as absent is removed, so a candidate that
    /// *added* a file leaves nothing behind after a rollback. This is what makes
    /// "the prior state is restored" a claim about the whole tracked set rather
    /// than about the files that happened to already exist.
    ///
    /// # Errors
    ///
    /// Returns the first filesystem error encountered while restoring.
    pub fn restore(&self, root: &Path) -> io::Result<()> {
        for file in &self.files {
            let target = root.join(&file.path);
            match &file.bytes {
                Some(bytes) => {
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&target, bytes)?;
                }
                None => match fs::remove_file(&target) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                },
            }
        }
        Ok(())
    }

    /// Whether the workspace under `root` currently matches this revision byte
    /// for byte.
    #[must_use]
    pub fn matches(&self, root: &Path) -> bool {
        self.files
            .iter()
            .all(|file| fs::read(root.join(&file.path)).ok() == file.bytes)
    }
}

/// What a version check observed about a candidate version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VersionVerdict {
    /// Whether the candidate compiled.
    pub compiled: bool,
    /// Compiler diagnostics, kept verbatim so a rollback can be explained.
    pub diagnostics: String,
    /// Baseline specifications that passed.
    pub baseline_passed: usize,
    /// Baseline specifications that failed.
    pub baseline_failed: usize,
}

impl VersionVerdict {
    /// A verdict describing a version that compiled and cleared `passed` specs.
    #[must_use]
    pub const fn green(passed: usize) -> Self {
        Self {
            compiled: true,
            diagnostics: String::new(),
            baseline_passed: passed,
            baseline_failed: 0,
        }
    }

    /// Whether this verdict permits a version switch: it compiled, and every
    /// baseline specification passed. A verdict with no baseline at all does not
    /// clear -- an absence of failures is not a pass.
    #[must_use]
    pub const fn permits_switch(&self) -> bool {
        self.compiled && self.baseline_failed == 0 && self.baseline_passed > 0
    }
}

/// Why a candidate version was rolled back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackReason {
    /// The candidate did not compile.
    CompileFailed,
    /// The candidate compiled, but the baseline did not fully pass.
    BaselineFailed,
    /// The candidate changed a pinned baseline file, so its verdict was never
    /// consulted.
    BaselineWeakened,
}

impl RollbackReason {
    /// Stable identifier a caller logs or looks a wording up by.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::CompileFailed => "memory_revision_compile_failed",
            Self::BaselineFailed => "memory_revision_baseline_failed",
            Self::BaselineWeakened => "memory_revision_baseline_weakened",
        }
    }
}

/// How one attempt at a new version resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptOutcome {
    /// The candidate became the new stable revision.
    Adopted {
        /// Id of the adopted revision.
        revision: String,
    },
    /// The candidate was discarded and the prior stable revision restored.
    RolledBack {
        /// Id of the revision the workspace was restored to.
        restored: String,
        /// Why the candidate did not survive.
        reason: RollbackReason,
        /// The baseline paths the candidate changed, when that was the reason.
        weakened: Vec<String>,
    },
}

impl AttemptOutcome {
    /// Stable slug used in serialization and in memory events.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Adopted { .. } => "adopted",
            Self::RolledBack { .. } => "rolled_back",
        }
    }
}

/// The append-only chain of recoverable memory states for one workspace.
#[derive(Debug, Clone)]
pub struct RevisionLedger {
    root: PathBuf,
    tracked: Vec<String>,
    baseline_paths: Vec<String>,
    revisions: Vec<MemoryRevision>,
    stable: usize,
    attempts: Vec<AttemptOutcome>,
}

impl RevisionLedger {
    /// Open a ledger over `root`, capturing the current state as revision zero.
    ///
    /// Revision zero is stable by definition: it is what the workspace was doing
    /// before this module was asked to watch it, and it is the state a first
    /// failed candidate goes back to.
    #[must_use]
    pub fn open(root: &Path, tracked: &[String], baseline_paths: &[String]) -> Self {
        let first = MemoryRevision::capture(root, tracked, baseline_paths, None);
        Self {
            root: root.to_path_buf(),
            tracked: tracked.to_vec(),
            baseline_paths: baseline_paths.to_vec(),
            revisions: vec![first],
            stable: 0,
            attempts: Vec::new(),
        }
    }

    /// The last revision that compiled and cleared the whole baseline.
    #[must_use]
    pub fn stable(&self) -> &MemoryRevision {
        &self.revisions[self.stable]
    }

    /// Every revision recorded so far, oldest first.
    #[must_use]
    pub fn revisions(&self) -> &[MemoryRevision] {
        &self.revisions
    }

    /// Every attempt resolved so far, oldest first.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptOutcome] {
        &self.attempts
    }

    /// Write a candidate version, check it, and adopt or roll back.
    ///
    /// `mutate` writes the candidate into the workspace; `check` decides whether
    /// it may be switched to. The prior stable state is captured before `mutate`
    /// runs, which is the whole point: a ledger that snapshots afterwards records
    /// the broken version.
    ///
    /// # Errors
    ///
    /// Returns the error `mutate` reported, or the first filesystem error hit
    /// while restoring after a rollback.
    pub fn attempt<M, C>(&mut self, mutate: M, check: C) -> io::Result<AttemptOutcome>
    where
        M: FnOnce(&Path) -> io::Result<()>,
        C: FnOnce(&Path) -> VersionVerdict,
    {
        let previous = self.stable().clone();
        mutate(&self.root)?;

        let weakened = previous.baseline.drift(&self.root);
        let outcome = if weakened.is_empty() {
            let verdict = check(&self.root);
            if verdict.permits_switch() {
                let adopted = MemoryRevision::capture(
                    &self.root,
                    &self.tracked,
                    &self.baseline_paths,
                    Some(previous.id.clone()),
                );
                let revision = adopted.id.clone();
                self.revisions.push(adopted);
                self.stable = self.revisions.len() - 1;
                AttemptOutcome::Adopted { revision }
            } else {
                AttemptOutcome::RolledBack {
                    restored: previous.id.clone(),
                    reason: if verdict.compiled {
                        RollbackReason::BaselineFailed
                    } else {
                        RollbackReason::CompileFailed
                    },
                    weakened: Vec::new(),
                }
            }
        } else {
            AttemptOutcome::RolledBack {
                restored: previous.id.clone(),
                reason: RollbackReason::BaselineWeakened,
                weakened,
            }
        };

        if matches!(outcome, AttemptOutcome::RolledBack { .. }) {
            previous.restore(&self.root)?;
        }
        self.attempts.push(outcome.clone());
        Ok(outcome)
    }

    /// The ledger as an append-only chain of memory events, so a recovery trail
    /// round-trips through the bundle export/import path exactly like a
    /// promotion decision does.
    #[must_use]
    pub fn memory_events(&self) -> Vec<MemoryEvent> {
        let mut events = Vec::with_capacity(self.revisions.len() + self.attempts.len());
        for revision in &self.revisions {
            events.push(MemoryEvent {
                id: stable_id("memory_revision", &revision.id),
                kind: Some(String::from("memory_revision")),
                role: Some(String::from("system")),
                intent: Some(String::from("recover")),
                inputs: Some(revision.parent.clone().unwrap_or_default()),
                outputs: Some(revision.id.clone()),
                evidence: revision
                    .baseline
                    .paths()
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect(),
                ..MemoryEvent::default()
            });
        }
        for (index, attempt) in self.attempts.iter().enumerate() {
            let (outputs, evidence) = match attempt {
                AttemptOutcome::Adopted { revision } => (revision.clone(), Vec::new()),
                AttemptOutcome::RolledBack {
                    restored,
                    reason,
                    weakened,
                } => {
                    let mut evidence = vec![reason.slug().to_owned()];
                    evidence.extend(weakened.iter().cloned());
                    (restored.clone(), evidence)
                }
            };
            events.push(MemoryEvent {
                id: stable_id("memory_revision_attempt", &format!("{index}:{outputs}")),
                kind: Some(String::from("memory_revision_attempt")),
                role: Some(String::from("system")),
                intent: Some(String::from("recover")),
                inputs: Some(attempt.slug().to_owned()),
                outputs: Some(outputs),
                evidence,
                ..MemoryEvent::default()
            });
        }
        events
    }
}

/// Compile `source` under `root` with the host `rustc` and report the verdict.
///
/// This is the real compile step behind "if compilation of next version of itself
/// fails": no simulated failure flag, an actual compiler invoked on an actual
/// file, so a test can break a version on purpose and watch the ledger recover.
/// Metadata is all that is emitted -- the question is whether the version builds,
/// not what it links to.
///
/// `baseline_passed` is carried through from the caller because a compile check
/// is not a test run; the ledger needs both numbers and this half only knows one.
///
/// The edition is the crate's own. What is being compiled here is the next
/// version of *this* crate, so a verdict rendered under an older edition would
/// reject Rust that `cargo build` accepts -- a let-chain would read as "does not
/// compile" and the ledger would roll back a version that was never broken.
#[must_use]
pub fn rustc_verdict(root: &Path, source: &str, baseline_passed: usize) -> VersionVerdict {
    let output = Command::new("rustc")
        .arg("--edition")
        .arg(CRATE_EDITION)
        .arg("--crate-type")
        .arg("lib")
        .arg("--emit=metadata")
        .arg("-o")
        .arg(root.join("version-check.rmeta"))
        .arg(root.join(source))
        .output();
    match output {
        Ok(output) => VersionVerdict {
            compiled: output.status.success(),
            diagnostics: String::from_utf8_lossy(&output.stderr).into_owned(),
            baseline_passed: if output.status.success() {
                baseline_passed
            } else {
                0
            },
            baseline_failed: 0,
        },
        Err(error) => VersionVerdict {
            compiled: false,
            diagnostics: error.to_string(),
            baseline_passed: 0,
            baseline_failed: 0,
        },
    }
}
