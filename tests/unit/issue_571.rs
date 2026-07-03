use formal_ai::FormalAiEngine;

#[test]
fn commercial_subscription_discount_questions_route_to_web_search_handler() {
    let cases = [
        (
            "Russian",
            "есть ли скидка если брать подписки $200 Claude MAX и ChatGPT Pro не помесячно, а на год?",
            "скидка если брать подписки 200 claude max и chatgpt pro не помесячно а на год",
        ),
        (
            "English",
            "Is there an annual discount for $200 Claude Max and ChatGPT Pro subscriptions?",
            "an annual discount for 200 claude max and chatgpt pro subscriptions",
        ),
        (
            "English broad yes/no",
            "Do Claude Max subscriptions have an annual discount?",
            "claude max subscriptions have an annual discount",
        ),
        (
            "English specific do-you-know",
            "Do you know the annual discount for Claude Max subscriptions?",
            "annual discount for claude max subscriptions",
        ),
        (
            "Hindi",
            "क्या Claude Max और ChatGPT Pro subscriptions पर वार्षिक discount है?",
            "claude max और chatgpt pro subscriptions पर वार्षिक discount है",
        ),
        (
            "Chinese",
            "有没有Claude Max和ChatGPT Pro订阅的年度折扣？",
            "claude max和chatgpt pro订阅的年度折扣",
        ),
    ];

    for (language, prompt, expected_query) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{language} commercial subscription discount question should route to web_search, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("web_search:request:{expected_query}")),
            "{language} web_search should extract the commercial research query without the question opener: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:implicit_research_question"),
            "{language} web_search should record the question as implicit research: {:?}",
            response.evidence_links,
        );
        assert_ne!(response.intent, "unknown");
    }
}

/// The maintainer asked for the *entire class* of externally verifiable
/// questions, not just the commercial subscription example. These prompts carry
/// no pricing/subscription/discount seed vocabulary at all: they route purely
/// because they interrogate a referential external entity — a brand written with
/// interior capitalisation (`ChatGPT`, `OpenAI`, `iPhone`, `TypeScript`) whose
/// current facts live on the public web, not in local memory. The topics are
/// deliberately non-commercial (release dates, hardware specs, features) to prove
/// the reasoning rule generalises beyond pricing and beyond any stored word list.
#[test]
fn external_entity_questions_route_to_web_search_by_reasoning_not_vocabulary() {
    let cases = [
        (
            "English release date",
            "When did OpenAI release its first model?",
        ),
        ("English hardware spec", "Is the iPhone 16 waterproof?"),
        (
            "English language feature",
            "Does TypeScript support pattern matching?",
        ),
        (
            "Russian release date",
            "Когда OpenAI выпустила свою первую модель?",
        ),
        ("Hindi hardware spec", "क्या iPhone 16 waterproof है?"),
        ("Chinese feature", "ChatGPT 支持语音输入吗？"),
    ];

    for (label, prompt) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "web_search",
            "{label} external-entity question should route to web_search by reasoning, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:implicit_research_question"),
            "{label} should be recorded as an implicit research question: {:?}",
            response.evidence_links,
        );
        assert_ne!(response.intent, "unknown");
    }
}
