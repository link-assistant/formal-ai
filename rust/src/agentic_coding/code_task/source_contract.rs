mod matcher;
use super::GeneratedSource;
use crate::seed::{self, Slot};
fn alternatives(mut values: Vec<String>) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    values.sort_by_key(|value| std::cmp::Reverse(value.len()));
    Some(format!(
        "(?:{})",
        values
            .iter()
            .map(|value| regex::escape(value))
            .collect::<Vec<_>>()
            .join("|")
    ))
}
fn role_alternatives(role: &str, slot: Slot) -> Option<String> {
    alternatives(
        seed::lexicon()
            .role_word_forms(role)
            .iter()
            .filter(|form| form.slot() == slot)
            .map(|form| {
                if slot == Slot::Prefix {
                    form.before_slot()
                } else {
                    form.text.as_str()
                }
                .trim()
                .to_owned()
            })
            .filter(|value| !value.is_empty())
            .collect(),
    )
}
fn meaning_alternatives(role: &str, concept: &str) -> Option<String> {
    alternatives(
        seed::lexicon()
            .first_role_match(role, concept)?
            .words()
            .filter(|surface| !surface.contains('…'))
            .map(str::to_owned)
            .collect(),
    )
}
pub(super) fn source_whitespace_supported(text: &str) -> bool {
    text.chars().all(|character| {
        !(character.is_whitespace() || character == '\u{feff}')
            || matches!(
                character,
                ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'
            )
    })
}

pub(super) fn source_description_contract(
    task: &str,
    artifact: &GeneratedSource,
) -> Option<serde_json::Value> {
    if let Some(contract) = matcher::match_source_description(task, artifact) {
        return Some(contract);
    }
    if !source_whitespace_supported(task) {
        return None;
    }
    let declaration =
        regex::Regex::new(r"^(pub )?fn ([A-Za-z_][A-Za-z_0-9]*)\(\) -> i64 \{\n    (-?\d+)\n\}\n$")
            .ok()?
            .captures(&artifact.content)?;
    declaration[3].parse::<i64>().ok()?;
    if !super::identifier_domain::rust_identifier_is_valid(&declaration[2]) {
        return None;
    }
    let roles = [
        role_alternatives("coding_request_verb", Slot::Bare)?,
        role_alternatives("file_declared_noun", Slot::Bare)?,
        role_alternatives("file_write_content_lead", Slot::Prefix)?,
        alternatives(
            seed::intent_routing()
                .article_prefixes
                .iter()
                .map(|value| value.trim().to_owned())
                .collect(),
        )?,
        role_alternatives("coding_visibility", Slot::Bare)?,
        meaning_alternatives("program_language_alias", "rust")?,
        meaning_alternatives("program_kind", "function")?,
        role_alternatives("coding_name_slot", Slot::Prefix)?,
        role_alternatives("coding_return_action", Slot::Bare)?,
    ];
    let visibility = if declaration.get(1).is_some() {
        format!(r"({})\s+", roles[4])
    } else {
        "()".to_owned()
    };
    let pattern = format!(
        r"^\s*({})\s+(?:({})\s+)?({})\s+({})\s+(?:({})\s+)?{}({})\s+({})\s+({})\s+({})\s+({})\s+({})[\s.!?。！？।]*$",
        roles[0],
        roles[1],
        regex::escape(&artifact.path),
        roles[2],
        roles[3],
        visibility,
        roles[5],
        roles[6],
        roles[7],
        regex::escape(&declaration[2]),
        roles[8],
        regex::escape(&declaration[3])
    );
    let expression = regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
        .ok()?;
    let matched = expression.captures(task)?;
    if matched.get(3)?.as_str() != artifact.path
        || matched.get(10)?.as_str() != &declaration[2]
        || matched.get(12)?.as_str() != &declaration[3]
    {
        return None;
    }
    let labels = [
        "coding_request_verb",
        "file_declared_noun",
        "path",
        "file_write_content_lead",
        "article",
        "coding_visibility",
        "program_language_alias",
        "program_kind",
        "coding_name_slot",
        "identifier",
        "coding_return_action",
        "value",
    ];
    let captures = labels.iter().enumerate().filter_map(|(index, role)| matched.get(index + 1)
        .map(|value| serde_json::json!({"role":role,"span":[value.start(),value.end()],"text":value.as_str()}))).collect::<Vec<_>>();
    Some(
        serde_json::json!({"unit":"utf8","full":[matched.get(0)?.start(),matched.get(0)?.end()],"source":task,"captures":captures,
        "output":{"path":artifact.path,"identifier":&declaration[2],"value":&declaration[3],"content":artifact.content},
        "unknownEffects":["module_initialization","tool_execution"],"compilation":"pending","wholeRequestConsumed":true,"whitespaceProfile":"ASCII"}),
    )
}
