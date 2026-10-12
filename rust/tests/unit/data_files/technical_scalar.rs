use regex::Regex;

pub(super) fn is_regex_template(content: &str, index: usize, keyword: &str, value: &str) -> bool {
    if declared_regex_pair(content, index, keyword, value) {
        return true;
    }
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
    let canonical_compound_prefix = content
        == include_str!("../../../../data/seed/source-authoring-grammar.lino")
        && parents.len() == 2
        && parents[0] == (0, "compound-task-grammar")
        && parents[1] == (2, "form")
        && indent == 4
        && keyword == "pattern"
        && super::strip_lino_comment(line)
            .trim()
            .strip_prefix("pattern ")
            == Some(value.trim());
    let parent = if canonical_compound_prefix {
        "compound-task-grammar"
    } else {
        parent
    };
    if parent == "compound-task-grammar" && !canonical_compound_prefix {
        return false;
    }
    if parent == "workspace-discovery-grammar" && (parents.len() != 1 || parents[0].0 != 0) {
        return false;
    }
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
    let grammar = include_str!("../../../../data/seed/workspace-discovery-grammar.lino");
    let (line_index, line) = grammar
        .lines()
        .enumerate()
        .find(|(_, line)| line.trim_start().starts_with("pattern "))
        .expect("actual workspace matcher pattern");
    let value = line.trim_start().strip_prefix("pattern ").unwrap();
    assert!(is_regex_template(grammar, line_index, "pattern", value));
    assert!(!is_regex_template(
        &grammar.replace("workspace-discovery-grammar", "ordinary-values"),
        line_index,
        "pattern",
        value,
    ));
    let nested_workspace = format!(
        "ordinary-values\n{}",
        grammar
            .lines()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(!is_regex_template(
        &nested_workspace,
        line_index + 1,
        "pattern",
        value,
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
    // Workspace discovery consumes a start-anchored prefix regex at its grammar root.
    // Quoted ordinary fields remain multi-value candidates, never exemptions.
    if keyword != "pattern"
        || !matches!(
            parent,
            "form" | "composition" | "workspace-discovery-grammar" | "compound-task-grammar"
        )
    {
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
    if !pattern.starts_with('^')
        || (!matches!(
            parent,
            "workspace-discovery-grammar" | "compound-task-grammar"
        ) && !pattern.ends_with('$'))
    {
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

fn declared_regex_pair(content: &str, index: usize, keyword: &str, value: &str) -> bool {
    let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../data/meta/technical-scalar-contracts.lino");
    let Ok(schema_text) = std::fs::read_to_string(schema_path) else {
        return false;
    };
    if links_notation::parse_lino(schema_text.trim()).is_err()
        || links_notation::parse_lino(content.trim()).is_err()
    {
        return false;
    }
    let Ok(schema) = formal_ai::seed::parse_lino_checked(&schema_text) else {
        return false;
    };
    let Ok(document) = formal_ai::seed::parse_lino_checked(content) else {
        return false;
    };
    let Some(schema_root) = schema.children.first() else {
        return false;
    };
    let Some(root) = document.children.first() else {
        return false;
    };
    if schema.children.len() != 1 || document.children.len() != 1 {
        return false;
    }
    let contracts = schema_root
        .children
        .iter()
        .filter(|node| node.name == "regex-pair")
        .collect::<Vec<_>>();
    if contracts.len() != 1 || schema_root.children.len() != 1 {
        return false;
    }
    let contract = contracts[0];
    if contract.children.iter().any(|node| {
        !node.children.is_empty()
            || !matches!(
                node.name.as_str(),
                "producer-root" | "container" | "record-kind" | "member" | "consumer"
            )
    }) {
        return false;
    }
    let mut declarations = std::collections::BTreeSet::new();
    if contract.children.iter().any(|node| {
        node.id.is_empty() || !declarations.insert((node.name.as_str(), node.id.as_str()))
    }) {
        return false;
    }
    let unique = |name: &str| {
        let nodes = contract
            .children
            .iter()
            .filter(|node| node.name == name)
            .collect::<Vec<_>>();
        (nodes.len() == 1).then(|| nodes[0].id.as_str())
    };
    let Some(producer) = unique("producer-root") else {
        return false;
    };
    let Some(container) = unique("container") else {
        return false;
    };
    let members = contract
        .children
        .iter()
        .filter(|node| node.name == "member")
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    if root.name != producer
        || members.len() != 2
        || members[0] == members[1]
        || !members.contains(&keyword)
    {
        return false;
    }
    let lines = content.lines().collect::<Vec<_>>();
    let Some(line) = lines.get(index) else {
        return false;
    };
    let indentation = |line: &str| line.bytes().take_while(|byte| *byte == b' ').count();
    let mut ancestors = Vec::<(usize, usize)>::new();
    for (position, line) in lines.iter().enumerate().take(index) {
        if super::strip_lino_comment(line).trim().is_empty() {
            continue;
        }
        let depth = indentation(line);
        while ancestors.last().is_some_and(|(level, _)| *level >= depth) {
            ancestors.pop();
        }
        ancestors.push((depth, position));
    }
    let depth = indentation(line);
    while ancestors.last().is_some_and(|(level, _)| *level >= depth) {
        ancestors.pop();
    }
    if ancestors.len() != 3 {
        return false;
    }
    let mut headers = Vec::new();
    for (_, position) in &ancestors {
        let Ok(parsed) =
            formal_ai::seed::parse_lino_checked(super::strip_lino_comment(lines[*position]).trim())
        else {
            return false;
        };
        if parsed.children.len() != 1 {
            return false;
        }
        headers.push(parsed.children[0].clone());
    }
    if headers[0].name != root.name
        || headers[1].name != container
        || !contract
            .children
            .iter()
            .any(|node| node.name == "record-kind" && node.id == headers[2].name)
    {
        return false;
    }
    let languages = root
        .children
        .iter()
        .filter(|node| node.name == container && node.id == headers[1].id)
        .collect::<Vec<_>>();
    if languages.len() != 1 {
        return false;
    }
    let records = languages[0]
        .children
        .iter()
        .filter(|node| node.name == headers[2].name && node.id == headers[2].id)
        .collect::<Vec<_>>();
    if records.len() != 1 || records[0].children.len() != 2 {
        return false;
    }
    let selected = super::strip_lino_comment(line).trim();
    let Ok(selected_tree) = formal_ai::seed::parse_lino_checked(selected) else {
        return false;
    };
    if selected_tree.children.len() != 1 || selected_tree.children[0].name != keyword {
        return false;
    }
    let placeholders = Regex::new(r"\{[a-z]+(?:-[a-z]+)*\}").expect("fixed placeholder grammar");
    for member in members {
        let fields = records[0]
            .children
            .iter()
            .filter(|node| node.name == member)
            .collect::<Vec<_>>();
        if fields.len() != 1 || !fields[0].children.is_empty() {
            return false;
        }
        let expression = &fields[0].id;
        if !expression.starts_with('^')
            || !expression.ends_with('$')
            || Regex::new(&placeholders.replace_all(expression, "(?:x)")).is_err()
        {
            return false;
        }
        let start = ancestors[2].1 + 1;
        let end = (start..lines.len())
            .find(|position| {
                !super::strip_lino_comment(lines[*position])
                    .trim()
                    .is_empty()
                    && indentation(lines[*position]) <= ancestors[2].0
            })
            .unwrap_or(lines.len());
        let quoted = lines[start..end]
            .iter()
            .filter(|source| indentation(source) == depth)
            .filter_map(|source| {
                let code = super::strip_lino_comment(source).trim();
                let (name, scalar) = code.split_once(char::is_whitespace)?;
                (name == member).then_some(scalar.trim())
            })
            .collect::<Vec<_>>();
        if quoted.len() != 1 || !quoted[0].starts_with('"') || !quoted[0].ends_with('"') {
            return false;
        }
        if member == keyword
            && (selected_tree.children[0].id != *expression || value.trim() != quoted[0])
        {
            return false;
        }
    }
    true
}

#[test]
fn declared_regex_pairs_preserve_nontechnical_pipe_rejection() {
    let valid = "source-context-grammar\n  language en\n    action-prologue issue\n      prefix \"^(a|b)$\"\n      suffix \"^$\"";
    assert!(is_regex_template(valid, 3, "prefix", "\"^(a|b)$\""));
    for invalid in [
        valid.replace("source-context-grammar", "ordinary-values"),
        valid.replace("language en", "aliases en"),
        valid.replace("action-prologue issue", "aliases issue"),
        valid.replace("suffix \"^$\"", "suffix ^$"),
        valid.replace("suffix \"^$\"", "aliases \"a|b\""),
        valid.replace("suffix \"^$\"", "prefix \"^$\""),
        valid.replace("      suffix", "        suffix"),
        valid.replace("^(a|b)$", "^(a|b$"),
        valid.replace("^(a|b)$", "a|b"),
    ] {
        assert!(!is_regex_template(&invalid, 3, "prefix", "\"^(a|b)$\""));
    }
    assert!(!is_regex_template(valid, 3, "aliases", "\"a|b\""));
    assert!(!is_regex_template(valid, 3, "prefix", "\"^(x|y)$\""));
}

#[test]
fn canonical_compound_prefix_patterns_keep_pipe_lists_and_spoofs_refused() {
    let grammar = include_str!("../../../../data/seed/source-authoring-grammar.lino");
    let (index, line) = grammar
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains("(?<minimumWords>"))
        .expect("actual compound receipt matcher");
    let value = line.trim().strip_prefix("pattern ").unwrap();
    assert!(is_regex_template(grammar, index, "pattern", value));
    assert!(!is_regex_template(grammar, index, "aliases", value));
    assert!(!is_regex_template(grammar, index, "pattern", "\"^a|b\""));
    assert!(!is_regex_template(
        &grammar.replace("compound-task-grammar", "ordinary-values"),
        index,
        "pattern",
        value,
    ));
    assert!(!is_regex_template(
        &format!(
            "ordinary-values\n{}",
            grammar
                .lines()
                .map(|line| format!("  {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        index + 1,
        "pattern",
        value,
    ));
    assert!(!is_regex_template(grammar, index + 1, "pattern", value));
    let direct_spoof = "compound-task-grammar\n  pattern \"^a|b\"\n";
    assert!(!is_regex_template(direct_spoof, 1, "pattern", "\"^a|b\""));
    let nested_spoof = "ordinary-values\n  compound-task-grammar\n    pattern \"^a|b\"\n";
    assert!(!is_regex_template(nested_spoof, 2, "pattern", "\"^a|b\""));
    let noncanonical_form = "compound-task-grammar\n  form 1\n    pattern \"^a|b\"\n";
    assert!(!is_regex_template(
        noncanonical_form,
        2,
        "pattern",
        "\"^a|b\""
    ));
}
