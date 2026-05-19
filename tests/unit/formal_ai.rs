use formal_ai::{
    create_chat_completion, create_response, environment_directory, environment_records,
    export_memory_bundle, export_memory_links_notation, extract_memory_from_bundle,
    handle_api_request, knowledge_links_notation, merged_bundle, parse_bundle,
    parse_memory_links_notation, seed_files, ChatCompletionRequest, ChatMessage, ConversationTurn,
    FormalAiEngine, MemoryEvent, MemoryStore, MessageContent, ResponsesRequest, UniversalSolver,
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

// Issue #152: "how are you?" small talk used to fall through to the unknown
// fallback. The follow-up review made this a supported-language invariant,
// not a Russian-only route.
#[test]
fn how_are_you_prompt_is_recognized_as_greeting_in_supported_languages() {
    let cases = [
        ("How are you?", "language:en"),
        ("Как твои дела?", "language:ru"),
        ("आप कैसे हैं?", "language:hi"),
        ("你好吗?", "language:zh"),
    ];

    for (prompt, language_link) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "greeting",
            "small-talk prompt {prompt:?} should be recognized as greeting, got intent {:?}",
            response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:greeting"),
            "response should cite response:greeting for {prompt:?}, got {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == language_link),
            "response should keep {language_link} for {prompt:?}, got {:?}",
            response.evidence_links
        );
    }
}

// Issue #67: "пока" and similar farewell words were returned as unknown intent.
#[test]
fn farewell_prompts_are_recognized_as_farewell() {
    let cases = [
        ("пока", "ru"),
        ("до свидания", "ru"),
        ("bye", "en"),
        ("goodbye", "en"),
    ];

    for (prompt, expected_language) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "farewell",
            "prompt {:?} should be recognized as farewell, got intent {:?}",
            prompt, response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:farewell"),
            "prompt {prompt:?} response should cite response:farewell",
        );
        if expected_language == "ru" {
            assert!(
                response.answer.contains("свидания") || response.answer.contains("Пока"),
                "Russian farewell {prompt:?} should get a Russian answer, got: {}",
                response.answer
            );
        }
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
fn how_you_work_prompts_return_meta_explanation() {
    let cases = [
        ("покажи как ты работаешь?", "ru"),
        ("как ты работаешь?", "ru"),
        ("how do you work?", "en"),
        ("show me how you work", "en"),
    ];

    for (prompt, expected_language) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "meta_explanation",
            "prompt '{prompt}' should resolve to meta_explanation, got '{}'",
            response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:meta_explanation"),
            "prompt '{prompt}' should include evidence link response:meta_explanation"
        );
        // Russian prompts must respond in Russian
        if expected_language == "ru" {
            assert!(
                response.answer.contains("работаешь")
                    || response.answer.contains("правил")
                    || response.answer.contains("Notation"),
                "Russian prompt '{prompt}' should get a Russian answer, got: {}",
                response.answer
            );
        }
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
fn write_script_prompt_returns_code_block() {
    // Regression test for issue #35: "Напиши скрипт на питоне" was returning
    // intent: unknown instead of routing to a code answer.
    let cases = [
        (
            "Напиши скрипт на питоне",
            "write_script_python",
            "```python",
        ),
        (
            "Write a script in Python",
            "write_script_python",
            "```python",
        ),
        ("Write a program in Rust", "write_script_rust", "```rust"),
        (
            "Write me some code in JavaScript",
            "write_script_javascript",
            "```javascript",
        ),
        (
            "написать скрипт на javascript",
            "write_script_javascript",
            "```javascript",
        ),
    ];

    for (prompt, intent, code_fence) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, intent,
            "prompt: {prompt:?} — answer was: {}",
            response.answer
        );
        assert!(
            response.answer.contains(code_fence),
            "prompt: {prompt:?} — expected {code_fence} in answer: {}",
            response.answer
        );
        assert_ne!(
            response.intent, "unknown",
            "prompt: {prompt:?} — got unknown intent"
        );
    }
}

