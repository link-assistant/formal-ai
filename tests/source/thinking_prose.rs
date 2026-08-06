//! Seed-backed prose for thinking traces (issue #889, parent #710).
//!
//! [`super::thinking`] owns the *shape* of a reasoning trace — which steps
//! exist, in what order, and which concrete value each one carries. This module
//! owns its *words*. Every sentence a non-UI surface prints (the CLI
//! `--thinking` trace, the OpenAI/Anthropic API `reasoning` fields, the Telegram
//! expandable blockquote) is looked up from
//! `data/seed/multilingual-responses-thinking*.lino` in the answer language,
//! the same way every other user-facing sentence in the system is (R379: no
//! hardcoded natural language in `src`).
//!
//! Before this module the sentences were English literals inside `thinking.rs`,
//! so the browser was localized (its own catalog) while a Russian, Hindi,
//! Chinese or Spanish answer on every other surface still narrated its
//! reasoning in English. Adding a language now means adding records to the seed
//! files, not editing Rust.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::seed::{multilingual_responses, ResponseRecord};

/// Intent prefix for the per-step sentences.
pub(crate) const STEP_INTENT_PREFIX: &str = "thinking_step_";
/// Intent suffix for the variant of a step that carries no concrete detail.
pub(crate) const PLAIN_INTENT_SUFFIX: &str = "_plain";
/// Intent prefix for the narrative headline (issue #676, R8).
pub(crate) const NARRATIVE_INTENT_PREFIX: &str = "thinking_narrative_";
/// Intent prefix for the name of a language as spoken in the answer language.
pub(crate) const LANGUAGE_NAME_INTENT_PREFIX: &str = "thinking_language_name_";

/// The language every lookup falls back to when a record is missing, matching
/// [`crate::seed::localized_response`]'s last resort.
const FALLBACK_LANGUAGE: &str = "en";

/// `(intent, language) -> text`, parsed once.
///
/// [`crate::seed::response_for`] re-parses every response file per call, which
/// is fine for the one response an answer needs but not for the dozen sentences
/// a thinking trace renders, so the thinking records get an index.
fn index() -> &'static HashMap<(String, String), String> {
    static INDEX: OnceLock<HashMap<(String, String), String>> = OnceLock::new();
    INDEX.get_or_init(|| {
        multilingual_responses()
            .into_iter()
            .filter(is_thinking_record)
            .map(|record| ((record.intent, record.language), record.text))
            .collect()
    })
}

/// Every intent this module serves shares one prefix, so the index holds the
/// thinking vocabulary and nothing else.
pub(crate) const THINKING_INTENT_PREFIX: &str = "thinking_";

fn is_thinking_record(record: &ResponseRecord) -> bool {
    record.intent.starts_with(THINKING_INTENT_PREFIX)
}

/// Normalize an answer-language slug (`ru-RU`, ` EN `) to its primary subtag.
#[must_use]
pub fn normalize_language(code: &str) -> String {
    let normalized = code.trim().to_ascii_lowercase();
    normalized
        .split(['-', '_'])
        .next()
        .unwrap_or(normalized.as_str())
        .to_owned()
}

/// Look up one thinking sentence and substitute its named template fields.
///
/// Falls back to English when the requested language has no record for the
/// intent, so a partially translated seed degrades to a readable trace instead
/// of an empty one. Returns `None` only when no language has the intent at all,
/// which lets callers fall back to a coarser template.
#[must_use]
pub fn thinking_prose(intent: &str, language: &str, fields: &[(&str, &str)]) -> Option<String> {
    let normalized = normalize_language(language);
    let table = index();
    let text = table
        .get(&(intent.to_owned(), normalized))
        .or_else(|| table.get(&(intent.to_owned(), FALLBACK_LANGUAGE.to_owned())))?;
    let mut rendered = text.clone();
    for (name, value) in fields {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    Some(rendered)
}

/// Name the language `code` as it is called in `answer_language`.
///
/// Issue #706 put the *English* name of every registered language in
/// `data/seed/languages.lino`; the remaining cells of that matrix live in
/// `data/seed/multilingual-responses-thinking-narrative.lino`, so a Russian
/// trace reads «Определить язык запроса: русский.» rather than naming the
/// language in English. A language registered without those records still
/// narrates by its ledger name rather than by a bare slug.
#[must_use]
pub fn language_label(answer_language: &str, code: &str) -> String {
    let primary = normalize_language(code);
    let slug = if primary.is_empty() {
        "unknown"
    } else {
        &primary
    };
    let intent = format!("{LANGUAGE_NAME_INTENT_PREFIX}{slug}");
    if let Some(name) = thinking_prose(&intent, answer_language, &[]) {
        return name;
    }
    if let Some(name) = crate::language::language_name(slug) {
        return name.to_owned();
    }
    slug.to_owned()
}

#[path = "source_tests/thinking_prose/tests.rs"]
mod tests;
