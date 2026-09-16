//! The runtime obligation tree an answer must discharge before it may finish (#1138 B5).
//!
//! Plan 05 owns this module. `ObligationOutcome::Satisfied` carries an
//! [`Evidence`] and nothing else, so the type system — not a convention —
//! forbids a satisfied obligation without an observation, and
//! [`need_ledger_with_execution`] is the single place `NeedStatus::Satisfied`
//! may be produced.
//!
//! Wave T lands the shapes only; every body is `todo!` until wave I5's leaves
//! 05-4 through 05-14.

use crate::execution_evidence::Evidence;
use crate::meta_frame::{NeedLedger, ProblemFrame};
use crate::protocol::ChatMessage;

/// What must be observed before this node may be called satisfied.
///
/// This is *not* plan 08's `TaskExpectation`, which declares the shape a task's
/// **answer** must take; the two are bridged by
/// `TaskExpectation::to_obligation_expectation` (plan 00 §9 R15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationExpectation {
    /// The named path must exist and its bytes must hash to `sha256` when given.
    FileBytes {
        /// Path the clause names.
        path: String,
        /// Expected digest of the file's bytes, when the clause states one.
        sha256: Option<String>,
    },
    /// The named command must run and exit with `expected_exit`.
    CommandExit {
        /// The command line the clause names.
        command: String,
        /// The exit status the clause requires.
        expected_exit: i64,
    },
    /// The named command's observed output must hash to `sha256`.
    OutputHash {
        /// The command line the clause names.
        command: String,
        /// Expected digest of the observed bytes.
        sha256: String,
    },
    /// One of the generated checks of loop step 6 must pass.
    ///
    /// `check_id` is `"<VerifiedAnswer::derivation_id>:<check slug>"`, owned by
    /// plan 08 (plan 00 §9 R15).
    SymbolicCheck {
        /// The check this node waits on.
        check_id: String,
    },
    /// No expectation could be derived from this clause. Never discarded: such a
    /// node is split, and when it cannot be split it is reported as a gap.
    Underivable {
        /// Why no expectation could be derived.
        reason: String,
    },
}

impl ObligationExpectation {
    /// Links Notation projection of one expectation.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 05 leaf 4")
    }
}

/// The outcome of one obligation node. `Satisfied` cannot exist without a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationOutcome {
    /// No observation has been made yet.
    Unattempted,
    /// An observation was made and did not meet the expectation.
    Refuted {
        /// The observation that refuted the expectation.
        record: Evidence,
        /// What did not match.
        mismatch: String,
    },
    /// An observation was made and met the expectation.
    Satisfied {
        /// The observation that discharged the expectation.
        record: Evidence,
    },
    /// No observation is reachable and no split helped; the reason is named.
    Unsatisfiable {
        /// Why nothing can be observed.
        reason: String,
    },
}

impl ObligationOutcome {
    /// Links Notation projection of one outcome.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 05 leaf 4")
    }
}

/// One node of the obligation tree: the runtime counterpart of a `WorkUnit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationNode {
    /// `stable_id("obligation", &format!("{parent:?}:{depth}:{clause}"))`.
    pub node_id: String,
    /// The parent node id, when this node came from a split.
    pub parent: Option<String>,
    /// The clause exactly as the user wrote it.
    pub clause: String,
    /// UTF-8 byte span of `clause` in the original request (R710-R9).
    pub span: (usize, usize),
    /// The `Need::need_id` this node discharges, when it maps to one.
    pub need_id: Option<String>,
    /// Split depth of this node.
    pub depth: u8,
    /// What must be observed here.
    pub expectation: ObligationExpectation,
    /// What has been observed here.
    pub outcome: ObligationOutcome,
    /// Nodes this clause split into.
    pub children: Vec<Self>,
}

