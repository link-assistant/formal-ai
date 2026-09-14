use formal_ai::handle_api_request;
use serde_json::{Value, json};

#[test]
fn mcp_refuses_computer_effects_without_agent_mode() {
    let body = json!({
        "jsonrpc":"2.0",
        "id":2,
        "method":"tools/call",
        "params":{
            "name":"fs.write",
            "arguments":{
                "plan_id":format!("denied-{}", std::process::id()),
                "step_id":"denied-01",
                "precondition":"permission",
                "postcondition":"written",
                "path":"denied.txt",
                "content":"must not be written",
                "confirmed":true
            }
        }
    });
    let response = handle_api_request("POST", "/mcp", &body.to_string());
    let body: Value = serde_json::from_str(&response.body).expect("MCP call JSON");
    assert_eq!(body["result"]["isError"], true);
    assert_eq!(
        body["result"]["structuredContent"]["events"][0]["phase"],
        "precondition"
    );
    assert_eq!(
        body["result"]["structuredContent"]["events"][0]["passed"],
        false
    );
    assert!(
        body["result"]["structuredContent"]["output"]["error"]
            .as_str()
            .is_some_and(|error| error.contains("agent_mode_required"))
    );
}
