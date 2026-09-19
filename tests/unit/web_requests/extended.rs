use super::*;

#[test]
fn research_result_followup_accepts_supported_language_research_contexts() {
    #[derive(Clone, Copy)]
    struct Case {
        language: &'static str,
        research_prompt: &'static str,
        prior_answer: &'static str,
    }

    let solver = UniversalSolver::default();
    let cases = [
        Case {
            language: "en",
            research_prompt: "research memory safety costs for Rust and C++",
            prior_answer: "No CORS-enabled web search results were returned for `memory safety costs`.",
        },
        Case {
            language: "ru",
            research_prompt: "исследование: затраты на безопасность памяти для Rust и C++",
            prior_answer: "Не получены результаты веб-поиска по запросу `затраты безопасность памяти`.",
        },
        Case {
            language: "hi",
            research_prompt: "अनुसंधान: Rust और C++ के लिए memory safety costs",
            prior_answer: "कोई खोज परिणाम नहीं मिला: `memory safety costs`.",
        },
        Case {
            language: "zh",
            research_prompt: "研究：Rust 和 C++ 的 memory safety costs",
            prior_answer: "未获取到可用的网页搜索结果：`memory safety costs`。",
        },
    ];

    for case in cases {
        let history = [
            ConversationTurn::user(case.research_prompt),
            ConversationTurn::assistant(case.prior_answer),
        ];
        let response = solver.solve_with_history("What is the result?", &history);
        assert_eq!(
            response.answer,
            expected_no_results_research_followup(case.research_prompt)
        );

        assert_eq!(
            response.intent, "research_result_followup",
            "{} research context should bind a terse result follow-up, got {} with answer {}",
            case.language, response.intent, response.answer,
        );
        assert!(
            response.answer.contains("verified source data"),
            "{} follow-up should not fabricate a sourced result, got: {}",
            case.language,
            response.answer,
        );
        assert!(
            response
                .answer
                .contains("no CORS-readable web search results were returned"),
            "{} follow-up should classify the localized prior no-results answer, got: {}",
            case.language,
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("research_result_followup:status:")),
            "{} follow-up should record the localized prior-search status evidence: {:?}",
            case.language,
            response.evidence_links,
        );
    }
}

#[test]
fn standalone_result_question_does_not_claim_research_followup() {
    let response = FormalAiEngine.answer("What is the result?");

    assert_ne!(
        response.intent, "research_result_followup",
        "without research history, the new follow-up handler should not claim the prompt, got {} with answer {}",
        response.intent, response.answer,
    );
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
fn term_information_request_routes_to_web_search_when_concept_lookup_misses() {
    for &(language, prompt, expected_query) in WEB_SEARCH_TERM_INFORMATION_CASES {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} term-information request should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("web_search:request:{expected_query}")),
            "{language} web_search should extract the requested public term: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:implicit_research_question"),
            "{language} web_search should record the term-information request as implicit research: {:?}",
            response.evidence_links,
        );
        assert_ne!(response.intent, "unknown");
    }
}

#[test]
fn term_information_request_preserves_seeded_concept_lookup() {
    for prompt in [
        "What is Rust?",
        "Tell me about Links Notation",
        "Что такое Rust?",
    ] {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "concept_lookup",
            "seeded concept prompt {prompt:?} should stay on concept_lookup, got {} with answer {}",
            response.intent, response.answer,
        );
    }
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
