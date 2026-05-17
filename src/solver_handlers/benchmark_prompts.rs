use super::finalize_simple;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
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

pub fn try_brainstorming_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let asks_for_brainstorm = normalized.contains("brainstorm")
        || normalized.contains("give me five ideas")
        || normalized.contains("give me 5 ideas")
        || normalized.contains("suggest five")
        || normalized.contains("suggest 5")
        || normalized.contains("ten names")
        || normalized.contains("10 names");
    if !asks_for_brainstorm {
        return None;
    }

    let requested_count = if normalized.contains("ten") || normalized.contains("10") {
        10
    } else {
        5
    };
    let (intent, category, body) = if normalized.contains("name") || normalized.contains("names") {
        (
            "brainstorm_names",
            "names",
            numbered(
                &[
                    "TraceLint",
                    "ReviewLink",
                    "PatchSignal",
                    "DiffAnchor",
                    "CodeLedger",
                    "SymbolScribe",
                    "RuleBeacon",
                    "LinkHarbor",
                    "TraceForge",
                    "PromptLedger",
                ],
                requested_count,
            ),
        )
    } else {
        (
            "brainstorm_project_ideas",
            "project_ideas",
            numbered(
                &[
                    "A local Links Notation notebook with searchable traces.",
                    "A deterministic code-review checklist generator.",
                    "A multilingual prompt-variation test corpus.",
                    "A CLI that converts issue requirements into traceable tests.",
                    "A source-cache inspector for reproducible agent runs.",
                    "A changelog-fragment consistency checker.",
                    "A prompt-matrix generator for four-language smoke tests.",
                    "A Wikidata anchor verifier for local seed records.",
                    "A trace viewer that groups events by solver phase.",
                    "A small offline issue-to-test planning tool.",
                ],
                requested_count,
            ),
        )
    };
    log.append("brainstorm:category", category.to_owned());
    Some(finalize_simple(
        prompt,
        log,
        intent,
        "response:brainstorm",
        &body,
        0.8,
    ))
}

fn numbered(items: &[&str], count: usize) -> String {
    items
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, item)| format!("{}. {item}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn try_fact_lookup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let fact = if normalized.contains("lord of the rings")
        && (normalized.contains("who wrote") || normalized.contains("author"))
    {
        Some((
            "The Lord of the Rings was written by J. R. R. Tolkien.",
            &["Q892", "Q15228"][..],
        ))
    } else if normalized.contains("eiffel tower")
        && (normalized.contains("built")
            || normalized.contains("construction")
            || normalized.contains("when"))
    {
        Some((
            "Construction of the Eiffel Tower started in 1887 and it opened in 1889.",
            &["Q243"][..],
        ))
    } else if normalized.contains("capital of japan")
        || (normalized.contains("japan") && normalized.contains("capital"))
    {
        Some(("The capital of Japan is Tokyo.", &["Q17", "Q1490"][..]))
    } else {
        None
    }?;

    log.append("fact_lookup:request", prompt.to_owned());
    for qid in fact.1 {
        log.append("wikidata", (*qid).to_owned());
    }
    Some(finalize_simple(
        prompt,
        log,
        "fact_lookup",
        "response:fact_lookup",
        fact.0,
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
