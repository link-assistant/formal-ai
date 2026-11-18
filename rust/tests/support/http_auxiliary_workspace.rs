//! A client workspace for observing auxiliary plan persistence over HTTP.
use formal_ai::agentic_coding::general_planner::{PLAN_PATH, compose_general_change_plan};
use serde_json::{Value, json};
use std::collections::HashMap;

pub fn observed_target_write(
    prompt: &str,
    target: &str,
    prior: &str,
    next: &mut dyn FnMut(&[Value]) -> Value,
) -> Value {
    let plan = compose_general_change_plan(prompt).expect("literal write plan");
    assert_eq!(plan.target, target);
    let expected = format!("{prior}{}", plan.links_notation());
    let mut files = HashMap::new();
    if !prior.is_empty() {
        files.insert(PLAN_PATH.to_owned(), prior.to_owned());
    }
    let mut messages = vec![json!({"role": "user", "content": prompt})];
    let mut stream_observed = false;
    for _ in 0..8 {
        let response = next(&messages);
        assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
        let calls = response["choices"][0]["message"]["tool_calls"]
            .as_array()
            .expect("actual HTTP tool calls");
        assert_eq!(calls.len(), 1);
        let call = &calls[0];
        let tool = call["function"]["name"].as_str().expect("tool name");
        let arguments: Value = serde_json::from_str(
            call["function"]["arguments"]
                .as_str()
                .expect("tool arguments"),
        )
        .expect("JSON arguments");
        let path = arguments["path"].as_str().expect("declared path");
        assert!(
            path == PLAN_PATH || path == target,
            "unexpected path: {arguments}"
        );
        if tool == "write_file" && path == target {
            assert!(
                stream_observed,
                "target write requires observed retained event"
            );
            assert_eq!(arguments["content"], plan.content);
            assert_eq!(files.get(PLAN_PATH), Some(&expected));
            return response;
        }
        let result = match tool {
            "read_file" => {
                if path == PLAN_PATH && files.get(path) == Some(&expected) {
                    stream_observed = true;
                }
                files.get(path).cloned().unwrap_or_else(|| {
                    json!({"is_error": true, "error": format!("File not found: {path}")})
                        .to_string()
                })
            }
            "write_file" => {
                assert_eq!(path, PLAN_PATH, "only the auxiliary write is consumed");
                let content = arguments["content"].as_str().expect("written stream");
                assert_eq!(content, expected, "earlier event bytes must be retained");
                files.insert(path.to_owned(), content.to_owned());
                json!({"success": true}).to_string()
            }
            _ => panic!("unexpected auxiliary tool: {tool}: {arguments}"),
        };
        messages.push(json!({"role": "assistant", "tool_calls": calls}));
        let read_observation = (tool == "read_file").then(|| {
            let success = files.contains_key(path);
            json!({"path": path, "format": "raw", "success": success, "complete": success,
                "error_code": if success { Value::Null } else { json!("ENOENT") }})
        });
        messages.push(json!({
            "role": "tool", "tool_call_id": call["id"], "name": tool,
            "content": result, "source_read": read_observation,
            "is_error": tool == "read_file" && !files.contains_key(path)
        }));
    }
    panic!("no target write after the bounded observed auxiliary setup");
}

/// Preserve the original edit-only request as a capability-gap control.
pub fn assert_read_capability_gap(text: &str, target: &str) {
    let gap: Value = serde_json::from_str(text).expect("typed read capability gap");
    assert_eq!(gap["schema"], "workspace-edit-capability-gap/v1");
    assert_eq!(gap["target"], target);
    assert_eq!(gap["reason"], "ReadToolUnavailable");
    assert_eq!(gap["missingCapabilities"], json!(["read"]));
    assert_eq!(gap["complete"], false);
    assert_eq!(gap["verified"], false);
}

/// Only this owning reader constructs the production complete-source frame.
fn actual_source_frame(target: &str, source: &str) -> String {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-http-source-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("isolated source workspace");
    let path = directory.join(target);
    std::fs::write(&path, source).expect("actual fixture source");
    let actual = std::fs::read_to_string(&path).expect("actual complete source read");
    assert_eq!(actual.as_bytes(), source.as_bytes());
    std::fs::remove_dir_all(directory).expect("remove isolated source workspace");
    formal_ai::agentic_coding::tool_result::complete_owned_source_read_frame(&actual)
}

