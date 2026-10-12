use super::final_result::{FinalDisposition, FinalPayloadRole, FinalResult, ResolvedPlan};
use super::planner::AgenticPlan;

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
    let artifact = super::final_result::SourceArtifact {
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
