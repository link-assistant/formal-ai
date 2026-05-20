// Multilingual coverage matrix (PR #134 feedback 4489651616: "all test cases
// are properly translated in all supported languages, and have lots of
// variations (at least 5-10 per language), so we have 100% tests coverage").
//
// Each block exercises the deterministic intent router with 8-10 distinct
// prompt variations per language so changes to the routing tables can't
// silently regress a single language. The cases intentionally mix:
//   - canonical greetings ("hello", "привет", "你好", "नमस्ते")
//   - punctuated/casing variants ("Hello!", "ПРИВЕТ", "你好.")
//   - polite forms ("good morning", "здравствуйте", "您好")
//   - regional/script variants where the seed routes have them
//
// `FormalAiEngine::answer` runs the full pipeline including
// `normalize_prompt` (which now lives in Rust + WASM, R194). A failure here
// catches drift between the JS worker and the Rust engine — e.g. a
// punctuation rule that collapses differently in one path.

use formal_ai::FormalAiEngine;

fn assert_intent(prompts: &[&str], expected_intent: &str, response_link: &str) {
    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, expected_intent,
            "prompt {prompt:?} should resolve to intent {expected_intent:?}, got {:?}",
            response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == response_link),
            "prompt {prompt:?} should cite evidence link {response_link:?}; got {:?}",
            response.evidence_links
        );
    }
}

// -----------------------------------------------------------------------------
// Greetings — 9 variants per language.
// -----------------------------------------------------------------------------

#[test]
fn greeting_english_variations_match() {
    let prompts = [
        "hi", "Hi!", "Hello", "hello,", "Hello!", "hey", "HEY", "Hey!", "HELLO",
    ];
    assert_intent(&prompts, "greeting", "response:greeting");
}

#[test]
fn greeting_russian_variations_match() {
    let prompts = [
        "привет",
        "Привет!",
        "ПРИВЕТ",
        "Привет.",
        "привет,",
        "Здравствуйте",
        "здравствуйте!",
        "шалом",
        "Шабат шалом!",
    ];
    assert_intent(&prompts, "greeting", "response:greeting");
}

#[test]
fn greeting_hindi_variations_match() {
    let prompts = [
        "नमस्ते",
        "नमस्ते!",
        "नमस्कार",
        "नमस्कार।",
        "हाय",
        "हाय!",
        "सलाम",
        "सलाम!",
        "राम राम",
    ];
    assert_intent(&prompts, "greeting", "response:greeting");
}

#[test]
fn greeting_chinese_variations_match() {
    let prompts = [
        "你好",
        "你好！",
        "您好",
        "您好！",
        "嗨",
        "嗨！",
        "哈喽",
        "早上好",
        "早安",
    ];
    assert_intent(&prompts, "greeting", "response:greeting");
}

// Issue #152 follow-up: "how are you?" small talk is greeting intent, not a
// language-specific one-off. Keep the semantic family covered in every
// language declared by `agent_info.supported_languages`.
#[test]
fn greeting_how_are_you_variations_match_across_languages() {
    let prompts = [
        "How are you?",
        "how are you doing?",
        "How do you do?",
        "Как твои дела?",
        "как дела",
        "как у вас дела?",
        "आप कैसे हैं?",
        "तुम कैसे हो?",
        "क्या हाल है?",
        "你好吗?",
        "你怎么样?",
        "最近怎么样?",
    ];
    assert_intent(&prompts, "greeting", "response:greeting");
}

// -----------------------------------------------------------------------------
// Farewells — 8-9 variants per language.
// -----------------------------------------------------------------------------

#[test]
fn farewell_english_variations_match() {
    let prompts = [
        "bye", "bye!", "Bye.", "Goodbye", "goodbye!", "Goodbye.", "ciao", "ciao!",
    ];
    assert_intent(&prompts, "farewell", "response:farewell");
}

#[test]
fn farewell_russian_variations_match() {
    let prompts = [
        "пока",
        "Пока!",
        "ПОКА",
        "пока.",
        "до свидания",
        "До свидания!",
        "до свидания.",
        "досвидания",
    ];
    assert_intent(&prompts, "farewell", "response:farewell");
}

#[test]
fn farewell_hindi_variations_match() {
    let prompts = [
        "अलविदा",
        "अलविदा!",
        "विदा",
        "विदा.",
        "बाय",
        "बाय!",
        "टाटा",
        "फिर मिलेंगे",
    ];
    assert_intent(&prompts, "farewell", "response:farewell");
}