#[test]
fn software_project_request_returns_reviewable_plan() {
    // Regression test for issue #80: a request to write an Owlbear/D&D
    // extension was returning intent: unknown. This must be handled as a
    // generalized software project request, not a memoized prompt.
    let prompt = concat!(
        "Hi, can you write for me extension for owlbear? I am currently leading some dnd games ",
        "and i want to try wargame. So, i need extensions that can track hp for different units, ",
        "that can track Protection and Resistance stacks on unit an will reduce damage count on ",
        "those stats. Also this extension should track cooldown of some abilities"
    );

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "software_project_plan",
        "answer was: {}",
        response.answer
    );
    assert_ne!(response.intent, "unknown");
    assert!(response.answer.contains("Formalized meaning"));
    assert!(response.answer.contains("software_project_request"));
    assert!(response.answer.contains("Reasoning steps"));
    assert!(response.answer.contains("Proposed plan"));
    assert!(response.answer.contains("Owlbear"));
    assert!(response.answer.contains("HP"));
    assert!(response.answer.contains("Protection"));
    assert!(response.answer.contains("Resistance"));
    assert!(response.answer.contains("cooldown"));
    assert!(response.answer.contains("approve plan"));
    assert!(!response.answer.contains("mitigateDamage"));
}

#[test]
fn software_project_variations_do_not_return_unknown() {
    let prompts = [
        "Build a browser extension that tracks reading progress and exports CSV",
        "Create a Discord bot for scheduling game sessions with reminders",
        "Implement a small web app for tracking invoices and overdue payments",
        "Make a plugin for a tabletop map that tracks unit status effects",
        "Develop a command line tool for renaming photos by date",
    ];

    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "software_project_plan",
            "prompt: {prompt:?} answer: {}",
            response.answer
        );
        assert_ne!(response.intent, "unknown");
        assert!(response.answer.contains("Formalized meaning"));
        assert!(response.answer.contains("Proposed plan"));
        assert!(response.answer.contains("approve plan"));
    }
}

#[test]
fn software_project_approval_returns_implementation_starter() {
    let solver = UniversalSolver::default();
    let prompt = concat!(
        "Write an extension for Owlbear that tracks HP, Protection, Resistance, ",
        "damage mitigation, and cooldowns for tabletop units"
    );

    let plan = solver.solve(prompt);
    let history = [
        ConversationTurn::user(prompt),
        ConversationTurn::assistant(plan.answer),
    ];
    let implementation = solver.solve_with_history("approve plan", &history);

    assert_eq!(
        implementation.intent, "software_project_implementation",
        "answer was: {}",
        implementation.answer
    );
    assert!(implementation.answer.contains("approval_state approved"));
    assert!(implementation.answer.contains("```typescript"));
    assert!(implementation.answer.contains("mitigateDamage"));
    assert!(implementation.answer.contains("tickCooldowns"));
}

#[test]
fn chat_completion_has_openai_compatible_shape() {
    let request = ChatCompletionRequest {
        model: Some(String::from("formal-symbolic-production")),
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: MessageContent::Text(String::from("Hello")),
        }],
        temperature: None,
        stream: false,
    };

    let completion = create_chat_completion(&request);

    assert_eq!(completion.object, "chat.completion");
    assert_eq!(completion.model, "formal-symbolic-production");
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
        model: Some(String::from("formal-symbolic-production")),
        input: serde_json::Value::String(String::from("Write hello world in Rust")),
        instructions: None,
        temperature: None,
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
        Some("formal-symbolic-production")
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
        "model": "formal-symbolic-production",
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
fn fetch_prompt_returns_http_fetch_intent_not_unknown() {
    // Regression test for issue #71: "fetch google.com" was returning
    // intent: unknown instead of routing to the http_fetch handler.
    //
    // Issue #125 follow-up: the http_fetch intent is reserved for prompts that
    // explicitly ask to perform an HTTP request (fetch, request, "Сделай
    // запрос к ..."). Navigation prompts ("Navigate to ...", "Visit ...")
    // route to the separate `url_navigate` intent instead — see
    // [`url_navigation_variations_return_https_link_without_fetch_advice`].
    let cases = [
        "fetch google.com",
        "fetch https://example.com",
        "fetch http://example.com/path",
        "fetch example.com",
        "Make a request to google.com",
        "Send a request to https://example.com",
        // Regression test for issue #107: the reported Russian prompt
        // "Сделай запрос к google.com" used to fall through to unknown.
        "Сделай запрос к google.com",
        "сделай запрос к https://example.com/path",
        "Выполни запрос к google.com",
        "запроси google.com",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "http_fetch",
            "prompt {prompt:?} should resolve to http_fetch, got {:?} — answer: {}",
            response.intent, response.answer
        );
        assert_ne!(
            response.intent, "unknown",
            "prompt {prompt:?} must not return unknown intent"
        );
    }
}

