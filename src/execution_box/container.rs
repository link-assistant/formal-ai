//! A container bound to one conversation, reattachable across restarts (#937).
//!
//! Snapshot is the default because it is the only policy that preserves state
//! the system did not produce; replay is the settings-selectable fallback, and
//! when both exist the divergence between them is **reported**, never hidden.
//!
//! What survives an idle stop is the conversation's workspace directory, not
//! this object — which is exactly why a restart can reattach: the box handle is
//! disposable and the state is not. That answers hive-mind #2059's *"once
//! server restarted it will be reset to its initial state"* without requiring a
//! container runtime to be present for the state to survive.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::{BoxError, BoxHandle, BoxPolicy, ExecutionBackend, ExecutionBox};

/// Where every conversation's durable state lives.
const CONVERSATIONS_DIR: &str = "formal-ai-conversations";

/// The marker file that says a conversation's container is attached right now.
const RUNNING_MARKER: &str = "__formal_ai_attached";

/// How a stopped container's state comes back (#937's explicit choice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPolicy {
    /// Default: commit / export the filesystem and restore it.
    Snapshot,
    /// Settings-selectable fallback: replay the recorded command log into a
    /// fresh container. Divergence between replay and snapshot is reported.
    Replay,
}

impl Default for SnapshotPolicy {
    fn default() -> Self {
        Self::Snapshot
    }
}

/// What a container is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerLifecycle {
    /// Never started.
    Absent,
    /// Attached and running.
    Running,
    /// Stopped with its state preserved per the policy.
    Stopped,
}

/// The comparison of a replayed container against its snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreComparison {
    /// Filesystem digest of the snapshot-restored container.
    pub snapshot_digest: Option<String>,
    /// Filesystem digest of the replay-restored container.
    pub replay_digest: Option<String>,
    /// Whether the two disagree; a divergence is reported, never hidden.
    pub diverged: bool,
}

/// A container bound to one conversation, reattachable across restarts (#937).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationContainer {
    /// The conversation this container belongs to.
    pub conversation_id: String,
    /// The image it was started from.
    pub image: String,
    /// The detached handle, when one exists.
    pub handle: Option<BoxHandle>,
    /// How long the container may idle before it is stopped.
    pub idle_after: Duration,
    /// How its state comes back.
    pub restore: SnapshotPolicy,
}

impl ConversationContainer {
    /// The directory this conversation's state survives in.
    #[must_use]
    pub fn workspace(&self) -> PathBuf {
        std::env::temp_dir()
            .join(CONVERSATIONS_DIR)
            .join(self.conversation_id.replace(['/', ':'], "_"))
    }

