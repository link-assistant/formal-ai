use serde_json::{json, Value};

use crate::server::ApiHttpResponse;
use crate::solver::UniversalSolver;

const PROTOCOL_VERSION: &str = "2025-06-18";

pub fn handle_mcp_request(body: &str, solver: &UniversalSolver) -> ApiHttpResponse {
    let Ok(request) = serde_json::from_str::<Value>(body) else {
        return json_rpc_error(&Value::Null, -32700, "Parse error");
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return json_rpc_error(&id, -32600, "Invalid Request");
    }
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return json_rpc_error(&id, -32600, "Invalid Request");
    };

    match method {
        "initialize" => {
            let protocol_version = request
                .pointer("/params/protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(PROTOCOL_VERSION);
            json_rpc_result(
                &id,
                &json!({
                    "protocolVersion": protocol_version,
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "formal-ai",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "instructions": mcp_text("mcp_instructions")
                }),
            )
        }
        "notifications/initialized" | "ping" => json_rpc_result(&id, &json!({})),
        "tools/list" => json_rpc_result(
            &id,
            &json!({
                "tools": [{
                    "name": "formal_ai_chat",
                    "description": mcp_text("mcp_tool_description"),
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prompt": {
                                "type": "string",
                                "description": mcp_text("mcp_prompt_description")
                            }
                        },
                        "required": ["prompt"],
                        "additionalProperties": false
                    }
                }]
            }),
        ),
        "tools/call" => call_tool(&request, &id, solver),
        _ => json_rpc_error(&id, -32601, "Method not found"),
    }
}

fn call_tool(request: &Value, id: &Value, solver: &UniversalSolver) -> ApiHttpResponse {
    if request.pointer("/params/name").and_then(Value::as_str) != Some("formal_ai_chat") {
        return json_rpc_error(id, -32601, "Tool not found");
    }
    let Some(prompt) = request
        .pointer("/params/arguments/prompt")
        .and_then(Value::as_str)
    else {
        return json_rpc_error(id, -32602, "prompt must be a string");
    };
    let answer = solver.solve(prompt);
    json_rpc_result(
        id,
        &json!({
            "content": [{ "type": "text", "text": answer.answer }],
            "isError": false
        }),
    )
}

fn mcp_text(intent: &str) -> String {
    crate::seed::response_for(intent, "en").unwrap_or_default()
}

fn json_rpc_result(id: &Value, result: &Value) -> ApiHttpResponse {
    json_response(&json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn json_rpc_error(id: &Value, code: i64, message: &str) -> ApiHttpResponse {
    json_response(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    }))
}

fn json_response(value: &Value) -> ApiHttpResponse {
    ApiHttpResponse {
        status_code: 200,
        content_type: "application/json",
        body: value.to_string(),
        deprecated: false,
    }
}
