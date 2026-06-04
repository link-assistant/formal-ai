use super::*;

#[test]
fn preserves_lowercase_and_question_mark() {
    assert_eq!(
        match_source_formatting("How are you?", "как у тебя дела?"),
        "how are you?",
    );
}

#[test]
fn preserves_uppercase_and_question_mark() {
    assert_eq!(
        match_source_formatting("How are you?", "Как у тебя дела?"),
        "How are you?",
    );
}

#[test]
fn drops_terminal_when_source_has_none() {
    assert_eq!(
        match_source_formatting("How are you?", "как дела"),
        "how are you",
    );
}

#[test]
fn keeps_target_when_source_has_no_letters() {
    assert_eq!(match_source_formatting("Hello", "..."), "Hello.");
}

#[test]
fn handles_empty_source() {
    assert_eq!(match_source_formatting("Hello", ""), "Hello");
}

#[test]
fn mirrors_chinese_terminal_punctuation() {
    assert_eq!(match_source_formatting("你好", "Hello?"), "你好?");
}
