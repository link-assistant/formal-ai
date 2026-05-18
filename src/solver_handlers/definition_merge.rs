use std::fmt::Write as _;

use crate::concepts::{extract_concept_query, lookup_concept_query, ConceptQuery, ConceptRecord};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

use super::{finalize_simple, render_source_link};

#[derive(Debug, Clone)]
struct DefinitionFragment {
    language: String,
    summary: String,
    source: String,
    source_kind: String,
}

pub fn try_definition_merge(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let term = extract_definition_merge_term(prompt, normalized)?;
    log.append("definition_merge:request", term.clone());
    let query = ConceptQuery {
        term: term.clone(),
        context: None,
    };
    let Some(lookup) = lookup_concept_query(&query) else {
        log.append("definition_merge:miss", term);
        return None;
    };
    let record = lookup.record;
    let fragments = definition_fragments(record);
    if fragments.is_empty() {
        log.append("definition_merge:empty", record.slug.clone());
        return None;
    }
    log.append("definition_merge:hit", record.slug.clone());
    if !record.wikidata.is_empty() {
        log.append("wikidata", record.wikidata.clone());
    }
    for language in source_languages(&fragments) {
        log.append("definition_merge:language", language);
    }
    for source in source_urls(&fragments) {
        log.append("source:http", source);
    }
    let facts = merged_definition_facts(&fragments);
    log.append("definition_merge:facts", facts.len().to_string());
    let body = render_definition_merge(record, &fragments, &facts);
    Some(finalize_simple(
        prompt,
        log,
        "definition_merge",
        "response:definition_merge",
        &body,
        0.9,
    ))
}

fn extract_definition_merge_term(prompt: &str, normalized: &str) -> Option<String> {
    let asks_merge = normalized.contains("merge")
        || normalized.contains("merged")
        || normalized.contains("combine")
        || normalized.contains("combined")
        || normalized.contains("fuse")
        || normalized.contains("fusion");
    let asks_definition = normalized.contains("definition")
        || normalized.contains("definitions")
        || normalized.contains("translation")
        || normalized.contains("translations")
        || normalized.contains("translated")
        || normalized.contains("wikipedia");
    if !asks_merge || !asks_definition {
        return None;
    }

    let lower = prompt.to_lowercase();
    let markers = [
        "translated definitions for ",
        "translated definitions of ",
        "wikipedia definitions for ",
        "wikipedia definitions of ",
        "definitions for ",
        "definitions of ",
        "definition for ",
        "definition of ",
        "translations for ",
        "translations of ",
        "translation for ",
        "translation of ",
    ];
    for marker in markers {
        if let Some(index) = lower.find(marker) {
            let start = index + marker.len();
            let candidate = trim_definition_merge_tail(&prompt[start..]);
            if !candidate.is_empty() {
                return Some(candidate.to_lowercase());
            }
        }
    }
    extract_concept_query(prompt).map(|query| query.term)
}

fn trim_definition_merge_tail(value: &str) -> String {
    let mut end = value.len();
    let lower = value.to_lowercase();
    for delimiter in [" from ", " using ", " with ", " by ", " into ", " across "] {
        if let Some(index) = lower.find(delimiter) {
            end = end.min(index);
        }
    }
    value[..end]
        .trim()
        .trim_matches(['\'', '"', '`', '“', '”', '«', '»'])
        .trim_end_matches(['?', '。', '.', '!', ',', ';', ':'])
        .trim()
        .to_owned()
}

fn definition_fragments(record: &ConceptRecord) -> Vec<DefinitionFragment> {
    let mut fragments = Vec::new();
    push_definition_fragment(
        &mut fragments,
        inferred_source_language(&record.source),
        &record.summary,
        &record.source,
        &record.source_kind,
    );
    for localized in &record.localized {
        push_definition_fragment(
            &mut fragments,
            &localized.language,
            &localized.summary,
            &localized.source,
            &localized.source_kind,
        );
    }
    fragments
}

