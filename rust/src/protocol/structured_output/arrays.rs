//! Source evidence for string arrays in structured responses.
use super::key_name;
use crate::concepts::{ConceptQuery, lookup_concept_query};
use crate::protocol::structured_schema as schema;
use regex::Regex;
use serde_json::Value;
use std::{collections::HashMap, sync::LazyLock};

struct Grammar {
    separator: Regex,
    label: Regex,
    lead: Regex,
    tail: Regex,
    topics: Vec<Vec<String>>,
}

static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| {
    let pattern = |key| {
        let body = crate::seed::agentic_message(key, &[]);
        Regex::new(&format!("(?i){body}")).expect("structured array grammar")
    };
    Grammar {
        separator: pattern("structured-array-separator"),
        label: pattern("structured-array-label"),
        lead: pattern("structured-array-lead"),
        tail: pattern("structured-array-tail"),
        topics: crate::seed::agentic_message("structured-array-topics", &[])
            .split(';')
            .map(|group| group.split('|').map(str::to_owned).collect())
            .collect(),
    }
});
static BULLET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[-*•–—]|[0-9]+[.)])[ \t]+(.+)$").expect("structured array bullet")
});
static CLAUSE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[.!?]+[ \t]+|;").expect("structured array clause boundary"));
static NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[.\p{Lu}\p{Lt}\p{N}][\p{L}\p{M}\p{N}_.+#'-]*)(?:[ \t]+[.\p{Lu}\p{Lt}\p{N}][\p{L}\p{M}\p{N}_.+#'-]*)*$")
        .expect("structured array name")
});

fn allows_type(rule: &Value, kind: &str) -> bool {
    rule.get("type").is_some_and(|value| {
        value == kind
            || value
                .as_array()
                .is_some_and(|types| types.iter().any(|t| t == kind))
    })
}

pub(super) fn item_rule<'a>(field: &'a Value, root: &'a Value) -> Option<&'a Value> {
    if !allows_type(field, "array") {
        return None;
    }
    let item = field.get("items")?;
    let item = item
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|pointer| schema::reference(root, pointer))
        .unwrap_or(item);
    (allows_type(item, "string")
        || (item.get("type").is_none()
            && item
                .get("enum")
                .and_then(Value::as_array)
                .is_some_and(|labels| labels.iter().all(Value::is_string))))
    .then_some(item)
}

enum Body<'a> {
    Text(&'a str),
    Bullets(Vec<&'a str>),
    Literal(Vec<Value>),
}

struct Candidate<'a> {
    context: String,
    labeled: bool,
    scoped: bool,
    body: Body<'a>,
}

fn clauses(source: &str) -> Vec<Candidate<'_>> {
    let mut result: Vec<Candidate<'_>> = Vec::new();
    let mut heading = "";
    let mut in_bullets = false;
    for line in source.split(['\r', '\n']).filter(|line| !line.is_empty()) {
        if let Some(item) = BULLET
            .captures(line.trim())
            .and_then(|captures| captures.get(1))
        {
            if in_bullets
                && let Some(Candidate {
                    body: Body::Bullets(items),
                    ..
                }) = result.last_mut()
            {
                items.push(item.as_str());
            } else {
                result.push(Candidate {
                    context: heading.to_owned(),
                    labeled: !heading.is_empty(),
                    scoped: true,
                    body: Body::Bullets(vec![item.as_str()]),
                });
            }
            in_bullets = true;
            continue;
        }
        heading = "";
        in_bullets = false;
        if let Some(captures) = GRAMMAR.label.captures(line.trim())
            && let Some(value) = literal_array(captures.get(2).expect("body capture").as_str())
        {
            result.push(Candidate {
                context: captures.get(1).expect("label capture").as_str().to_owned(),
                labeled: true,
                scoped: true,
                body: Body::Literal(value),
            });
            continue;
        }
        for clause in CLAUSE
            .split(line)
            .map(str::trim)
            .filter(|clause| !clause.is_empty())
        {
            if let Some(captures) = GRAMMAR.label.captures(clause) {
                let label = captures.get(1).expect("label capture").as_str();
                let text = captures.get(2).expect("body capture").as_str();
                heading = "";
                if text.is_empty() {
                    heading = label;
                } else {
                    result.push(Candidate {
                        context: label.to_owned(),
                        labeled: true,
                        scoped: true,
                        body: Body::Text(text),
                    });
                }
            } else {
                heading = "";
                let start = GRAMMAR.lead.find(clause).map_or(0, |lead| lead.end());
                let clean = clause[start..].trim_end_matches(['.', '!', '?']);
                let end = GRAMMAR
                    .tail
                    .find(clean)
                    .map_or(clean.len(), |tail| tail.start());
                result.push(Candidate {
                    context: format!("{} {}", &clause[..start], &clean[end..]),
                    labeled: false,
                    scoped: start != 0 || end != clean.len(),
                    body: Body::Text(&clean[..end]),
                });
            }
        }
    }
    result
}

fn contains_phrase(text: &str, part: &str) -> bool {
    !part.is_empty()
        && text.match_indices(part).any(|(start, _)| {
            let end = start + part.len();
            (start == 0 || text.as_bytes()[start - 1] == b' ')
                && (end == text.len() || text.as_bytes()[end] == b' ')
        })
}

