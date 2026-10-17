//! Actual native planner route using shared explicitly synthetic provider observations.
use super::super::final_result::FinalResult;
use super::super::planner::{AgenticPlan, plan_chat_step_resolved};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

fn projected(plan: Option<AgenticPlan>, result: Option<FinalResult>) -> Value {
    match plan {
        None => Value::Null,
        Some(AgenticPlan::ToolCalls(calls)) => {
            json!({"kind":"tool_calls","calls":calls.iter().map(|call| {
            json!({"tool":call.tool,"arguments":serde_json::from_str::<Value>(&call.arguments).unwrap()})
        }).collect::<Vec<_>>()})
        }
        Some(AgenticPlan::Final(answer)) => {
            let result = result.expect("typed final disposition");
            json!({"kind":"final","answer":answer,"disposition":format!("{:?}",result.disposition).to_lowercase(),"origin":result.origin})
        }
    }
}

#[test]
fn shared_declarative_observer_vectors() {
    let fixture: Value = serde_json::from_str(include_str!("observations.json")).unwrap();
    for row in fixture["cases"].as_array().unwrap() {
        let result = super::derive_source_maintenance(
            row["property"].as_str().unwrap(),
            row["language"].as_str().unwrap(),
            row["source"].as_str().unwrap(),
        );
        let actual = match result {
            Ok(source) => json!({"kind":"normalized","source":source}),
            Err(reason) => json!({"kind":"gap","reason":reason}),
        };
        assert_eq!(actual, row["expected"], "{}", row["id"]);
    }
}

#[test]
fn actual_native_planner_consumes_same_canonical_states_and_tool_receipts() {
    let fixture: Value = serde_json::from_str(include_str!("routes.json")).unwrap();
    for row in fixture["cases"].as_array().unwrap() {
        let messages: Vec<ChatMessage> = serde_json::from_value(row["messages"].clone()).unwrap();
        let tools = row["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let mut result = None;
        let plan = super::plan_source_maintenance_step(
            row["prompt"].as_str().unwrap(),
            &messages,
            &tools,
            &mut result,
        );
        assert_eq!(projected(plan, result), row["expected"], "{}", row["id"]);
        if row["expected"].is_null() {
            continue;
        }
        let selected =
            plan_chat_step_resolved(&messages, &tools).expect("actual native production entry");
        assert_eq!(
            projected(Some(selected.plan), selected.result),
            row["expected"],
            "{} production route",
            row["id"]
        );
    }
}