#[test]
fn url_navigation_variations_return_https_link_without_fetch_advice() {
    // Regression test for issue #125: navigation-style prompts must route to
    // the `url_navigate` intent (no HTTP fetch attempted) and surface a direct
    // HTTPS link plus iframe preview controls. They must not be conflated
    // with the `http_fetch` intent, which is reserved for explicit requests
    // such as `Make a request to google.com`.
    let cases = [
        ("Navigate to github.com", "https://github.com"),
        ("Go to github.com", "https://github.com"),
        ("Goto github.com", "https://github.com"),
        ("Visit github.com", "https://github.com"),
        ("Browse to github.com", "https://github.com"),
        ("Show github.com", "https://github.com"),
        ("Show me github.com", "https://github.com"),
        ("Display github.com", "https://github.com"),
        ("Load github.com", "https://github.com"),
        ("Take me to github.com", "https://github.com"),
        ("Preview github.com", "https://github.com"),
        ("View github.com", "https://github.com"),
        ("Open github.com", "https://github.com"),
        ("Open url github.com", "https://github.com"),
        ("Open the page github.com", "https://github.com"),
        (
            "Open https://github.com/link-assistant/formal-ai",
            "https://github.com/link-assistant/formal-ai",
        ),
        ("github.com", "https://github.com"),
        ("https://github.com", "https://github.com"),
        ("Перейди на github.com", "https://github.com"),
        ("Перейдите на github.com", "https://github.com"),
        ("Переходи на github.com", "https://github.com"),
        ("Открой github.com", "https://github.com"),
        ("Открой сайт github.com", "https://github.com"),
        ("Открой страницу github.com", "https://github.com"),
        ("Открой ссылку github.com", "https://github.com"),
        ("Покажи github.com", "https://github.com"),
        ("Покажи сайт github.com", "https://github.com"),
        ("Загрузи github.com", "https://github.com"),
        ("Посети github.com", "https://github.com"),
        ("Зайди на github.com", "https://github.com"),
    ];

    for (prompt, expected_url) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "url_navigate",
            "prompt {prompt:?} should resolve to url_navigate, got {:?} — answer: {}",
            response.intent, response.answer
        );
        assert!(
            response.answer.contains(expected_url),
            "prompt {prompt:?} should return a proper HTTPS link {expected_url:?}, got: {}",
            response.answer
        );
        assert!(
            !response.answer.to_lowercase().contains("cors")
                && !response.answer.contains("fetch()"),
            "prompt {prompt:?} should not tell the user that the browser will try fetch/CORS first, got: {}",
            response.answer
        );
    }
}

#[test]
fn http_fetch_and_url_navigate_intents_are_distinct() {
    // Issue #125: ensure the two flows do not collide. `Make a request to X`
    // must keep going through http_fetch (so the browser attempts an actual
    // network request) while `Navigate to X` must surface the url_navigate
    // intent (iframe-only, no fetch attempt). Both must surface the URL.
    let fetch_prompt = "Make a request to google.com";
    let navigate_prompt = "Navigate to google.com";

    let fetch_response = FormalAiEngine.answer(fetch_prompt);
    let navigate_response = FormalAiEngine.answer(navigate_prompt);

    assert_eq!(
        fetch_response.intent, "http_fetch",
        "Make a request prompt must keep using http_fetch; got {:?}",
        fetch_response.intent,
    );
    assert_eq!(
        navigate_response.intent, "url_navigate",
        "Navigate prompt must route to url_navigate; got {:?}",
        navigate_response.intent,
    );
    assert!(fetch_response.answer.contains("https://google.com"));
    assert!(navigate_response.answer.contains("https://google.com"));
    // The navigation copy must not mention fetch()/CORS — the user explicitly
    // asked us not to imply a network request will be attempted.
    assert!(
        !navigate_response.answer.to_lowercase().contains("cors")
            && !navigate_response.answer.contains("fetch()"),
        "navigate_response must not mention fetch/CORS, got: {}",
        navigate_response.answer
    );
}

