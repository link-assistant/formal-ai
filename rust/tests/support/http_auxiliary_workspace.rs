//! A client workspace for observing auxiliary plan persistence over HTTP.
use formal_ai::agentic_coding::general_planner::{PLAN_PATH, compose_general_change_plan};
use serde_json::{Value, json};
use std::collections::HashMap;

pub(super) fn observed_target_write(
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
        messages.push(
            json!({"role": "tool", "tool_call_id": call["id"], "name": tool, "content": result}),
        );
    }
    panic!("no target write after the bounded observed auxiliary setup");
}
