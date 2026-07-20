//! Issue #746: hosted and protocol-native tool advertisements reach the shared router.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

fn post(path: &str, body: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let response = handle_api_request("POST", path, &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("valid JSON response")
}

fn responses_call(response: &Value) -> &Value {
    response["output"]
        .as_array()
        .expect("Responses output array")
        .iter()
        .find(|item| item["type"] == "function_call")
        .unwrap_or_else(|| panic!("expected a real function_call, got {response}"))
}

fn hosted_web_search_call(response: &Value) -> &Value {
    response["output"]
        .as_array()
        .expect("Responses output array")
        .iter()
        .find(|item| item["type"] == "web_search_call")
        .unwrap_or_else(|| panic!("expected a native web_search_call, got {response}"))
}

fn gemini_call(response: &Value) -> &Value {
    response["candidates"][0]["content"]["parts"]
        .as_array()
        .expect("Gemini parts array")
        .iter()
        .find_map(|part| part.get("functionCall"))
        .unwrap_or_else(|| panic!("expected a real functionCall, got {response}"))
}

#[test]
fn responses_type_only_web_search_routes_instead_of_wildcard_refusal() {
    // Explicit language: "en" coverage complements the multilingual protocol cases below.
    let response = post(
        "/v1/responses",
        &json!({
            "model": "formal-ai",
            "input": "Search online for Elon Musk",
            "tools": [{"type": "web_search"}],
            "tool_choice": "auto"
        }),
    );
    let call = hosted_web_search_call(&response);
    assert_eq!(call["status"], "completed");
    assert_eq!(call["action"]["type"], "search");
    assert!(call["action"]["query"]
        .as_str()
        .is_some_and(|query| query.contains("elon musk")));
}

#[test]
fn responses_preview_alias_maps_to_the_web_search_capability() {
    let response = post(
        "/v1/responses",
        &json!({
            "model": "formal-ai",
            "input": "Найди в интернете Rust ownership",
            "tools": [{"type": "web_search_preview"}]
        }),
    );
    assert_eq!(hosted_web_search_call(&response)["type"], "web_search_call");
}

#[test]
fn responses_prefers_client_executed_tools_inside_an_mcp_namespace() {
    let response = post(
        "/v1/responses",
        &json!({
            "model": "formal-ai",
            "input": "Найди мне совместимую зарядку для Acer Aspire 3 A325-45?",
            "tools": [
                {
                    "type": "namespace",
                    "name": "mcp__issue781",
                    "description": "Deterministic research tools",
                    "tools": [
                        {
                            "type": "function",
                            "name": "websearch",
                            "parameters": {
                                "type": "object",
                                "properties": {"query": {"type": "string"}},
                                "required": ["query"]
                            }
                        },
                        {
                            "type": "function",
                            "name": "webfetch",
                            "parameters": {
                                "type": "object",
                                "properties": {"url": {"type": "string"}},
                                "required": ["url"]
                            }
                        }
                    ]
                },
                {"type": "web_search"}
            ]
        }),
    );

    let call = responses_call(&response);
    assert_eq!(call["name"], "websearch", "{response}");
    assert_eq!(call["namespace"], "mcp__issue781", "{response}");
    assert!(
        call["arguments"]
            .as_str()
            .is_some_and(|arguments| arguments.contains("query")),
        "the nested MCP schema must be applied: {call}"
    );
    assert!(
        response["output"]
            .as_array()
            .is_some_and(|items| items.first().is_some_and(|item| item["type"] == "message")),
        "the explanation must precede the MCP call: {response}"
    );
}

#[test]
fn responses_stream_uses_native_web_search_lifecycle_events() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "input": "Search online for Rust ownership",
        "tools": [{"type": "web_search"}],
        "stream": true
    });
    let response = handle_api_request("POST", "/v1/responses", &body.to_string());

    assert_eq!(response.status_code, 200, "{}", response.body);
    assert!(response.content_type.starts_with("text/event-stream"));
    for event in [
        "response.web_search_call.in_progress",
        "response.web_search_call.searching",
        "response.web_search_call.completed",
    ] {
        assert!(
            response.body.contains(event),
            "missing {event}: {}",
            response.body
        );
    }
}

#[test]
fn anthropic_server_web_search_type_routes_to_tool_use() {
    let response = post(
        "/api/anthropic/v1/messages",
        &json!({
            "model": "formal-ai",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "वेब पर खोजें Rust ownership"}],
            "tools": [{
                "type": "web_search_20250305",
                "name": "web_search",
                "input_schema": {"type": "object", "properties": {"query": {"type": "string"}}}
            }]
        }),
    );
    assert_eq!(response["stop_reason"], "tool_use");
    let call = response["content"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["type"] == "tool_use")
        .expect("Anthropic tool_use block");
    assert_eq!(call["name"], "web_search");
}