#[test]
fn web_search_prompt_returns_web_search_intent_not_unknown() {
    let cases = [
        "Search the web for Nikola Tesla",
        "Search internet for formal verification",
        "Найди в интернете Никола Тесла",
        "Поищи в интернете формальную верификацию",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "prompt {prompt:?} should resolve to web_search, got {:?} - answer: {}",
            response.intent, response.answer
        );
        assert_ne!(
            response.intent, "unknown",
            "prompt {prompt:?} must not return unknown intent"
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

#[test]
fn opinion_questions_return_no_opinion_response() {
    // Issue #42: "Do you think space is continuous or discrete" previously fell
    // through to the generic unknown-intent error. Opinion/belief questions
    // must now return a deterministic explanation instead.
    let cases = [
        "Do you think space is continuous or discrete",
        "What do you think about quantum mechanics?",
        "Do you believe in free will?",
        "What is your opinion on climate change?",
        "In your opinion, is consciousness physical?",
        "What are your thoughts on recursion?",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "opinion_question",
            "prompt {prompt:?} should resolve to opinion_question intent"
        );
        assert!(
            response.answer.contains("deterministic"),
            "response for {prompt:?} should mention deterministic nature"
        );
        assert!(
            !response
                .answer
                .contains("cannot answer that from local Links Notation rules"),
            "prompt {prompt:?} should not return the unknown-intent error"
        );
    }
}

#[test]
fn who_is_question_does_not_return_unknown_intent() {
    // Issue #69: "who is elon mask" (typo of Musk) previously returned
    // intent: unknown.  "Who is X" prompts must be treated as a question
    // and return a deterministic response even when the entity is not in
    // the knowledge base.
    let cases = [
        ("who is elon mask", Some("Elon Musk")),
        ("who is elon musk", None),
        ("who was albert einstein", None),
    ];

    for (prompt, expected_suggestion) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_ne!(
            response.intent, "unknown",
            "prompt {prompt:?} should not return unknown intent"
        );
        assert!(
            !response
                .answer
                .contains("cannot answer that from local Links Notation rules"),
            "prompt {prompt:?} should not return the unknown-intent error"
        );
        if let Some(suggestion) = expected_suggestion {
            assert!(
                response.answer.contains(suggestion),
                "response for {prompt:?} should suggest \"{suggestion}\", got: {}",
                response.answer
            );
        }
    }
}

#[test]
fn who_is_elon_mask_suggests_elon_musk() {
    // Issue #69: specific reproduction case — typo "mask" instead of "musk".
    let response = FormalAiEngine.answer("who is elon mask");

    assert_eq!(
        response.intent, "who_is_question",
        "prompt should resolve to who_is_question intent"
    );
    assert!(
        response.answer.contains("Elon Musk"),
        "response should suggest \"Elon Musk\" for typo \"elon mask\", got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Did you mean"),
        "response should contain \"Did you mean\" correction, got: {}",
        response.answer
    );
}

// Issue #66: "Расскажи за Telegram Ads" was returning intent: unknown because
// the colloquial Russian prefix "расскажи за" was not in the prompt-patterns,
// and Telegram Ads had no concept entry in the knowledge base.
#[test]
fn rasskazhi_za_telegram_ads_resolves_to_concept_lookup() {
    let cases = [
        // Exact issue report
        "Расскажи за Telegram Ads",
        // Variants with "расскажи мне за"
        "Расскажи мне за Telegram Ads",
        // Other supported Russian concept-lookup prefixes for the same concept
        "Расскажи про Telegram Ads",
        "Расскажи о Telegram Ads",
        "Что такое Telegram Ads",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert!(
            response.intent == "concept_lookup" || response.intent == "concept_lookup_in_context",
            "[{prompt}] expected concept_lookup, got intent: {}",
            response.intent
        );
        assert!(
            response.answer.contains("Telegram"),
            "[{prompt}] answer should mention Telegram, got: {}",
            response.answer
        );
        assert_ne!(
            response.intent, "unknown",
            "[{prompt}] must not fall through to unknown intent"
        );
    }
}

// Issue #64: "Расскажи о теории связей" should resolve to Link Foundation's
// links meta-theory, while making clear that similarly named theories may mean
// something else.
#[test]
fn links_theory_prompts_resolve_to_meta_theory_concept() {
    let cases = [
        // Exact issue report
        "Расскажи о теории связей",
        // Russian variants covered by concept-lookup prefixes and aliases
        "Расскажи про теорию связей",
        "Что такое теория связей?",
        "Что такое глубокая теория связей?",
        // English aliases for the same Link Foundation product
        "Tell me about links theory",
        "What is the links meta-theory?",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);
        let lower = response.answer.to_lowercase();

        assert_eq!(
            response.intent, "concept_lookup",
            "[{prompt}] expected concept_lookup, got intent: {}",
            response.intent
        );
        assert_ne!(
            response.intent, "unknown",
            "[{prompt}] must not fall through to unknown intent"
        );
        assert!(
            lower.contains("meta-theory")
                || lower.contains("метатеор")
                || lower.contains("мета-теор"),
            "[{prompt}] answer should identify the Link Foundation meta-theory, got: {}",
            response.answer
        );
        assert!(
            lower.contains("similar") || lower.contains("похож"),
            "[{prompt}] answer should mention similarly named theories, got: {}",
            response.answer
        );
        assert!(
            response
                .answer
                .contains("https://github.com/link-foundation/meta-theory"),
            "[{prompt}] should cite the meta-theory repository, got: {}",
            response.answer
        );
    }
}
