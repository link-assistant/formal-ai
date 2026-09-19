//! A coding task that names several obligations is finished when all of them are.
//!
//! Issue #1099 exposed the structural failure this module guards: the planner
//! treated "the task" and "the first artifact named by the task" as the same
//! thing. Plan 05 removes that second model of an obligation. The public
//! compatibility surface below is now a projection of the runtime
//! [`ObligationNode`] tree, so a
//! clause that has no immediately derivable artifact remains an underivable
//! node instead of disappearing from the request.

use crate::engine::stable_id;
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::obligation_ledger::{
    ObligationExpectation, ObligationLedger, ObligationNode, ObligationOutcome, clauses_with_spans,
};
use crate::protocol::ChatMessage;

pub use crate::obligation_ledger::ObligationStep;

/// Compatibility name for one projected runtime obligation.
///
/// This is deliberately an alias, not a second record. The clause, span,
/// expectation and evidence-bearing outcome therefore cannot drift from the
/// ledger the completion gate reads.
pub type Obligation = ObligationNode;

/// Project the leaves of an enumerated request's obligation tree.
///
/// Returns `None` for a request with fewer than two enumerated clauses, keeping
/// the established single-target routes unchanged. Once a request is
/// enumerated, every leaf is returned: in particular, failure to derive a file
/// target produces an `Underivable` node rather than the old silent `continue`.
#[must_use]
pub fn obligations(request: &str) -> Option<Vec<Obligation>> {
    (clauses_with_spans(request).len() >= 2).then(|| {
        let root = agentic_root(request);
        let mut leaves = Vec::new();
        root.collect_leaves(&mut leaves);
        leaves.into_iter().cloned().collect()
    })
}

/// What the live agentic session must do next about an enumerated request.
///
/// The runtime implementation lives with the ledger. This compatibility entry
/// point keeps the agentic route and callers of `obligations()` on the same tree
/// and deliberately declines single-clause tasks.
#[must_use]
pub fn next_step(request: &str, messages: &[ChatMessage]) -> Option<ObligationStep> {
    obligations(request)?;
    let ledger = observed_ledger(request, messages);
    let mut leaves = Vec::new();
    ledger.root.collect_leaves(&mut leaves);
    let open: Vec<&ObligationNode> = leaves
        .into_iter()
        .filter(|node| !node.discharged())
        .collect();
    if let Some(node) = open
        .iter()
        .find(|node| node.expectation.is_observable())
    {
        return Some(ObligationStep::Observe((*node).clone()));
    }
    if let Some(node) = open.iter().find(|node| {
        node.depth < crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND
            && crate::task_decomposition::split_once_checkable(&node.clause).len() >= 2
    }) {
        return Some(ObligationStep::Decompose((*node).clone()));
    }
    let node = *open.first()?;
    let reason = match &node.expectation {
        ObligationExpectation::Underivable { reason } => reason.clone(),
        expectation => expectation.slug().to_owned(),
    };
    Some(ObligationStep::ReportGap {
        node_id: node.node_id.clone(),
        clause: node.clause.clone(),
        span: node.span,
        reason,
    })
}

/// Whether an enumerated request has crossed the successful completion gate.
///
/// Unsatisfiable nodes are reportable, but they are not successful completion.
/// At least one evidence-backed satisfaction is required in addition to every
/// node being discharged.
#[must_use]
pub(super) fn successfully_discharged(request: &str, messages: &[ChatMessage]) -> bool {
    let Some(_) = obligations(request) else {
        return false;
    };
    let ledger = observed_ledger(request, messages);
    ledger.every_obligation_discharged()
        && ledger.unsatisfiable_count() == 0
        && ledger.satisfied_count() >= 1
}

/// Render a terminal, machine-readable account of an obligation that cannot be
/// observed or split. The record names the original clause, its UTF-8 byte
/// span, and the derivation reason; it contains no completion claim.
#[must_use]
pub(super) fn gap_answer(
    node_id: &str,
    clause: &str,
    span: (usize, usize),
    reason: &str,
) -> String {
    crate::links_format::format_lino_record(
        node_id,
        &[
            ("record_type", String::from("obligation_gap")),
            ("clause", clause.to_owned()),
            ("span_start", span.0.to_string()),
            ("span_end", span.1.to_string()),
            ("reason", reason.to_owned()),
        ],
    )
}

