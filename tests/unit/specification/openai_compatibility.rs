//! `OpenAI`-compatible Chat Completions and Responses tests.
//!
//! `VISION.md` and `REQUIREMENTS.md` (R39-R42, R55-R70) require that any
//! production-ready AI assistant talks the `OpenAI` Chat Completions and
//! Responses dialects so that existing tooling can call `formal-ai` as a
//! drop-in symbolic engine.

use std::sync::{Mutex, MutexGuard, OnceLock};

use formal_ai::{
    create_chat_completion, create_response, export_memory_links_notation, handle_api_request,
    handle_api_request_with_auth, model_aliases, resolve_model_id, ApiAuthConfig,
    ChatCompletionRequest, ChatMessage, MemoryEvent, ResponsesRequest, DEFAULT_MODEL,
};

// ---------------------------------------------------------------------------
// Active expectations: the OpenAI surfaces already implemented today.
// ---------------------------------------------------------------------------
#[test]
fn chat_completion_round_trips_user_prompt_to_assistant_response() {
    let request = ChatCompletionRequest {
        model: Some(String::from("formal-ai")),
        messages: vec![ChatMessage::user("Hi")],
        temperature: Some(0.0),
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);

    assert_eq!(completion.object, "chat.completion");
    assert_eq!(completion.choices.len(), 1);
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert_eq!(completion.choices[0].message.role, "assistant");
    assert_eq!(
        completion.choices[0].message.content.plain_text(),
        "Hi, how may I help you?"
    );
}
#[test]
fn chat_completion_accepts_multipart_content() {
    let parts = serde_json::json!([
        {"type": "text", "text": "Write hello world in Rust"}
    ]);
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: serde_json::from_value(parts).unwrap(),
            ..Default::default()
        }],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);
    assert!(completion.choices[0]
        .message
        .content
        .plain_text()
        .contains("```rust"));
}
#[test]
fn chat_completion_reports_token_usage() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user("Hello")],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    let completion = create_chat_completion(&request);
    assert!(completion.usage.prompt_tokens > 0);
    assert!(completion.usage.completion_tokens > 0);
    assert_eq!(
        completion.usage.total_tokens,
        completion.usage.prompt_tokens + completion.usage.completion_tokens
    );
}
#[test]
fn chat_completion_includes_ordered_thinking_steps() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user("Hi")],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);
    let steps = &completion.choices[0].message.thinking_steps;

    assert!(
        !steps.is_empty(),
        "assistant message should expose thinking steps"
    );
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[0].step, "impulse");
    assert!(steps.iter().any(|step| step.step == "formalize"));
    assert!(steps.iter().any(|step| step.step == "deformalize"));
}
#[test]
fn chat_completion_includes_standard_reasoning_content() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user("Hi")],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);
    let message = &completion.choices[0].message;

    assert!(!message.reasoning_content.is_empty());
    assert_eq!(message.reasoning, message.reasoning_content);
    assert!(
        message.reasoning_content.contains("Read the request"),
        "reasoning_content should render concrete thinking"
    );
}
#[test]
fn responses_endpoint_returns_completed_response() {
    let request = ResponsesRequest {
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: Some(0.0),
        stream: false,
        ..ResponsesRequest::default()
    };
    let response = create_response(&request);
    assert_eq!(response.object, "response");
    assert_eq!(response.status, "completed");
    let messages = response.output_messages();
    assert_eq!(messages[0].role, "assistant");
    assert_eq!(messages[0].content[0].kind, "output_text");
}
#[test]
fn responses_instructions_do_not_prefix_the_latest_user_turn() {
    let bare = ResponsesRequest {
        input: serde_json::json!([{
            "role": "user",
            "content": [{"type": "input_text", "text": "hi"}]
        }]),
        ..ResponsesRequest::default()
    };
    let steered = ResponsesRequest {
        instructions: Some(String::from(
            "You are a coding agent. Decompose requests into sub-tasks.",
        )),
        ..bare.clone()
    };

    let bare_response = create_response(&bare);
    let steered_response = create_response(&steered);
    let bare_text = &bare_response.output_messages()[0].content[0].text;
    let steered_text = &steered_response.output_messages()[0].content[0].text;
    assert_eq!(steered_text, bare_text);
    assert!(steered_text.to_ascii_lowercase().contains("hi"));
}
#[test]
fn responses_endpoint_includes_top_level_and_message_thinking_steps() {
    let request = ResponsesRequest {
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: Some(0.0),
        stream: false,
        ..ResponsesRequest::default()
    };

    let response = create_response(&request);
    let messages = response.output_messages();

    assert!(!response.thinking_steps.is_empty());
    assert_eq!(messages[0].thinking_steps, response.thinking_steps);
    assert!(response
        .thinking_steps
        .iter()
        .any(|step| step.source_event == "response"));
}
#[test]
fn responses_endpoint_includes_standard_reasoning_output_item() {
    let request = ResponsesRequest {
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: Some(0.0),
        stream: false,
        ..ResponsesRequest::default()
    };

    let response = create_response(&request);
    let serialized = serde_json::to_value(&response).unwrap();
    let output = serialized["output"]
        .as_array()
        .expect("Responses output should be an array");
    let reasoning = output
        .iter()
        .find(|item| item["type"] == "reasoning")
        .expect("Responses output should include a reasoning item");

    assert_eq!(reasoning["summary"][0]["type"], "summary_text");
    assert!(
        reasoning["summary"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("Read the request"),
        "reasoning summary should render concrete thinking, got: {reasoning}"
    );
}
#[test]
fn openai_requests_accept_temperature_parameter() {
    let chat: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}],
        "temperature": 0.0
    }))
    .unwrap();
    assert_eq!(chat.temperature, Some(0.0));

    let response: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": "Hi",
        "temperature": 0.25
    }))
    .unwrap();
    assert_eq!(response.temperature, Some(0.25));
}
#[test]
fn http_health_endpoint_returns_ok() {
    let response = handle_api_request("GET", "/health", "");
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("ok") || response.body.contains("healthy"));
}
#[test]
fn http_models_endpoint_lists_at_least_one_model() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "list");
    assert!(json["data"].as_array().map_or(0, Vec::len) >= 1);
    assert!(
        json["models"].as_array().map_or(0, Vec::len) >= 1,
        "model list should also expose Codex-compatible `models` metadata"
    );
}
#[test]
fn canonical_model_id_is_formal_ai() {
    assert_eq!(DEFAULT_MODEL, "formal-ai");
}
#[test]
fn model_aliases_are_loaded_from_seed_data() {
    let registry = model_aliases();
    assert_eq!(registry.canonical_id, DEFAULT_MODEL);
    assert_eq!(
        registry.aliases,
        vec![
            "formal-ai",
            "@link-assistant/formal-ai",
            "link-assistant/formal-ai",
            "formal-ai-latest",
            "latest",
        ]
    );
    assert_eq!(
        resolve_model_id(Some("@LINK-ASSISTANT/FORMAL-AI")),
        DEFAULT_MODEL
    );
    assert_eq!(resolve_model_id(Some("latest")), DEFAULT_MODEL);
}
#[test]
fn http_models_endpoint_advertises_only_formal_ai() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let ids: Vec<&str> = json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|model| model["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["formal-ai"]);
    let retired_model_id = ["formal", "symbolic", "production"].join("-");
    assert!(!response.body.contains(&retired_model_id));
}
#[test]
fn http_openai_surfaces_accept_model_aliases_and_return_canonical_model() {
    let aliases = [
        "formal-ai",
        "FORMAL-AI",
        "@link-assistant/formal-ai",
        "@LINK-ASSISTANT/FORMAL-AI",
        "link-assistant/formal-ai",
        "formal-ai-latest",
        "latest",
    ];

    for alias in aliases {
        assert_chat_alias_resolves_to_formal_ai(alias);
        assert_response_alias_resolves_to_formal_ai(alias);
        assert_anthropic_alias_resolves_to_formal_ai(alias);
    }
}
#[test]
fn http_openai_surfaces_reject_unsupported_explicit_model_ids() {
    let requests = [
        (
            "/v1/chat/completions",
            serde_json::json!({
                "model": "unsupported-model",
                "messages": [{"role": "user", "content": "Hi"}]
            }),
        ),
        (
            "/v1/responses",
            serde_json::json!({
                "model": "unsupported-model",
                "input": "Hi"
            }),
        ),
        (
            "/v1/messages",
            serde_json::json!({
                "model": "unsupported-model",
                "messages": [{"role": "user", "content": "Hi"}]
            }),
        ),
    ];

    for (path, body) in requests {
        let response = handle_api_request("POST", path, &body.to_string());
        assert_eq!(response.status_code, 400, "{path}");
        assert!(response.body.contains("unsupported model"), "{path}");
        assert!(response.body.contains(DEFAULT_MODEL), "{path}");
    }
}
#[test]
fn http_chat_completion_returns_canonical_model_for_multilingual_prompts() {
    struct PromptCase<'a> {
        language: &'a str,
        prompt: &'a str,
    }

    let cases = [
        PromptCase {
            language: "en",
            prompt: "Say hello.",
        },
        PromptCase {
            language: "ru",
            prompt: "Скажи привет.",
        },
        PromptCase {
            language: "hi",
            prompt: "नमस्ते कहो.",
        },
        PromptCase {
            language: "zh",
            prompt: "说你好。",
        },
    ];

    for case in cases {
        let body = serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": case.prompt}]
        })
        .to_string();
        let response = handle_api_request("POST", "/v1/chat/completions", &body);
        assert_eq!(response.status_code, 200, "{}", case.language);
        let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(json["model"], "formal-ai", "{}", case.language);
    }
}

