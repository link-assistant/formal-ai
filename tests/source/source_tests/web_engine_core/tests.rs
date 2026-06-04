use super::*;
use alloc::vec;

#[test]
fn normalize_collapses_punctuation_to_single_space() {
    assert_eq!(normalize_prompt("Hello,  world!"), "hello world");
    assert_eq!(normalize_prompt("  what's 2+2?"), "what s 2 2");
}

#[test]
fn normalize_keeps_cjk_codepoints() {
    let out = normalize_prompt("你好，世界！");
    assert!(out.contains('你'));
    assert!(out.contains('好'));
    assert!(out.contains('世'));
    assert!(out.contains('界'));
}

#[test]
fn normalize_handles_devanagari() {
    let out = normalize_prompt("नमस्ते, दुनिया!");
    assert!(out.contains('न'));
    assert!(out.contains('द'));
    assert!(!out.contains(','));
}

#[test]
fn normalize_lowercases_cyrillic() {
    // `char::to_lowercase` handles Cyrillic correctly.
    let out = normalize_prompt("ПРИВЕТ, МИР!");
    assert!(out.contains("привет"));
    assert!(out.contains("мир"));
}

#[test]
fn tokenize_returns_individual_words() {
    assert_eq!(
        tokenize_prompt("  Hello,  world  again!"),
        vec![
            "hello".to_string(),
            "world".to_string(),
            "again".to_string()
        ],
    );
}

#[test]
fn detect_language_matches_existing_rules() {
    assert_eq!(detect_language("Hello"), Language::English);
    assert_eq!(detect_language("Привет"), Language::Russian);
    assert_eq!(detect_language("नमस्ते"), Language::Hindi);
    assert_eq!(detect_language("你好"), Language::Chinese);
}

#[test]
fn evaluate_arithmetic_handles_word_operators() {
    assert_eq!(
        evaluate_arithmetic_expression("two plus two"),
        Ok("4".to_string())
    );
    assert_eq!(
        evaluate_arithmetic_expression("3 multiplied by 4"),
        Ok("12".to_string())
    );
}

#[test]
fn evaluate_arithmetic_handles_percent_of_word_problems() {
    // Issue #334 step 2: the WASM worker must evaluate the reduced
    // "55 * 8% of 500" word problem to 2200 (8% of 500 = 40, 55 * 40).
    assert_eq!(
        evaluate_arithmetic_expression("55 * 8% of 500"),
        Ok("2200".to_string())
    );
    assert_eq!(
        evaluate_arithmetic_expression("8% of 500"),
        Ok("40".to_string())
    );
    // A bare `%` not followed by `of` still means modulo.
    assert_eq!(
        evaluate_arithmetic_expression("10 % 3"),
        Ok("1".to_string())
    );
}

#[test]
fn evaluate_arithmetic_returns_localizable_errors() {
    assert!(evaluate_arithmetic_expression("1 / 0").is_err());
    assert!(evaluate_arithmetic_expression("").is_err());
}

#[test]
fn stable_id_hashes_utf8_bytes_for_non_ascii_prompts() {
    assert_eq!(
        stable_id("unknown_opener", "неведомослово"),
        "unknown_opener_3f0af77ee5085861"
    );
}

#[test]
fn unknown_opener_selection_matches_native_solver_for_russian() {
    assert_eq!(
        select_unknown_opener("неведомослово", "ru"),
        "Я ещё не научился отвечать на это."
    );
}

#[test]
fn route_parts_match_keywords_tokens_and_combos() {
    let keywords = vec!["hello".to_string()];
    let phrases = vec!["what s your name".to_string()];
    let tokens = vec!["greet".to_string()];
    let combos = vec![vec!["who".to_string(), "you".to_string()]];

    assert!(matches_intent_route_parts(
        "hello", "hello", &keywords, &phrases, &tokens, &combos
    ));
    assert!(matches_intent_route_parts(
        "please greet",
        "please greet",
        &keywords,
        &phrases,
        &tokens,
        &combos
    ));
    assert!(matches_intent_route_parts(
        "who are you",
        "who are you",
        &keywords,
        &phrases,
        &tokens,
        &combos
    ));
    assert!(!matches_intent_route_parts(
        "world", "world", &keywords, &phrases, &tokens, &combos
    ));
}

#[test]
fn route_payload_parser_preserves_raw_phrase_compatibility() {
    let payload = "what s your name\nWhat's your name?\nP\twhat's your name";
    assert!(matches_intent_route_payload(payload));
}
