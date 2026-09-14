use formal_ai::computer_use::{COMPUTER_USE_PRIMITIVES, benchmark_tasks};
use formal_ai::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{Value, json};

#[test]
fn mcp_advertises_the_complete_computer_use_taxonomy_with_schemas() {
    let response = handle_api_request(
        "POST",
        "/mcp",
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#,
    );
    let body: Value = serde_json::from_str(&response.body).expect("MCP list JSON");
    let tools = body["result"]["tools"].as_array().expect("tools");
    assert_eq!(tools.len(), 13);
    assert_eq!(tools[0]["name"], "formal_ai_chat");
    for primitive in COMPUTER_USE_PRIMITIVES {
        let definition = tools
            .iter()
            .find(|tool| tool["name"] == primitive.name())
            .unwrap_or_else(|| panic!("missing {}", primitive.name()));
        assert_eq!(definition["inputSchema"]["type"], "object");
        assert_eq!(definition["inputSchema"]["additionalProperties"], false);
        let required = definition["inputSchema"]["required"]
            .as_array()
            .expect("required");
        for common in ["plan_id", "step_id", "precondition", "postcondition"] {
            assert!(
                required.iter().any(|value| value == common),
                "{} missing {common}",
                primitive.name()
            );
        }
    }
}

#[test]
fn mcp_executes_and_verifies_all_ten_plans_in_agent_mode() {
    enable_http_agent_mode_for_current_process();
    for task in benchmark_tasks() {
        let plan_id = format!("mcp-{}-{}", std::process::id(), task.id);
        for step in &task.steps {
            let mut arguments = step.arguments.clone();
            let object = arguments.as_object_mut().expect("arguments object");
            object.insert("plan_id".to_owned(), Value::String(plan_id.clone()));
            object.insert("step_id".to_owned(), Value::String(step.id.clone()));
            object.insert(
                "precondition".to_owned(),
                Value::String(step.precondition.clone()),
            );
            object.insert(
                "postcondition".to_owned(),
                Value::String(step.postcondition.clone()),
            );
            let body = json!({
                "jsonrpc":"2.0",
                "id":step.id,
                "method":"tools/call",
                "params":{"name":step.primitive.name(),"arguments":arguments}
            });
            let response = handle_api_request("POST", "/mcp", &body.to_string());
            let body: Value = serde_json::from_str(&response.body).expect("MCP result");
            assert_eq!(
                body["result"]["isError"], false,
                "{} {}: {}",
                task.id, step.id, body
            );
            let record = &body["result"]["structuredContent"];
            assert_eq!(record["verified"], true);
            assert_eq!(record["events"].as_array().map(Vec::len), Some(3));
            assert_eq!(record["events"][0]["phase"], "precondition");
            assert_eq!(record["events"][1]["phase"], "effect");
            assert_eq!(record["events"][2]["phase"], "postcondition");
        }
    }
}

#[test]
fn mcp_rejects_a_primitive_call_without_verification_context() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "jsonrpc":"2.0",
        "id":3,
        "method":"tools/call",
        "params":{
            "name":"fs.write",
            "arguments":{
                "path":"unscoped.txt",
                "content":"must not be written",
                "confirmed":true
            }
        }
    });
    let response = handle_api_request("POST", "/mcp", &body.to_string());
    let body: Value = serde_json::from_str(&response.body).expect("MCP call JSON");
    assert_eq!(body["error"]["code"], -32602, "{body}");
    assert!(body.get("result").is_none(), "{body}");
}