fn topical_rank(context: &str, name: &str, description: &str) -> u8 {
    let label = key_name(context);
    if label.is_empty() {
        return 0;
    }
    if label == name {
        return 4;
    }
    if contains_phrase(&label, name) {
        return 3;
    }
    if contains_phrase(description, &label) || contains_phrase(&label, description) {
        return 2;
    }
    let shared_word = label
        .split(' ')
        .any(|word| word.chars().count() >= 4 && contains_phrase(description, word));
    let shared_topic = GRAMMAR.topics.iter().any(|group| {
        group.iter().any(|word| contains_phrase(&label, word))
            && group
                .iter()
                .any(|word| contains_phrase(name, word) || contains_phrase(description, word))
    });
    u8::from(shared_word || shared_topic)
}

fn trim_item(text: &str) -> &str {
    let item = text.trim().trim_end_matches(['.', '!', '?']).trim_end();
    for (left, right) in [
        ('"', '"'),
        ('\'', '\''),
        ('«', '»'),
        ('“', '”'),
        ('\u{60}', '\u{60}'),
    ] {
        if let Some(inner) = item
            .strip_prefix(left)
            .and_then(|item| item.strip_suffix(right))
        {
            return inner.trim();
        }
    }
    item
}

fn label_key(value: &str) -> String {
    value
        .trim()
        .split([' ', '\t'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn exact_label<'a>(item: &str, labels: &[&'a str]) -> Option<&'a str> {
    if let Some(&label) = labels.iter().find(|&&label| label == item) {
        return Some(label);
    }
    let key = label_key(item);
    let mut matches = labels
        .iter()
        .copied()
        .filter(|label| label_key(label) == key);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn edge_item(text: &str, first: bool, recognized: &mut impl FnMut(&str) -> bool) -> String {
    if recognized(text) {
        return text.to_owned();
    }
    let words: Vec<_> = text
        .split([' ', '\t'])
        .filter(|part| !part.is_empty())
        .collect();
    for size in (1..words.len().min(7)).rev() {
        let part = if first {
            &words[words.len() - size..]
        } else {
            &words[..size]
        }
        .join(" ");
        if recognized(&part) {
            return part;
        }
    }
    text.to_owned()
}

fn candidate_items(
    candidate: &Candidate<'_>,
    mut recognized: impl FnMut(&str) -> bool,
) -> Option<Vec<String>> {
    let texts: Vec<&str> = match &candidate.body {
        Body::Bullets(items) => items.clone(),
        Body::Text(text) => GRAMMAR.separator.split(text).collect(),
        Body::Literal(_) => return None,
    };
    let mut items: Vec<_> = texts
        .into_iter()
        .map(trim_item)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect();
    if !matches!(candidate.body, Body::Bullets(_)) && items.len() < 2 {
        return None;
    }
    if candidate.labeled || matches!(candidate.body, Body::Bullets(_)) {
        return Some(items);
    }
    let last = items.len() - 1;
    items[0] = edge_item(&items[0], true, &mut recognized);
    items[last] = edge_item(&items[last], false, &mut recognized);
    (candidate.scoped
        || items.iter().all(|item| {
            !item.contains(char::is_whitespace) || NAME.is_match(item) || recognized(item)
        }))
    .then_some(items)
}

fn literal_array(text: &str) -> Option<Vec<Value>> {
    if !text.starts_with('[') {
        return None;
    }
    serde_json::from_str(text.trim_end().trim_end_matches(['.', '!', '?'])).ok()
}

// Absent evidence is distinct from a labeled JSON array containing zero items.
pub(super) fn evidence(
    source: &str,
    name: &str,
    description: Option<&str>,
    rule: &Value,
) -> Option<Vec<Value>> {
    let name = key_name(name);
    let description = description.map_or_else(String::new, key_name);
    let labels = rule.get("enum").and_then(Value::as_array);
    let strings: Vec<_> = labels
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut known = HashMap::new();
    let mut known_item = |item: &str| {
        if let Some(&found) = known.get(item) {
            return found;
        }
        let query = ConceptQuery {
            term: item.to_owned(),
            context: None,
            response_language: None,
        };
        let found = lookup_concept_query(&query).is_some();
        known.insert(query.term, found);
        found
    };
    let mut best = None;
    let mut answer = None;
    for candidate in clauses(source) {
        let topic = topical_rank(&candidate.context, &name, &description);
        if (candidate.labeled && topic == 0) || best.is_some_and(|(rank, _)| topic < rank) {
            continue;
        }
        let literal = match &candidate.body {
            Body::Literal(value) => Some(value.clone()),
            Body::Text(text) if candidate.labeled => literal_array(text),
            _ => None,
        };
        let value = if let Some(value) = literal {
            value
        } else {
            let Some(items) = candidate_items(&candidate, |item| {
                known_item(item) || exact_label(item, &strings).is_some()
            }) else {
                continue;
            };
            let value: Vec<_> = if labels.is_some() {
                items
                    .iter()
                    .filter_map(|item| exact_label(item, &strings))
                    .map(Value::from)
                    .collect()
            } else {
                items.into_iter().map(Value::String).collect()
            };
            if value.is_empty() {
                continue;
            }
            value
        };
        let rank = (
            topic,
            value
                .iter()
                .filter_map(Value::as_str)
                .filter(|item| known_item(item))
                .count(),
        );
        if best.is_none_or(|previous| rank > previous) {
            best = Some(rank);
            answer = Some(value);
        } else if best == Some(rank) && answer.as_ref() != Some(&value) {
            answer = None;
        }
    }
    answer
}
