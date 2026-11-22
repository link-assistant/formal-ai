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
pub(super) struct FactTransport<'a> {
    pub(super) subject: &'a str,
    pub(super) term: &'a str,
}

impl SourceTransport for FactTransport<'_> {
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
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fact-claims");
        std::fs::read(root.join("entities").join(name))
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    return std::fs::read(root.join("live-entities").join(name));
                }
                Err(error)
            })
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
        FactTransport {
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

#[test]
fn publisher_quantity_units_and_painting_attribution_are_resolved() {
    let speed = fact_answer("What is the speed of light?", "Q2111");
    assert_eq!(
        speed.answer,
        "Wikidata states that the numeric value of speed of light in vacuum is 299792458 metre per second. Source: https://www.finlex.fi/fi/lainsaadanto/saadoskokoelma/2020/431"
    );
    assert!(
        speed
            .evidence_links
            .iter()
            .any(|link| link.contains("Q182429"))
    );
    let painter = fact_answer("Кто написал Мону Лизу?", "Q12418");
    assert_eq!(
        painter.answer,
        "По данным Викиданных, художник для Мона Лиза: Леонардо да Винчи. Источник: https://rkd.nl/explore/images/302327"
    );
    assert!(
        painter
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q762")
    );
}

#[test]
fn publisher_temporal_claims_keep_the_requested_precision_and_all_equal_ranks() {
    let built = fact_answer("When was the Eiffel Tower built?", "Q243");
    assert_eq!(
        built.answer,
        "Wikidata states that the inception year of Eiffel Tower is 1887. Source: https://www.wikidata.org/wiki/Special:EntityData/Q243.json"
    );
    let opening = fact_answer("When did the Eiffel Tower open?", "Q243");
    assert_eq!(
        opening.answer,
        "Wikidata lists these opening dates for Eiffel Tower: 1889-03-31, 1889-05-15. Source: https://www.wikidata.org/wiki/Special:EntityData/Q243.json"
    );
    assert!(
        opening.evidence_links.iter().any(|link| {
            link.contains("Special:EntityData/Q243.json") && link.contains("sha256=")
        })
    );
    let foundation = fact_answer("When was UNESCO founded?", "Q7809");
    assert_eq!(
        foundation.answer,
        "Wikidata states that the inception year of UNESCO is 1945. Source: https://www.un.org/youthenvoy/2013/08/unesco-united-nations-educational-scientific-and-cultural-organization/"
    );
    assert!(
        foundation
            .evidence_links
            .contains(&"wikidata:Q7809".to_owned())
    );
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
    "Pretend you are Albert Einstein: A moving clock can measure a different elapsed time.",
    "Act as Einstein: A moving clock can measure a different elapsed time.",
    "Roleplay as a teacher: A moving clock can measure a different elapsed time.",
    "Explain like you are Ada Lovelace: An algorithm is a sequence of steps.",
    "Pretend you are a patient teacher and explain time dilation.",
];

#[test]
fn roleplay_intent_routes_to_roleplay_handler() {
    const EXPECTED_ANSWERS: &[&str] = &[
        "Roleplay frame for Albert Einstein. Supplied facts: A moving clock can measure a different elapsed time.",
        "Roleplay frame for Albert Einstein. Supplied facts: A moving clock can measure a different elapsed time.",
        "Roleplay frame for a teacher. Supplied facts: A moving clock can measure a different elapsed time.",
        "Roleplay frame for Ada Lovelace. Supplied facts: An algorithm is a sequence of steps.",
        "Name the role, then put the facts to explain after a colon.",
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
