use std::collections::{BTreeMap, BTreeSet};

use formal_ai::{agent_info, ConversationTurn, FormalAiEngine, UniversalSolver};

const WEB_SEARCH_SOURCE_MARKER_CASES: &[(&str, &str, &str)] = &[
    ("en", "Find apple on the internet", "apple"),
    ("ru", "Найди яблоко в интернете", "яблоко"),
    ("hi", "सेब के बारे में इंटरनेट पर खोजो", "सेब"),
    ("zh", "查找苹果网上信息", "苹果"),
];

const WEB_SEARCH_ENUMERATION_RESEARCH_CASES: &[(&str, &str, &str)] = &[
    (
        "en",
        "list all genshin characters with off-field DMG",
        "genshin characters with off field dmg",
    ),
    (
        "ru",
        "перечисли всех персонажей genshin с уроном вне поля",
        "персонажей genshin с уроном вне поля",
    ),
    (
        "hi",
        "सभी Genshin पात्र जिनके पास off-field DMG है",
        "genshin पात्र जिनके पास off field dmg है",
    ),
    (
        "zh",
        "列出所有 Genshin 角色 具有 off-field DMG",
        "genshin 角色 具有 off field dmg",
    ),
];

const WEB_SEARCH_LATEST_NEWS_CASES: &[(&str, &str, &str)] = &[
    ("English", "latest news", "latest news"),
    ("Russian", "последние новости", "последние новости"),
    ("Hindi", "नवीनतम समाचार", "नवीनतम समाचार"),
    ("Chinese", "最新新闻", "最新新闻"),
];

#[test]
fn latest_news_routes_to_wikinews_search_across_supported_languages() {
    for &(language, prompt, expected_query) in WEB_SEARCH_LATEST_NEWS_CASES {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} latest-news prompt should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("web_search:request:{expected_query}")),
            "{language} latest-news prompt should preserve the requested news query: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:latest_news"),
            "{language} latest-news prompt should record the specialized query kind: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:provider:wikinews"),
            "{language} latest-news prompt should include Wikinews as a search provider: {:?}",
            response.evidence_links,
        );
        assert!(
            response.answer.to_lowercase().contains("wikinews")
                || response.answer.to_lowercase().contains("викиновости"),
            "{language} latest-news answer should direct the user to Wikinews, got: {}",
            response.answer,
        );
        assert_ne!(response.intent, "unknown");
    }
}

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
fn web_search_enumeration_research_cases_cover_every_supported_language() {
    let info = agent_info();
    let supported_languages = info
        .get("supported_languages")
        .expect("agent-info.lino should define supported_languages")
        .split('|')
        .filter(|language| !language.is_empty())
        .collect::<BTreeSet<_>>();
    let mut case_languages = BTreeMap::<&str, usize>::new();
    for &(language, _, _) in WEB_SEARCH_ENUMERATION_RESEARCH_CASES {
        *case_languages.entry(language).or_insert(0) += 1;
    }
    assert_eq!(
        case_languages.keys().copied().collect::<BTreeSet<_>>(),
        supported_languages,
        "enumeration-research web-search prompts must cover every supported language",
    );
    assert!(
        case_languages.values().all(|count| *count == 1),
        "enumeration-research prompts should add one case per supported language: {case_languages:?}",
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
fn source_search_prompts_drop_follow_up_instruction_clauses() {
    let cases = [
        (
            "Search Wikipedia for \"War of Currents\" and summarize who won and why",
            "war of currents",
        ),
        (
            "Search Wikipedia for Nikola Tesla and Thomas Edison. Compare their number of patents.",
            "nikola tesla and thomas edison",
        ),
    ];

    for (prompt, expected_query) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "prompt {prompt:?} should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("web_search:request:{expected_query}")),
            "web_search should keep only the source query term {expected_query:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:explicit_prefix"),
            "source-specific searches should record explicit-prefix routing: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn source_search_prompts_still_cover_supported_languages() {
    let cases = [
        ("English", "Find information about Rust programming"),
        (
            "Russian",
            "Поищи материалы по Rust программированию в Википедии",
        ),
        ("Hindi", "Rust programming के बारे में विकिपीडिया में खोजें"),
        ("Chinese", "在维基百科上查一下 Rust 编程"),
    ];

    for (language, prompt) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} source-search prompt should still route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.to_lowercase().contains("rust"),
            "{language} web-search answer should preserve the requested topic, got: {}",
            response.answer,
        );
    }
}

#[test]
fn research_comparison_table_followup_uses_prior_search_topics() {
    let solver = UniversalSolver::default();
    let search_prompt = "Search for information about:\n\
                         1. Machine learning algorithms\n\
                         2. Deep learning vs traditional ML\n\
                         3. Neural networks basics";
    let search_response = solver.solve(search_prompt);
    assert_eq!(search_response.intent, "web_search");

    let history = [
        ConversationTurn::user(search_prompt),
        ConversationTurn::assistant(search_response.answer),
    ];
    let response = solver.solve_with_history(
        "create a comparison table showing:\n\
         - Key differences\n\
         - Use cases for each\n\
         - Advantages and disadvantages",
        &history,
    );

    assert_eq!(
        response.intent, "research_comparison_table",
        "agent follow-up should create a comparison table instead of falling through, got {} with answer {}",
        response.intent, response.answer,
    );
    assert!(response
        .answer
        .contains("| Topic | Key differences | Use cases | Advantages | Disadvantages |"));
    assert!(response.answer.contains("Machine learning algorithms"));
    assert!(response.answer.contains("Deep learning vs traditional ML"));
    assert!(response.answer.contains("Neural networks basics"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("research_table:prior_search:")),
        "comparison table should record the prior search it reused: {:?}",
        response.evidence_links,
    );
    assert_ne!(response.intent, "unknown");
}

#[test]
fn research_comparison_table_change_preserves_supported_language_web_search_routing() {
    let solver = UniversalSolver::default();
    let cases = [
        ("English", WEB_SEARCH_SOURCE_MARKER_CASES[0].1),
        ("Russian", WEB_SEARCH_SOURCE_MARKER_CASES[1].1),
        ("Hindi", WEB_SEARCH_SOURCE_MARKER_CASES[2].1),
        ("Chinese", WEB_SEARCH_SOURCE_MARKER_CASES[3].1),
    ];

    for (language, prompt) in cases {
        let response = solver.solve(prompt);
        assert_eq!(
            response.intent, "web_search",
            "{language} web-search routing should remain ahead of the research table follow-up handler for prompt {prompt:?}",
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
    for &(language, prompt, expected_query) in WEB_SEARCH_ENUMERATION_RESEARCH_CASES {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} enumeration research request should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("web_search:request:{expected_query}")),
            "{language} web_search should extract the list target without the enumeration prefix: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:enumeration_research_request"),
            "{language} web_search should record why an enumeration request was routed: {:?}",
            response.evidence_links,
        );
        assert_ne!(response.intent, "unknown");
    }
}