#[test]
fn farewell_chinese_variations_match() {
    let prompts = [
        "再见",
        "再见！",
        "拜拜",
        "拜拜！",
        "回见",
        "改天见",
        "后会有期",
        "再见.",
    ];
    assert_intent(&prompts, "farewell", "response:farewell");
}

// -----------------------------------------------------------------------------
// Identity questions — 8-9 variants per language.
// -----------------------------------------------------------------------------

#[test]
fn identity_english_variations_match() {
    let prompts = [
        "who are you",
        "Who are you?",
        "what are you",
        "What are you?",
        "tell me about yourself",
        "Tell me about yourself.",
        "introduce yourself",
        "Introduce yourself!",
        "what is formal-ai?",
    ];
    assert_intent(&prompts, "identity", "response:identity");
}

#[test]
fn identity_russian_variations_match() {
    let prompts = [
        "кто ты",
        "Кто ты?",
        "что ты",
        "Что ты?",
        "кто ты такой",
        "кто ты такая",
        "кто ты, ассистент?",
        "что ты, formal-ai?",
    ];
    assert_intent(&prompts, "identity", "response:identity");
}

// The "X कौन है" / "X कौन हैं" suffix is intercepted by `try_who_is_question`
// before intent routing fires (see `src/solver_handlers/user_intent.rs`). The
// Hindi identity routing therefore relies on `तुम कौन हो` (different copula),
// `अपना परिचय दो`, and `अपने बारे में बताओ` to reach the `identity` intent.
#[test]
fn identity_hindi_variations_match() {
    let prompts = [
        "तुम कौन हो",
        "तुम कौन हो?",
        "तुम कौन हो!",
        "तुम कौन हो।",
        "अपना परिचय दो",
        "अपना परिचय दो।",
        "अपने बारे में बताओ",
        "अपने बारे में बताओ।",
    ];
    assert_intent(&prompts, "identity", "response:identity");
}

// The "X 是谁" suffix is intercepted by `try_who_is_question` before intent
// routing, so Chinese identity coverage uses "你是什么"/"您是什么" (different
// interrogative), plus the `介绍一下…` / `告诉我…` self-introduction phrases.
#[test]
fn identity_chinese_variations_match() {
    let prompts = [
        "你是什么",
        "你是什么？",
        "你是什么.",
        "你是什么!",
        "介绍一下你自己",
        "介绍一下你自己。",
        "告诉我你自己",
        "告诉我你自己。",
    ];
    assert_intent(&prompts, "identity", "response:identity");
}

// -----------------------------------------------------------------------------
// Arithmetic — 8 variants per language, exercising the shared Rust evaluator
// that the JS worker now delegates to via WASM (R194).
// -----------------------------------------------------------------------------

fn assert_calculation(prompts_and_results: &[(&str, &str)]) {
    for (prompt, expected_value) in prompts_and_results {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "calculation",
            "prompt {prompt:?} should resolve to calculation, got {:?}",
            response.intent
        );
        assert!(
            response.answer.contains(expected_value),
            "prompt {prompt:?} answer {:?} should contain {expected_value:?}",
            response.answer
        );
    }
}

#[test]
fn arithmetic_english_word_variations_match() {
    let cases = [
        ("two plus two", "4"),
        ("three plus five", "8"),
        ("seven minus four", "3"),
        ("six times seven", "42"),
        ("nine multiplied by nine", "81"),
        ("ten divided by two", "5"),
        ("eight modulo three", "2"),
        ("one plus two plus three", "6"),
    ];
    assert_calculation(&cases);
}

#[test]
fn arithmetic_russian_word_variations_match() {
    let cases = [
        ("два плюс два", "4"),
        ("три плюс пять", "8"),
        ("семь минус четыре", "3"),
        ("шесть умножить на семь", "42"),
        ("девять умножить на девять", "81"),
        ("десять разделить на два", "5"),
        ("один плюс два плюс три", "6"),
        ("пять минус один", "4"),
    ];
    assert_calculation(&cases);
}

#[test]
fn arithmetic_symbolic_variations_match() {
    let cases = [
        ("2 + 2", "4"),
        ("3 + 5", "8"),
        ("7 - 4", "3"),
        ("6 * 7", "42"),
        ("9 * 9", "81"),
        ("10 / 2", "5"),
        ("8 % 3", "2"),
        ("(1 + 2) * 3", "9"),
    ];
    assert_calculation(&cases);
}