fn assert_chat_alias_resolves_to_formal_ai(alias: &str) {
    let body = serde_json::json!({
        "model": alias,
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert_eq!(response.status_code, 200, "chat alias {alias}");
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["model"], "formal-ai", "chat alias {alias}");
    assert_eq!(
        json["choices"][0]["message"]["content"], "Hi, how may I help you?",
        "chat alias {alias}"
    );
}

fn assert_response_alias_resolves_to_formal_ai(alias: &str) {
    let body = serde_json::json!({
        "model": alias,
        "input": "Hi"
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/responses", &body);
    assert_eq!(response.status_code, 200, "responses alias {alias}");
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["model"], "formal-ai", "responses alias {alias}");
    assert_eq!(
        json["output"][0]["content"][0]["text"], "Hi, how may I help you?",
        "responses alias {alias}"
    );
}

fn assert_anthropic_alias_resolves_to_formal_ai(alias: &str) {
    let body = serde_json::json!({
        "model": alias,
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/messages", &body);
    assert_eq!(response.status_code, 200, "messages alias {alias}");
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["model"], "formal-ai", "messages alias {alias}");
    assert_eq!(
        json["content"][0]["text"], "Hi, how may I help you?",
        "messages alias {alias}"
    );
}
#[test]
fn http_chat_completions_route_returns_completion_object() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "chat.completion");
}
#[test]
fn protocol_namespaces_route_to_the_same_openai_and_formal_ai_surfaces() {
    let openai_models = handle_api_request("GET", "/api/openai/v1/models", "");
    assert_eq!(openai_models.status_code, 200);
    let openai_json: serde_json::Value = serde_json::from_str(&openai_models.body).unwrap();
    assert_eq!(openai_json["data"][0]["id"], "formal-ai");

    let legacy_models = handle_api_request("GET", "/v1/models", "");
    assert_eq!(legacy_models.status_code, 200);
    let legacy_json: serde_json::Value = serde_json::from_str(&legacy_models.body).unwrap();
    for pointer in ["/object", "/data/0/id", "/models/0/name", "/rate_limit"] {
        assert_eq!(openai_json.pointer(pointer), legacy_json.pointer(pointer));
    }

    let graph = handle_api_request("GET", "/api/formal-ai/v1/graph", "");
    assert_eq!(graph.status_code, 200);
    assert!(
        graph.body.contains("nodes"),
        "Formal AI native graph should be available under /api/formal-ai/v1"
    );
}
#[test]
fn responses_stream_true_emits_responses_sse_protocol_on_openai_routes() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "input": "Hi",
        "stream": true
    })
    .to_string();
    for path in ["/v1/responses", "/api/openai/v1/responses"] {
        let response = handle_api_request("POST", path, &body);
        assert_eq!(response.status_code, 200, "{path}");
        assert!(
            response.content_type.contains("text/event-stream"),
            "streaming Responses must use SSE content-type on {path}, got: {}",
            response.content_type
        );
        let events = sse_event_names(&response.body);
        assert_eq!(events.first().copied(), Some("response.created"), "{path}");
        assert!(
            events.contains(&"response.output_item.added"),
            "{path}: {events:?}"
        );
        assert!(
            events.contains(&"response.output_text.delta"),
            "{path}: {events:?}"
        );
        assert!(
            events.contains(&"response.output_item.done"),
            "{path}: {events:?}"
        );
        assert_eq!(events.last().copied(), Some("response.completed"), "{path}");
        assert!(
            response.body.contains("Hi, how may I help you?"),
            "stream should contain output_text delta data on {path}: {}",
            response.body
        );
    }
}
#[test]
fn gemini_and_vertex_protocols_share_the_solver_with_native_model_lists() {
    let gemini_models = handle_api_request("GET", "/api/gemini/v1beta/models", "");
    assert_eq!(gemini_models.status_code, 200);
    let gemini_json: serde_json::Value = serde_json::from_str(&gemini_models.body).unwrap();
    assert_eq!(gemini_json["models"][0]["name"], "models/formal-ai");
    assert!(gemini_json["models"][0]["supportedGenerationMethods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method == "generateContent"));

    let gemini_body = serde_json::json!({
        "contents": [{"role": "user", "parts": [{"text": "Hi"}]}]
    })
    .to_string();
    let gemini_response = handle_api_request(
        "POST",
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        &gemini_body,
    );
    assert_eq!(gemini_response.status_code, 200);
    let gemini: serde_json::Value = serde_json::from_str(&gemini_response.body).unwrap();
    assert_eq!(
        gemini["candidates"][0]["content"]["parts"][0]["text"],
        "Hi, how may I help you?"
    );

    let vertex_models = handle_api_request(
        "GET",
        "/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models",
        "",
    );
    assert_eq!(vertex_models.status_code, 200);
    let vertex_json: serde_json::Value = serde_json::from_str(&vertex_models.body).unwrap();
    assert_eq!(
        vertex_json["publisherModels"][0]["name"],
        "projects/local/locations/us-central1/publishers/google/models/formal-ai"
    );

    let vertex_response = handle_api_request(
        "POST",
        "/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models/formal-ai:generateContent",
        &gemini_body,
    );
    assert_eq!(vertex_response.status_code, 200);
    let vertex: serde_json::Value = serde_json::from_str(&vertex_response.body).unwrap();
    assert_eq!(
        vertex["candidates"][0]["content"]["parts"][0]["text"],
        "Hi, how may I help you?"
    );
}
#[test]
fn http_chat_completion_queries_persisted_memory_with_natural_language() {
    let response = with_recall_memory(|| {
        let body = serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": "Find Rust in another conversation"}]
        })
        .to_string();
        handle_api_request("POST", "/v1/chat/completions", &body)
    });

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default();
    assert!(content.contains("Rust Notes"), "{content}");
    assert!(content.contains("user: What is Rust?"), "{content}");
    assert!(
        !content.contains("What is Wikipedia?"),
        "query should not include unrelated memory: {content}"
    );
}
#[test]
fn http_responses_route_returns_response_object() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "input": "Hi"
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/responses", &body);
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "response");
}
#[test]
fn http_responses_route_queries_persisted_memory_with_natural_language() {
    let response = with_recall_memory(|| {
        let body = serde_json::json!({
            "model": "formal-ai",
            "input": "Find Rust in another conversation"
        })
        .to_string();
        handle_api_request("POST", "/v1/responses", &body)
    });

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let content = json["output"][0]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(content.contains("Rust Notes"), "{content}");
    assert!(content.contains("user: What is Rust?"), "{content}");
    assert!(
        !content.contains("What is Wikipedia?"),
        "query should not include unrelated memory: {content}"
    );
}

