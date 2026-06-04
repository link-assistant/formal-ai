use super::*;

#[test]
fn detects_each_source_language() {
    assert_eq!(
        detect_source_language("translate apple from english"),
        Some("en")
    );
    assert_eq!(detect_source_language("apple с русского"), Some("ru"));
    assert_eq!(detect_source_language("apple हिंदी से"), Some("hi"));
    assert_eq!(detect_source_language("从中文翻译 apple"), Some("zh"));
    assert_eq!(detect_source_language("what is apple"), None);
}

#[test]
fn detects_each_target_language() {
    assert_eq!(
        detect_target_language("translate apple to english"),
        Some("en")
    );
    assert_eq!(detect_target_language("apple на русский"), Some("ru"));
    assert_eq!(detect_target_language("apple हिंदी में"), Some("hi"));
    assert_eq!(detect_target_language("apple 成中文"), Some("zh"));
    assert_eq!(detect_target_language("what is apple"), None);
}

#[test]
fn combined_prompt_reads_both_directions() {
    let normalized = "translate apple from english to russian";
    assert_eq!(detect_source_language(normalized), Some("en"));
    assert_eq!(detect_target_language(normalized), Some("ru"));
}

#[test]
fn source_language_names_do_not_leak_into_target() {
    // "english to russian" must not register English as the *target*:
    // "to english" is not a substring here.
    assert_eq!(
        detect_target_language("translate apple from english to russian"),
        Some("ru")
    );
}
