//! Issue #16 follow-up: one seed, and one memory, across every surface.
//!
//! The tests these were extracted from ask what the engine answers. These ask
//! something narrower and shared: that the seed names every surface the agent
//! runs on, and that a memory written on one surface reads back on the next --
//! through the links-notation projection, the bundle, and the split bundle
//! files. They were already fenced off by a section comment in the file they
//! came from; the fence is now a module.

use formal_ai::{
    FormalAiEngine, MemoryEvent, MemoryStore, environment_directory, environment_records,
    export_memory_bundle, export_memory_links_notation, extract_memory_from_bundle, merged_bundle,
    parse_bundle, parse_memory_links_notation, seed_files,
};

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
        "desktop",
        "vscode",
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
    // HTTPS external link. They must not be conflated with the `http_fetch`
    // intent, which is reserved for explicit requests such as
    // `Make a request to google.com`.
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
    // intent (direct external link, no fetch attempt). Both must surface the
    // URL.
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
        "Найди яблоко в интернете",
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
                .contains("cannot answer that from local links rules"),
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
                .contains("cannot answer that from local links rules"),
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
