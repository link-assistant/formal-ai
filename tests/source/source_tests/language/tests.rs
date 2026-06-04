use super::{detect, Language};

#[test]
fn latin_text_is_english() {
    assert_eq!(detect("Hello"), Language::English);
}

#[test]
fn cyrillic_text_is_russian() {
    assert_eq!(detect("Привет"), Language::Russian);
}

#[test]
fn devanagari_text_is_hindi() {
    assert_eq!(detect("नमस्ते"), Language::Hindi);
}

#[test]
fn cjk_text_is_chinese() {
    assert_eq!(detect("你好"), Language::Chinese);
}

#[test]
fn arabic_text_is_unknown() {
    assert_eq!(detect("لطفاً سلام بگو"), Language::Unknown);
}

#[test]
fn empty_prompt_defaults_to_english() {
    assert_eq!(detect(""), Language::English);
}
