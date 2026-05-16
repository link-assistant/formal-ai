use formal_ai::{
    create_chat_completion, create_response, environment_directory, environment_records,
    export_memory_bundle, export_memory_links_notation, extract_memory_from_bundle,
    handle_api_request, knowledge_links_notation, merged_bundle, parse_bundle,
    parse_memory_links_notation, seed_files, ChatCompletionRequest, ChatMessage, FormalAiEngine,
    MemoryEvent, MemoryStore, MessageContent, ResponsesRequest,
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
fn shabbat_shalom_greeting_is_recognized_as_greeting() {
    for prompt in ["шабат шалом!", "шабат шалом", "шалом"] {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "greeting",
            "prompt {:?} should be recognized as a greeting, got intent {:?}",
            prompt, response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:greeting"),
            "prompt {prompt:?} response should cite response:greeting",
        );
    }
}

#[test]
fn identity_questions_return_standard_self_description() {
    let cases = [
        "Who are you?",
        "what are you",
        "Tell me about yourself",
        "What is formal-ai?",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(response.intent, "identity");
        assert!(response.answer.contains("formal-ai"));
        assert!(response
            .evidence_links
            .iter()
            .any(|link| link == "response:identity"));
    }
}

#[test]
fn rust_hello_world_prompt_returns_code_block() {
    let response = FormalAiEngine.answer("Write me hello world program in Rust");

    assert_eq!(response.intent, "hello_world_rust");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("fn main()"));
    assert!(response.answer.contains("println!(\"Hello, world!\");"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
    assert!(response.answer.contains("Output:"));
    assert!(response.answer.contains("```text\nHello, world!\n```"));
}

// Issue #31: Queries about KISS in a programming context should return the
// software design principle, not the rock band KISS.
#[test]
fn kiss_in_programming_context_returns_design_principle_not_band() {
    let cases = [
        // Exact issue report (Russian, misspelled programming word, "в рамках" delimiter)
        "что такое Kiss в рамках програмирования",
        // English equivalents
        "what is KISS in programming",
        "what is kiss in software development",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        // Must resolve as a concept lookup (offline, deterministic — no Wikipedia
        // network call needed because the KISS principle is in the concept corpus).
        assert!(
            response.intent == "concept_lookup_in_context" || response.intent == "concept_lookup",
            "[{prompt}] unexpected intent: {}",
            response.intent
        );
        // Answer must mention the design principle, not the rock band.
        assert!(
            response.answer.contains("принцип")
                || response.answer.contains("KISS")
                || response.answer.contains("simple"),
            "[{prompt}] answer does not mention the design principle: {}",
            response.answer
        );
        assert!(
            !response.answer.contains("рок-группа") && !response.answer.contains("rock band"),
            "[{prompt}] answer incorrectly describes the rock band: {}",
            response.answer
        );
    }
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

#[test]
fn telegram_webhook_supports_private_messages() {
    let body = serde_json::json!({
        "update_id": 1000,
        "message": {
            "message_id": 7,
            "date": 1,
            "chat": {"id": 42, "type": "private"},
            "text": "Hi"
        }
    })
    .to_string();

    let response = handle_api_request("POST", "/telegram/webhook", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["method"], "sendMessage");
    assert_eq!(json["chat_id"], 42);
    assert_eq!(json["parse_mode"], "HTML");
    let text = json["text"].as_str().expect("text should be a string");
    assert!(text.starts_with("Hi, how may I help you?"));
    assert!(text.contains("/trace "));
}

#[test]
fn telegram_webhook_supports_public_chat_code_replies() {
    let body = serde_json::json!({
        "update_id": 1001,
        "message": {
            "message_id": 8,
            "date": 1,
            "chat": {"id": -100_123, "type": "supergroup", "title": "formal-ai"},
            "text": "Write me hello world program in Rust"
        }
    })
    .to_string();

    let response = handle_api_request("POST", "/telegram/webhook", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["method"], "sendMessage");
    assert_eq!(json["chat_id"], -100_123);
    assert_eq!(json["parse_mode"], "HTML");
    let text = json["text"]
        .as_str()
        .expect("telegram reply text should be a string");
    assert!(text.contains("<pre><code class=\"language-rust\">"));
    assert!(text.contains("Execution status: compiled and ran"));
    assert!(text.contains("Hello, world!"));
}

// --- Issue #16 follow-up: universal-seed and cross-surface memory tests ---

#[test]
fn environment_directory_declares_every_supported_surface() {
    // R106: the seed itself must name every interface the agent supports.
    let directory = environment_directory();
    let ids: Vec<&str> = directory
        .environments
        .iter()
        .map(|env| env.id.as_str())
        .collect();
    for expected in [
        "browser",
        "rust_library",
        "cli",
        "http_server",
        "telegram",
        "docker_microservice",
    ] {
        assert!(
            ids.contains(&expected),
            "environments.lino must declare a `{expected}` environment; got {ids:?}",
        );
    }
    // Every environment must declare a non-empty memory store description
    // so chat surfaces can explain where state lives.
    for env in &directory.environments {
        assert!(
            !env.memory_store.is_empty(),
            "environment {} should declare a memory_store",
            env.id,
        );
        assert!(
            !env.runtime.is_empty(),
            "environment {} should declare a runtime",
            env.id,
        );
    }
    // The migration block must enumerate the documented cross-surface flows.
    let flow_ids: Vec<&str> = directory.flows.iter().map(|f| f.id.as_str()).collect();
    for expected in [
        "browser_to_cli",
        "cli_to_browser",
        "browser_to_browser",
        "cli_to_cli",
    ] {
        assert!(
            flow_ids.contains(&expected),
            "migration flow `{expected}` is missing; got {flow_ids:?}",
        );
    }
}

#[test]
fn environment_records_match_directory() {
    // R108: every CLI capability must also be reachable from the library.
    // `environment_records` is the convenience accessor the CLI uses.
    let records = environment_records();
    let directory = environment_directory();
    assert_eq!(records.len(), directory.environments.len());
    for (record, env) in records.iter().zip(directory.environments.iter()) {
        assert_eq!(record.id, env.id);
        assert_eq!(record.label, env.label);
        assert_eq!(record.tools, env.tools);
    }
}

#[test]
fn library_memory_round_trips_through_links_notation() {
    // R107: events written on one surface must replay on another via the
    // shared `demo_memory` wire format. The library accessors must be
    // sufficient for that round-trip (no CLI/HTTP detour required).
    let mut store = MemoryStore::new();
    store.append(MemoryEvent::user("Привет"));
    store.append(MemoryEvent::assistant("Hi, how may I help you?"));
    let text = export_memory_links_notation(store.events());
    assert!(text.starts_with("demo_memory\n"));
    let parsed = parse_memory_links_notation(&text);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].content.as_deref(), Some("Привет"));
    assert_eq!(parsed[1].role.as_deref(), Some("assistant"));
}

