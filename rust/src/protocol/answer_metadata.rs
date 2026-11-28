use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::capability_routing::claim_rows;
use crate::engine::SymbolicAnswer;

/// Whether the request produced an answer or a usable tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerStatus {
    Answered,
    Unanswered,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerProvenance {
    LocalRules,
    StructuredDerivation,
    NetworkRetrieval,
}

/// Formal AI's extension to OpenAI result envelopes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerMetadata {
    pub status: AnswerStatus,
    pub provenance: Vec<AnswerProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholders: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<BTreeMap<String, String>>,
}

fn has_capability_refusal(answer: &SymbolicAnswer) -> bool {
    claim_rows()
        .iter()
        .flat_map(|row| &row.refusal_events)
        .any(|kind| {
            answer.evidence_links.iter().any(|link| {
                link == kind
                    || link
                        .strip_prefix(kind.as_str())
                        .is_some_and(|identifier| identifier.starts_with(':'))
            })
        })
}

impl AnswerMetadata {
    pub(super) fn new(status: AnswerStatus) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }

    pub(super) fn from_symbolic(answer: &SymbolicAnswer) -> Self {
        let status = if matches!(
            answer.intent.as_str(),
            "tool_call_refused"
                | "code_execution_refused"
                | "policy_inappropriate_content"
                | "policy_bounded_autonomy"
                | "policy_add_only_history"
                | "policy_cache_flush_requires_confirmation"
                | "policy_destructive_action_requires_confirmation"
                | "policy_agent_time_budget"
        ) {
            AnswerStatus::Refused
        } else if answer.is_inconclusive() || has_capability_refusal(answer) {
            AnswerStatus::Unanswered
        } else {
            AnswerStatus::Answered
        };
        Self::new(status)
    }

    pub(super) fn observe_retrieval_since(&mut self, before: u64) {
        if crate::source_fetch::network_retrieval_count() != before {
            self.provenance.push(AnswerProvenance::NetworkRetrieval);
        }
    }
}

impl Default for AnswerMetadata {
    fn default() -> Self {
        Self {
            status: AnswerStatus::Answered,
            provenance: vec![AnswerProvenance::LocalRules],
            placeholders: None,
            rules: None,
        }
    }
}
