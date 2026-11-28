// Encode contributor-controlled human diagnostics, keeping ordinary output exact.
// A JSON string with escaped command delimiters cannot be an Actions command.
pub fn safe_log_text(value: &str) -> String {
    if !value.contains("::") && !value.contains("##[") && !value.chars().any(char::is_control) {
        return value.to_owned();
    }
    use std::fmt::Write as _;
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0c}' => result.push_str("\\f"),
            ':' => result.push_str("\\u003a"),
            '#' => result.push_str("\\u0023"),
            character if character.is_control() => {
                write!(&mut result, "\\u{:04x}", u32::from(character)).expect("write to String");
            }
            character => result.push(character),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::safe_log_text;

    #[test]
    fn ordinary_diagnostics_stay_exact() {
        for value in ["normal/path.rs", "ordinary Unicode é 🙂", "count: 3", "a#b"] {
            assert_eq!(safe_log_text(value), value);
        }
    }

    #[test]
    fn both_command_protocols_and_line_breaks_are_encoded() {
        assert_eq!(
            safe_log_text("::error::fake"),
            r#""\u003a\u003aerror\u003a\u003afake""#
        );
        assert_eq!(
            safe_log_text("prefix##[add-mask]fake"),
            r#""prefix\u0023\u0023[add-mask]fake""#
        );
        assert_eq!(
            safe_log_text("a\n::warning::fake"),
            r#""a\n\u003a\u003awarning\u003a\u003afake""#
        );
        assert_eq!(
            safe_log_text("quote\"\\##[error]é🙂"),
            r#""quote\"\\\u0023\u0023[error]é🙂""#
        );
        assert_eq!(
            safe_log_text("\u{0}\u{08}\u{0c}\u{7f}\u{80}\u{9f}"),
            r#""\u0000\b\f\u007f\u0080\u009f""#
        );
    }
}
