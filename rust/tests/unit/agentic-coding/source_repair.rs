//! Native intake parity vectors; CI only.
use super::super::planner::AgenticPlan;
use super::{plan_owned_source_repair, source_repair_contract};
use crate::protocol::ChatMessage;
use serde_json::Value;
#[test]
fn shared_source_repair_contracts() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../fixtures/source-repair-intake-cases.json"
    ))
    .expect("fixtures");
    for entry in fixture["cases"].as_array().expect("cases") {
        let contract = source_repair_contract(entry["task"].as_str().expect("task"));
        if entry["unowned"] == true {
            assert!(contract.is_none());
            continue;
        }
        let contract = contract.expect("owned contract");
        assert_eq!(contract["owner"], entry["owner"]);
        assert_eq!(contract["authority"], entry["authority"]);
    }
}
#[test]
fn original_repair_first_step_reads_source() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../fixtures/source-repair-intake-cases.json"
    ))
    .expect("fixtures");
    let entry = &fixture["cases"][0];
    let task = entry["task"].as_str().expect("task");
    let plan = plan_owned_source_repair(
        task,
        &[ChatMessage::new("user", task)],
        &["read", "edit", "write"],
        None,
        &mut None,
    )
    .expect("owned route");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("Read required")
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "read");
    let arguments: Value = serde_json::from_str(&calls[0].arguments).expect("arguments");
    assert_eq!(arguments["filePath"], entry["owner"]);
}

#[test]
fn dictionary_operation_requires_owned_retention_goal() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../fixtures/source-repair-goal-cases.json"))
            .expect("goal fixtures");
    for entry in fixture["cases"].as_array().expect("cases") {
        let contract =
            source_repair_contract(entry["task"].as_str().expect("task")).expect("owned source");
        assert_eq!(
            contract["operation"], entry["operation"],
            "{}",
            entry["name"]
        );
    }
}
