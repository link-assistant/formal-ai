//! Warm only retained publisher captures, then let the full offline solver
//! replay them in a child process with an exclusive source-cache directory.
use std::path::Path;

use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::try_fact_live_answer;

const QUESTIONS: [&str; 4] = [
    "Make sense of this: what is the capital of France?",
    "What is the capital of the USA?",
    "столица россии",
    "谁创建了 Rust?",
];

struct PublisherFixtures;

impl SourceTransport for PublisherFixtures {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if !url.starts_with("https://www.wikidata.org/") {
            return Err(FetchError::Transport(format!(
                "unexpected publisher: {url}"
            )));
        }
        if url.contains("wbsearchentities") {
            let lower = url.to_ascii_lowercase();
            let subject = [
                ("france", "Q142", "France"),
                ("usa", "Q30", "USA"),
                ("россии", "Q159", "россии"),
                ("rust", "Q575650", "Rust"),
            ]
            .into_iter()
            .find(|(term, _, _)| {
                lower.contains(&format!(
                    "search={}",
                    formal_ai::seed::percent_encode(term).to_ascii_lowercase()
                ))
            });
            let body = subject.map_or_else(
                || serde_json::json!({"search": []}),
                |(_, id, label)| serde_json::json!({"search": [{"id": id, "label": label}]}),
            );
            return Ok(serde_json::to_vec(&body).expect("fixture search JSON"));
        }
        let id = url
            .rsplit('/')
            .next()
            .and_then(|tail| tail.strip_suffix(".json"))
            .filter(|id| {
                id.strip_prefix('Q').is_some_and(|digits| {
                    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
                })
            })
            .ok_or_else(|| FetchError::Transport(format!("unexpected entity URL: {url}")))?;
        let directory = if matches!(id, "Q575650" | "Q28937563") {
            "live-entities"
        } else {
            "entities"
        };
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/fact-claims")
            .join(directory)
            .join(format!("{id}.json"));
        std::fs::read(&path)
            .map_err(|error| FetchError::Transport(format!("{}: {error}", path.display())))
    }
}

pub(super) fn warm_cache(directory: &Path) {
    let cold =
        CachedSourceClient::new(directory.join("empty"), PublisherFixtures).with_online(false);
    assert!(
        try_fact_live_answer(QUESTIONS[3], &mut EventLog::new(), &cold).is_none(),
        "a creator cannot be invented without a publisher capture"
    );
    let client = CachedSourceClient::new(directory, PublisherFixtures).with_online(true);
    for prompt in QUESTIONS {
        let answer = try_fact_live_answer(prompt, &mut EventLog::new(), &client)
            .unwrap_or_else(|| panic!("publisher replay must answer {prompt:?}"));
        assert_eq!(answer.intent, "fact_query", "{prompt}");
    }
}

pub(super) fn assert_fact_answer(id: &str, answer: &str, evidence: &[String]) {
    let (subject, value, label) = match id {
        "p109" => ("Q142", "Q90", "Paris"),
        "p110" => ("Q30", "Q61", "Washington"),
        "p111" => ("Q159", "Q649", "Москва"),
        "p112" => ("Q575650", "Q28937563", "Graydon Hoare"),
        _ => panic!("unexpected factual probe: {id}"),
    };
    assert!(answer.contains(label), "{id}: {answer}");
    for qid in [subject, value] {
        assert!(
            evidence
                .iter()
                .any(|link| link == &format!("wikidata:{qid}")),
            "{id} retains {qid}"
        );
    }
}
