//! Proposal-only learning of reusable methods from solved-problem event logs.
//!
//! Algorithm discovery already performs link-native sequence compression and
//! held-out validation. This module supplies the missing lifecycle bridge for
//! issue #922: normalize real append-only logs into operation traces, expose
//! every discovered abstraction for review, and turn only validated candidates
//! into issue-#656 promotion proposals. Discovery never mutates the registry.

use crate::algorithm_discovery::{
    discover_algorithms, AlgorithmCandidate, AlgorithmDiscoveryRun, ExecutionTrace, TraceStep,
};
use crate::event_log::EventLog;
use crate::links_format::push_lino_node;
use crate::promotion::{PromotionProposal, SeedEdit};

/// Seed file into which benchmark-cleared method abstractions are materialized.
pub const LEARNED_METHODS_SEED_FILE: &str = "data/seed/learned-methods.lino";

/// A reviewable method abstraction derived from recurring event-log operations.
///
/// Payloads are intentionally excluded. They contain problem-specific text and,
/// for `method_registry`, the current catalogue bytes. Learning the event kinds
/// captures control flow while avoiding a self-referential algorithm identity
/// after an adopted method appears in that catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodProposal {
    /// Stable registry name derived from the algorithm schema identity.
    pub name: String,
    /// Stable id of the underlying parameterized algorithm.
    pub algorithm_id: String,
    /// Integrity id covering support and held-out evidence.
    pub evidence_id: String,
    /// Ordered event kinds in the learned abstraction.
    pub operations: Vec<String>,
    /// Traces used to infer the abstraction.
    pub support_trace_ids: Vec<String>,
    /// Traces withheld from inference and used only for validation.
    pub held_out_trace_ids: Vec<String>,
    /// Why a held-out validation rejected the candidate, if it did.
    pub rejection_reasons: Vec<String>,
}

impl MethodProposal {
    /// Proposals are inert until their seed edit clears the promotion protocol.
    #[must_use]
    pub const fn mode(&self) -> &'static str {
        "proposal_only"
    }

    /// Whether every held-out occurrence reproduced the learned operation flow.
    #[must_use]
    pub const fn validated(&self) -> bool {
        self.rejection_reasons.is_empty() && !self.held_out_trace_ids.is_empty()
    }

    fn adopted_seed_lino(&self) -> String {
        let mut output = String::new();
        push_lino_node(&mut output, 0, "learned_method", Some(&self.name));
        push_lino_node(&mut output, 2, "status", Some("adopted"));
        push_lino_node(
            &mut output,
            2,
            "source",
            Some(&format!("algorithm_candidate:{}", self.algorithm_id)),
        );
        push_lino_node(&mut output, 2, "algorithm_id", Some(&self.algorithm_id));
        push_lino_node(&mut output, 2, "evidence_id", Some(&self.evidence_id));
        for trace_id in &self.support_trace_ids {
            push_lino_node(&mut output, 2, "support_trace", Some(trace_id));
        }
        for trace_id in &self.held_out_trace_ids {
            push_lino_node(&mut output, 2, "held_out_trace", Some(trace_id));
        }
        for operation in &self.operations {
            push_lino_node(&mut output, 2, "operation", Some(operation));
        }
        output.trim_end().to_owned()
    }
}

/// One deterministic learning pass over append-only solved-problem histories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodLearningRun {
    /// Complete discovery evidence, including candidates rejected by held-out
    /// validation and their exact mismatch reasons.
    pub discovery: AlgorithmDiscoveryRun,
    /// Reviewable method projections of every retained candidate.
    pub proposals: Vec<MethodProposal>,
}

impl MethodLearningRun {
    /// Held-out-validated proposals eligible to enter benchmark promotion.
    #[must_use]
    pub fn validated_proposals(&self) -> Vec<&MethodProposal> {
        self.proposals
            .iter()
            .filter(|proposal| proposal.validated())
            .collect()
    }

    /// Bridge validated abstractions to the canonical issue-#656 protocol.
    ///
    /// Gates are deliberately empty here. Proposal data cannot declare runner
    /// commands or claim observations; `replay_promotion_gates*` replaces this
    /// with fresh evidence from the canonical allow-list before evaluation.
    #[must_use]
    pub fn promotion_proposals(&self) -> Vec<PromotionProposal> {
        self.validated_proposals()
            .into_iter()
            .map(|proposal| {
                PromotionProposal::new(
                    format!("algorithm_candidate:{}", proposal.algorithm_id),
                    format!(
                        "adopt_learned_method:{}:support={}:held_out={}",
                        proposal.name,
                        proposal.support_trace_ids.len(),
                        proposal.held_out_trace_ids.len()
                    ),
                    SeedEdit::new(LEARNED_METHODS_SEED_FILE, proposal.adopted_seed_lino()),
                    Vec::new(),
                )
            })
            .collect()
    }
}

/// Mine reusable methods from real append-only event-log traces.
///
/// Each tuple supplies the stable observation id used by support and held-out
/// evidence. At least three matching occurrences are required by algorithm
/// discovery: two for inference and one withheld for validation.
#[must_use]
pub fn learn_methods_from_event_logs(observations: &[(&str, &EventLog)]) -> MethodLearningRun {
    let traces = observations
        .iter()
        .map(|(id, log)| {
            ExecutionTrace::new(
                *id,
                log.events()
                    .iter()
                    .map(|event| TraceStep::new(event.kind))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    let discovery = discover_algorithms(&traces);
    let mut proposals = discovery
        .candidates
        .iter()
        .map(method_proposal)
        .collect::<Vec<_>>();
    proposals.sort_by(|left, right| {
        right
            .validated()
            .cmp(&left.validated())
            .then_with(|| right.operations.len().cmp(&left.operations.len()))
            .then_with(|| left.name.cmp(&right.name))
    });
    MethodLearningRun {
        discovery,
        proposals,
    }
}

fn method_proposal(candidate: &AlgorithmCandidate) -> MethodProposal {
    let suffix = candidate
        .id
        .strip_prefix("algorithm_")
        .unwrap_or(&candidate.id);
    MethodProposal {
        name: format!("learned_recursive_core_{suffix}"),
        algorithm_id: candidate.id.clone(),
        evidence_id: candidate.evidence_id.clone(),
        operations: candidate
            .steps
            .iter()
            .map(|step| step.operation.clone())
            .collect(),
        support_trace_ids: candidate.support_trace_ids.clone(),
        held_out_trace_ids: candidate
            .held_out
            .iter()
            .map(|test| test.trace_id.clone())
            .collect(),
        rejection_reasons: candidate
            .held_out
            .iter()
            .flat_map(|test| {
                test.failures
                    .iter()
                    .map(|failure| format!("{}:{failure}", test.trace_id))
            })
            .collect(),
    }
}