#[test]
fn gemini_function_declarations_and_hosted_google_search_both_route() {
    let declarations = post(
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        &json!({
            "contents": [{"role": "user", "parts": [{"text": "搜索网络 Rust ownership"}]}],
            "tools": [{"functionDeclarations": [{
                "name": "google_web_search",
                "parameters": {"type": "object", "properties": {"query": {"type": "string"}}}
            }]}]
        }),
    );
    assert_eq!(gemini_call(&declarations)["name"], "google_web_search");

    for hosted in [json!({"google_search": {}}), json!({"googleSearch": {}})] {
        let response = post(
            "/api/gemini/v1beta/models/formal-ai:generateContent",
            &json!({
                "contents": [{"role": "user", "parts": [{"text": "Search online for Rust ownership"}]}],
                "tools": [hosted]
            }),
        );
        assert_eq!(gemini_call(&response)["name"], "web_search");
    }
}

#[test]
fn gemini_url_context_routes_as_web_fetch() {
    let response = post(
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        &json!({
            "contents": [{"role": "user", "parts": [{"text": "Summarize https://example.com"}]}],
            "tools": [{"url_context": {}}]
        }),
    );
    assert_eq!(gemini_call(&response)["name"], "web_fetch");
}

#[test]
fn every_specified_hosted_type_reaches_its_capability_through_the_public_api() {
    for (definition, prompt, expected) in [
        (
            json!({"type": "web_search"}),
            "Search online for Rust",
            "web_search",
        ),
        (
            json!({"type": "web_search_preview"}),
            "Search online for Rust",
            "web_search",
        ),
        (
            json!({"type": "file_search"}),
            "Search the code for TODO",
            "file_search",
        ),
        (
            json!({"type": "computer_use_preview"}),
            "Execute pwd",
            "computer_use",
        ),
        (
            json!({"type": "code_interpreter"}),
            "Execute pwd",
            "code_interpreter",
        ),
    ] {
        let response = post(
            "/v1/responses",
            &json!({
                "model": "formal-ai",
                "input": prompt,
                "tools": [definition]
            }),
        );
        if expected == "web_search" {
            assert_eq!(hosted_web_search_call(&response)["type"], "web_search_call");
        } else {
            assert_eq!(responses_call(&response)["name"], expected);
        }
    }
}

#[test]
fn whole_task_routes_web_search_across_all_advertisement_protocols() {
    let openai = post(
        "/v1/responses",
        &json!({"input": "Search the web for Rust", "tools": [{"type": "web_search"}]}),
    );
    assert_eq!(hosted_web_search_call(&openai)["type"], "web_search_call");

    let anthropic = post(
        "/api/anthropic/v1/messages",
        &json!({
            "max_tokens": 256,
            "messages": [{"role": "user", "content": "Search the web for Rust"}],
            "tools": [{"type": "web_search_20250305", "name": "web_search", "input_schema": {"type": "object"}}]
        }),
    );
    assert_eq!(anthropic["stop_reason"], "tool_use");

    let gemini = post(
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        &json!({
            "contents": [{"role": "user", "parts": [{"text": "Search the web for Rust"}]}],
            "tools": [{"googleSearch": {}}]
        }),
    );
    assert_eq!(gemini_call(&gemini)["name"], "web_search");
}

#[test]
fn qwen_discovers_a_deferred_web_tool_instead_of_searching_local_files() {
    let response = post(
        "/v1/chat/completions",
        &json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": "Search online for Elon Musk"}],
            "tools": [
                {"type": "function", "function": {
                    "name": "grep_search",
                    "parameters": {"type": "object", "properties": {"pattern": {"type": "string"}}, "required": ["pattern"]}
                }},
                {"type": "function", "function": {
                    "name": "tool_search",
                    "parameters": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}
                }}
            ]
        }),
    );
    let call = &response["choices"][0]["message"]["tool_calls"][0]["function"];
    assert_eq!(call["name"], "tool_search");
    assert!(call["arguments"]
        .as_str()
        .is_some_and(|arguments| arguments.contains("web search")));

    let call_id = response["choices"][0]["message"]["tool_calls"][0]["id"]
        .as_str()
        .expect("tool call id");
    let follow_up = post(
        "/v1/chat/completions",
        &json!({
            "model": "formal-ai",
            "messages": [
                {"role": "user", "content": "Search online for Elon Musk"},
                response["choices"][0]["message"].clone(),
                {"role": "tool", "tool_call_id": call_id, "content": "Loaded 5 tool(s)"}
            ],
            "tools": [
                {"type": "function", "function": {
                    "name": "grep_search",
                    "parameters": {"type": "object", "properties": {"pattern": {"type": "string"}}, "required": ["pattern"]}
                }},
                {"type": "function", "function": {
                    "name": "tool_search",
                    "parameters": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}
                }}
            ]
        }),
    );
    assert!(
        follow_up["choices"][0]["message"]["tool_calls"]
            .as_array()
            .is_none_or(|calls| calls
                .iter()
                .all(|call| call["function"]["name"] != "tool_search")),
        "deferred discovery must not repeat after its result: {follow_up}"
    );
}
