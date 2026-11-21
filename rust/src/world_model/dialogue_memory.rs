//! Permissioned access between dialogue-local and general world contexts.

use super::{Action, Context, ContextDiff, Prediction, RecalculationReport, stable_id};
use std::error::Error;
use std::fmt;

/// Append-only event kinds for the current/general context boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAccessEventKind {
    /// A user explicitly approved general-memory access.
    PermissionGranted,
    /// General-memory access was considered without approval.
    PermissionDenied,
    /// A permission token was used to read/audit general memory.
    GeneralContextRead,
    /// Current-dialogue knowledge was committed to general memory.
    CurrentCommitted,
}

/// An append-only record of a permission decision or boundary crossing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAccessEvent {
    /// Zero-based append sequence.
    pub sequence: usize,
    /// Event kind.
    pub kind: ContextAccessEventKind,
    /// Stable permission id when a grant exists.
    pub permission_id: Option<String>,
    /// Caller-supplied purpose/reason retained in the trace.
    pub reason: String,
}

/// Whether dialogue-local state may be promoted into shared general memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralMemoryPermission {
    /// Keep dialogue state confined to its current context.
    Denied,
    /// Permit an explicit promotion into shared general memory.
    Allowed,
}

/// A rejected attempt to cross the dialogue-to-general-memory boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralMemoryCommitError {
    PermissionDenied,
}

impl fmt::Display for GeneralMemoryCommitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => {
                formatter.write_str("general-memory promotion requires explicit permission")
            }
        }
    }
}

impl Error for GeneralMemoryCommitError {}

/// The per-dialogue world model: a `current` context, a `target` context, and a
/// shared `general` context that per-dialogue contexts merge into.
///
/// This is the top-level holder issue-649 describes — *"at any stage of the
/// dialogue we have the current representation of the world … and also the
/// target representation the user wants"* — with `general` as the *"entire world
/// model / general context"* a dialogue context can be merged into or split
/// from.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorldModel {
    /// The current state of the (partial) world discussed in the dialogue.
    pub current: Context,
    /// The target state the user wants.
    pub target: Context,
    /// The shared world model per-dialogue contexts merge into.
    general: Context,
    /// Append-only permission/crossing trace.
    context_access_events: Vec<ContextAccessEvent>,
}

impl WorldModel {
    /// A fresh world model with three empty, distinctly-named contexts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: Context::new("current"),
            target: Context::new("target"),
            general: Context::new("general"),
            context_access_events: Vec::new(),
        }
    }

    /// The difference from the current state to the target state (what the agent
    /// must achieve). Directly exposes issue-649's *"difference from the current
    /// state"* at any dialogue stage.
    #[must_use]
    pub fn difference(&self) -> ContextDiff {
        self.current.difference(&self.target)
    }

    /// Predict an action's consequences against the current state without
    /// mutating it.
    #[must_use]
    pub fn predict(&self, action: &Action) -> Prediction {
        self.current.predict(action)
    }

    /// Whether the current state already satisfies the target (the difference is
    /// empty) — i.e. the dialogue's goal is reached.
    #[must_use]
    pub fn target_reached(&self) -> bool {
        self.difference().is_empty()
    }

    /// Read the shared context without exposing a mutation path around the
    /// dialogue-to-general permission gate.
    #[must_use]
    pub const fn general(&self) -> &Context {
        &self.general
    }

    /// Merge the current dialogue context into the shared general world model
    /// (ATMS context combination) only when the caller explicitly permits that
    /// boundary crossing.
    pub fn commit_current_to_general(
        &mut self,
        permission: GeneralMemoryPermission,
    ) -> Result<RecalculationReport, GeneralMemoryCommitError> {
        let reason = "current-to-general-memory commit";
        let permission_id = self.record_general_memory_permission(permission, reason)?;
        let report = self.general.merge_from(&self.current);
        self.push_context_access_event(
            ContextAccessEventKind::CurrentCommitted,
            Some(permission_id),
            reason,
        );
        Ok(report)
    }

    /// Append-only permission and boundary-crossing trace.
    #[must_use]
    pub fn context_access_events(&self) -> &[ContextAccessEvent] {
        &self.context_access_events
    }

    /// Permissioned mutable access used by whole-memory fact-check operations.
    pub(crate) fn general_context_for_audit(
        &mut self,
        permission: GeneralMemoryPermission,
    ) -> Result<&mut Context, GeneralMemoryCommitError> {
        let reason = "fact-check general-memory audit";
        let permission_id = self.record_general_memory_permission(permission, reason)?;
        self.push_context_access_event(
            ContextAccessEventKind::GeneralContextRead,
            Some(permission_id),
            reason,
        );
        Ok(&mut self.general)
    }

    fn record_general_memory_permission(
        &mut self,
        permission: GeneralMemoryPermission,
        reason: &str,
    ) -> Result<String, GeneralMemoryCommitError> {
        if permission == GeneralMemoryPermission::Denied {
            self.push_context_access_event(ContextAccessEventKind::PermissionDenied, None, reason);
            return Err(GeneralMemoryCommitError::PermissionDenied);
        }
        let sequence = self.context_access_events.len();
        let permission_id = stable_id(
            "general_memory_permission",
            &format!("sequence:{sequence};reason:{reason}"),
        );
        self.push_context_access_event(
            ContextAccessEventKind::PermissionGranted,
            Some(permission_id.clone()),
            reason,
        );
        Ok(permission_id)
    }

    fn push_context_access_event(
        &mut self,
        kind: ContextAccessEventKind,
        permission_id: Option<String>,
        reason: &str,
    ) {
        self.context_access_events.push(ContextAccessEvent {
            sequence: self.context_access_events.len(),
            kind,
            permission_id,
            reason: reason.to_owned(),
        });
    }
}
