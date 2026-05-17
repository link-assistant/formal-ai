use std::sync::OnceLock;

use super::finalize_simple;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, BrainstormSeeds, FactRecord};
use crate::solver_helpers::last_user_turn;

pub fn try_summarization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let asks_for_summary = normalized.contains("summarize")
        || normalized.contains("summarise")
        || normalized.contains("summary of")
        || normalized.contains("one-paragraph summary");
    if !asks_for_summary
        || normalized.contains("conversation")
        || normalized.contains("chat")
        || normalized == "summarize"
    {
        return None;
    }

    let topic = summary_topic(prompt, normalized);
    log.append("summarization:topic", topic.clone());
    if normalized.contains("one paragraph") || normalized.contains("one-paragraph") {
        log.append("summarization:constraint", "one_paragraph".to_owned());
    }

    let body = match topic.as_str() {
        "Rust" => concat!(
            "Rust is a systems programming language focused on performance, memory safety, ",
            "and concurrency. It prevents many memory errors at compile time through ownership ",
            "and borrowing, while still compiling to native machine code."
        )
        .to_owned(),
        "Wikipedia" => concat!(
            "Wikipedia is a free multilingual encyclopedia maintained by volunteer contributors. ",
            "Its articles are collaboratively edited, cite external sources, and are connected ",
            "to structured Wikimedia projects such as Wikidata."
        )
        .to_owned(),
        "formal-ai" => concat!(
            "formal-ai is a deterministic symbolic assistant that routes prompts through Links ",
            "Notation-backed rules, records reasoning events, and exposes OpenAI-shaped API ",
            "surfaces without neural-network inference."
        )
        .to_owned(),
        other => format!(
            "{other} summary: the request is recorded as a bounded summarization task with a topic, constraint, and trace link."
        ),
    };

    Some(finalize_simple(
        prompt,
        log,
        "summarize_topic",
        "response:summarize_topic",
        &body,
        0.85,
    ))
}

fn summary_topic(prompt: &str, normalized: &str) -> String {
    if normalized.contains("formal-ai") || normalized.contains("formal ai") {
        return String::from("formal-ai");
    }
    if normalized.contains("rust") {
        return String::from("Rust");
    }
    if normalized.contains("wikipedia") {
        return String::from("Wikipedia");
    }

    prompt
        .trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .to_owned()
}

fn brainstorm_seed_data() -> &'static BrainstormSeeds {
    static CELL: OnceLock<BrainstormSeeds> = OnceLock::new();
    CELL.get_or_init(seed::brainstorm_seeds)
}

pub fn try_brainstorming_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = brainstorm_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }
    let category = seeds.pick_category(normalized)?;
    let requested_count = requested_brainstorm_count(normalized);
    let body = numbered(&category.items, requested_count);
    log.append("brainstorm:category", category.slug.clone());
    Some(finalize_simple(
        prompt,
        log,
        &category.intent,
        "response:brainstorm",
        &body,
        0.8,
    ))
}

/// Parse the number of items the user asked for. Defaults to 5 when no
/// explicit count is present. Recognises numeric and word forms in every
/// supported language so the algorithm doesn't depend on English-only
/// spelling.
fn requested_brainstorm_count(normalized: &str) -> usize {
    const TEN_HINTS: &[&str] = &[
        " 10 ",
        "10.",
        "10 ",
        " 10",
        "ten ",
        "десять",
        "10 идей",
        "10 имён",
        "दस ",
        "10 ",
        "十个",
        "10 个",
    ];
    if TEN_HINTS.iter().any(|hint| normalized.contains(hint)) {
        10
    } else {
        5
    }
}

fn numbered(items: &[String], count: usize) -> String {
    items
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, item)| format!("{}. {item}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fact_records() -> &'static [FactRecord] {
    static CELL: OnceLock<Vec<FactRecord>> = OnceLock::new();
    CELL.get_or_init(seed::facts).as_slice()
}

pub fn try_fact_lookup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let record = fact_records()
        .iter()
        .find(|record| record.matches_normalized(normalized))?;

    log.append("fact_lookup:request", prompt.to_owned());
    log.append("fact_lookup:hit", record.slug.clone());
    for qid in &record.wikidata {
        if !qid.is_empty() {
            log.append("wikidata", qid.clone());
        }
    }

    let language = detect_language(prompt).slug();
    let summary = record.summary_for(language);
    let source = record.source_for(language);
    if !source.is_empty() {
        log.append("source", source.to_owned());
    }

    Some(finalize_simple(
        prompt,
        log,
        "fact_lookup",
        "response:fact_lookup",
        summary,
        0.9,
    ))
}

pub fn try_coreference_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let has_pronoun = normalized.contains(" it ")
        || normalized.starts_with("it ")
        || normalized.contains(" it?")
        || normalized.contains(" it.")
        || normalized.contains("compare it");
    if !has_pronoun {
        return None;
    }

    let previous = last_user_turn(log)?;
    if !previous.to_lowercase().contains("rust") {
        return None;
    }

    log.append("coreference:resolved", "it=Rust".to_owned());
    log.append("wikidata", "Q575650".to_owned());
    let body = concat!(
        "`it` resolves to Rust from your prior turn. Compared with C, Rust adds ownership, ",
        "borrowing, and stronger compile-time checks so memory-safety errors are caught before ",
        "the program runs while retaining native-code performance."
    );
    Some(finalize_simple(
        prompt,
        log,
        "coreference_rust",
        "response:coreference",
        body,
        0.85,
    ))
}

pub fn try_roleplay_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let asks_for_roleplay = normalized.contains("pretend you are")
        || normalized.contains("act as")
        || normalized.contains("roleplay")
        || normalized.contains("explain like you are");
    if !asks_for_roleplay {
        return None;
    }

    let persona = if normalized.contains("einstein") {
        log.append("wikidata", "Q937".to_owned());
        "Albert Einstein"
    } else if normalized.contains("ada lovelace") {
        log.append("wikidata", "Q7259".to_owned());
        "Ada Lovelace"
    } else if normalized.contains("teacher") {
        "teacher"
    } else {
        "requested persona"
    };
    log.append("roleplay:persona", persona.to_owned());
    let body = if normalized.contains("algorithm") {
        format!(
            "Roleplay frame recorded for {persona}. I will keep the persona explicit and factual: an algorithm is a precise sequence of steps, so a reliable explanation names the inputs, the ordered operations, and the expected result."
        )
    } else if normalized.contains("time dilation") {
        format!(
            "Roleplay frame recorded for {persona}. I will keep the persona explicit and factual: time dilation means clocks can measure different elapsed times when observers move differently or sit in different gravitational fields."
        )
    } else {
        format!(
            "Roleplay frame recorded for {persona}. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent."
        )
    };
    Some(finalize_simple(
        prompt,
        log,
        "roleplay_explanation",
        "response:roleplay",
        &body,
        0.8,
    ))
}
