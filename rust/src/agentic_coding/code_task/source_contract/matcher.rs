use super::super::GeneratedSource;

pub(super) fn match_source_description(
    task: &str,
    artifact: &GeneratedSource,
) -> Option<serde_json::Value> {
    if !super::source_whitespace_supported(task) {
        return None;
    }
    let (kind, declared) = super::declaration::source_declaration_slots(&artifact.content)?;
    let mut slots = declared.into_iter().collect::<Vec<_>>();
    slots.push(("path", artifact.path.clone()));
    let parsed = crate::seed::parser::parse_lino(include_str!(
        "../../../../embedded/data/seed/source-authoring-grammar.lino"
    ));
    let root = source_grammar_root(&parsed)?;
    let placeholders = regex::Regex::new(r"\{([a-z]+(?:-[a-z]+)*)\}").ok()?;
    for form in root.children.iter().filter(|node| node.name == "form") {
        let field = |name: &str| {
            form.children
                .iter()
                .find(|node| node.name == name)
                .or_else(|| root.children.iter().find(|node| node.name == name))
                .map(|node| node.id.as_str())
        };
        if field("kind") != Some(kind) {
            continue;
        }
        let (Some(pattern), Some(language)) = (field("pattern"), field("language")) else {
            continue;
        };
        let mut seen = Vec::new();
        let mut missing = false;
        let pattern = placeholders.replace_all(pattern, |captures: &regex::Captures<'_>| {
            let role = captures[1].to_owned();
            let Some((_, value)) = slots.iter().find(|(name, _)| *name == role) else {
                missing = true;
                return String::new();
            };
            if seen.contains(&role) {
                missing = true;
                return String::new();
            }
            seen.push(role);
            format!("({})", regex::escape(value))
        });
        if missing || seen.len() != slots.len() {
            continue;
        }
        let Ok(expression) = regex::RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
        else {
            continue;
        };
        let Some(matched) = expression.captures(task) else {
            continue;
        };
        if matched.get(0)?.as_str() != task {
            continue;
        }
        if seen.iter().enumerate().any(|(index, role)| {
            let expected = slots
                .iter()
                .find(|(name, _)| *name == role)
                .map(|(_, value)| value.as_str());
            matched.get(index + 1).map(|capture| capture.as_str()) != expected
        }) {
            continue;
        }
        let captures = seen.iter().enumerate().map(|(index, role)| {
            let capture = matched.get(index + 1).expect("one capture for each bound slot");
            serde_json::json!({"role":role,"span":[capture.start(),capture.end()],"text":capture.as_str()})
        }).collect::<Vec<_>>();
        let mut output = serde_json::json!({"content": artifact.content});
        for (role, value) in &slots {
            output[*role] = serde_json::json!(value);
        }
        return Some(
            serde_json::json!({"unit":"utf8","full":[0,task.len()],"language":language,"kind":kind,"captures":captures,
            "output":output,"wholeRequestConsumed":true,
            "compilation":"pending","unknownEffects":["module_initialization","tool_execution"],"whitespaceProfile":"ASCII"}),
        );
    }
    None
}

/// Select exactly one named grammar container without first-child assumptions.
pub(super) fn source_grammar_root(
    document: &crate::seed::parser::LinoNode,
) -> Option<&crate::seed::parser::LinoNode> {
    if document.name == "source-authoring-grammar" {
        return Some(document);
    }
    let mut grammar_roots = document
        .children
        .iter()
        .filter(|node| node.name == "source-authoring-grammar");
    let root = grammar_roots.next()?;
    if grammar_roots.next().is_some() {
        return None;
    }
    Some(root)
}
