use super::super::GeneratedSource;

pub(in crate::agentic_coding) fn source_registration_contract(
    task: &str,
    source_for: fn(&str) -> Option<GeneratedSource>,
) -> Option<serde_json::Value> {
    let action =
        super::role_alternatives("coding_module_registration_action", crate::seed::Slot::Bare)?;
    let slots = [
        ("source-clause", ".+?"),
        ("registration-action", action.as_str()),
        ("registration-path", r"[^ \t\n\r\x0b\x0c,;:]+\.rs"),
    ];
    let parsed = crate::seed::parser::parse_lino(include_str!(
        "../../../../embedded/data/seed/source-authoring-grammar.lino"
    ));
    let root = parsed.children.first()?;
    let placeholders = regex::Regex::new(r"\{([a-z]+(?:-[a-z]+)*)\}").ok()?;
    for form in root
        .children
        .iter()
        .filter(|node| node.name == "composition")
    {
        let Some(template) = form.children.iter().find(|node| node.name == "pattern") else {
            continue;
        };
        if !template.id.starts_with('^') || !template.id.ends_with('$') {
            continue;
        }
        let mut seen = Vec::new();
        let mut missing = false;
        let prefix = &template.id[..template.id.len() - 1];
        let pattern = placeholders.replace_all(prefix, |captures: &regex::Captures<'_>| {
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
            format!("({value})")
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
        let capture = |role: &str| {
            seen.iter()
                .position(|value| value == role)
                .and_then(|index| matched.get(index + 1))
        };
        let declaration = capture("source-clause")?;
        let registration = capture("registration-path")?;
        let Some(artifact) = source_for(declaration.as_str()) else {
            continue;
        };
        let Some(module) = artifact
            .path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".rs"))
        else {
            continue;
        };
        if !super::super::identifier_domain::rust_identifier_is_valid(module) {
            continue;
        }
        let Some(source) = super::source_description_contract(declaration.as_str(), &artifact)
        else {
            continue;
        };
        if declaration.start() != 0
            || registration.as_str().strip_suffix(".rs").is_none()
            || !crate::agentic_coding::write_request::safe_relative_path(registration.as_str())
            || registration.as_str() == artifact.path
        {
            continue;
        }
        let action = capture("registration-action")?;
        let end = matched.get(0)?.end();
        return Some(serde_json::json!({
            "unit":"utf8", "full":[0,end], "wholeRequestConsumed":end==task.len(),
            "declaration":source,
            "registration":{
                "action":{"text":action.as_str(),"span":[action.start(),action.end()]},
                "target":{"text":registration.as_str(),"span":[registration.start(),registration.end()]},
                "module":module,"span":[declaration.end(),end]},
            "remainingSpan":[end,task.len()],"source":task,"compilation":"pending",
            "unknownEffects":["module_initialization","tool_execution"],
            "condition":"both-source-owned-operations-and-entire-request-before-effects",
            "whitespaceProfile":"ASCII"
        }));
    }
    None
}