#[test]
fn library_bundle_round_trips_seed_and_memory() {
    // R107 + R108: build a bundle from the library, then recover the seed
    // and memory sections — both must round-trip. This is the exact code
    // path the CLI's `bundle export|import` and the browser's
    // `Download bundle` button rely on.
    let events = vec![
        MemoryEvent::user("hello"),
        MemoryEvent::assistant("hi back"),
    ];
    let bundle = export_memory_bundle(&seed_files(), &events);
    let recovered_memory = extract_memory_from_bundle(&bundle).expect("recover memory");
    assert_eq!(recovered_memory.len(), 2);
    assert_eq!(recovered_memory[0].content.as_deref(), Some("hello"));
    let recovered_seed = parse_bundle(&bundle);
    let names: Vec<&str> = recovered_seed.iter().map(|(n, _)| n.as_str()).collect();
    for (expected, _) in seed_files() {
        assert!(
            names.contains(&expected),
            "bundle round-trip should recover seed file {expected}",
        );
    }
}

#[test]
fn merged_bundle_and_parse_bundle_round_trip_split_files() {
    // R104: the static seed bundle must round-trip through parse_bundle
    // back to the same per-category split. This protects the
    // single-file-import-on-any-surface invariant from R107.
    let bundle = merged_bundle();
    let parsed = parse_bundle(&bundle);
    let files = seed_files();
    assert_eq!(parsed.len(), files.len());
    for ((parsed_name, _), (orig_name, _)) in parsed.iter().zip(files.iter()) {
        assert_eq!(parsed_name, orig_name);
    }
}
