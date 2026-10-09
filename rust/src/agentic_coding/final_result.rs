//! Internal final-result provenance; public plans retain their text projection.

use super::planner::AgenticPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FinalDisposition {
    Finding,
    Clarification,
    Gap,
    Failure,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FinalPayloadRole {
    Finding,
    AuditReport,
    SourceModule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceArtifact {
    pub(super) complete: bool,
    pub(super) content: String,
    pub(super) content_id: String,
    pub(super) syntax_content: String,
    pub(super) syntax_content_id: String,
    pub(super) syntax_exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FinalResult {
    pub(super) text: String,
    pub(super) disposition: FinalDisposition,
    pub(super) origin: String,
    pub(super) payload_role: FinalPayloadRole,
    pub(super) artifact: Option<SourceArtifact>,
}

pub(super) struct ResolvedPlan {
    pub(super) plan: AgenticPlan,
    pub(super) result: Option<FinalResult>,
}

impl ResolvedPlan {
    pub(super) fn new(plan: AgenticPlan, result: Option<FinalResult>) -> Self {
        let result = match &plan {
            AgenticPlan::ToolCalls(_) => None,
            AgenticPlan::Final(text) => Some(
                result
                    .filter(|result| result.text == *text)
                    .unwrap_or_else(|| FinalResult {
                        text: text.clone(),
                        disposition: FinalDisposition::Unknown,
                        origin: String::new(),
                        payload_role: FinalPayloadRole::Finding,
                        artifact: None,
                    }),
            ),
        };
        Self { plan, result }
    }

    pub(super) fn can_deliver(&self) -> bool {
        self.can_deliver_as(FinalPayloadRole::Finding)
    }

    pub(super) fn can_deliver_as(&self, required: FinalPayloadRole) -> bool {
        self.result.as_ref().is_some_and(|result| {
            result.disposition == FinalDisposition::Finding
                && !result.origin.is_empty()
                && (required != FinalPayloadRole::SourceModule
                    || (result.payload_role == FinalPayloadRole::SourceModule
                        && result.artifact.as_ref().is_some_and(|artifact| {
                            artifact.complete
                                && artifact.content == result.text
                                && !artifact.content_id.is_empty()
                                && artifact.syntax_exit_code == 0
                                && artifact.syntax_content == artifact.content
                                && artifact.syntax_content_id == artifact.content_id
                        })))
        })
    }

    pub(super) fn into_plan(self, result: &mut Option<FinalResult>) -> AgenticPlan {
        *result = self.result;
        self.plan
    }
}

/// Capture metadata only for the final plan actually selected by its route.
pub(super) fn record(
    plan: AgenticPlan,
    disposition: FinalDisposition,
    origin: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    record_with_role(plan, disposition, origin, FinalPayloadRole::Finding, result)
}

pub(super) fn record_with_role(
    plan: AgenticPlan,
    disposition: FinalDisposition,
    origin: &str,
    payload_role: FinalPayloadRole,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    if let AgenticPlan::Final(text) = &plan {
        *result = Some(FinalResult {
            text: text.clone(),
            disposition,
            origin: origin.to_owned(),
            payload_role,
            artifact: None,
        });
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::{FinalDisposition, FinalPayloadRole, FinalResult, ResolvedPlan};
    use crate::agentic_coding::AgenticPlan;

    #[test]
    fn certificate_belongs_only_to_selected_final_bytes() {
        let observed = FinalResult {
            text: "observed bytes".to_owned(),
            disposition: FinalDisposition::Finding,
            origin: "fixture_observed".to_owned(),
            payload_role: FinalPayloadRole::AuditReport,
            artifact: None,
        };
        let selected = ResolvedPlan::new(
            AgenticPlan::Final(observed.text.clone()),
            Some(observed.clone()),
        );
        assert!(selected.can_deliver());
        assert!(!selected.can_deliver_as(FinalPayloadRole::SourceModule));
        let other = ResolvedPlan::new(
            AgenticPlan::Final("different bytes".to_owned()),
            Some(observed.clone()),
        );
        assert!(!other.can_deliver());
        assert_eq!(
            other.result.as_ref().map(|result| result.disposition),
            Some(FinalDisposition::Unknown)
        );
        let tool = ResolvedPlan::new(AgenticPlan::ToolCalls(Vec::new()), Some(observed));
        assert!(tool.result.is_none());
    }

    #[test]
    fn source_operand_requires_a_complete_matching_successful_receipt() {
        let content = "export function identity(value) { return value; }";
        let artifact = super::SourceArtifact {
            complete: true,
            content: content.to_owned(),
            content_id: "observed-content-id".to_owned(),
            syntax_content: content.to_owned(),
            syntax_content_id: "observed-content-id".to_owned(),
            syntax_exit_code: 0,
        };
        let outcome = |artifact| {
            ResolvedPlan::new(
                AgenticPlan::Final(content.to_owned()),
                Some(FinalResult {
                    text: content.to_owned(),
                    disposition: FinalDisposition::Finding,
                    origin: "source_syntax_observed".to_owned(),
                    payload_role: FinalPayloadRole::SourceModule,
                    artifact: Some(artifact),
                }),
            )
        };
        assert!(outcome(artifact.clone()).can_deliver_as(FinalPayloadRole::SourceModule));
        let mut failed = artifact.clone();
        failed.syntax_exit_code = 1;
        assert!(!outcome(failed).can_deliver_as(FinalPayloadRole::SourceModule));
        let mut stale = artifact.clone();
        stale.syntax_content_id = "stale-content-id".to_owned();
        assert!(!outcome(stale).can_deliver_as(FinalPayloadRole::SourceModule));
        let mut partial = artifact;
        partial.content = "Line 2: source excerpt".to_owned();
        assert!(!outcome(partial).can_deliver_as(FinalPayloadRole::SourceModule));
    }
}