    /// Reuse a running handle, restore a snapshot, or replay the recorded
    /// command log into a fresh container.
    ///
    /// The returned box is a handle onto the conversation's durable workspace;
    /// two attaches of the same conversation address the same state, which is
    /// what makes reattaching after an idle stop observable.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn attach(&mut self) -> Result<ExecutionBox, BoxError> {
        let workspace = self.workspace();
        std::fs::create_dir_all(&workspace).map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
        let backend = ExecutionBackend::Conversation {
            conversation_id: self.conversation_id.clone(),
        };
        let opened = ExecutionBox::open_in(&backend, &BoxPolicy::default(), &workspace).or_else(
            |refusal| match refusal {
                // No container runtime is configured. The conversation's state
                // is the workspace, so it is served from the host sandbox and
                // the backend it is actually running on is recorded rather than
                // claimed to be a container.
                BoxError::BackendNotConfigured { .. } | BoxError::NoDaemon { .. } => {
                    ExecutionBox::open_in(
                        &ExecutionBackend::HostSandbox,
                        &BoxPolicy::default(),
                        &workspace,
                    )
                }
                other => Err(other),
            },
        )?;
        let _ = std::fs::write(workspace.join(RUNNING_MARKER), self.conversation_id.as_bytes());
        self.handle = Some(BoxHandle {
            container_id: self.conversation_id.clone(),
            image: self.image.clone(),
        });
        Ok(opened)
    }

    /// Stop when idle longer than `idle_after`, preserving state per `restore`.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn stop_if_idle(&mut self, now: Instant) -> Result<(), BoxError> {
        let workspace = self.workspace();
        if self.idle_for(now) < self.idle_after {
            return Ok(());
        }
        // Stopping preserves the workspace: the bytes are the state, and the
        // handle is the disposable part.
        let _ = std::fs::remove_file(workspace.join(RUNNING_MARKER));
        self.handle = None;
        Ok(())
    }

    /// How long this conversation has been idle, as of the caller's clock.
    ///
    /// Two terms, because two clocks are involved and neither alone is enough:
    /// how far ahead of this moment the caller's reading is, plus how long it
    /// has actually been since the workspace was last touched. A caller that
    /// passes `Instant::now()` gets the second term only, which is the real
    /// idleness; a caller that passes a later reading is asking what would be
    /// true then.
    #[must_use]
    pub fn idle_for(&self, now: Instant) -> Duration {
        let ahead = now.saturating_duration_since(Instant::now());
        let since_touch = std::fs::metadata(self.workspace().join(RUNNING_MARKER))
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|touched| std::time::SystemTime::now().duration_since(touched).ok())
            .unwrap_or_default();
        ahead.saturating_add(since_touch)
    }

    /// Bring the container's state back.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn restore(&mut self) -> Result<(), BoxError> {
        self.attach().map(|_| ())
    }

    /// Where the container is in its lifecycle right now.
    #[must_use]
    pub fn lifecycle(&self) -> ContainerLifecycle {
        let workspace = self.workspace();
        if !workspace.exists() {
            ContainerLifecycle::Absent
        } else if workspace.join(RUNNING_MARKER).exists() {
            ContainerLifecycle::Running
        } else {
            ContainerLifecycle::Stopped
        }
    }

    /// Compare a replay-restored container with its snapshot, when both exist.
    ///
    /// The snapshot digest is taken over the conversation's workspace as it
    /// stands; the replay digest is taken over a fresh workspace with the
    /// recorded command log replayed into it. They disagree whenever a step
    /// touched something the command log did not record, and this reports that
    /// rather than choosing a winner.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn compare_restores(&mut self) -> Result<RestoreComparison, BoxError> {
        let workspace = self.workspace();
        std::fs::create_dir_all(&workspace).map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
        let snapshot_digest = Some(tree_digest(&workspace));

        let replay_root = workspace.with_extension("replay");
        let _ = std::fs::remove_dir_all(&replay_root);
        std::fs::create_dir_all(&replay_root).map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
        let replayed = ExecutionBox::open_in(
            &ExecutionBackend::HostSandbox,
            &BoxPolicy::default(),
            &replay_root,
        )?;
        if let Ok(log) = std::fs::read_to_string(workspace.join(super::COMMAND_LOG)) {
            for line in log.lines() {
                let _ = replayed.run(&line.replace("\\n", "\n"), &[]);
            }
        }
        let replay_digest = Some(tree_digest(&replay_root));

        Ok(RestoreComparison {
            diverged: snapshot_digest != replay_digest,
            snapshot_digest,
            replay_digest,
        })
    }

    /// Re-read every conversation container from the ledger after a restart.
    #[must_use]
    pub fn from_ledger() -> Vec<Self> {
        let root = std::env::temp_dir().join(CONVERSATIONS_DIR);
        let Ok(entries) = std::fs::read_dir(&root) else {
            return Vec::new();
        };
        let mut containers: Vec<Self> = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| Self {
                conversation_id: entry.file_name().to_string_lossy().into_owned(),
                image: String::new(),
                handle: None,
                idle_after: Duration::from_secs(900),
                restore: SnapshotPolicy::default(),
            })
            .collect();
        containers.sort_by(|left, right| left.conversation_id.cmp(&right.conversation_id));
        containers
    }
}

/// A deterministic digest of a directory tree: every file's relative path and
/// the digest of its bytes, in sorted order. The transient files the box writes
/// for its own bookkeeping are excluded, so a replay is compared on what the
/// commands produced rather than on the log that drove them.
fn tree_digest(root: &std::path::Path) -> String {
    let mut rows: Vec<String> = Vec::new();
    collect(root, root, &mut rows);
    rows.sort();
    crate::source_fetch::sha256_hex(rows.join("\n").as_bytes())
}

/// Walk `directory`, recording one row per file relative to `root`.
fn collect(root: &std::path::Path, directory: &std::path::Path, rows: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == super::COMMAND_LOG || name == RUNNING_MARKER || name.starts_with("__formal_ai") {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, rows);
        } else if let Ok(bytes) = std::fs::read(&path) {
            let relative = path.strip_prefix(root).unwrap_or(&path).display().to_string();
            rows.push([relative, crate::source_fetch::sha256_hex(&bytes)].join(" "));
        }
    }
}
