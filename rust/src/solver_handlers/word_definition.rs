//! Word definitions from a dictionary source (issue #1172 R1172-5).
//!
//! "What does X mean?" is answered from the registry's dictionary sources
//! (Wiktionary through the Free Dictionary entry API for English and each
//! edition's own MediaWiki extracts for the other languages), never from a
//! canned paragraph. Everything this module reads is seed data — the
//! `word_definition` rows of `data/seed/prompt-patterns.lino` hold the frames
//! that read the word (`what does {term} mean`, `что означает {term}`,
//! `{term}是什么意思`, …), the registry kind that answers a definition, and
//! each edition's definition-section titles, example markers, annotation
//! labels and inflection markers — and the answer is a localized template of
//! `data/seed/multilingual-responses-concept-lookup.lino` that cites the
//! captured URL, its SHA-256 and the source's license. A word no dictionary
//! source defines falls through to the next row, which records the consulted
//! sources. The browser twin is `tryWordDefinition` in
//! `js/worker/formal_ai_worker_concept_lookup.js`.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::{PromptPattern, localized_response, prompt_patterns, source_registry};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

const INTENT: &str = "word_definition";
const MAX_SENSES: usize = 3;

/// The `word_definition` prompt-pattern rows of one kind, optionally of one
/// language, in seed order.
fn patterns(kind: &str, language: Option<&str>) -> Vec<PromptPattern> {
    prompt_patterns()
        .into_iter()
        .filter(|row| {
            row.intent == INTENT
                && row.kind == kind
                && !row.text.is_empty()
                && language.is_none_or(|language| row.language == language)
        })
        .collect()
}

/// The word a definition request asks about and the frame's language, read
/// from the longest seeded frame that encloses it.
#[must_use]
pub fn definition_term(prompt: &str) -> Option<(String, String)> {
    let lowered = prompt.to_lowercase();
    let lowered = lowered
        .trim()
        .trim_end_matches(['?', '？', '.', '。', '!', '！'])
        .trim_end()
        .trim_start_matches(['¿', '¡'])
        .trim_start();
    let mut frames: Vec<PromptPattern> = patterns("frame", None)
        .into_iter()
        .filter(|row| row.text.contains("{term}"))
        .collect();
    frames.sort_by_key(|row| std::cmp::Reverse(row.text.encode_utf16().count()));
    for frame in frames {
        let Some((before, after)) = frame.text.split_once("{term}") else {
            continue;
        };
        if lowered.len() <= before.len() + after.len()
            || !lowered.starts_with(before)
            || !lowered.ends_with(after)
        {
            continue;
        }
        let quotes: &[char] = &['"', '\'', '`', '“', '”', '‘', '’', '«', '»'];
        let term = lowered[before.len()..lowered.len() - after.len()]
            .trim()
            .trim_matches(quotes)
            .trim();
        let words = term.split_whitespace().count();
        let lexical = term.chars().all(|ch| {
            ch.is_alphabetic()
                || ch.is_whitespace()
                || ch == '\''
                || ch == '-'
                || matches!(
                    unicode_general_category::get_general_category(ch),
                    unicode_general_category::GeneralCategory::NonspacingMark
                        | unicode_general_category::GeneralCategory::SpacingMark
                        | unicode_general_category::GeneralCategory::EnclosingMark
                )
        });
        if words == 0 || words > 3 || !lexical {
            continue;
        }
        return Some((term.to_owned(), frame.language.clone()));
    }
    None
}