/// The unchanged request refuses without Read, then follows a real owning read.
pub fn observed_chat_edit(
    request: &Value,
    target: &str,
    source: &str,
    edit_tool: &str,
    next: &mut dyn FnMut(&Value) -> Value,
) -> Value {
    let refused = next(request);
    let refused_message = &refused["choices"][0]["message"];
    assert!(
        refused_message["tool_calls"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
    assert_read_capability_gap(refused_message["content"].as_str().unwrap(), target);
    let mut grounded = request.clone();
    grounded["tools"].as_array_mut().unwrap().push(json!({
        "type": "function", "function": {"name": "read_file", "parameters": {"type": "object"}}
    }));
    let response = next(&grounded);
    let message = &response["choices"][0]["message"];
    let calls = message["tool_calls"]
        .as_array()
        .expect("grounding read calls");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call["function"]["name"], "read_file");
    let arguments: Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments["path"], target);
    let frame = actual_source_frame(target, source);
    let messages = grounded["messages"].as_array_mut().unwrap();
    messages.push(message.clone());
    messages.push(
        json!({"role": "tool", "tool_call_id": call["id"], "name": "read_file", "content": frame}),
    );
    let edited = next(&grounded);
    assert_eq!(
        edited["choices"][0]["message"]["tool_calls"][0]["function"]["name"],
        edit_tool
    );
    edited
}

/// Gemini uses the same original request and complete observed source bytes.
pub fn observed_gemini_edit(
    request: &Value,
    target: &str,
    source: &str,
    edit_tool: &str,
    next: &mut dyn FnMut(&Value) -> Value,
) -> Value {
    let refused = next(request);
    let parts = refused["candidates"][0]["content"]["parts"]
        .as_array()
        .unwrap();
    assert!(parts.iter().all(|part| part.get("functionCall").is_none()));
    let text = parts
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect::<String>();
    assert_read_capability_gap(&text, target);
    let mut grounded = request.clone();
    grounded["tools"][0]["functionDeclarations"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "name": "read_file", "parameters": {"type": "object"}
        }));
    let response = next(&grounded);
    let call = response["candidates"][0]["content"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|part| part.get("functionCall"))
        .expect("grounding Gemini read");
    assert_eq!(call["name"], "read_file");
    assert_eq!(call["args"]["path"], target);
    let frame = actual_source_frame(target, source);
    let mut delivered = json!({"name": "read_file", "response": {"output": frame}});
    if let Some(id) = call.get("id") {
        delivered["id"] = id.clone();
    }
    let contents = grounded["contents"].as_array_mut().unwrap();
    contents.push(json!({"role": "model", "parts": [{"functionCall": call}]}));
    contents.push(json!({"role": "user", "parts": [{"functionResponse": delivered}]}));
    let edited = next(&grounded);
    let call = edited["candidates"][0]["content"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|part| part.get("functionCall"))
        .expect("grounded Gemini edit");
    assert_eq!(call["name"], edit_tool);
    edited
}

/// Original creation has no authority to append a plan or read an absent target.
pub fn observed_target_only_write(
    prompt: &str,
    target: &str,
    prior: &str,
    next: &mut dyn FnMut(&[Value]) -> Value,
) -> Value {
    let plan = compose_general_change_plan(prompt).expect("original literal creation");
    assert_eq!(plan.target, target);
    let response = next(&[json!({"role": "user", "content": prompt})]);
    let calls = response["choices"][0]["message"]["tool_calls"]
        .as_array()
        .expect("target-only creation");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call["function"]["name"], "write_file");
    let arguments: Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments["path"], target);
    assert_eq!(arguments["content"], plan.content);
    let mut files = HashMap::from([(PLAN_PATH.to_owned(), prior.to_owned())]);
    files.insert(
        target.to_owned(),
        arguments["content"].as_str().unwrap().to_owned(),
    );
    assert_eq!(files.get(PLAN_PATH).map(String::as_str), Some(prior));
    assert_eq!(
        files.get(target).map(String::as_str),
        Some(plan.content.as_str())
    );
    let _actual_read = actual_source_frame(target, arguments["content"].as_str().unwrap());
    response
}

/// Responses retains its original refusal and binds the next edit to a real read.
pub fn observed_responses_edit(
    request: &Value,
    target: &str,
    source: &str,
    edit_tool: &str,
    next: &mut dyn FnMut(&Value) -> Value,
) -> Value {
    let refused = next(request);
    let output = refused["output"].as_array().expect("Responses output");
    assert!(output.iter().all(|item| item["type"] != "function_call"));
    let text = output
        .iter()
        .filter_map(|item| item["content"].as_array())
        .flatten()
        .filter_map(|part| part["text"].as_str())
        .collect::<String>();
    assert_read_capability_gap(&text, target);
    let mut grounded = request.clone();
    grounded["tools"].as_array_mut().unwrap().push(json!({
        "type": "function", "name": "read_file", "parameters": {"type": "object"}
    }));
    let response = next(&grounded);
    let call = response["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses grounding read");
    assert_eq!(call["name"], "read_file");
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments["path"], target);
    let frame = actual_source_frame(target, source);
    grounded["input"] = json!([
        {"role": "user", "content": request["input"]},
        call,
        {"type": "function_call_output", "call_id": call["call_id"], "output": frame}
    ]);
    let edited = next(&grounded);
    let call = edited["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses grounded edit");
    assert_eq!(call["name"], edit_tool);
    edited
}
