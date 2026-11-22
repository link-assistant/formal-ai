//! Issue #103 prompt-variation matrix, continued: structured capital fact
//! queries, multi-turn coreference and roleplay routing.
//!
//! Split from `prompt_variations.rs` to keep each test file under the
//! 1000-line Rust ceiling (R1016).

use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};
use formal_ai::{live_fact_question, try_fact_lookup_with_client};

// Replay publisher entities through an explicitly supplied test transport.
// The search response binds each exercised spelling to its fixture item.
struct CapitalTransport<'a> {
    subject: &'a str,
    term: &'a str,
}

impl SourceTransport for CapitalTransport<'_> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.contains("action=wbsearchentities") {
            return Ok(serde_json::json!({
                "search": [{"id": self.subject, "label": self.term}]
            })
            .to_string()
            .into_bytes());
        }
        let name = url.rsplit('/').next().unwrap_or_default();
        if !name.starts_with('Q') || !name.ends_with(".json") || name.contains(['?', '#', '%']) {
            return Err(FetchError::Transport(format!(
                "unhandled fixture URL: {url}"
            )));
        }
        std::fs::read(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/fact-claims/entities/"
            )
            .to_owned()
                + name,
        )
        .map_err(|error| FetchError::Transport(error.to_string()))
    }
}

pub(super) fn fact_answer(prompt: &str, subject: &str) -> SymbolicAnswer {
    let question = live_fact_question(prompt).unwrap_or_else(|| panic!("unparsed fact: {prompt}"));
    let client = CachedSourceClient::new(
        std::env::temp_dir().join(format!(
            "formal-ai-127-capitals-{}-{}",
            std::process::id(),
            formal_ai::sha256_hex(prompt.as_bytes())
        )),
        CapitalTransport {
            subject,
            term: &question.subject_term,
        },
    )
    .with_online(true);
    try_fact_lookup_with_client(
        prompt,
        &normalize_prompt(prompt),
        &mut EventLog::new(),
        &client,
    )
    .unwrap_or_else(|| panic!("unanswered fact: {prompt}"))
}
//
// (country_label, expected_subject_qid, expected_value_qid, expected_answer, prompts)
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
        "Wikidata states that the capital of Russia is Moscow.",
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
        "Wikidata states that the capital of Japan is Tokyo.",
        &[
            "What is the capital of Japan?",
            "Which city is Japan's capital?",
        ],
    ),
    (
        "France",
        "Q142",
        "Q90",
        "Wikidata states that the capital of France is Paris.",
        &[
            "What is the capital of France?",
            "What is the capital of the French Republic?",
        ],
    ),
    (
        "Germany",
        "Q183",
        "Q64",
        "Wikidata states that the capital of Germany is Berlin.",
        &[
            "What is the capital of Germany?",
            "What is Germany's capital?",
        ],
    ),
    (
        "China",
        "Q148",
        "Q956",
        "Wikidata states that the capital of People's Republic of China is Beijing.",
        &[
            "What is the capital of China?",
            "Which city is the capital of the People's Republic of China?",
        ],
    ),
    (
        "India",
        "Q668",
        "Q987",
        "Wikidata states that the capital of India is New Delhi.",
        &[
            "What is the capital of India?",
            "Which city is India's capital?",
        ],
    ),
    (
        "Brazil",
        "Q155",
        "Q2844",
        "Wikidata states that the capital of Brazil is Brasília.",
        &[
            "What is the capital of Brazil?",
            "Which city is Brazil's capital?",
        ],
    ),
    (
        "United States",
        "Q30",
        "Q61",
        "Wikidata states that the capital of United States is Washington, D.C..",
        &[
            "What is the capital of the United States?",
            "What is the capital of the USA?",
        ],
    ),
    (
        "United Kingdom",
        "Q145",
        "Q84",
        "Wikidata states that the capital of United Kingdom is London.",
        &[
            "What is the capital of the United Kingdom?",
            "What is the capital of the UK?",
        ],
    ),
];

#[test]
fn capital_matrix_resolves_every_publisher_fixture() {
    for (country, subject_qid, value_qid, expected_answer, prompts) in CAPITAL_CASES {
        for prompt in *prompts {
            let response = fact_answer(prompt, subject_qid);
            assert_eq!(
                response.intent, "fact_query",
                "{country} prompt {prompt:?} should route to fact_query, got {}",
                response.intent,
            );
            let expected = format!(
                "{expected_answer} Source: https://www.wikidata.org/wiki/Special:EntityData/{subject_qid}.json"
            );
            assert_eq!(response.answer, expected, "{prompt}");
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
    let response = fact_answer("столица россии", "Q159");
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
    let has_capture = response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("source:http:") && link.contains("sha256="));
    assert!(
        has_capture,
        "structured trace should record publisher capture identity, got: {:?}",
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
        let response = FormalAiEngine.answer(prompt);
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

fn russian_capital_fixture(prompt: &str) -> SymbolicAnswer {
    fact_answer(prompt, "Q159")
}

#[test]
fn russian_capital_russia_prompt_returns_moscow() {
    let response = russian_capital_fixture("столица россии");
    assert_eq!(
        response.intent, "fact_query",
        "reported prompt should route to fact_query, got {} with answer {}",
        response.intent, response.answer,
    );
    assert_eq!(
        response.answer,
        "По данным Викиданных, столица для Россия: Москва. Источник: https://www.wikidata.org/wiki/Special:EntityData/Q159.json"
    );
    assert!(
        response.answer.contains("Москва"),
        "reported prompt should answer in Russian with Moscow, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q159"),
        "reported prompt should record the Russia Wikidata anchor, got links: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q649"),
        "reported prompt should record the Moscow Wikidata anchor, got links: {:?}",
        response.evidence_links,
    );
}
