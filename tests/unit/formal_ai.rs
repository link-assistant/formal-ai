use formal_ai::{
    create_chat_completion, create_response, handle_api_request, knowledge_links_notation,
    ChatCompletionRequest, ChatMessage, FormalAiEngine, MessageContent, ResponsesRequest,
};
use lino_objects_codec::format::parse_indented;

#[test]
fn greeting_prompt_returns_symbolic_greeting() {
    let response = FormalAiEngine.answer("Hi");

    assert_eq!(response.intent, "greeting");
    assert_eq!(response.answer, "Hi, how may I help you?");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "response:greeting"));
}

#[test]
fn rust_hello_world_prompt_returns_code_block() {
    let response = FormalAiEngine.answer("Write me hello world program in Rust");

    assert_eq!(response.intent, "hello_world_rust");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("fn main()"));
    assert!(response.answer.contains("println!(\"Hello, world!\");"));
}

#[test]
fn hello_world_prompt_supports_multiple_programming_languages() {
    let cases = [
        (
            "Write hello world in Python",
            "hello_world_python",
            "```python",
        ),
        (
            "Create a hello world example in JavaScript",
            "hello_world_javascript",
            "```javascript",
        ),
        ("hello world in Go", "hello_world_go", "```go"),
    ];

    for (prompt, intent, code_fence) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(response.intent, intent);
        assert!(response.answer.contains(code_fence));
        assert!(response.answer.contains("Hello, world!"));
    }
}

#[test]
fn chat_completion_has_openai_compatible_shape() {
    let request = ChatCompletionRequest {
        model: Some(String::from("formal-symbolic-poc")),
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: MessageContent::Text(String::from("Hello")),
        }],
        stream: false,
    };

    let completion = create_chat_completion(&request);

    assert_eq!(completion.object, "chat.completion");
    assert_eq!(completion.model, "formal-symbolic-poc");
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert_eq!(
        completion.choices[0].message.content.plain_text(),
        "Hi, how may I help you?"
    );
    assert!(completion.usage.total_tokens >= completion.usage.prompt_tokens);
}

#[test]
fn responses_api_shape_contains_output_text() {
    let request = ResponsesRequest {
        model: Some(String::from("formal-symbolic-poc")),
        input: serde_json::Value::String(String::from("Write hello world in Rust")),
        instructions: None,
        stream: false,
    };

    let response = create_response(&request);

    assert_eq!(response.object, "response");
    assert_eq!(response.status, "completed");
    assert_eq!(response.output[0].role, "assistant");
    assert_eq!(response.output[0].content[0].kind, "output_text");
    assert!(response.output[0].content[0].text.contains("```rust"));
}

#[test]
fn knowledge_export_is_valid_links_notation() {
    let notation = knowledge_links_notation();
    let records = notation.split("\n\n").collect::<Vec<_>>();
    let (id, root) = parse_indented(records[0]).expect("root record should parse");

    assert_eq!(id, "formal_ai_knowledge");
    assert_eq!(
        root.get("model").map(String::as_str),
        Some("formal-symbolic-poc")
    );
    assert!(records.iter().any(|record| {
        let Ok((_id, parsed)) = parse_indented(record) else {
            return false;
        };

        parsed.get("intent").map(String::as_str) == Some("hello_world_rust")
    }));
    assert!(!notation.contains("(str "));
}

#[test]
fn server_handler_supports_chat_completions_route() {
    let body = serde_json::json!({
        "model": "formal-symbolic-poc",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();

    let response = handle_api_request("POST", "/v1/chat/completions", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["object"], "chat.completion");
    assert_eq!(
        json["choices"][0]["message"]["content"],
        "Hi, how may I help you?"
    );
}
