//! Source-qualified definitions bind two seeded slots before any lookup.
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::{SourceRecord, localized_response, prompt_patterns, source_registry};
use crate::solver_handlers::finalize_simple;
use crate::source_fetch::{CachedSourceClient, SourceTransport};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Subject, provider and language bound by a canonical source-definition frame.
pub struct Request {
    pub term: String,
    pub source: String,
    pub language: String,
}
fn matches_source(label: &str, source: &SourceRecord) -> bool {
    [source.id.as_str(), source.name.as_str()]
        .iter()
        .any(|name| name.to_lowercase() == label.to_lowercase())
}
fn unquote(value: &str) -> String {
    let value = value.trim();
    let pairs = [
        ('"', '"'),
        ('\'', '\''),
        ('`', '`'),
        ('«', '»'),
        ('“', '”'),
        ('‘', '’'),
    ];
    for (first, last) in pairs {
        if value.starts_with(first)
            && value.ends_with(last)
            && value.len() >= first.len_utf8() + last.len_utf8()
        {
            return value[first.len_utf8()..value.len() - last.len_utf8()]
                .trim()
                .to_owned();
        }
    }
    value.to_owned()
}
/// Match a seeded literal at a UTF-8 boundary, with case and whitespace folding.
fn literal_end(input: &str, start: usize, literal: &str) -> Option<usize> {
    let mut cursor = start;
    let mut expected = literal.chars().peekable();
    while let Some(character) = expected.next() {
        if character.is_whitespace() {
            while expected.peek().is_some_and(|next| next.is_whitespace()) {
                expected.next();
            }
            let mut found = false;
            for actual in input.get(cursor..)?.chars() {
                if !actual.is_whitespace() {
                    break;
                }
                cursor += actual.len_utf8();
                found = true;
            }
            if !found {
                return None;
            }
        } else {
            let actual = input.get(cursor..)?.chars().next()?;
            if !actual.to_lowercase().eq(character.to_lowercase()) {
                return None;
            }
            cursor += actual.len_utf8();
        }
    }
    Some(cursor)
}
/// Bind the seeded subject/provider slots without performing a lookup.
#[must_use]
pub fn request(prompt: &str) -> Option<Request> {
    let input = prompt
        .trim()
        .trim_end_matches(['.', '?', '!', '。', '？', '！'])
        .trim();
    let registry = source_registry();
    let mut frames: Vec<_> = prompt_patterns()
        .into_iter()
        .filter(|row| row.intent == "source-qualified-definition" && row.kind == "frame")
        .collect();
    frames.sort_by_key(|row| std::cmp::Reverse(row.text.encode_utf16().count()));
    for frame in frames {
        let term_slot = concat!("{", "term", "}");
        let source_slot = concat!("{", "source", "}");
        let (Some(term_at), Some(source_at)) =
            (frame.text.find(term_slot), frame.text.find(source_slot))
        else {
            continue;
        };
        let term_first = term_at < source_at;
        let (first, second) = if term_first {
            (term_slot, source_slot)
        } else {
            (source_slot, term_slot)
        };
        let Some((before, tail)) = frame.text.split_once(first) else {
            continue;
        };
        let Some((middle, after)) = tail.split_once(second) else {
            continue;
        };
        if middle.trim().is_empty()
            || [before, middle, after]
                .iter()
                .any(|text| text.contains(['{', '}']))
        {
            continue;
        }
        let Some(body_start) = literal_end(input, 0, before) else {
            continue;
        };
        let Some(body_end) = input
            .char_indices()
            .map(|(start, _)| start)
            .chain(std::iter::once(input.len()))
            .filter(|start| *start >= body_start)
            .rev()
            .find(|start| literal_end(input, *start, after) == Some(input.len()))
        else {
            continue;
        };
        let body = &input[body_start..body_end];
        let mut candidates = Vec::new();
        let mut next_boundary = 0;
        for (start, _) in body.char_indices() {
            if start < next_boundary {
                continue;
            }
            let Some(end) = literal_end(body, start, middle) else {
                continue;
            };
            next_boundary = end;
            let left = unquote(&body[..start]);
            let right = unquote(&body[end..]);
            let (term, source) = if term_first {
                (left, right)
            } else {
                (right, left)
            };
            if term.is_empty()
                || source.is_empty()
                || term.contains(['\n', '\r'])
                || source.contains(['\n', '\r'])
            {
                continue;
            }
            candidates.push(Request {
                term,
                source,
                language: frame.language.clone(),
            });
        }
        let bound = candidates
            .iter()
            .filter(|candidate| {
                registry
                    .iter()
                    .any(|record| matches_source(&candidate.source, record))
            })
            .max_by_key(|candidate| candidate.source.len());
        if let Some(bound) = bound {
            return Some(bound.clone());
        }
        if !candidates.is_empty() {
            return if term_first {
                candidates.pop()
            } else {
                Some(candidates.remove(0))
            };
        }
    }
    None
}
fn render(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut output = localized_response(intent, language).unwrap_or_default();
    for (name, value) in values {
        output = output.replace(&format!("{{{name}}}"), value);
    }
    output
}
fn unresolved(prompt: &str, log: &mut EventLog, request: &Request, status: &str) -> SymbolicAnswer {
    log.append("source-qualified-definition:status", status.to_owned());
    let body = render(
        "source-qualified-definition-unresolved",
        &request.language,
        &[
            ("term", &request.term),
            ("source", &request.source),
            ("status", status),
        ],
    );
    finalize_simple(
        prompt,
        log,
        "concept_lookup",
        "response:source-qualified-definition",
        &body,
        0.85,
    )
}
pub(crate) fn try_definition<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let request = request(prompt)?;
    log.append("source-qualified-definition:term", request.term.clone());
    let matches: Vec<_> = source_registry()
        .into_iter()
        .filter(|record| matches_source(&request.source, record))
        .collect();
    let Some(record) = matches.first() else {
        return Some(unresolved(prompt, log, &request, "missing-source"));
    };
    if matches.len() > 1 {
        return Some(unresolved(prompt, log, &request, "ambiguous-source"));
    }
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
    if !preferences.allows(record) {
        return Some(unresolved(prompt, log, &request, "disabled-source"));
    }
    let Some(url) = crate::source_walk::entry_url_in(record, &request.term, &request.language)
    else {
        return Some(unresolved(prompt, log, &request, "unbound-endpoint"));
    };
    let Ok(capture) = client.fetch(&url) else {
        return Some(unresolved(prompt, log, &request, "no-capture"));
    };
    let value: serde_json::Value = serde_json::from_slice(capture.bytes()).unwrap_or_default();
    let ambiguous = record.extractor == "mediawiki_summary_v1"
        && value.get("type").and_then(serde_json::Value::as_str) == Some("disambiguation");
    let items = if ambiguous {
        crate::how_to_guide::extract::list_items(
            value
                .get("extract_html")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        )
        .iter()
        .map(|item| {
            crate::how_to_guide::extract::decode_entities(
                &crate::how_to_guide::extract::strip_html(item),
            )
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
        })
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
    } else {
        crate::concept_lookup::extractor_gloss_texts(
            &record.extractor,
            capture.bytes(),
            &request.term,
        )
    };
    if items.is_empty() {
        return Some(unresolved(prompt, log, &request, "no-items"));
    }
    capture.record(log);
    log.append("source-qualified-definition:source", record.id.clone());
    log.append("source", capture.source_url().to_owned());
    let kind = if ambiguous {
        "disambiguation"
    } else {
        "definition"
    };
    log.append("source-qualified-definition:kind", kind.to_owned());
    log.append("source-qualified-definition:status", "captured".to_owned());
    let senses = items
        .iter()
        .enumerate()
        .map(|(index, gloss)| {
            render(
                "word_definition_sense",
                &request.language,
                &[
                    ("n", &(index + 1).to_string()),
                    ("pos", ""),
                    ("gloss", gloss),
                ],
            )
        })
        .collect::<String>();
    let body = render(
        "source-qualified-definition-answer",
        &request.language,
        &[
            ("term", &request.term),
            ("source", &request.source),
            ("kind", kind),
            ("senses", &senses),
            ("url", capture.source_url()),
            ("sha256", capture.sha256()),
            ("captured-at", capture.fetched_at()),
            ("cached", if capture.cached() { "true" } else { "false" }),
            ("license", &record.license_name),
            ("license-url", &record.license_url),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        "concept_lookup",
        "response:source-qualified-definition",
        &body,
        0.85,
    ))
}
