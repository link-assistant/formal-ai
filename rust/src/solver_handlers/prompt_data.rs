//! Bind classification, record questions and extraction to caller-supplied data.
//! The worker implements the same operations from the same Links Notation seed.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{LazyLock, OnceLock};

use regex::Regex;

use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::{
    detect_input, forced_response_language_slug, language_name, surface_matches_language,
};
use crate::seed::parser::{LinoNode, parse_lino};
use crate::seed::{lexicon, surface_present};

const SEED: &str = include_str!("../../embedded/data/seed/prompt-data.lino");

fn rules() -> &'static [LinoNode] {
    static RULES: OnceLock<Vec<LinoNode>> = OnceLock::new();
    RULES.get_or_init(|| {
        parse_lino(SEED)
            .children
            .into_iter()
            .flat_map(|root| root.children)
            .collect()
    })
}

fn rule(kind: &str, id: &str) -> Option<&'static LinoNode> {
    rules()
        .iter()
        .find(|node| node.name == kind && node.id == id)
}

fn values<'a>(node: &'a LinoNode, name: &'a str) -> impl Iterator<Item = &'a str> {
    node.children
        .iter()
        .filter(move |child| child.name == name)
        .map(|child| child.id.as_str())
}

fn cues(id: &str) -> impl Iterator<Item = &'static str> {
    rule("cue", id)
        .into_iter()
        .flat_map(|node| values(node, "surface"))
}

fn starts(text: &str, id: &str) -> bool {
    cues(id).any(|cue| {
        text == cue
            || text
                .strip_prefix(cue)
                .is_some_and(|suffix| suffix.starts_with(' '))
    })
}

fn localized(locale: &str, key: &str) -> &'static str {
    [locale, "en"]
        .into_iter()
        .filter_map(|name| rule("locale", name))
        .map(|node| node.find_child_value(key))
        .find(|value| !value.is_empty())
        .unwrap_or("")
}

struct Reply {
    operation: &'static str,
    body: String,
    evidence: Vec<String>,
    confidence: f32,
}

fn reply(operation: &'static str, answer: &str, spans: &[&str], locale: &str) -> Reply {
    let mut body = answer.to_owned();
    let mut evidence = vec!["prompt_data:network:none".to_owned()];
    let mut seen = HashSet::new();
    for &span in spans {
        if seen.insert(span) {
            body.push('\n');
            body.push_str(&localized(locale, "evidence").replace("{span}", span));
            evidence.push(format!("prompt_data:span:{span}"));
        }
    }
    Reply {
        operation,
        body,
        evidence,
        confidence: if spans.is_empty() { 0.5 } else { 0.9 },
    }
}

fn gap(operation: &'static str, locale: &str) -> Reply {
    reply(operation, localized(locale, "gap"), &[], locale)
}

fn facet(label: &str) -> Option<&'static LinoNode> {
    rules()
        .iter()
        .find(|node| node.name == "facet" && values(node, "label").any(|surface| surface == label))
}

fn choices(text: &str, split_conjunctions: bool) -> Vec<String> {
    let mut results = Vec::new();
    for part in text.split([',', ';', '|']) {
        let mut phrase = Vec::new();
        for word in part.split_whitespace() {
            if (phrase.is_empty() || split_conjunctions)
                && cues("option_join").any(|cue| cue == word.to_lowercase())
            {
                if !phrase.is_empty() {
                    results.push(phrase.join(" "));
                    phrase.clear();
                }
            } else {
                phrase.push(word);
            }
        }
        if !phrase.is_empty() {
            results.push(phrase.join(" "));
        }
    }
    let mut labels = Vec::<String>::new();
    for result in results {
        let label = result.trim_matches(['"', '\'', '«', '»', '“', '”']);
        let key = normalize_prompt(label);
        if key.is_empty() {
            continue;
        }
        if let Some(existing) = labels
            .iter_mut()
            .find(|other| normalize_prompt(other) == key)
        {
            *existing = label.to_owned();
        } else {
            labels.push(label.to_owned());
        }
    }
    labels
}

