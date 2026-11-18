//! These tests cover the declared lexical corpus, not general CNL conformance.
use ::formal_ai::cnl_output::*;

/// The projections of the declared "omniscient" meaning that the JavaScript
/// reference certifies (`node experiments/issue-1584/round-trip-certificates.mjs`):
/// only Minimal English and Special English, whose checks are all decidable.
const CERTIFIED_OMNISCIENT: [&str; 9] = [
    "en minimal-english standard",
    "en special-english standard",
    "ru minimal-english standard",
    "ru minimal-english top-37",
    "hi minimal-english standard",
    "hi minimal-english top-37",
    "zh minimal-english standard",
    "es minimal-english standard",
    "es minimal-english top-37",
];

#[test]
fn declared_multilingual_meanings_round_trip_without_certifying_unchecked_standards() {
    let policy = cnl_policy();
    let mut certified = Vec::new();
    for (language, simple, dense) in [
        ("en", "I know everything.", "I am omniscient."),
        ("ru", "Я знаю всё.", "Я всеведущ."),
        ("hi", "मैं सब कुछ जानता हूँ।", "मैं सर्वज्ञ हूँ।"),
        ("zh", "我知道一切。", "我无所不知。"),
        // Spanish participates in the same realization and profile matrix.
        ("es", "Yo sé todo.", "Soy omnisciente."),
    ] {
        let document = formalize_output(simple, language, policy);
        let graph = output_links_notation(&document, policy);
        let configuration = output_configuration(None, Some("max-precision"), policy).unwrap();
        let rendered = render_output(&document, configuration, policy, &());
        assert_eq!(rendered.text, dense);
        assert!(!rendered.notice.is_empty());
        assert_eq!(output_source(&rendered.content, language, policy), dense);
        assert_eq!(rendered.links_notation, graph);
        let dense_document = formalize_output(dense, language, policy);
        for profile in &policy.profiles {
            for density in ["standard", "primes", "top-37", "max-precision"] {
                let configuration =
                    output_configuration(Some(&profile.id), Some(density), policy).unwrap();
                let projected = project_output(&dense_document, configuration, policy);
                assert_eq!(
                    output_links_notation(&formalize_output(&projected, language, policy), policy),
                    graph
                );
                let report = validate_output(&projected, language, configuration, policy, &());
                if report.compliant {
                    // A certificate needs every check to pass with evidence.
                    assert!(report.violations.is_empty() && report.unchecked.is_empty());
                    certified.push(format!("{language} {} {density}", profile.id));
                }
            }
        }
        let simple = project_output(&dense_document, OutputConfig::default(), policy);
        assert_eq!(
            output_links_notation(&formalize_output(&simple, language, policy), policy),
            graph
        );
    }
    assert_eq!(certified, CERTIFIED_OMNISCIENT);
}

#[test]
fn lexical_density_preserves_known_named_profile_constraints() {
    let policy = cnl_policy();
    let document = formalize_output("I am omniscient.", "en", policy);
    for density in ["max-precision", "top-200"] {
        let configuration = output_configuration(Some("e-prime"), Some(density), policy).unwrap();
        let rendered = render_output(&document, configuration, policy, &());
        assert_eq!(rendered.text, "I know everything.");
        assert!(rendered.report.violations.is_empty());
        assert!(!rendered.report.compliant);
        assert_eq!(
            output_links_notation(&formalize_output(&rendered.text, "en", policy), policy),
            rendered.links_notation
        );
    }
}