/// Read a file-delivery outcome through the same ledger judgement as every
/// other agentic obligation.
///
/// `evidence_record` previously had its own private `Obligation` and decided
/// completion from two write-only booleans. Its delivery parser still owns the
/// constraints needed to render the file, but the decision about what the
/// transcript proves now passes through `ObligationLedger::observe` here.
#[must_use]
pub(super) fn observed_file_outcome(
    request: &str,
    path: &str,
    messages: &[ChatMessage],
) -> ObligationOutcome {
    let node_id = stable_id("obligation", &format!("delivery:{path}:{request}"));
    let mut ledger = ObligationLedger {
        frame_id: stable_id("obligation_ledger", request),
        root: ObligationNode {
            node_id,
            parent: None,
            clause: request.to_owned(),
            span: (0, request.len()),
            need_id: None,
            depth: 0,
            expectation: ObligationExpectation::FileBytes {
                path: path.to_owned(),
                // A derived evidence file has no caller-declared digest. The
                // record must still name the path and report success.
                sha256: None,
            },
            outcome: ObligationOutcome::Unattempted,
            children: Vec::new(),
        },
    };
    let progress = super::progress::Progress::scan(messages);
    if progress.attempted_write_for(path) && !progress.successful_write_for(path) {
        // The client reported an attempted write but no successful one. Keep
        // the exit status explicitly absent and observe no file bytes; this
        // refutes a file expectation without upgrading a harness claim into a
        // local-process result.
        ledger.observe(&Evidence::observed(
            crate::repository_workspace::render_protocol_template(
                "obligation_write_observation",
                &[("path", path)],
            )
            .unwrap_or_else(|| String::from("obligation_write_observation")),
            vec![path.to_owned()],
            None,
            b"",
            ObservationKind::ToolResult,
            EvidenceSource::Harness,
        ));
        return ledger.root.outcome;
    }
    for record in super::transcript_evidence::records(messages) {
        let is_write = record
            .command
            .split_whitespace()
            .next()
            .and_then(super::capability_router::classify_tool)
            == Some(super::planner::Capability::Write);
        if is_write {
            ledger.observe(&record);
        }
    }
    ledger.root.outcome
}

/// Rebuild the ledger from transcript evidence; planner state remains purely
/// derived from the conversation and survives process boundaries.
fn observed_ledger(request: &str, messages: &[ChatMessage]) -> ObligationLedger {
    let mut ledger = ObligationLedger {
        frame_id: stable_id("obligation_ledger", request),
        root: agentic_root(request),
    };
    for record in super::transcript_evidence::records(messages) {
        ledger.observe(&record);
    }
    ledger
}

/// The runtime tree projected onto the bytes the agentic writer actually asks
/// the client to persist.
///
/// The general derivation trims a sentence's final full stop before hashing,
/// because it has no execution plan yet. The live composer does: its `content`
/// field is the exact write operand. Hash that operand exactly so the
/// observation and expectation describe the same bytes instead of repeatedly
/// reopening an already verified node. A read tool may display a trailing
/// newline, but it must not invent one in the file's content address.
fn agentic_root(request: &str) -> ObligationNode {
    let mut root = ObligationNode::build(
        request,
        crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND,
    );
    align_file_expectations(&mut root);
    root
}

fn align_file_expectations(node: &mut ObligationNode) {
    if node.children.is_empty()
        && let Some(plan) = super::general_planner::compose_general_change_plan(&node.clause)
        && matches!(node.expectation, ObligationExpectation::FileBytes { .. })
    {
        node.expectation = ObligationExpectation::FileBytes {
            path: plan.target,
            sha256: Some(crate::source_fetch::sha256_hex(plan.content.as_bytes())),
        };
    }
    for child in &mut node.children {
        align_file_expectations(child);
    }
}
