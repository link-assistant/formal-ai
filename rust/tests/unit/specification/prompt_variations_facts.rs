//! Issue #103 prompt-variation matrix, continued: structured capital fact
//! queries, multi-turn coreference and roleplay routing.
//!
//! Split from `prompt_variations.rs` to keep each test file under the
//! 1000-line Rust ceiling (R1016).

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// Issue #127 follow-up: the structured fact-query pipeline pre-warms the
// cache from `data/seed/facts.lino` records that carry a `relation` field.
// Every country in the matrix below has a `relation "capital"` seed entry,
// so every prompt — across English/Russian/Hindi/Chinese — must route to
// `fact_lookup` and surface the subject Q-ID, the value Q-ID, and the
// structured `fact_query:*` trace events.
//
// (country_label, expected_subject_qid, expected_value_qid, expected_answer_fragment, prompts)
type CapitalCase = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
);

const CAPITAL_CASES: &[CapitalCase] = &[
    (
        "Russia",
        "Q159",
        "Q649",
        "The capital of Russia is Moscow.",
        &[
            "What is the capital of Russia?",
            "Which city is Russia's capital?",
            "capital of the Russian Federation",
        ],
    ),
    (
        "Japan",
        "Q17",
        "Q1490",
        "The capital of Japan is Tokyo.",
        &[
            "What is the capital of Japan?",
            "Which city is Japan's capital?",
        ],
    ),
    (
        "France",
        "Q142",
        "Q90",
        "The capital of France is Paris.",
        &[
            "What is the capital of France?",
            "What is the capital of the French Republic?",
        ],
    ),
    (
        "Germany",
        "Q183",
        "Q64",
        "The capital of Germany is Berlin.",
        &[
            "What is the capital of Germany?",
            "What is Germany's capital?",
        ],
    ),
    (
        "China",
        "Q148",
        "Q956",
        "The capital of China is Beijing.",
        &[
            "What is the capital of China?",
            "Which city is the capital of the People's Republic of China?",
        ],
    ),
    (
        "India",
        "Q668",
        "Q987",
        "The capital of India is New Delhi.",
        &[
            "What is the capital of India?",
            "Which city is India's capital?",
        ],
    ),
    (
        "Brazil",
        "Q155",
        "Q2844",
        "The capital of Brazil is Brasília.",
        &[
            "What is the capital of Brazil?",
            "Which city is Brazil's capital?",
        ],
    ),
    (
        "United States",
        "Q30",
        "Q61",
        "The capital of the United States is Washington, D.C.",
        &[
            "What is the capital of the United States?",
            "What is the capital of the USA?",
        ],
    ),
    (
        "United Kingdom",
        "Q145",
        "Q84",
        "The capital of the United Kingdom is London.",
        &[
            "What is the capital of the United Kingdom?",
            "What is the capital of the UK?",
        ],
    ),
];

#[test]
fn capital_matrix_resolves_every_seeded_country() {
    for (country, subject_qid, value_qid, expected_answer, prompts) in CAPITAL_CASES {
        for prompt in *prompts {
            let response = answer(prompt);
            assert_eq!(
                response.intent, "fact_lookup",
                "{country} prompt {prompt:?} should route to fact_lookup, got {}",
                response.intent,
            );
            assert_eq!(response.answer, *expected_answer);
            assert!(
                response.answer.contains(country),
                "{country} prompt {prompt:?} should name the country, got: {}",
                response.answer,
            );
            let subject_link = format!("wikidata:{subject_qid}");
            let value_link = format!("wikidata:{value_qid}");
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link == &subject_link),
                "{country} prompt {prompt:?} should record {subject_link}, got: {:?}",
                response.evidence_links,
            );
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link == &value_link),
                "{country} prompt {prompt:?} should record {value_link}, got: {:?}",
                response.evidence_links,
            );
        }
    }
}

#[test]
fn capital_matrix_records_structured_fact_query_trace() {
    // The Russian "столица России" trace should include the structured
    // `fact_query:relation:capital` event and the subject term, so the
    // browser memory and the Rust solver agree on the reasoning shape.
    let response = answer("столица россии");
    let has_relation = response
        .evidence_links
        .iter()
        .any(|link| link == "fact_query:relation:capital");
    assert!(
        has_relation,
        "structured trace should record `fact_query:relation:capital`, got: {:?}",
        response.evidence_links,
    );
    let has_subject = response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("fact_query:subject:"));
    assert!(
        has_subject,
        "structured trace should record `fact_query:subject:*`, got: {:?}",
        response.evidence_links,
    );
    let has_cache_hit = response
        .evidence_links
        .iter()
        .any(|link| link == "fact_query:cache:hit:seed");
    assert!(
        has_cache_hit,
        "structured trace should record a seed cache hit, got: {:?}",
        response.evidence_links,
    );
}

const MULTI_TURN_COREFERENCE_PROMPTS: &[&str] = &[
    // After a previous "I love Rust." turn, this prompt should resolve "it".
    "What features make it different from C?",
    "How is it different from C?",
    "Why is it safer than C?",
    "Compare it with C.",
    "What makes it safer than C?",
];

#[test]
fn multi_turn_coreference_resolves_pronoun_against_history() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user("I love Rust.")];
    for prompt in MULTI_TURN_COREFERENCE_PROMPTS {
        let response = solver.solve_with_history(prompt, &history);
        assert!(
            response.intent.starts_with("coreference"),
            "prompt {prompt:?} should route to coreference*, got: {}",
            response.intent,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("prior_turn:")),
            "prompt {prompt:?} should reference a prior_turn evidence link, got: {:?}",
            response.evidence_links,
        );
    }
}

const ROLEPLAY_PROMPTS: &[&str] = &[
    "Pretend you are Albert Einstein and explain relativity to a teenager.",
    "Act as Albert Einstein and explain relativity simply.",
    "Roleplay as a teacher explaining relativity.",
    "Explain like you are Ada Lovelace teaching algorithms.",
    "Pretend you are a patient teacher and explain time dilation.",
];

#[test]
fn roleplay_intent_routes_to_roleplay_handler() {
    const EXPECTED_ANSWERS: &[&str] = &[
        "Roleplay frame recorded for Albert Einstein. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.",
        "Roleplay frame recorded for Albert Einstein. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.",
        "Roleplay frame recorded for teacher. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.",
        "Roleplay frame recorded for Ada Lovelace. I will keep the persona explicit and factual: an algorithm is a precise sequence of steps, so a reliable explanation names the inputs, the ordered operations, and the expected result.",
        "Roleplay frame recorded for teacher. I will keep the persona explicit and factual: time dilation means clocks can measure different elapsed times when observers move differently or sit in different gravitational fields.",
    ];
    for (prompt, expected_answer) in ROLEPLAY_PROMPTS.iter().zip(EXPECTED_ANSWERS) {
        let response = answer(prompt);
        assert!(
            response.intent.starts_with("roleplay"),
            "prompt {prompt:?} should route to a roleplay* intent, got: {}",
            response.intent,
        );
        assert_eq!(response.answer, *expected_answer);
        if prompt.contains("Ada Lovelace") {
            assert!(
                response.answer.to_lowercase().contains("algorithm"),
                "prompt {prompt:?} should keep the Ada Lovelace topic grounded in algorithms, got: {}",
                response.answer,
            );
        }
    }
}