#[test]
fn max_precision_notices_preserve_chat_response_contracts() {
    let solver = ::formal_ai::UniversalSolver::default();
    for (instruction, prompt, expected) in [
        ("Answer in JSON.", "2 + 2", r#"{"answer":"2 + 2 = 4"}"#),
        (
            "Answer with exactly one word: yes or no.",
            "Is 3 greater than 2?",
            "yes",
        ),
    ] {
        let request = serde_json::from_value(serde_json::json!({
            "lexical_density": "max-precision",
            "messages": [
                {"role": "system", "content": instruction},
                {"role": "user", "content": prompt},
            ],
        }))
        .unwrap();
        let completion = ::formal_ai::create_chat_completion_with_solver(&request, &solver);
        assert_eq!(completion.choices[0].message.content.plain_text(), expected);
        let projection = completion.output_projection.unwrap();
        assert!(
            projection["notice"]
                .as_str()
                .unwrap()
                .contains("dictionary")
        );
        assert_eq!(projection["content"], expected);
    }
}

#[test]
fn opaque_input_and_configuration_limits_are_lossless_on_every_target() {
    let policy = cnl_policy();
    for input in [
        "I do not know everything.",
        "I know everything except the password.",
        "`I am omniscient.`",
        "uhh?? nope lol 🤔",
        "Not every person knows everything.",
    ] {
        let document = formalize_output(input, "en", policy);
        for profile in &policy.profiles {
            let configuration =
                output_configuration(Some(&profile.id), Some("max-precision"), policy).unwrap();
            assert_eq!(project_output(&document, configuration, policy), input);
        }
    }
    assert_eq!(
        frequency_tier("top-9007199254740991"),
        Some(9_007_199_254_740_991)
    );
    for invalid in [
        "top-0",
        "top-01",
        "top-1.5",
        "top-1e3",
        "top-9007199254740992",
    ] {
        assert!(output_configuration(None, Some(invalid), policy).is_err());
    }
    assert_eq!(output_words("मैं सब कुछ जानता हूँ।", "hi").len(), 5);
    assert_eq!(output_words("我知道。", "zh").len(), 3);
}

#[test]
fn e_prime_contractions_and_missing_frequency_evidence_do_not_receive_certificates() {
    let policy = cnl_policy();
    let configuration = output_configuration(Some("e-prime"), None, policy).unwrap();
    for text in [
        "I’m here.",
        "John's tired.",
        "John's hat fell.",
        "I ain't ready.",
        "Y'all're here.",
        "’Tis ready.",
        "Thou wast here.",
        "Thou beest wise.",
    ] {
        assert!(!validate_output(text, "en", configuration, policy, &()).compliant);
    }
    let report = validate_output("I know this.", "en", configuration, policy, &());
    assert!(!report.compliant);
    assert!(report.unchecked.contains(&"e-prime-english".to_owned()));
    let configuration = output_configuration(Some("e-prime"), Some("top-1"), policy).unwrap();
    let report = validate_output("Alice sees Bob.", "en", configuration, policy, &());
    assert!(!report.compliant);
    assert!(
        report
            .unchecked
            .contains(&"frequency-vocabulary".to_owned())
    );
    for invalid in [
        r#"{"output_standard":"unknown"}"#,
        r#"{"lexical_density":"top-0"}"#,
    ] {
        assert!(
            serde_json::from_str::<::formal_ai::protocol::ChatCompletionRequest>(invalid).is_err()
        );
    }
}

// Issue #1584: the controlled dialogue through the real solver (twin of
// rust/tests/web/cnl-solver-dialogue.test.mjs).
#[test]
fn controlled_dialogue_answers_declared_facts_and_asks_which_reading() {
    use ::formal_ai::{ConversationTurn, FormalAiEngine, solve_with_history};
    let cat = FormalAiEngine.answer("Is a cat an animal?");
    assert_eq!(cat.intent, "cnl_answer");
    assert_eq!(cat.answer, "Yes. Every cat is an animal.");
    assert!(cat.evidence_links.contains(&"cnl:en".to_owned()));
    let question = "Is a mouse an animal?";
    let clarification = FormalAiEngine.answer(question);
    assert_eq!(clarification.intent, "cnl_clarification");
    assert!(clarification.answer.contains("\n1) "));
    let history = [
        ConversationTurn::user(question),
        ConversationTurn::assistant(clarification.answer.as_str()),
    ];
    let chosen = solve_with_history("1", &history);
    assert_eq!(chosen.intent, "cnl_answer");
    assert!(chosen.evidence_links.contains(&"cnl:en".to_owned()));
}

#[test]
fn task_paraphrases_reach_the_solver_and_understood_prompts_keep_their_answers() {
    use ::formal_ai::FormalAiEngine;
    // The task paraphrase reaches the canonical calculation.
    let paraphrase = FormalAiEngine.answer("add 10 and 3");
    assert_eq!(paraphrase.intent, "calculation");
    assert_eq!(paraphrase.answer, "10 + 3 = 13");
    assert!(
        paraphrase
            .evidence_links
            .contains(&"cnl:task:en".to_owned()),
        "{:?}",
        paraphrase.evidence_links
    );
    // Issue #962: no CNL fallback for a prompt the solver already understood.
    let sum = FormalAiEngine.answer("What is 2 plus 2?");
    assert_eq!(sum.answer, "2 plus 2 = 4");
    assert!(
        !sum.evidence_links
            .iter()
            .any(|item| item.starts_with("cnl:task"))
    );
}

const MAX_PRECISION_CAT: &str = "Maximum precision uses specific words. Look up unfamiliar words in a \
     dictionary to check their exact meaning.\n\nYes. Cats are animals.";

#[test]
fn an_in_dialog_density_switch_rerenders_the_controlled_answer_deterministically() {
    use ::formal_ai::{ConversationTurn, FormalAiEngine, solve_with_history};
    let question = "Is a cat an animal?";
    let first = FormalAiEngine.answer(question);
    assert_eq!(first.intent, "cnl_answer");
    assert_eq!(first.answer, "Yes. Every cat is an animal.");
    let history = [
        ConversationTurn::user(question),
        ConversationTurn::assistant(first.answer.as_str()),
    ];
    // A declared directive (a bare `max-precision` is not one) re-renders the previous answer.
    let switched = solve_with_history("Use max-precision.", &history);
    let again = solve_with_history("Use max-precision.", &history);
    assert_eq!(switched.intent, "cnl_paraphrase");
    assert_eq!(switched.answer, MAX_PRECISION_CAT);
    assert_eq!(again.answer, switched.answer);
    let named = solve_with_history("Rewrite in max-precision mode", &history);
    assert_eq!(named.answer, MAX_PRECISION_CAT);
}

// Issue #1550: a controlled dialogue answer and a density switch are drafts
// that still run the recursive procedure (twin of "controlled dialogue answers
// retain the recursive procedure trace" in issue-1550-procedure-census.test.mjs).
#[test]
fn controlled_dialogue_answers_retain_the_recursive_procedure_trace() {
    use ::formal_ai::meta_executor::{last_turn_trace, remember_trace};
    use ::formal_ai::{ConversationTurn, UniversalSolver};
    let solver = UniversalSolver::default();
    let question = "Is a cat an animal?";
    remember_trace(&[]);
    let cat = solver.solve(question);
    let trace = last_turn_trace();
    assert_eq!(cat.intent, "cnl_answer");
    assert_eq!(cat.answer, "Yes. Every cat is an animal.");
    assert!(trace.iter().any(|event| event.kind == "meta_exec:result"));
    let history = [
        ConversationTurn::user(question),
        ConversationTurn::assistant(cat.answer.as_str()),
    ];
    remember_trace(&[]);
    let switched = solver.solve_with_history("Use max-precision.", &history);
    let trace = last_turn_trace();
    assert_eq!(switched.answer, MAX_PRECISION_CAT);
    assert!(trace.iter().any(|event| event.kind == "meta_exec:result"));
}
