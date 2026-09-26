use super::*;

#[test]
fn extracts_unquoted_english_surface() {
    assert_eq!(
        extract_unquoted_translation_surface("translate apple to russian"),
        Some("apple".to_owned()),
    );
}

#[test]
fn preserves_capitalization() {
    assert_eq!(
        extract_unquoted_translation_surface("Translate Apple to Russian"),
        Some("Apple".to_owned()),
    );
}

#[test]
fn extracts_unquoted_russian_surface() {
    assert_eq!(
        extract_unquoted_translation_surface("переведи яблоко на английский"),
        Some("яблоко".to_owned()),
    );
}

#[test]
fn extracts_surface_before_postpositive_translation_frame() {
    assert_eq!(
        extract_unquoted_translation_surface(
            "любая формальная система либо неполна, либо противоречива - translate to english",
        ),
        Some("любая формальная система либо неполна, либо противоречива".to_owned()),
    );
}

#[test]
fn extracts_unquoted_hindi_surface() {
    assert_eq!(
        extract_unquoted_translation_surface("apple का हिंदी में अनुवाद करो"),
        Some("apple".to_owned()),
    );
    assert_eq!(
        extract_unquoted_translation_surface("सेब को अंग्रेजी में अनुवाद करो"),
        Some("सेब".to_owned()),
    );
}

#[test]
fn extracts_unquoted_chinese_surface() {
    assert_eq!(
        extract_unquoted_translation_surface("把 apple 翻译成中文"),
        Some("apple".to_owned()),
    );
    assert_eq!(
        extract_unquoted_translation_surface("将苹果翻译成英文"),
        Some("苹果".to_owned()),
    );
    assert_eq!(
        extract_unquoted_translation_surface("翻译 apple 成中文"),
        Some("apple".to_owned()),
    );
}

#[test]
fn ignores_trailing_punctuation() {
    assert_eq!(
        extract_unquoted_translation_surface("translate apple to russian."),
        Some("apple".to_owned()),
    );
}

#[test]
fn returns_none_for_quoted_prompts() {
    assert_eq!(
        extract_unquoted_translation_surface("translate \"apple\" to russian"),
        None,
    );
}

#[test]
fn returns_none_without_verb() {
    assert_eq!(extract_unquoted_translation_surface("what is apple"), None,);
}

#[test]
fn returns_none_without_preposition() {
    assert_eq!(
        extract_unquoted_translation_surface("translate apple"),
        None,
    );
}
