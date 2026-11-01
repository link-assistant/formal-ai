//! Shared finite file-command grammar. Native execution is qualified only in CI.
use crate::normal_markov::quoted_segment_spans;
use crate::portable_string_contracts::quote_shell_word;
use serde_json::{Map, Value, json};
use std::sync::OnceLock;

fn grammar() -> &'static Value {
    static GRAMMAR: OnceLock<Value> = OnceLock::new();
    GRAMMAR.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../embedded/data/meta/file-command-grammar.json"
        ))
        .expect("StaticFileCommandGrammar")
    })
}
fn expression(text: &str) -> regex::Regex {
    regex::RegexBuilder::new(text)
        .case_insensitive(true)
        .build()
        .expect("StaticFileCommandExpression")
}
fn unquote(text: &str) -> &str {
    if text.len() >= 2
        && matches!(text.as_bytes()[0], b'\'' | b'"' | b'`')
        && text.as_bytes().last() == text.as_bytes().first()
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}
fn path_valid(path: &str) -> bool {
    static PATH: OnceLock<regex::Regex> = OnceLock::new();
    !path.is_empty()
        && path.len() <= 4096
        && PATH
            .get_or_init(|| {
                regex::Regex::new(r"^[A-Za-z0-9_.][A-Za-z0-9_./ -]*$")
                    .expect("StaticFileCommandPath")
            })
            .is_match(path)
        && path.split('/').all(|part| !matches!(part, "" | ".."))
}
fn shell_control(text: &str) -> bool {
    let mut quoted = None;
    let mut escaped = false;
    for (index, character) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted == Some('\'') {
            if character == '\'' {
                quoted = None;
            }
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if Some(character) == quoted {
            quoted = None;
            continue;
        }
        if matches!(character, '\'' | '"') && quoted.is_none() {
            quoted = Some(character);
            continue;
        }
        if text[index..].starts_with(
            grammar()["substitutionOpening"]
                .as_str()
                .expect("StaticSubstitutionOpening"),
        ) || grammar()["controlCharacters"]
            .as_array()
            .expect("StaticShellControls")
            .iter()
            .any(|value| {
                value
                    .as_str()
                    .is_some_and(|value| value.chars().eq(std::iter::once(character)))
            })
        {
            return true;
        }
    }
    quoted.is_some() || escaped
}
fn one_edit(left: &str, right: &str) -> bool {
    if left.len().abs_diff(right.len()) > 1 || left == right {
        return false;
    }
    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut first = 0;
    let mut second = 0;
    let mut edits = 0;
    while first < left.len() && second < right.len() {
        if left[first] == right[second] {
            first += 1;
            second += 1;
            continue;
        }
        edits += 1;
        if edits > 1 {
            return false;
        }
        if left.len() >= right.len() {
            first += 1;
        }
        if right.len() >= left.len() {
            second += 1;
        }
    }
    edits + (left.len() - first) + (right.len() - second) == 1
}
pub(super) fn normalize_listing(task: &str) -> (String, Vec<Value>) {
    if task.len() > 8192 {
        return (task.to_owned(), Vec::new());
    }
    static FIRST: OnceLock<regex::Regex> = OnceLock::new();
    let first = FIRST
        .get_or_init(|| {
            regex::Regex::new(r"^\s*(?:please\s+)?([A-Za-z]+)").expect("StaticListingLead")
        })
        .captures(task)
        .and_then(|found| found.get(1))
        .map(|word| word.as_str().to_lowercase());
    if !grammar()["listingVerbs"]
        .as_array()
        .expect("StaticListingVerbs")
        .iter()
        .any(|word| word.as_str() == first.as_deref())
    {
        return (task.to_owned(), Vec::new());
    }
    static ADJECTIVE: OnceLock<regex::Regex> = OnceLock::new();
    let pattern = ADJECTIVE.get_or_init(|| {
        expression(
            grammar()["adjectiveExpression"]
                .as_str()
                .expect("StaticAdjectiveExpression"),
        )
    });
    let quoted = quoted_segment_spans(task);
    let mut replacements = Vec::new();
    let mut normalizations = Vec::new();
    for found in pattern.captures_iter(task) {
        let Some(word) = found.name("adjective") else {
            continue;
        };
        let start = word.start();
        let end = word.end();
        let lower = word.as_str().to_lowercase();
        let adjectives = grammar()["listingAdjectives"]
            .as_array()
            .expect("StaticListingAdjectives");
        if quoted
            .iter()
            .any(|span| start < span.end && end > span.start)
            || adjectives
                .iter()
                .any(|value| value.as_str() == Some(lower.as_str()))
        {
            continue;
        }
        let candidates = adjectives
            .iter()
            .filter_map(Value::as_str)
            .filter(|candidate| one_edit(&lower, candidate))
            .collect::<Vec<_>>();
        if candidates.len() == 1 {
            normalizations.push(json!({"span":[task[..start].encode_utf16().count(),task[..end].encode_utf16().count()],"observed":word.as_str(),"canonical":candidates[0],"basis":"unique-one-edit-adjective-before-container"}));
            replacements.push((start, end, candidates[0].to_owned()));
        }
    }
    let mut text = task.to_owned();
    for (start, end, canonical) in replacements.into_iter().rev() {
        text.replace_range(start..end, &canonical);
    }
    (text, normalizations)
}
pub(super) fn file_command_contract(task: &str) -> Option<Value> {
    if task.len() > 8192 {
        return None;
    }
    let base = || json!({"schema":"file-command-contract/v1","task":task,"verified":false,"complete":false});
    let raw = unquote(task.trim());
    let first = raw
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || matches!(*character, '_' | '-')
        })
        .collect::<String>()
        .to_lowercase();
    if crate::seed::terminal_command_vocabulary()
        .bare_shell_tokens
        .contains(&first)
        && shell_control(raw)
    {
        let mut result = base();
        result["operation"] = json!("unsupported");
        result["reason"] = json!("CompoundShellInputUnsupported");
        return Some(result);
    }
    static PRODUCTIONS: OnceLock<Vec<(String, Option<String>, regex::Regex)>> = OnceLock::new();
    let productions = PRODUCTIONS.get_or_init(|| {
        grammar()["patterns"]
            .as_array()
            .expect("StaticFileCommandProductions")
            .iter()
            .map(|row| {
                (
                    row["operation"]
                        .as_str()
                        .expect("StaticFileCommandOperation")
                        .to_owned(),
                    row["reason"].as_str().map(str::to_owned),
                    expression(
                        row["expression"]
                            .as_str()
                            .expect("StaticFileCommandProduction"),
                    ),
                )
            })
            .collect()
    });
    let (text, normalizations) = normalize_listing(task);
    for (operation, reason, pattern) in productions {
        let Some(found) = pattern.captures(&text) else {
            continue;
        };
        let mut slots = Map::new();
        for name in pattern.capture_names().flatten() {
            if let Some(value) = found.name(name) {
                slots.insert(name.to_owned(), json!(value.as_str()));
            }
        }
        let path = unquote(found.name("path").map_or(".", |value| value.as_str())).to_owned();
        slots.insert("path".to_owned(), json!(path));
        let mut result = base();
        result["operation"] = json!(operation);
        result["slots"] = Value::Object(slots);
        result["normalizations"] = json!(normalizations);
        let unsupported = |mut result: Value, reason: &str| {
            result["operation"] = json!("unsupported");
            result["reason"] = json!(reason);
            Some(result)
        };
        if operation == "unsupported" {
            return unsupported(result, reason.as_deref().unwrap_or("UnresolvedFileCommand"));
        }
        if !path_valid(&path) {
            return unsupported(result, "FileCommandPathUnsupported");
        }
        let arguments: Vec<String> = match operation.as_str() {
            "listing" => vec!["ls".to_owned()],
            "head" => {
                let count = found
                    .name("count")
                    .and_then(|value| value.as_str().parse::<usize>().ok())
                    .filter(|value| (1..=1000).contains(value));
                let Some(count) = count else {
                    return unsupported(result, "FileCommandLineLimit");
                };
                vec![
                    "head".to_owned(),
                    "-n".to_owned(),
                    count.to_string(),
                    "--".to_owned(),
                    path,
                ]
            }
            "search" => {
                let query = unquote(found.name("query")?.as_str());
                if query.is_empty() || query.len() > 512 || query.contains(['\0', '\r', '\n']) {
                    return unsupported(result, "FileCommandQueryUnsupported");
                }
                vec![
                    "grep".to_owned(),
                    "-rFn".to_owned(),
                    "--".to_owned(),
                    query.to_owned(),
                    path,
                ]
            }
            "count" => vec![
                "node".to_owned(),
                "-e".to_owned(),
                include_str!("../../embedded/data/meta/file-count-program.cjs").to_owned(),
                "--".to_owned(),
                path,
            ],
            _ => return unsupported(result, "FileCommandOperationUnsupported"),
        };
        let command = std::iter::once(arguments[0].clone())
            .chain(
                arguments
                    .iter()
                    .skip(1)
                    .map(|value| quote_shell_word(value)),
            )
            .collect::<Vec<_>>()
            .join(" ");
        result["arguments"] = json!(arguments);
        result["command"] = json!(command);
        return Some(result);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_operation_arguments_and_refusals_match() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/file-command-cases.json"))
                .expect("FileCommandFixtures");
        for case in fixture["cases"].as_array().expect("FileCommandCases") {
            let result = file_command_contract(case["task"].as_str().expect("FileCommandTask"));
            if case["unclaimed"] == true {
                assert!(result.is_none(), "{case}");
                continue;
            }
            let result = result.expect("FileCommandContract");
            assert_eq!(result["operation"], case["operation"], "{case}");
            if !case["reason"].is_null() {
                assert_eq!(result["reason"], case["reason"]);
            }
            if !case["arguments"].is_null() {
                assert_eq!(result["arguments"], case["arguments"]);
            }
            if let Some(path) = case["countPath"].as_str() {
                let arguments = result["arguments"].as_array().expect("FileCountArguments");
                assert_eq!(arguments[0], "node");
                assert_eq!(arguments[1], "-e");
                assert_eq!(
                    arguments[2],
                    include_str!("../../embedded/data/meta/file-count-program.cjs")
                );
                assert_eq!(arguments[4], path);
            }
        }
    }
}
