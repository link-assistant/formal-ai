use std::sync::OnceLock;

use crate::seed;

const FALLBACK_ASSISTANT_NAME_ANSWER: &str =
    "I'm formal AI, and currently I don't have a name. But you can name me as you like.";
const FALLBACK_ASSISTANT_NAME_ANSWER_RU: &str =
    "Я formal AI, и сейчас у меня нет имени. Но вы можете назвать меня как хотите.";
const FALLBACK_ASSISTANT_NAME_ANSWER_HI: &str =
    "मैं formal AI हूँ, और अभी मेरा कोई नाम नहीं है। लेकिन आप मुझे अपनी पसंद का नाम दे सकते हैं।";
const FALLBACK_ASSISTANT_NAME_ANSWER_ZH: &str =
    "我是 formal AI,目前还没有名字。不过您可以按自己的喜好给我起名。";

pub const ASSISTANT_NAME_EXAMPLES: &[&str] = &[
    "What is your name?",
    "Как твое имя?",
    "Как тебя зовут?",
    "आपका नाम क्या है?",
    "你叫什么名字?",
];

fn cached_response(
    cell: &'static OnceLock<String>,
    language: &str,
    fallback: &str,
) -> &'static str {
    cell.get_or_init(|| {
        seed::response_for("assistant_name", language).unwrap_or_else(|| String::from(fallback))
    })
    .as_str()
}

pub fn assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "en", FALLBACK_ASSISTANT_NAME_ANSWER)
}

pub fn russian_assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "ru", FALLBACK_ASSISTANT_NAME_ANSWER_RU)
}

pub fn hindi_assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "hi", FALLBACK_ASSISTANT_NAME_ANSWER_HI)
}

pub fn chinese_assistant_name_answer() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    cached_response(&CELL, "zh", FALLBACK_ASSISTANT_NAME_ANSWER_ZH)
}
