use regex::Regex;

pub(super) fn is_regex_template(content: &str, index: usize, keyword: &str, value: &str) -> bool {
    let mut parents: Vec<(usize, &str)> = Vec::new();
    for line in content.lines().take(index) {
        let code = super::strip_lino_comment(line).trim();
        if code.is_empty() {
            continue;
        }
        let indent = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        while parents.last().is_some_and(|(level, _)| *level >= indent) {
            parents.pop();
        }
        parents.push((indent, code.split_whitespace().next().unwrap_or("")));
    }
    let Some(line) = content.lines().nth(index) else {
        return false;
    };
    let indent = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    while parents.last().is_some_and(|(level, _)| *level >= indent) {
        parents.pop();
    }
    let parent = parents.last().map_or("", |(_, name)| *name);
    lino_pipe_value_is_technical_regex(keyword, value.trim(), parent)
}

#[test]
fn regex_scalar_context_uses_the_actual_parent_and_line() {
    let positive = "root\n  form 1\n    pattern \"^(?:Create|Write)$\"";
    assert!(is_regex_template(
        positive,
        2,
        "pattern",
        "\"^(?:Create|Write)$\""
    ));
    let nested = "root\n  form 1\n    aliases\n      pattern \"^(?:Create|Write)$\"";
    assert!(!is_regex_template(
        nested,
        3,
        "pattern",
        "\"^(?:Create|Write)$\""
    ));
    assert!(!is_regex_template(
        positive,
        99,
        "pattern",
        "\"^(?:Create|Write)$\""
    ));
}

fn lino_pipe_value_is_technical_regex(keyword: &str, value: &str, parent: &str) -> bool {
    // Matcher contracts consume pattern scalars under form/composition nodes.
    // Quoted ordinary fields remain multi-value candidates, never exemptions.
    if keyword != "pattern" || !matches!(parent, "form" | "composition") {
        return false;
    }
    if !value.starts_with('"') || !value.ends_with('"') {
        return false;
    }
    let parsed = formal_ai::seed::parse_lino(&format!("pattern {value}"));
    let Some(node) = parsed.children.first() else {
        return false;
    };
    if parsed.children.len() != 1 || !node.children.is_empty() {
        return false;
    }
    let pattern = node.id.as_str();
    if !pattern.starts_with('^') || !pattern.ends_with('$') {
        return false;
    }
    let placeholders = Regex::new(r"\{[a-z]+(?:-[a-z]+)*\}").expect("fixed placeholder grammar");
    let expression = placeholders.replace_all(pattern, "(?:x)");
    Regex::new(&expression).is_ok()
}

#[test]
fn pipe_value_classification_keeps_regex_and_reference_roles_distinct() {
    assert!(lino_pipe_value_is_technical_regex(
        "pattern",
        r#""^(?:Create|Write)\\s+{path}$""#,
        "form"
    ));
    assert!(lino_pipe_value_is_technical_regex(
        "pattern",
        r#""^{source-clause}(?:and|or){registration-path}$""#,
        "composition"
    ));
    for (keyword, value, parent) in [
        ("aliases", r#""a|b""#, "form"),
        ("tasks", "a|b", "form"),
        ("pattern", r#""a|b""#, "form"),
        ("pattern", r#""^(?:a|b)$""#, "aliases"),
        ("pattern", "^(?:a|b)$", "form"),
        ("pattern", r#""^(a|b$""#, "form"),
        ("pattern", r#""^(?:a|b)$" "extra""#, "form"),
    ] {
        assert!(
            !lino_pipe_value_is_technical_regex(keyword, value, parent),
            "{keyword} {value} under {parent}"
        );
    }
}
