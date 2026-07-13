//! Cached localized seed responses used by the symbolic engine.

use std::sync::OnceLock;

use crate::seed;

/// Hardcoded English fallbacks used only when `data/seed/multilingual-responses.lino`
/// cannot be parsed (which would be a build-time bug since the file is
/// embedded via `include_str!`). All real reads come from [`crate::seed`].
const FALLBACK_GREETING_ANSWER: &str = "Hi, how may I help you?";
const FALLBACK_WELLBEING_ANSWER: &str =
    "I'm doing great, thanks for asking! I'm ready to help — what would you like to do?";
const FALLBACK_FAREWELL_ANSWER: &str = "Goodbye! Feel free to return any time.";
const FALLBACK_TEST_STATUS_ANSWER: &str = "Test passed. I'm here.";
const FALLBACK_COURTESY_RESPONSE_ANSWER: &str = "Glad to hear it. What would you like to do next?";
const FALLBACK_ASSISTANT_FREE_TIME_ANSWER: &str = "I do not have free time the way a person does. Between prompts I am idle; when the dialog is active, I help with tasks, rules, and explanations.";
const FALLBACK_IDENTITY_ANSWER: &str = "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";
const FALLBACK_UNKNOWN_ANSWER: &str = "I don't know how to answer that yet. I cannot answer that from local Links Notation rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";
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

fn cached_response(
    cell: &'static OnceLock<String>,
    intent: &str,
    language: &str,
    fallback: &str,
) -> &'static str {
    cell.get_or_init(|| {
        seed::response_for(intent, language).unwrap_or_else(|| String::from(fallback))
    })
    .as_str()
}

pub fn greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "en", FALLBACK_GREETING_ANSWER)
}

pub fn wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "en", FALLBACK_WELLBEING_ANSWER)
}

pub fn farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "en", FALLBACK_FAREWELL_ANSWER)
}

pub fn courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "en",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

pub fn assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "assistant_free_time",
        "en",
        FALLBACK_ASSISTANT_FREE_TIME_ANSWER,
    )
}

pub fn russian_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "ru", FALLBACK_FAREWELL_ANSWER)
}

pub fn hindi_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "hi", FALLBACK_FAREWELL_ANSWER)
}

pub fn chinese_farewell_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "farewell", "zh", FALLBACK_FAREWELL_ANSWER)
}

pub fn test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "en", FALLBACK_TEST_STATUS_ANSWER)
}

pub fn russian_test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "ru", FALLBACK_TEST_STATUS_ANSWER)
}

pub fn hindi_test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "hi", FALLBACK_TEST_STATUS_ANSWER)
}

pub fn chinese_test_status_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "test_status", "zh", FALLBACK_TEST_STATUS_ANSWER)
}

pub fn russian_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "ru",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

pub fn hindi_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "hi",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

pub fn chinese_courtesy_response_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "courtesy_response",
        "zh",
        FALLBACK_COURTESY_RESPONSE_ANSWER,
    )
}

pub fn russian_assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "assistant_free_time",
        "ru",
        FALLBACK_ASSISTANT_FREE_TIME_ANSWER,
    )
}

pub fn hindi_assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "assistant_free_time",
        "hi",
        FALLBACK_ASSISTANT_FREE_TIME_ANSWER,
    )
}

pub fn chinese_assistant_free_time_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(
        &CELL,
        "assistant_free_time",
        "zh",
        FALLBACK_ASSISTANT_FREE_TIME_ANSWER,
    )
}

pub fn identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "en", FALLBACK_IDENTITY_ANSWER)
}

pub fn unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "en", FALLBACK_UNKNOWN_ANSWER)
}

pub fn russian_wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "ru", FALLBACK_WELLBEING_ANSWER)
}

pub fn hindi_wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "hi", FALLBACK_WELLBEING_ANSWER)
}

pub fn chinese_wellbeing_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "wellbeing", "zh", FALLBACK_WELLBEING_ANSWER)
}

pub fn russian_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "ru", FALLBACK_GREETING_ANSWER)
}

pub fn hindi_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "hi", FALLBACK_GREETING_ANSWER)
}

pub fn chinese_greeting_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "greeting", "zh", FALLBACK_GREETING_ANSWER)
}

pub fn russian_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "ru", FALLBACK_IDENTITY_ANSWER)
}

pub fn hindi_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "hi", FALLBACK_IDENTITY_ANSWER)
}

pub fn chinese_identity_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "identity", "zh", FALLBACK_IDENTITY_ANSWER)
}

pub fn russian_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "ru", FALLBACK_UNKNOWN_ANSWER)
}

pub fn hindi_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "hi", FALLBACK_UNKNOWN_ANSWER)
}

pub fn chinese_unknown_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "unknown", "zh", FALLBACK_UNKNOWN_ANSWER)
}

pub const fn unknown_language_fallback_answer() -> &'static str {
    FALLBACK_UNKNOWN_LANGUAGE_ANSWER
}