/// Collapse every whitespace run to one space.
fn compact(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `gloss` is only a respelling of the headword (same letters and
/// digits once marks and punctuation are dropped).
fn respelling(gloss: &str, term: &str) -> bool {
    let letters = |value: &str| -> String {
        value
            .chars()
            .filter(|ch| ch.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect()
    };
    let gloss = letters(gloss);
    !gloss.is_empty() && gloss == letters(term)
}

/// Whether a trimmed extract line is a MediaWiki section heading.
fn is_heading(line: &str) -> bool {
    line.chars().count() > 1 && line.starts_with('=') && line.ends_with('=')
}

/// A heading's title, lowercased.
fn title_of(line: &str) -> String {
    line.trim_matches('=').trim().to_lowercase()
}

/// The definitions one dictionary capture states, as `(part of speech,
/// gloss)` pairs: the Free Dictionary entry array, or a MediaWiki extract read
/// by its seeded definition sections (the first section when the edition
/// declares none or the page carries none).
fn read_senses(text: &str, term: &str, language: &str) -> Vec<(String, String)> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text.trim()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(entries) = value.as_array() {
        for meaning in entries
            .iter()
            .filter_map(|entry| entry.get("meanings").and_then(|value| value.as_array()))
            .flatten()
        {
            let pos = meaning
                .get("partOfSpeech")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            for definition in meaning
                .get("definitions")
                .and_then(|value| value.as_array())
                .into_iter()
                .flatten()
            {
                let gloss = compact(
                    definition
                        .get("definition")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default(),
                );
                if !gloss.is_empty() {
                    out.push((pos.to_owned(), gloss));
                }
            }
        }
        return out;
    }
    let Some(pages) = value
        .pointer("/query/pages")
        .and_then(|value| value.as_object())
    else {
        return out;
    };
    let texts = |kind: &str| -> Vec<String> {
        patterns(kind, Some(language))
            .into_iter()
            .map(|row| row.text)
            .collect()
    };
    let titles: Vec<String> = texts("definition_section")
        .iter()
        .map(|title| title.to_lowercase())
        .collect();
    let markers = texts("example_marker");
    let labels: Vec<String> = texts("annotation_prefix")
        .iter()
        .map(|label| label.to_lowercase())
        .collect();
    let inflections = texts("inflection_marker");
    let opens_definitions = |line: &str| {
        titles
            .iter()
            .any(|title| title_of(line).starts_with(title.as_str()))
    };
    for extract in pages
        .values()
        .filter_map(|page| page.get("extract").and_then(|value| value.as_str()))
    {
        let lines: Vec<&str> = extract.lines().map(str::trim).collect();
        let sectioned = !titles.is_empty()
            && lines
                .iter()
                .any(|line| is_heading(line) && opens_definitions(line));
        let mut in_section = false;
        let mut headings = 0usize;
        for line in lines {
            if is_heading(line) {
                headings += 1;
                in_section = if sectioned {
                    opens_definitions(line)
                } else {
                    headings == 1
                };
                continue;
            }
            if !in_section || line.is_empty() || line.chars().all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            let lower = line.to_lowercase();
            if labels.iter().any(|label| lower.starts_with(label.as_str()))
                || inflections
                    .iter()
                    .any(|marker| line.contains(marker.as_str()))
            {
                continue;
            }
            let mut gloss = line;
            for marker in &markers {
                if let Some(cut) = gloss.find(marker.as_str()) {
                    gloss = &gloss[..cut];
                }
            }
            let gloss = compact(gloss);
            if gloss.is_empty() || respelling(&gloss, term) {
                continue;
            }
            out.push((String::new(), gloss));
        }
    }
    out
}

/// Fill a localized `word_definition_*` template.
fn render(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut out = localized_response(intent, language).unwrap_or_default();
    for (name, value) in values {
        out = out.replace(&["{", name, "}"].concat(), value);
    }
    out
}

/// Answer a definition request from the dictionary sources the registry
/// declares, through `client` (the capture cache, then the network when the
/// client is online). `None` when the prompt is not a definition request or
/// no dictionary source states a sense.
pub fn try_word_definition_with_client<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let (term, language) = definition_term(prompt)?;
    log.append("word_definition:term", term.clone());
    let kinds: Vec<String> = patterns("source_kind", None)
        .into_iter()
        .map(|row| row.text)
        .collect();
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
    for record in source_registry()
        .into_iter()
        .filter(|record| kinds.contains(&record.kind) && preferences.allows(record))
    {
        let Some(url) = crate::source_walk::entry_url_in(&record, &term, &language) else {
            log.append("word_definition:unbound", record.id.clone());
            continue;
        };
        let capture = match client.fetch(&url) {
            Ok(capture) => capture,
            Err(error) => {
                log.append(
                    "word_definition:miss",
                    [record.id.as_str(), " ", &error.to_string()].concat(),
                );
                continue;
            }
        };
        let text = String::from_utf8_lossy(capture.bytes());
        let senses: Vec<(String, String)> = read_senses(&text, &term, &language)
            .into_iter()
            .take(MAX_SENSES)
            .collect();
        if senses.is_empty() {
            log.append("word_definition:no_sense", record.id.clone());
            continue;
        }
        let mut rendered = String::new();
        for (index, (pos, gloss)) in senses.iter().enumerate() {
            let pos = if pos.is_empty() {
                String::new()
            } else {
                ["(", pos, ") "].concat()
            };
            rendered.push_str(&render(
                "word_definition_sense",
                &language,
                &[
                    ("n", &(index + 1).to_string()),
                    ("pos", &pos),
                    ("gloss", gloss),
                ],
            ));
        }
        let digest = capture.sha256().get(..16).unwrap_or(capture.sha256());
        let body = render(
            "word_definition_answer",
            &language,
            &[
                ("term", &term),
                ("source", &record.name),
                ("senses", &rendered),
                ("url", capture.source_url()),
                ("sha256", digest),
                ("license", &record.license_name),
            ],
        );
        log.append("word_definition:source", record.id.clone());
        log.append("source", capture.source_url().to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "concept_lookup",
            "response:word_definition",
            &body,
            0.85,
        ));
    }
    None
}

/// The `concept_lookup` row's dictionary step over the process capture cache;
/// transport is enabled only when the runtime opted into live fetches.
pub fn try_word_definition_with_offline(
    prompt: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    definition_term(prompt)?;
    let cache_root = crate::coding::synthesis_runtime::source_cache_root();
    let client = CachedSourceClient::new(&cache_root, CurlSourceTransport).with_online(!offline);
    try_word_definition_with_client(prompt, log, &client)
}