fn classification(head: &str, payload: &str, locale: &str) -> Reply {
    static CLAUSES: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[^,]+?(?:[.!?](?:\s+|$)|,|$)").expect("valid clause pattern")
    });
    let domain = head
        .find(['[', '('])
        .and_then(|start| {
            let tail = &head[start + 1..];
            let parenthesized = head.as_bytes()[start] == b'(';
            let end = tail.find(if parenthesized { ')' } else { ']' })?;
            Some(choices(&tail[..end], parenthesized))
        })
        .unwrap_or_default();
    let unknown = localized(locale, "unknown");
    if !(2..=64).contains(&domain.len()) {
        return reply("classification", unknown, &[], locale);
    }
    let clauses: Vec<_> = CLAUSES
        .find_iter(payload)
        .map(|part| part.as_str().trim_end_matches(',').trim())
        .filter(|part| !part.is_empty())
        .map(|part| (part, normalize_prompt(part)))
        .collect();
    let sentiment = cues("sentiment").any(|cue| surface_present(&normalize_prompt(head), cue));
    let mut candidates = Vec::new();
    let mut fallback = None;
    let mut neutral = None;
    for label in &domain {
        let key = normalize_prompt(label);
        let facet = facet(&key);
        match facet.map(|node| node.id.as_str()) {
            Some("fallback") => {
                fallback.get_or_insert(label);
                continue;
            }
            Some("neutral") if sentiment => neutral = Some(label),
            _ => {}
        }
        let explicit = clauses.iter().find(|(_, part)| surface_present(part, &key));
        let hits: Vec<_> = facet
            .into_iter()
            .flat_map(|node| values(node, "term"))
            .filter(|term| clauses.iter().any(|(_, part)| surface_present(part, term)))
            .collect();
        let support = explicit.or_else(|| {
            clauses
                .iter()
                .find(|(_, part)| hits.iter().any(|term| surface_present(part, term)))
        });
        if let Some((span, _)) = support {
            let score = hits.len() + if explicit.is_some() { 2 } else { 0 };
            candidates.push((score, label, *span));
        }
    }
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.0));
    if let Some(&(_, label, span)) = candidates
        .first()
        .filter(|first| candidates.get(1).is_none_or(|next| first.0 > next.0))
    {
        return reply("classification", label, &[span], locale);
    }
    if let Some(label) = neutral.filter(|_| candidates.is_empty()) {
        return reply("classification", label, &[payload], locale);
    }
    reply(
        "classification",
        fallback.map_or(unknown, String::as_str),
        &[],
        locale,
    )
}

fn pair(text: &str) -> Option<(&str, &str)> {
    let (field, value) = text.split_once([':', '：'])?;
    let (field, value) = (field.trim(), value.trim());
    (!field.is_empty() && !value.is_empty()).then_some((field, value))
}

fn strip_list_prefix(line: &str) -> &str {
    static PREFIX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?:\d+[.)]|[-*])\s+").expect("valid list prefix"));
    let line = line.trim();
    PREFIX
        .find(line)
        .map_or(line, |prefix| &line[prefix.end()..])
}

struct RecordLink<'a> {
    subject: &'a str,
    field: &'a str,
    value: &'a str,
    span: Rc<str>,
    header: Option<&'a str>,
}

fn identity(field: &str) -> bool {
    cues("identity_field").any(|cue| normalize_prompt(field) == cue)
}

fn append_block<'a>(
    links: &mut Vec<RecordLink<'a>>,
    fields: &mut Vec<(&'a str, &'a str)>,
    rows: &mut Vec<&'a str>,
) {
    let subject = fields
        .iter()
        .find(|(field, _)| identity(field))
        .map_or("", |(_, value)| *value);
    let span: Rc<str> = rows.join("\n").into();
    links.extend(fields.drain(..).map(|(field, value)| RecordLink {
        subject,
        field,
        value,
        span: span.clone(),
        header: None,
    }));
    rows.clear();
}

fn table_cells(line: &str) -> impl Iterator<Item = &str> {
    line.trim().trim_matches('|').split('|').map(str::trim)
}