fn with_recall_memory<T>(run: impl FnOnce() -> T) -> T {
    let _guard = memory_env_lock();
    let dir = std::env::temp_dir().join(format!("formal-ai-memory-query-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("memory.lino");
    let memory = export_memory_links_notation(&[
        memory_event(
            "a1",
            "message",
            "user",
            "conv-a",
            "Rust Notes",
            "What is Rust?",
        ),
        memory_event(
            "a2",
            "message",
            "assistant",
            "conv-a",
            "Rust Notes",
            "Rust is a systems programming language.",
        ),
        memory_event(
            "b1",
            "message",
            "user",
            "conv-b",
            "Wikipedia Notes",
            "What is Wikipedia?",
        ),
    ]);
    std::fs::write(&path, memory).expect("write memory");

    let previous = std::env::var_os("FORMAL_AI_MEMORY_PATH");
    std::env::set_var("FORMAL_AI_MEMORY_PATH", &path);
    let result = run();
    match previous {
        Some(value) => std::env::set_var("FORMAL_AI_MEMORY_PATH", value),
        None => std::env::remove_var("FORMAL_AI_MEMORY_PATH"),
    }
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn memory_event(
    id: &str,
    kind: &str,
    role: &str,
    conversation_id: &str,
    conversation_title: &str,
    content: &str,
) -> MemoryEvent {
    MemoryEvent {
        id: id.to_owned(),
        kind: Some(kind.to_owned()),
        role: Some(role.to_owned()),
        content: Some(content.to_owned()),
        conversation_id: Some(conversation_id.to_owned()),
        conversation_title: Some(conversation_title.to_owned()),
        ..MemoryEvent::default()
    }
}

fn memory_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn sse_event_names(body: &str) -> Vec<&str> {
    body.lines()
        .filter_map(|line| line.strip_prefix("event: "))
        .collect()
}
#[test]
fn http_unknown_route_returns_404() {
    let response = handle_api_request("GET", "/this-route-does-not-exist", "");
    assert_eq!(response.status_code, 404);
}
#[test]
fn http_chat_completions_route_rejects_invalid_json() {
    let response = handle_api_request("POST", "/v1/chat/completions", "{not json");
    assert!(response.status_code >= 400);
    assert!(response.status_code < 500);
}
#[test]
fn http_responses_declare_a_content_type() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert!(
        !response.content_type.is_empty(),
        "every HTTP response must declare a Content-Type so clients can parse it"
    );
}

// ---------------------------------------------------------------------------
// full-scope expectations: behaviors documented in VISION.md/GOALS.md/REQUIREMENTS.md
// that are not yet implemented.
// ---------------------------------------------------------------------------
#[test]
fn chat_completion_supports_multi_turn_conversation() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![
            ChatMessage::user("My name is Ada."),
            ChatMessage::assistant("Nice to meet you, Ada."),
            ChatMessage::user("What is my name?"),
        ],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    let completion = create_chat_completion(&request);
    assert!(
        completion.choices[0]
            .message
            .content
            .plain_text()
            .to_lowercase()
            .contains("ada"),
        "multi-turn chat should remember names introduced earlier in the conversation"
    );
}
#[test]
fn chat_completion_applies_behavior_rule_from_prior_messages() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![
            ChatMessage::user(
                "When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`",
            ),
            ChatMessage::assistant("Behavior rule recorded for this dialog."),
            ChatMessage::user("Какая у тебя модель личности?"),
        ],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);
    assert_eq!(
        completion.choices[0].message.content.plain_text(),
        "У меня символьная модель личности."
    );
}
#[test]
fn streaming_chat_completion_emits_server_sent_events() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}],
        "stream": true
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert!(
        response.content_type.contains("text/event-stream"),
        "streaming responses must use SSE content-type, got: {}",
        response.content_type
    );
    assert!(response.body.contains("data: "));
    assert!(response.body.contains("[DONE]"));
}
#[test]
fn streaming_chat_completion_emits_reasoning_content_delta() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}],
        "stream": true
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);

    assert_eq!(response.status_code, 200);
    assert!(
        response.body.contains("\"reasoning_content\""),
        "streaming chat should emit delta.reasoning_content, got: {}",
        response.body
    );
    assert!(
        response.body.contains("Read the request"),
        "reasoning delta should include concrete thinking, got: {}",
        response.body
    );
    let reasoning_index = response
        .body
        .find("\"reasoning_content\"")
        .expect("reasoning_content delta should be present");
    let content_index = response
        .body
        .find("\"content\"")
        .expect("content delta should be present");
    assert!(
        reasoning_index < content_index,
        "reasoning should stream before answer content"
    );
}
#[test]
fn streaming_responses_emit_reasoning_summary_events() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "input": "Hi",
        "stream": true
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/responses", &body);

    assert_eq!(response.status_code, 200);
    assert!(
        response.content_type.contains("text/event-stream"),
        "streaming Responses must use SSE content-type, got: {}",
        response.content_type
    );
    assert!(
        response
            .body
            .contains("event: response.reasoning_summary_text.delta"),
        "Responses stream should emit reasoning summary deltas, got: {}",
        response.body
    );
    assert!(
        response.body.contains("Read the request"),
        "reasoning summary should include concrete thinking, got: {}",
        response.body
    );
    assert!(
        response.body.contains("event: response.output_text.delta"),
        "Responses stream should emit output_text deltas, got: {}",
        response.body
    );
    let reasoning_index = response
        .body
        .find("event: response.reasoning_summary_text.delta")
        .expect("reasoning summary delta should be present");
    let text_index = response
        .body
        .find("event: response.output_text.delta")
        .expect("output_text delta should be present");
    assert!(
        reasoning_index < text_index,
        "Responses reasoning summary should stream before output text"
    );
}
#[test]
fn authenticated_routes_accept_bearer_token() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let auth = ApiAuthConfig::bearer_token("local-test-token");
    let response = handle_api_request_with_auth(
        "POST",
        "/v1/chat/completions",
        &[("Authorization", "Bearer local-test-token")],
        &body,
        &auth,
    );
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "chat.completion");
}
#[test]
fn authenticated_routes_reject_missing_bearer_token() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let auth = ApiAuthConfig::bearer_token("local-test-token");
    let response = handle_api_request_with_auth("POST", "/v1/chat/completions", &[], &body, &auth);
    assert_eq!(response.status_code, 401);
    assert!(response.body.to_lowercase().contains("bearer"));
}
#[test]
fn rate_limit_metadata_is_exposed() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert!(
        response.body.contains("\"rate_limit\":") || response.content_type.contains("ratelimit"),
        "full-scope API should publish RateLimit metadata for fair use"
    );
}
#[test]
fn responses_api_attaches_trace_link() {
    let request = ResponsesRequest {
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: None,
        stream: false,
        ..ResponsesRequest::default()
    };
    let response = create_response(&request);
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(
        serialized.contains("trace:"),
        "Responses payload should carry a trace link in evidence_links"
    );
}
#[test]
fn chat_completion_refuses_tool_call_without_agent_mode() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": "Use the provided local_shell tool to list the working directory"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "local_shell",
                "description": "Run a shell command on the host",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    }
                }
            }
        }],
        "tool_choice": {
            "type": "function",
            "function": {"name": "local_shell"}
        }
    }))
    .unwrap();
    let completion = create_chat_completion(&request);
    let body = completion.choices[0].message.content.plain_text();
    assert!(
        body.to_lowercase().contains("agent mode") || body.to_lowercase().contains("not allowed"),
        "chat surface must refuse autonomous shell execution by default, got: {body}"
    );
}
#[test]
fn chat_completion_allows_declared_tools_when_tool_choice_is_none() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": "Say hello"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "local_shell",
                "description": "Run a shell command on the host",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    }
                }
            }
        }],
        "tool_choice": "none"
    }))
    .unwrap();
    let completion = create_chat_completion(&request);
    let body = completion.choices[0].message.content.plain_text();
    assert!(
        !body.to_lowercase().contains("agent mode"),
        "declared tools with tool_choice=none should remain a normal chat request, got: {body}"
    );
}
