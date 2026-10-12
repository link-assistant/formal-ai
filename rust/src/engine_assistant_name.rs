//! Cached localized assistant-name answers.

use std::sync::OnceLock;

use crate::seed;

/// The seeded `assistant_name` answer in `language`, read once.
///
/// Every supported language has a record in the seed, so the lookup always
/// succeeds at runtime; on the build-time-impossible parse failure it degrades
/// to the intent slug (a meaning), never to hardcoded natural language, as
/// `engine_responses` does (R379, R1188-U1).
fn cached_response(cell: &'static OnceLock<String>, language: &str) -> &'static str {
    cell.get_or_init(|| {
        seed::localized_response("assistant_name", language)
            .unwrap_or_else(|| String::from("assistant_name"))
    })
    .as_str()
}

pub fn assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "en")
}

pub fn russian_assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "ru")
}

pub fn hindi_assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "hi")
}

pub fn chinese_assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "zh")
}
