//! Regression coverage for issue #465: pronoun follow-ups should preserve the
//! prior topic when asking a creator question.

use formal_ai::event_log::EventLog;
use formal_ai::{
    CachedSourceClient, ConversationTurn, UniversalSolver, normalize_prompt_text,
    try_coreference_with_client,
};

#[allow(clippy::duplicate_mod)]
#[path = "../../fixtures/fact-claims/publisher_transport.rs"]
mod publisher_transport;
use publisher_transport::{FactTransport, fact_answer};

#[test]
fn pronoun_followup_resolves_prior_rust_topic_for_creator_question() {
    let solver = UniversalSolver::default();
    let first = solver.solve("What is Rust?");
    assert!(first.intent.starts_with("concept_lookup"));
    let history = [
        ConversationTurn::user("What is Rust?"),
        ConversationTurn::assistant(first.answer),
    ];
    let response = solver.solve_with_history("Who created it?", &history);
    assert!(response.links_notation.contains("coreference:resolved"));
    assert!(response.links_notation.contains("coreference:rewrite"));
}

#[test]
fn resolved_creator_question_reads_the_publisher_statement() {
    let prompt = "Who created it?";
    let history = [ConversationTurn::user("What is Rust?")];
    let mut log = EventLog::new();
    log.append("prior_turn:user", "What is Rust?");
    let client = CachedSourceClient::new(
        std::env::temp_dir().join(format!("formal-ai-465-creator-{}", std::process::id())),
        FactTransport {
            subject: "Q575650",
            term: "rust",
        },
    )
    .with_online(true);
    let response =
        try_coreference_with_client(prompt, &normalize_prompt_text(prompt), &mut log, &client)
            .expect("the resolved subject has a publisher statement");
    assert_eq!(response.intent, "fact_query");
    assert_eq!(
        response.answer,
        "Wikidata states that the inventor of Rust is Graydon Hoare. Source: https://prev.rust-lang.org/id-ID/faq.html"
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q575650")
    );
    assert!(response.links_notation.contains("coreference:resolved"));
    assert!(response.links_notation.contains("coreference:rewrite"));

    let solver = UniversalSolver::new(formal_ai::SolverConfig {
        offline: true,
        ..formal_ai::SolverConfig::default()
    });
    let cold = solver.solve_with_history(prompt, &history);
    assert!(!cold.answer.contains("Graydon Hoare"));
}

#[test]
fn rust_creator_fact_is_available_across_supported_languages() {
    for prompt in [
        "Who created Rust?",
        "Кто создал Rust?",
        "Rust किसने बनाया?",
        "谁创建 Rust?",
    ] {
        let response = fact_answer(prompt, "Q575650");
        assert_eq!(response.intent, "fact_query", "{prompt}");
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "wikidata:Q575650"),
            "{prompt}"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "wikidata:Q28937563"),
            "{prompt}"
        );
    }
}

#[test]
fn seeded_coreference_contexts_normalize_terminal_multilingual_surfaces() {
    let solver = UniversalSolver::default();
    let original = "Write me a Rust program that lists the files in the current directory";
    let first = solver.solve(original);
    assert_eq!(first.intent, "write_program");
    let history = [
        ConversationTurn::user(original),
        ConversationTurn::assistant(first.answer),
    ];
    let prompts = [
        "Explain the program?",
        "Объясни результаты?",
        "Объясни программу.",
        "इन परिणामों?",
        "यह प्रोग्राम?",
        "解释结果。",
        "解释程序？",
    ];
    for prompt in prompts {
        let reply = solver.solve_with_history(prompt, &history);
        assert_eq!(reply.intent, "coreference_program_artifact", "{prompt}");
        assert!(
            reply.links_notation.contains("coreference:resolved"),
            "{}",
            prompt
        );
    }
}

#[test]
fn seeded_coreference_contexts_reject_embedded_word_surfaces() {
    let solver = UniversalSolver::default();
    let original = "Write me a Rust program that lists the files in the current directory";
    let first = solver.solve(original);
    assert_eq!(first.intent, "write_program");
    let history = [
        ConversationTurn::user(original),
        ConversationTurn::assistant(first.answer),
    ];
    for prompt in [
        "Who created with?",
        "Explain programming?",
        "xрезультатыx",
        "xपरिणामोंx",
        "itself",
    ] {
        let reply = solver.solve_with_history(prompt, &history);
        assert!(
            !reply.links_notation.contains("coreference:resolved"),
            "{}",
            prompt
        );
    }
}

#[test]
fn seeded_antecedents_reject_embedded_alias_words() {
    let seeds = formal_ai::seed::coreference_seeds();
    for previous in [
        "Rustic furniture is handmade.",
        "A programming textbook.",
        "xпрограммаx",
    ] {
        assert!(
            seeds.pick_antecedent(&previous.to_lowercase()).is_none(),
            "{previous}"
        );
    }
    for previous in [
        "What is Rust?",
        "Объясни программу.",
        "यह प्रोग्राम?",
        "解释程序？",
    ] {
        assert!(
            seeds.pick_antecedent(&previous.to_lowercase()).is_some(),
            "{previous}"
        );
    }
}

#[test]
fn coreference_requires_real_user_antecedents_and_command_head_references() {
    let solver = UniversalSolver::default();
    let original = "Write me a Rust program that lists the files in the current directory";
    let first = solver.solve(original);
    assert_eq!(first.intent, "write_program");
    let history = [
        ConversationTurn::user(original),
        ConversationTurn::assistant(first.answer),
    ];
    for turns in [
        Vec::new(),
        vec![ConversationTurn::user("I enjoy fresh apples.")],
    ] {
        let reply = solver.solve_with_history("Explain the program?", &turns);
        assert!(!reply.links_notation.contains("coreference:resolved"));
    }
    for prompt in [
        "Explain programming?",
        "itself",
        "xрезультатыx",
        "xपरिणामोंx",
        "Summarize: Explain the program? It has three steps.",
        "Rewrite \"Explain the program? It has three steps.\"",
    ] {
        let reply = solver.solve_with_history(prompt, &history);
        assert!(
            !reply.links_notation.contains("coreference:resolved"),
            "{prompt}"
        );
    }
}

#[test]
fn seed_claim_and_handler_share_normalized_bounded_pronoun_contexts() {
    let seeds = formal_ai::seed::coreference_seeds();
    for prompt in [
        "Who created it?",
        "Explain the program?",
        "Объясни результаты?",
        "यह प्रोग्राम?",
        "解释程序？",
    ] {
        let normalized = formal_ai::normalize_prompt_text(prompt);
        assert!(seeds.matches_pronoun(&normalized), "{prompt}");
        assert!(seeds.matching_pronoun(&normalized).is_some(), "{prompt}");
    }
    for prompt in [
        "Who created with?",
        "Explain programming?",
        "itself",
        "xрезультатыx",
        "xपरिणामोंx",
    ] {
        let normalized = formal_ai::normalize_prompt_text(prompt);
        assert!(!seeds.matches_pronoun(&normalized), "{prompt}");
        assert!(seeds.matching_pronoun(&normalized).is_none(), "{prompt}");
    }
}
