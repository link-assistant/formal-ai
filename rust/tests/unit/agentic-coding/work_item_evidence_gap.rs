//! Issue #1483: a completed evidence collection names the narrowest gap it
//! leaves; a work item that links only related pages names each as unresolved.
use super::super::planner::AgenticPlan;
use super::{evidence_gap, plan_work_item_evidence, requested_interfaces};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

fn requirement(text: &str) -> Value {
    json!({ "text": text })
}

fn complete(path: &str) -> Value {
    json!({ "path": path, "complete": true })
}

#[test]
fn call_shaped_identifiers_in_requirements_are_requested_interfaces() {
    let requirements = [
        requirement("Proposed addition: assessContext(evidence) and same_meaning(a, b)."),
        requirement("Keep assessContext(evidence) stable; Math.max(a) is a member call."),
    ];
    assert_eq!(
        requested_interfaces(&requirements),
        vec!["assessContext".to_owned(), "same_meaning".to_owned()]
    );
}

#[test]
fn an_interface_no_read_source_defines_is_the_named_gap() {
    let requirements = [requirement("Add assessContext(evidence) to js/rules.mjs.")];
    let sources = ["export function aggregate(evidence) { return evidence; }".to_owned()];
    let gap = evidence_gap(&requirements, &[complete("js/rules.mjs")], &sources);
    assert_eq!(
        gap,
        Some((
            "RequestedInterfaceAbsent",
            vec!["create-interface:assessContext".to_owned()]
        ))
    );
}

#[test]
fn a_defined_interface_leaves_the_named_files_without_a_change_recipe() {
    let requirements = [requirement(
        "Change rules.mjs so aggregate(evidence) keeps provenance.",
    )];
    let sources = ["export function aggregate(evidence) { return evidence; }".to_owned()];
    let observations = [complete("js/rules.mjs"), complete("js/other.mjs")];
    assert_eq!(
        evidence_gap(&requirements, &observations, &sources),
        Some((
            "NoChangeRecipe",
            vec!["change-recipe-unavailable:js/rules.mjs".to_owned()]
        ))
    );
    assert_eq!(evidence_gap(&requirements, &[], &sources), None);
}

#[test]
fn a_work_item_linking_only_related_pages_names_each_as_unresolved() {
    let task = "Organize the blockers.\n\nSee https://github.com/example/other/issues/1 and https://github.com/example/tool.";
    let messages = vec![ChatMessage::user(task)];
    let mut result = None;
    let plan = plan_work_item_evidence(
        task,
        &messages,
        &["read", "write"],
        "https://github.com/example/project/issues/9",
        None,
        &mut result,
    );
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("a related-only work item ends with a final report");
    };
    let record = crate::agentic_coding::work_item_report::reported_work_item_record(&answer)
        .expect("a work-item final keeps its record");
    assert_eq!(record["reason"], json!("RelatedEvidenceUnread"));
    let rows = record["unresolved"]
        .as_array()
        .expect("unresolved rows")
        .iter()
        .map(|row| {
            (
                row["reference"].as_str().unwrap_or_default().to_owned(),
                row["reason"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            (
                "https://github.com/example/other/issues/1".to_owned(),
                "RelatedPageNotRead".to_owned()
            ),
            (
                "https://github.com/example/tool".to_owned(),
                "RelatedPageNotRead".to_owned()
            ),
        ]
    );
    let mut none = None;
    assert!(
        plan_work_item_evidence(
            "Organize the blockers.",
            &[],
            &["read"],
            "",
            None,
            &mut none
        )
        .is_none(),
        "a work item with no lead is left to the general planner"
    );
}
