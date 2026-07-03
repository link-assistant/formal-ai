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
