//! Seed-backed words for the summarization quality protocol.
//!
//! [`super`] owns the *protocol* — which files are drawn, which criteria are
//! scored, when the ratchet holds. This module owns its *words*: the published
//! description of each criterion, the sentences the ratchet emits when it
//! refuses a run, and the sentences `formal-ai summarization` prints. They live
//! in `data/seed/multilingual-responses-summarization-quality.lino` rather than
//! as literals in Rust, the way every other user-facing sentence in the system
//! does (R379: no hardcoded natural language in `src/`).
//!
//! The criterion *names* stay in Rust, because they are language-neutral keys
//! the committed baseline and the report parser read back. Only the prose moves.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::seed::{ResponseRecord, multilingual_responses};

/// Every intent this module serves shares one prefix, so the index holds the
/// summarization-quality vocabulary and nothing else.
const INTENT_PREFIX: &str = "summarization_";

/// The language this operator surface is published in. The protocol is a CI
/// gate whose report is read next to `docs/case-studies/issue-893/`; a
/// translated record for a given intent is preferred when the seed grows one.
const PUBLICATION_LANGUAGE: &str = "en";

/// `(intent, language) -> text`, parsed once.
///
/// [`crate::seed::response_for`] re-parses every response file per call, which
/// a run scoring ten criteria over two dozen files would pay for hundreds of
/// times.
fn index() -> &'static HashMap<(String, String), String> {
    static INDEX: OnceLock<HashMap<(String, String), String>> = OnceLock::new();
    INDEX.get_or_init(|| {
        multilingual_responses()
            .into_iter()
            .filter(is_summarization_record)
            .map(|record| ((record.intent, record.language), record.text))
            .collect()
    })
}

fn is_summarization_record(record: &ResponseRecord) -> bool {
    record.intent.starts_with(INTENT_PREFIX)
}

/// Render the seeded sentence for `intent`, substituting its named fields.
///
/// A missing record yields the intent itself rather than an empty string, so a
/// seed gap shows up in the report as an obviously wrong word instead of
/// silently deleting a violation the ratchet meant to state.
#[must_use]
pub fn sentence(intent: &str, fields: &[(&str, &str)]) -> String {
    let key = (intent.to_owned(), PUBLICATION_LANGUAGE.to_owned());
    let mut rendered = index()
        .get(&key)
        .cloned()
        .unwrap_or_else(|| intent.to_owned());
    for (name, value) in fields {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}
