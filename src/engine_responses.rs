//! Cached localized seed responses used by the symbolic engine.

use std::sync::OnceLock;

use crate::seed;

// R379 burn-down (#659): the per-intent English prose fallbacks that used to
// live here have been migrated out — every one of these answers already exists
// for all four languages in `data/seed/multilingual-responses.lino`, so the
// hardcoded copies were pure duplication. `cached_response` now reads only from
// the seed and, on the build-time-impossible parse failure, degrades to the
// intent slug (a meaning), never to hardcoded natural language.
//
// The one remaining fallback below fires on a real runtime path (an
// unsupported language was detected) and is still tracked in
// `scripts/hardcoded-language-allowlist.txt` pending its own migration.
const FALLBACK_UNKNOWN_LANGUAGE_ANSWER: &str = concat!(
    "I detected an unsupported language. Falling back to English: I cannot ",
    "answer that from local Links Notation rules yet. Please add a fact or ",
    "add a rule in Links Notation, then run the request again."
);

pub const GREETING_EXAMPLES: &[&str] = &["Hi", "Hello", "Hey"];
pub const TEST_STATUS_EXAMPLES: &[&str] =
    &["Test", "Test passed", "I'm here", "test passed, I'm here"];
pub const COURTESY_RESPONSE_EXAMPLES: &[&str] = &["I am fine, thank you", "thanks"];
pub const ASSISTANT_FREE_TIME_EXAMPLES: &[&str] = &[
    "What do you do in your free time?",
    "Что делаешь в свободное время?",
];
pub const IDENTITY_EXAMPLES: &[&str] = &[
    "Who are you?",
    "What are you?",
    "Tell me about yourself",
    "What is formal-ai?",
];
pub const UNKNOWN_EXAMPLES: &[&str] = &["Any prompt without a matching symbolic rule"];

/// Resolve a localized response from the seed, caching the first read.
///
/// Reads come from [`crate::seed`] (backed by
/// `data/seed/multilingual-responses.lino`). Every intent used here has a
/// record for all four supported languages, so the lookups always succeed at
/// runtime. On the build-time-impossible seed-parse failure the value degrades
/// to the requested language's English record and finally to the intent slug —
/// a meaning, never hardcoded natural language (R379).
fn cached_response(cell: &'static OnceLock<String>, intent: &str, language: &str) -> &'static str {
    cell.get_or_init(|| {
        seed::response_for(intent, language)
            .or_else(|| seed::response_for(intent, "en"))
            .unwrap_or_else(|| intent.to_string())
    })
    .as_str()
}

pub fn greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "en")
}

pub fn wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "en")
}

pub fn farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "en")
}

pub fn courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "courtesy_response", "en")
}

pub fn assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "assistant_free_time", "en")
}

pub fn russian_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "ru")
}

pub fn hindi_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "hi")
}

pub fn chinese_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "zh")
}

pub fn test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "en")
}

pub fn russian_test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "ru")
}

pub fn hindi_test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "hi")
}

pub fn chinese_test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "zh")
}

pub fn russian_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "courtesy_response", "ru")
}

pub fn hindi_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "courtesy_response", "hi")
}

pub fn chinese_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "courtesy_response", "zh")
}

pub fn russian_assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "assistant_free_time", "ru")
}

pub fn hindi_assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "assistant_free_time", "hi")
}

pub fn chinese_assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "assistant_free_time", "zh")
}

pub fn identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "en")
}

pub fn unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "en")
}

pub fn russian_wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "ru")
}

pub fn hindi_wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "hi")
}

pub fn chinese_wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "zh")
}

pub fn russian_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "ru")
}

pub fn hindi_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "hi")
}

pub fn chinese_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "zh")
}

pub fn russian_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "ru")
}

pub fn hindi_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "hi")
}

pub fn chinese_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "zh")
}

pub fn russian_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "ru")
}

pub fn hindi_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "hi")
}

pub fn chinese_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "zh")
}

pub const fn unknown_language_fallback_answer() -> &'static str {
    FALLBACK_UNKNOWN_LANGUAGE_ANSWER
}