fn push_definition_fragment(
    fragments: &mut Vec<DefinitionFragment>,
    language: &str,
    summary: &str,
    source: &str,
    source_kind: &str,
) {
    let summary = summary.trim();
    if summary.is_empty() {
        return;
    }
    let duplicate = fragments.iter().any(|fragment| {
        fragment.language == language
            && normalize_fact(&fragment.summary) == normalize_fact(summary)
    });
    if duplicate {
        return;
    }
    fragments.push(DefinitionFragment {
        language: language.to_owned(),
        summary: summary.to_owned(),
        source: source.trim().to_owned(),
        source_kind: source_kind.trim().to_owned(),
    });
}

fn inferred_source_language(source: &str) -> &str {
    if source.contains("://ru.wikipedia.org/") {
        "ru"
    } else if source.contains("://hi.wikipedia.org/") {
        "hi"
    } else if source.contains("://zh.wikipedia.org/") {
        "zh"
    } else {
        "en"
    }
}

fn source_languages(fragments: &[DefinitionFragment]) -> Vec<String> {
    let mut languages = Vec::new();
    for fragment in fragments {
        if !languages
            .iter()
            .any(|language| language == &fragment.language)
        {
            languages.push(fragment.language.clone());
        }
    }
    languages
}

fn source_urls(fragments: &[DefinitionFragment]) -> Vec<String> {
    let mut sources = Vec::new();
    for fragment in fragments {
        if fragment.source.is_empty() {
            continue;
        }
        if !sources.iter().any(|source| source == &fragment.source) {
            sources.push(fragment.source.clone());
        }
    }
    sources
}

fn merged_definition_facts(fragments: &[DefinitionFragment]) -> Vec<(String, String)> {
    let mut facts = Vec::new();
    let mut seen = Vec::new();
    for fragment in fragments {
        for sentence in split_definition_sentences(&fragment.summary) {
            let key = normalize_fact(&sentence);
            if key.is_empty() || seen.iter().any(|existing| existing == &key) {
                continue;
            }
            seen.push(key);
            facts.push((fragment.language.clone(), sentence));
        }
    }
    facts
}

fn split_definition_sentences(summary: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for character in summary.chars() {
        current.push(character);
        if matches!(character, '.' | '!' | '?' | '।' | '。') {
            let sentence = current.trim();
            if !sentence.is_empty() {
                sentences.push(sentence.to_owned());
            }
            current.clear();
        }
    }
    let tail = current.trim();
    if !tail.is_empty() {
        sentences.push(tail.to_owned());
    }
    sentences
}

fn normalize_fact(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn render_definition_merge(
    record: &ConceptRecord,
    fragments: &[DefinitionFragment],
    facts: &[(String, String)],
) -> String {
    let display_term = record
        .localized_for("en")
        .map(|localized| localized.term.as_str())
        .filter(|term| !term.is_empty())
        .unwrap_or(record.term.as_str());
    let languages = source_languages(fragments).join(", ");
    let anchor = if record.wikidata.is_empty() {
        String::new()
    } else {
        format!(" [{}]", record.wikidata)
    };
    let mut body = format!(
        "Merged definition of {display_term}{anchor}\nSource languages: {languages}\n\nFacts:"
    );
    for (language, fact) in facts {
        let _ = writeln!(body, "\n- [{language}] {fact}");
    }
    body.push_str("\nSources:");
    for fragment in unique_source_fragments(fragments) {
        let source = render_source_link(&fragment.source);
        let _ = writeln!(
            body,
            "\n- [{language}] {source} ({source_kind})",
            language = fragment.language,
            source_kind = fragment.source_kind,
        );
    }
    body
}

fn unique_source_fragments(fragments: &[DefinitionFragment]) -> Vec<&DefinitionFragment> {
    let mut unique = Vec::new();
    for fragment in fragments {
        if fragment.source.is_empty() {
            continue;
        }
        let exists = unique.iter().any(|existing: &&DefinitionFragment| {
            existing.language == fragment.language && existing.source == fragment.source
        });
        if !exists {
            unique.push(fragment);
        }
    }
    unique
}
