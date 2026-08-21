use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};

use crate::computer_use::{
    ComputerUsePolicy, ComputerUsePrimitive, ComputerUseSession, mcp_tool_definitions,
};
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
        "tools/list" => json_rpc_result(&id, &json!({"tools": listed_tools()})),
        "tools/call" => call_tool(&request, &id, solver),
        _ => json_rpc_error(&id, -32601, "Method not found"),
    }
}

fn call_tool(request: &Value, id: &Value, solver: &UniversalSolver) -> ApiHttpResponse {
    let Some(name) = request.pointer("/params/name").and_then(Value::as_str) else {
        return json_rpc_error(id, -32602, "tool name must be a string");
    };
    if name == "formal_ai_chat" {
        return call_chat(request, id, solver);
    }
    let Some(primitive) = ComputerUsePrimitive::from_tool_name(name) else {
        return json_rpc_error(id, -32601, "Tool not found");
    };
    call_computer_primitive(request, id, solver, primitive)
}

fn call_chat(request: &Value, id: &Value, solver: &UniversalSolver) -> ApiHttpResponse {
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

fn call_computer_primitive(
    request: &Value,
    id: &Value,
    solver: &UniversalSolver,
    primitive: ComputerUsePrimitive,
) -> ApiHttpResponse {
    let arguments = request
        .pointer("/params/arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let context = (
        required_computer_argument(&arguments, "plan_id"),
        required_computer_argument(&arguments, "step_id"),
        required_computer_argument(&arguments, "precondition"),
        required_computer_argument(&arguments, "postcondition"),
    );
    let (Some(plan_id), Some(step_id), Some(precondition), Some(postcondition)) = context else {
        return json_rpc_error(id, -32602, "computer_use_verification_context_required");
    };
    let policy = if solver.config.agent_mode {
        ComputerUsePolicy::agent_mode_all()
    } else {
        ComputerUsePolicy::deny_all()
    };
    let sessions = computer_sessions();
    let Ok(mut sessions) = sessions.lock() else {
        return json_rpc_error(id, -32603, "computer-use session lock failed");
    };
    // A server can be switched into agent mode for a later request. Keep denied
    // and enabled sessions distinct so a default-deny probe cannot poison the
    // workspace/policy later used by an explicitly enabled agent.
    let session_key = format!("{}:agent_mode={}", plan_id, solver.config.agent_mode);
    let session = match sessions.entry(session_key) {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
            let Ok(session) = ComputerUseSession::new(&plan_id, policy) else {
                return json_rpc_error(id, -32603, "computer-use workspace creation failed");
            };
            entry.insert(session)
        }
    };
    let mut record = session.execute_primitive(
        &step_id,
        primitive,
        arguments,
        &precondition,
        &postcondition,
    );
    drop(sessions);
    if let Err(error) = append_computer_use_audit(&record) {
        record.verified = false;
        if let Some(event) = record.events.last_mut() {
            event.passed = false;
            event.detail = format!("audit_persistence_failed:{error}");
        }
    }
    let text = serde_json::to_string(&record).unwrap_or_default();
    let is_error = !record.verified;
    json_rpc_result(
        id,
        &json!({
            "content": [{ "type": "text", "text": text }],
            "structuredContent": record,
            "isError": is_error
        }),
    )
}

fn required_computer_argument(arguments: &Value, name: &str) -> Option<String> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
}

fn listed_tools() -> Vec<Value> {
    let mut tools = vec![json!({
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
    })];
    tools.extend(mcp_tool_definitions());
    tools
}

fn computer_sessions() -> &'static Mutex<HashMap<String, ComputerUseSession>> {
    static SESSIONS: OnceLock<Mutex<HashMap<String, ComputerUseSession>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn append_computer_use_audit(
    record: &crate::computer_use::ComputerStepRecord,
) -> std::io::Result<()> {
    let Some(path) = std::env::var_os("FORMAL_AI_COMPUTER_USE_AUDIT_PATH") else {
        return Ok(());
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")
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