fn record_links<'a>(lines: &[&'a str]) -> Vec<RecordLink<'a>> {
    static TABLE_SEPARATOR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^:?-{3,}:?$").expect("valid table separator"));
    static FIELD_SEPARATOR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r" [-–—] |; ").expect("valid field separator"));
    let mut links = Vec::new();
    let mut fields = Vec::new();
    let mut rows = Vec::new();
    let mut input = lines.iter().copied().peekable();
    while let Some(raw) = input.next() {
        let line = raw.trim();
        if line.contains('|')
            && input.peek().is_some_and(|next| {
                next.contains('|') && table_cells(next).all(|cell| TABLE_SEPARATOR.is_match(cell))
            })
        {
            append_block(&mut links, &mut fields, &mut rows);
            let headers: Vec<_> = table_cells(line).collect();
            input.next();
            while input.peek().is_some_and(|row| row.contains('|')) {
                let row = input.next().unwrap_or_default().trim();
                let cells: Vec<_> = table_cells(row).collect();
                if cells.len() != headers.len() || cells[0].is_empty() {
                    continue;
                }
                let span: Rc<str> = row.into();
                links.extend(
                    headers
                        .iter()
                        .zip(&cells)
                        .skip(1)
                        .filter(|(field, value)| !field.is_empty() && !value.is_empty())
                        .map(|(field, value)| RecordLink {
                            subject: cells[0],
                            field,
                            value,
                            span: span.clone(),
                            header: Some(line),
                        }),
                );
            }
            continue;
        }
        if line.is_empty() {
            append_block(&mut links, &mut fields, &mut rows);
            continue;
        }
        let parts: Vec<_> = FIELD_SEPARATOR.split(strip_list_prefix(line)).collect();
        if parts.len() > 1 && !parts[0].contains([':', '：']) {
            append_block(&mut links, &mut fields, &mut rows);
            let span: Rc<str> = line.into();
            links.extend(parts.iter().skip(1).filter_map(|part| pair(part)).map(
                |(field, value)| RecordLink {
                    subject: parts[0],
                    field,
                    value,
                    span: span.clone(),
                    header: None,
                },
            ));
            continue;
        }
        let next_fields: Vec<_> = parts.iter().filter_map(|part| pair(part)).collect();
        if next_fields.iter().any(|(field, _)| identity(field)) {
            append_block(&mut links, &mut fields, &mut rows);
        }
        fields.extend(next_fields);
        rows.push(raw);
    }
    append_block(&mut links, &mut fields, &mut rows);
    links
}

fn question(line: &str) -> bool {
    line.split_once([':', '：'])
        .is_some_and(|(field, _)| cues("question").any(|cue| cue == normalize_prompt(field)))
}

fn record_answer(payload: &str, locale: &str) -> Reply {
    let lines: Vec<_> = payload.lines().collect();
    let Some(index) = lines.iter().rposition(|line| question(line)).or_else(|| {
        lines
            .last()
            .filter(|line| line.trim_end().ends_with('?'))
            .map(|_| lines.len() - 1)
    }) else {
        return gap("record_qa", locale);
    };
    let prompt = lines[index..].join(" ");
    let query = normalize_prompt(if question(lines[index]) {
        prompt
            .split_once([':', '：'])
            .map_or("", |(_, query)| query)
    } else {
        &prompt
    });
    let links = record_links(&lines[..index]);
    let field_keys: HashSet<_> = links
        .iter()
        .map(|link| normalize_prompt(link.field))
        .filter(|key| surface_present(&query, key))
        .collect();
    if field_keys.len() != 1 {
        return gap("record_qa", locale);
    }
    let matches: Vec<_> = links
        .iter()
        .filter(|link| field_keys.contains(&normalize_prompt(link.field)))
        .collect();
    let named: Vec<_> = matches
        .iter()
        .copied()
        .filter(|link| surface_present(&query, &normalize_prompt(link.subject)))
        .collect();
    let by_subject = named.is_empty()
        && starts(&query, "subject_question")
        && matches.iter().any(|link| !link.subject.is_empty());
    let selected = if matches.iter().all(|link| link.subject.is_empty()) {
        matches
    } else if by_subject {
        let explicit: Vec<_> = matches
            .iter()
            .copied()
            .filter(|link| surface_present(&query, &normalize_prompt(link.value)))
            .collect();
        if explicit.is_empty() {
            let polarity = if cues("negation").any(|cue| surface_present(&query, cue)) {
                "negative"
            } else {
                "affirmative"
            };
            matches
                .iter()
                .copied()
                .filter(|link| {
                    !link.subject.is_empty()
                        && cues(polarity).any(|cue| cue == normalize_prompt(link.value))
                })
                .collect()
        } else {
            explicit
        }
    } else {
        named
    };
    let answers: HashSet<_> = selected
        .iter()
        .map(|link| if by_subject { link.subject } else { link.value })
        .collect();
    if answers.len() != 1 {
        return gap("record_qa", locale);
    }
    let spans: Vec<_> = selected
        .iter()
        .flat_map(|link| std::iter::once(link.span.as_ref()).chain(link.header))
        .collect();
    let mut result = reply(
        "record_qa",
        answers.into_iter().next().unwrap_or_default(),
        &spans,
        locale,
    );
    result.evidence.extend(selected.iter().map(|link| {
        format!(
            "prompt_data:link:{}→{}→{}",
            link.subject, link.field, link.value
        )
    }));
    result
}