impl ObligationNode {
    /// Build the obligation tree for a request: derive an expectation per clause,
    /// and recurse through `task_decomposition::split_once_checkable` for clauses
    /// whose expectation is `Underivable`.
    #[must_use]
    pub fn build(_request: &str, _max_split_depth: u8) -> Self {
        todo!("plan 05 leaf 5")
    }

    /// Post-order: discharged when every child is discharged and this node's own
    /// outcome is `Satisfied` or `Unsatisfiable`.
    #[must_use]
    pub fn discharged(&self) -> bool {
        todo!("plan 05 leaf 5")
    }

    /// The first node that is neither discharged nor has an undischarged child.
    #[must_use]
    pub fn next_open(&self) -> Option<&Self> {
        todo!("plan 05 leaf 5")
    }

    /// Collect every leaf of the tree, in source order.
    pub fn collect_leaves<'a>(&'a self, _out: &mut Vec<&'a Self>) {
        todo!("plan 05 leaf 5")
    }

    /// Links Notation projection of the node and its children.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 05 leaf 5")
    }
}

/// The runtime counterpart of `NeedLedger`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationLedger {
    /// The frame this ledger belongs to.
    pub frame_id: String,
    /// The obligation tree for the frame's request.
    pub root: ObligationNode,
}

impl ObligationLedger {
    /// Build the ledger for one frame's request.
    #[must_use]
    pub fn for_frame(_frame: &ProblemFrame, _request: &str, _max_split_depth: u8) -> Self {
        todo!("plan 05 leaf 8")
    }

    /// Apply one observation to the node whose expectation it answers. Returns the
    /// discharged node id, or `None` when no node expected it — an unrelated
    /// result can never clear a step (R710-R4).
    pub fn observe(&mut self, _record: Evidence) -> Option<String> {
        todo!("plan 05 leaf 8")
    }

    /// Every obligation is `Satisfied` or `Unsatisfiable` with a named reason.
    #[must_use]
    pub fn every_obligation_discharged(&self) -> bool {
        todo!("plan 05 leaf 8")
    }

    /// Number of nodes whose outcome is `Satisfied`.
    #[must_use]
    pub fn satisfied_count(&self) -> usize {
        todo!("plan 05 leaf 8")
    }

    /// Number of nodes whose outcome is `Unsatisfiable`.
    #[must_use]
    pub fn unsatisfiable_count(&self) -> usize {
        todo!("plan 05 leaf 8")
    }

    /// Number of nodes whose outcome is `Unattempted`.
    #[must_use]
    pub fn unattempted_count(&self) -> usize {
        todo!("plan 05 leaf 8")
    }

    /// Links Notation projection of the whole ledger.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 05 leaf 8")
    }
}

/// What the session must do next about its obligations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationStep {
    /// Make the observation this node expects.
    Observe(ObligationNode),
    /// The node's expectation is `Underivable` and it can still be split.
    Decompose(ObligationNode),
    /// Nothing is left to split and nothing can be observed; report the gap.
    ReportGap {
        /// The node that cannot be discharged.
        node_id: String,
        /// The clause exactly as the user wrote it.
        clause: String,
        /// Its byte span in the original request.
        span: (usize, usize),
        /// Why nothing can be observed.
        reason: String,
    },
}

/// The join: a *new* need ledger whose rows are upgraded from `Planned` to
/// `Satisfied` exactly where an obligation carrying an [`Evidence`] discharged
/// the same need. Never mutates its input.
///
/// This is the **only** function in the tree that may produce
/// `NeedStatus::Satisfied` (plan 05 leaf 9).
#[must_use]
pub fn need_ledger_with_execution(
    _planned: &NeedLedger,
    _obligations: &ObligationLedger,
) -> NeedLedger {
    todo!("plan 05 leaf 9")
}

/// Replaces `task_obligations::outstanding`.
#[must_use]
pub fn next_step(_request: &str, _messages: &[ChatMessage]) -> Option<ObligationStep> {
    todo!("plan 05 leaf 14")
}
