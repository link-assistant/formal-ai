use super::*;

pub(super) fn build_dialogue_fact_check(
    turns: &[String],
    templates: FactCheckTemplates<'_>,
) -> String {
    let mut statements = Vec::new();
    for text in turns
        .iter()
        .map(|turn| turn.trim())
        .filter(|turn| !turn.is_empty())
    {
        if !looks_declarative(text) {
            continue;
        }
        let id = stable_id("world_statement", text);
        if statements
            .iter()
            .any(|item: &DialogueStatement| item.id == id)
        {
            continue;
        }
        statements.push(assess_dialogue_statement(id, text, &templates));
    }
    statements.sort_by(|left, right| left.id.cmp(&right.id));

    let summary = if statements.is_empty() {
        templates.no_statements.to_string()
    } else {
        statements
            .iter()
            .map(|statement| render_dialogue_statement(statement, &templates))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let content = fill_template(
        templates.audit,
        &[
            ("count", statements.len().to_string()),
            ("formal_system", String::from("current")),
            ("summary", summary),
        ],
    );
    serialize_dialogue_answer(&content, &statements)
}

pub(super) fn serialize_dialogue_answer(content: &str, statements: &[DialogueStatement]) -> String {
    let mut output = String::from("{\"intent\":\"fact_check_current_dialogue\",\"content\":");
    push_json_string(&mut output, content);
    output.push_str(",\"confidence\":1,\"evidence\":[");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        let mut evidence = format!("fact_check:statement:{}", statement.id);
        evidence.push(' ');
        evidence.push_str(statement.probability);
        evidence.push(' ');
        evidence.push_str(statement.basis);
        push_json_string(&mut output, &evidence);
    }
    if !statements.is_empty() {
        output.push(',');
    }
    for (index, evidence) in [
        format!(
            "fact_check:audit:scope=current_dialogue;statements={}",
            statements.len()
        ),
        String::from("fact_check:scope:current_dialogue"),
        format!(
            "fact_check:formal_system:{}",
            stable_id("formal_system", "name:current;universe:;interpretation:;")
        ),
    ]
    .iter()
    .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        push_json_string(&mut output, evidence);
    }
    output.push_str("],\"trace\":[");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(
            &mut output,
            &format!(
                "fact_check:refutation:{}:disprove_statement:{}",
                statement.id, statement.outcome
            ),
        );
    }
    output.push_str("]}");
    output
}
