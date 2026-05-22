use std::collections::{BTreeMap, BTreeSet};

use formal_ai::{agent_info, FormalAiEngine};

const WEB_SEARCH_SOURCE_MARKER_CASES: &[(&str, &str, &str)] = &[
    ("en", "Find apple on the internet", "apple"),
    ("ru", "Найди яблоко в интернете", "яблоко"),
    ("hi", "सेब के बारे में इंटरनेट पर खोजो", "सेब"),
    ("zh", "查找苹果网上信息", "苹果"),
];

#[test]
fn navigation_describes_frame_policy_check_before_iframe_preview() {
    // Regression test for issue #169: navigation must not use a host-specific
    // blocklist or blindly render an iframe that may show a blocked-frame page.
    // The browser checks frame-policy metadata first, then chooses an iframe or
    // a direct external link.
    let response = FormalAiEngine.answer("Navigate to github.com");

    assert_eq!(response.intent, "url_navigate");
    assert!(response.answer.contains("https://github.com"));
    assert!(
        response
            .answer
            .contains("I suggest opening this in a new tab"),
        "Navigation should be phrased as a polite suggestion, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Open this"),
        "Navigation copy should not command the user, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("browser web app"),
        "Navigation copy should describe the browser web app behavior, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("frame-policy metadata"),
        "Navigation copy should mention frame-policy metadata, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("X-Frame-Options"),
        "Navigation copy should mention X-Frame-Options, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("CSP frame-ancestors"),
        "Navigation copy should mention CSP frame-ancestors, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("cannot reliably confirm"),
        "Navigation copy should not give up before checking headers, got: {}",
        response.answer
    );
    assert!(
        !response.answer.to_lowercase().contains("demo"),
        "Navigation copy should not call the product a demo, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("URL requested for"),
        "GitHub navigation copy should be natural, got: {}",
        response.answer
    );
    assert!(
        !response.answer.to_lowercase().contains("preview below"),
        "GitHub navigation must not blindly promise a preview below, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:frame_policy_check:")),
        "Navigation should record the frame-policy check path: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:external_link:")),
        "Navigation should record the direct external-link preview path: {:?}",
        response.evidence_links
    );
}

#[test]
fn generic_navigation_uses_same_frame_policy_path_as_github() {
    let response = FormalAiEngine.answer("Navigate to example.com");

    assert_eq!(response.intent, "url_navigate");
    assert!(response.answer.contains("https://example.com"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:frame_policy_check:")),
        "Generic navigation should not depend on a hardcoded frame-blocked host table: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:external_link:")),
        "The offline Rust answer should still expose a direct-link fallback: {:?}",
        response.evidence_links
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:blocked:")),
        "Generic navigation should not hardcode a blocked host verdict: {:?}",
        response.evidence_links
    );
}

#[test]
fn web_search_source_marker_cases_cover_every_supported_language() {
    let info = agent_info();
    let supported_languages = info
        .get("supported_languages")
        .expect("agent-info.lino should define supported_languages")
        .split('|')
        .filter(|language| !language.is_empty())
        .collect::<BTreeSet<_>>();
    let mut case_languages = BTreeMap::<&str, usize>::new();
    for &(language, _, _) in WEB_SEARCH_SOURCE_MARKER_CASES {
        *case_languages.entry(language).or_insert(0) += 1;
    }
    assert_eq!(
        case_languages.keys().copied().collect::<BTreeSet<_>>(),
        supported_languages,
        "source-marker web-search prompts must cover every supported language",
    );
    assert!(
        case_languages.values().all(|count| *count == 1),
        "source-marker web-search prompts should add one case per supported language: {case_languages:?}",
    );
}

#[test]
fn web_search_source_marker_prompts_extract_query_without_source_marker() {
    for &(language, prompt, expected_query) in WEB_SEARCH_SOURCE_MARKER_CASES {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} prompt {prompt:?} should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("web_search:request:{expected_query}")),
            "{language} web search should extract only the query term {expected_query:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            response.answer.contains(&format!("`{expected_query}`")),
            "{language} web-search answer should echo the extracted query, got: {}",
            response.answer,
        );
        assert_ne!(response.intent, "unknown");
    }
}

#[test]
fn information_search_variants_route_to_web_search_handler() {
    let prompts = [
        "Найди информацию о Rust программировании",
        "Поищи информацию про Rust программирование",
        "Найди подробные сведения о Rust программировании",
        "Поищи материалы по Rust программированию в Википедии",
        "Find information about Rust programming",
        "Look up information on Rust programming",
        "Find detailed information about Rust programming",
        "Research Rust programming online",
        "Rust programming के बारे में जानकारी खोजो",
        "Rust programming पर जानकारी ढूंढो",
        "Rust programming के बारे में विकिपीडिया में खोजें",
        "查找关于 Rust 编程的信息",
        "搜索 Rust 编程 的资料",
        "在维基百科上查一下 Rust 编程",
    ];
    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "web_search",
            "prompt {prompt:?} should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.to_lowercase().contains("rust"),
            "web search response should preserve the query, got {}",
            response.answer,
        );
    }
}

#[test]
fn implicit_research_question_routes_to_web_search_handler() {
    let prompt = "What is the most popular dataset for translation quality validation?";
    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "web_search",
        "implicit research questions should route to web_search, got {} with answer {}",
        response.intent, response.answer,
    );
    assert!(
        response.evidence_links.iter().any(|link| link
            == "web_search:request:most popular dataset for translation quality validation"),
        "web_search should extract the research query without the question prefix: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "web_search:query_kind:implicit_research_question"),
        "web_search should record why a question without an explicit search verb was routed: {:?}",
        response.evidence_links,
    );
    assert_ne!(response.intent, "unknown");
}

#[test]
fn enumeration_research_request_routes_to_web_search_handler() {
    let prompt = "list all genshin characters with off-field DMG";
    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "web_search",
        "enumeration research requests should route to web_search, got {} with answer {}",
        response.intent, response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "web_search:request:genshin characters with off field dmg"),
        "web_search should extract the list target without the list-all prefix: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "web_search:query_kind:enumeration_research_request"),
        "web_search should record why a list-all request was routed: {:?}",
        response.evidence_links,
    );
    assert_ne!(response.intent, "unknown");
}
