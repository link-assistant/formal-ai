//! Physical publisher replay shared by full and isolated test targets.

use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{SymbolicAnswer, live_fact_question, try_fact_lookup_with_client};

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
