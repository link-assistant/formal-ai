//! The `repository-protocol.lino` evidence every protocol caller writes.
//!
//! SWE-bench, the #848 coding ladder and self-authoring all run the stages
//! `data/meta/repository-workspace-protocol.lino` declares (#1138
//! R1138-3-5). This module renders what one run observed at each declared
//! stage, in declared order, so the three callers' evidence names the same
//! stages and differs only in what each stage observed.

use std::fmt::Write as _;

use super::{ProtocolOutcome, WorkspaceProtocol};
use crate::meta_frame::NeedStatus;

/// The editor that changes the tree by structural, registry-grounded edits
/// (SWE-bench and the coding ladder).
pub const EDITOR_STRUCTURAL: &str = "structural";

/// The editor that changes the tree through a live Agent CLI session
/// (self-authoring).
pub const EDITOR_AGENT_SESSION: &str = "agent_session";

/// The record head of a rendered trace.
pub const TRACE_RECORD: &str = "repository_protocol_trace";

/// The child name each stage line carries.
const STAGE_FIELD: &str = "stage";

/// How one declared stage ended in one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    /// The stage ran and its postcondition was observed.
    Observed,
    /// The stage ran and recorded nothing to observe (no named tests).
    Unobserved,
    /// The stage ran and its postcondition was not observed; the run stopped.
    Stopped,
    /// An earlier stage stopped the run, so this one never ran.
    NotReached,
    /// The stage belongs to another editor.
    NotApplicable,
    /// The commit gate stayed shut: no commit was asked for, or nothing was
    /// authored.
    Refused,
    /// The commit gate opened; the commit carries this document.
    Requested,
}

impl StageStatus {
    /// The token a trace records for this status.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Unobserved => "unobserved",
            Self::Stopped => "stopped",
            Self::NotReached => "not_reached",
            Self::NotApplicable => "not_applicable",
            Self::Refused => "refused",
            Self::Requested => "requested",
        }
    }
}

/// One declared stage and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageRecord {
    /// The stage id the protocol document declares.
    pub id: String,
    /// How it ended in this run.
    pub status: StageStatus,
}

/// What one caller observed at every declared stage of one run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolTrace {
    /// Which caller ran the protocol (`solve`, `benchmark`, `authoring`).
    pub caller: String,
    /// Which editor changed the tree.
    pub editor: String,
    /// Caller-specific facts (session, model, counts), in insertion order.
    pub fields: Vec<(String, String)>,
    /// Every declared stage, in protocol order.
    pub stages: Vec<StageRecord>,
    /// Requirements the run could not satisfy, stated plainly.
    pub open: Vec<String>,
}

impl ProtocolTrace {
    /// A trace with every declared stage not reached yet, except the stages
    /// that belong to another editor.
    #[must_use]
    pub fn new(protocol: &WorkspaceProtocol, caller: &str, editor: &str) -> Self {
        Self {
            caller: caller.to_owned(),
            editor: editor.to_owned(),
            fields: Vec::new(),
            stages: protocol
                .steps()
                .iter()
                .map(|step| StageRecord {
                    id: step.id.clone(),
                    status: if step.applies_to(editor) {
                        StageStatus::NotReached
                    } else {
                        StageStatus::NotApplicable
                    },
                })
                .collect(),
            open: Vec::new(),
        }
    }

    /// The trace of a structural run, read from what
    /// [`WorkspaceProtocol::execute`] observed.
    #[must_use]
    pub fn from_outcome(
        protocol: &WorkspaceProtocol,
        caller: &str,
        outcome: &ProtocolOutcome,
    ) -> Self {
        let mut trace = Self::new(protocol, caller, EDITOR_STRUCTURAL);
        let stopped = outcome.stopped_at.as_ref().map(|step| step.id.as_str());
        let mut reached = true;
        for stage in &mut trace.stages {
            if stage.status == StageStatus::NotApplicable || !reached {
                continue;
            }
            if stopped == Some(stage.id.as_str()) {
                stage.status = StageStatus::Stopped;
                reached = false;
                continue;
            }
            if !outcome
                .need_ledger
                .rows
                .iter()
                .any(|row| row.route.as_deref() == Some(stage.id.as_str()))
            {
                stage.status = StageStatus::NotApplicable;
                continue;
            }
            let satisfied = outcome.need_ledger.rows.iter().any(|row| {
                row.route.as_deref() == Some(stage.id.as_str())
                    && row.status == NeedStatus::Satisfied
            });
            stage.status = if satisfied {
                StageStatus::Observed
            } else {
                StageStatus::Unobserved
            };
        }
        trace.set_field("located", &outcome.located.len().to_string());
        trace.set_field("edited", &outcome.edited.len().to_string());
        trace.set_field("observations", &outcome.observations.len().to_string());
        trace.set_field("diff_bytes", &outcome.diff.len().to_string());
        trace.open.clone_from(&outcome.open);
        trace
    }

    /// Record how stage `id` ended. A stage the document does not declare is
    /// ignored: the document, not the caller, names the stages.
    pub fn record(&mut self, id: &str, status: StageStatus) {
        if let Some(stage) = self.stages.iter_mut().find(|stage| stage.id == id) {
            stage.status = status;
        }
    }

    /// Set one caller-specific fact, replacing an earlier value.
    pub fn set_field(&mut self, name: &str, value: &str) {
        for field in &mut self.fields {
            if field.0 == name {
                value.clone_into(&mut field.1);
                return;
            }
        }
        self.fields.push((name.to_owned(), value.to_owned()));
    }

    /// How stage `id` ended, when the document declares it.
    #[must_use]
    pub fn status(&self, id: &str) -> Option<StageStatus> {
        self.stages
            .iter()
            .find(|stage| stage.id == id)
            .map(|stage| stage.status)
    }

    /// The document `repository-protocol.lino` carries.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = format!("{TRACE_RECORD}\n");
        let _ = writeln!(out, "  caller \"{}\"", quote(&self.caller));
        let _ = writeln!(out, "  editor \"{}\"", quote(&self.editor));
        for (name, value) in &self.fields {
            let _ = writeln!(out, "  {name} \"{}\"", quote(value));
        }
        for stage in &self.stages {
            let _ = writeln!(
                out,
                "  {STAGE_FIELD} {} \"{}\"",
                stage.id,
                stage.status.token()
            );
        }
        for open in &self.open {
            let _ = writeln!(out, "  open \"{}\"", quote(open));
        }
        out
    }
}

/// The stage ids a rendered trace names, in the order it names them.
#[must_use]
pub fn stage_ids(document: &str) -> Vec<String> {
    stage_lines(document).map(|(id, _)| id.to_owned()).collect()
}

/// Every `(stage, status)` pair a rendered trace records, in order.
#[must_use]
pub fn stage_statuses(document: &str) -> Vec<(String, String)> {
    stage_lines(document)
        .map(|(id, status)| (id.to_owned(), status.trim_matches('"').to_owned()))
        .collect()
}

fn stage_lines(document: &str) -> impl Iterator<Item = (&str, &str)> {
    document.lines().filter_map(|line| {
        line.trim_start()
            .strip_prefix(STAGE_FIELD)?
            .strip_prefix(' ')?
            .split_once(' ')
    })
}

fn quote(value: &str) -> String {
    value.replace('"', "\"\"")
}
