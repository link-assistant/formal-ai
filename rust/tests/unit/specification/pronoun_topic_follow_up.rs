//! Regression coverage for issue #465: pronoun follow-ups should preserve the
//! prior topic when asking a creator question.

use formal_ai::{ConversationTurn, UniversalSolver};

struct RustCreatorCase {
    language: &'static str,
    prompt: &'static str,
}

#[test]
fn pronoun_followup_resolves_prior_rust_topic_for_creator_question() {
    let solver = UniversalSolver::default();
    let first = solver.solve("What is Rust?");
    assert!(
        first.intent.starts_with("concept_lookup"),
        "first turn should establish the Rust topic, got: {}",
        first.intent
    );

    let history = [
        ConversationTurn::user("What is Rust?"),
        ConversationTurn::assistant(first.answer),
    ];
    let response = solver.solve_with_history("Who created it?", &history);

    assert_eq!(response.intent, "fact_lookup");
    assert_eq!(
        response.answer,
        "Rust was originally created by Graydon Hoare at Mozilla Research, and Mozilla sponsored the project."
    );
    assert!(
        response.answer.contains("Graydon Hoare"),
        "creator answer should name Graydon Hoare, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Mozilla"),
        "creator answer should include the Mozilla context, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q575650"),
        "Rust follow-up should keep the Rust Wikidata anchor, got: {:?}",
        response.evidence_links
    );
    assert!(
        response.links_notation.contains("coreference:resolved")
            && response.links_notation.contains("coreference:rewrite"),
        "follow-up trace should record the coreference resolution and rewrite: {}",
        response.links_notation
    );
}

#[test]
fn rust_creator_fact_is_available_across_supported_languages() {
    let solver = UniversalSolver::default();
    let cases = [
        RustCreatorCase {
            language: "en",
            prompt: "Who created Rust?",
        },
        RustCreatorCase {
            language: "ru",
            prompt: "Кто создал Rust?",
        },
        RustCreatorCase {
            language: "hi",
            prompt: "Rust किसने बनाया?",
        },
        RustCreatorCase {
            language: "zh",
            prompt: "谁创建 Rust?",
        },
    ];

    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "fact_lookup",
            "{} Rust creator prompt should route to fact_lookup, got {} -> {}",
            case.language, response.intent, response.answer
        );
        let documented_answer = match case.language {
            "en" => {
                "Rust was originally created by Graydon Hoare at Mozilla Research, and Mozilla sponsored the project."
            }
            "ru" => {
                "Rust изначально создал Graydon Hoare в Mozilla Research, а Mozilla спонсировала проект."
            }
            "hi" => {
                "Rust को मूल रूप से Graydon Hoare ने Mozilla Research में बनाया था, और Mozilla ने इस परियोजना को प्रायोजित किया."
            }
            "zh" => "Rust 最初由 Graydon Hoare 在 Mozilla Research 创建, Mozilla 赞助了该项目。",
            _ => unreachable!("the case table documents every supported language"),
        };
        assert_eq!(response.answer, documented_answer);
        assert!(
            response.answer.contains("Graydon Hoare"),
            "{} Rust creator answer should name Graydon Hoare, got: {}",
            case.language,
            response.answer
        );
        assert!(
            response.answer.contains("Mozilla"),
            "{} Rust creator answer should include Mozilla context, got: {}",
            case.language,
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "wikidata:Q575650"),
            "{} Rust creator fact should keep the Rust Wikidata anchor, got: {:?}",
            case.language,
            response.evidence_links
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