fn extraction(head: &str, payload: &str, locale: &str) -> Option<Reply> {
    let cue = cues("extraction").find(|cue| head.starts_with(cue))?;
    let request = head[cue.len()..].trim();
    let requested = choices(request.strip_prefix("the ").unwrap_or(request), true);
    let fields: Vec<_> = payload
        .lines()
        .filter_map(|line| {
            pair(strip_list_prefix(line)).map(|(field, value)| (field, value, line.trim()))
        })
        .collect();
    if fields.is_empty() && !payload.contains('\n') {
        return None;
    }
    let mut answers = Vec::new();
    let mut spans = Vec::new();
    for key in requested {
        let matches: Vec<_> = fields
            .iter()
            .filter(|(field, _, _)| normalize_prompt(field) == normalize_prompt(&key))
            .collect();
        let unique: HashSet<_> = matches.iter().map(|(_, value, _)| value).collect();
        if unique.len() == 1 {
            let (field, value, _) = matches[0];
            answers.push(format!("{field}: {value}"));
            spans.extend(matches.iter().map(|(_, _, span)| *span));
        } else {
            answers.push(format!("{key}: {}", localized(locale, "missing")));
        }
    }
    Some(if answers.is_empty() {
        gap("extraction", locale)
    } else {
        reply("extraction", &answers.join("\n"), &spans, locale)
    })
}

fn identify_language(payload: &str, locale: &str) -> Reply {
    let slug = detect_input(payload).slug();
    let normalized = normalize_prompt(payload);
    let words: HashSet<_> = normalized.split_whitespace().collect();
    let mut dictionary = if surface_matches_language(payload, slug) {
        lexicon()
            .meanings
            .iter()
            .flat_map(|meaning| &meaning.lexemes)
            .filter(|lexeme| lexeme.language == slug)
            .flat_map(|lexeme| &lexeme.words)
            .filter_map(|word| words.get(word.text.as_str()).copied())
            .collect::<HashSet<_>>()
    } else {
        HashSet::new()
    };
    if dictionary.is_empty() {
        return reply("language", localized(locale, "unknown"), &[], locale);
    }
    let matches: Vec<_> = normalized
        .split_whitespace()
        .filter(|word| dictionary.remove(*word))
        .collect();
    let matches = matches.join(", ");
    let name = localized(locale, slug);
    let name = if name.is_empty() {
        language_name(slug).unwrap_or(slug)
    } else {
        name
    };
    let answer = format!("{name}\n{}: {matches}", localized(locale, "dictionary"));
    let mut result = reply("language", &answer, &[payload], locale);
    result
        .evidence
        .push(format!("prompt_data:dictionary:{slug}:{matches}"));
    result
}

pub(crate) fn try_prompt_data(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let (head, payload) = prompt.split_once([':', '：'])?;
    let (head, payload) = (head.trim(), payload.trim());
    if payload.is_empty() {
        return None;
    }
    let key = normalize_prompt(head);
    let locale = forced_response_language_slug().unwrap_or_else(|| detect_input(head).slug());
    let result = if starts(&key, "classification") {
        classification(head, payload, locale)
    } else if starts(&key, "language") {
        identify_language(payload, locale)
    } else if cues("records").any(|cue| cue == key) {
        record_answer(payload, locale)
    } else if identity(&key) && payload.lines().any(question) {
        record_answer(prompt, locale)
    } else if starts(&key, "extraction") {
        extraction(&key, payload, locale)?
    } else if starts(&key, "rewrite") && cues("omit").any(|cue| surface_present(&key, cue)) {
        gap("rewrite", locale)
    } else {
        return None;
    };
    let intent = format!("prompt_data_{}", result.operation);
    for evidence in &result.evidence {
        log.append("prompt_data", evidence);
    }
    let mut answer = super::finalize_simple(
        prompt,
        log,
        &intent,
        "response:prompt_data",
        &result.body,
        result.confidence,
    );
    answer.evidence_links.extend(result.evidence);
    Some(answer)
}
