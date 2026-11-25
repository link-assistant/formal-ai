//! Free-text summarization handler (issue #1174, E139 text-transform family;
//! R1188-U21).
//!
//! Recognizes requests that carry the text to summarize themselves — a
//! paragraph after a command colon, a line break, or quotes — and answers with
//! the dependency summary of that text
//! ([`crate::summarization::dependency::summarize_free_text`]): standalone
//! greetings and farewells are filtered, then complete sentences are ranked
//! by dependency and deduplicated.
//! Requests that carry no text (for example "summarize the rust language", a
//! seeded topic) stay with the handlers that own them: this handler returns
//! `None` when no payload follows the command, and when the payload holds a
//! single sentence that would be echoed back unchanged.
//!
//! The evidence trace names every statement kept or dropped and every
//! duplicate removed. Recognition vocabulary lives in
//! `data/seed/meanings-summarization.lino` (`text_summarization_action`, in
//! all five supported languages). The browser twin is `trySummarizationText`
//! in `js/worker/formal_ai_worker_text_transform.js`, which runs the
//! JavaScript root's `js/agentic/crate/dependency_summarization.mjs`.

use super::finalize_simple;
use super::text_rewrite::{command_head, free_text_payload};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed;
use crate::summarization::dependency::summarize_free_text;

/// The role naming the act of summarizing a text the request itself
/// carries. Surfaces live in `data/seed/meanings-summarization.lino`.
const ROLE_TEXT_SUMMARIZATION_ACTION: &str = "text_summarization_action";

/// Recognize a summarization request that carries its own text and answer
/// with the statements the rest of the text depends on.
///
/// Returns `None` when there is no payload or no shorter summary.
pub fn handle_summarization_request(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // Words from the supplied passage cannot introduce a new request.
    let folded = crate::engine::normalize_prompt(command_head(prompt));
    let lexicon = crate::seed::lexicon();
    if !lexicon.mentions_role(ROLE_TEXT_SUMMARIZATION_ACTION, &folded) {
        return None;
    }
    // "Summarize conversation: …" belongs to `conversation_memory`.
    if lexicon.mentions_role(seed::ROLE_CONVERSATION_REFERENCE, &folded)
        || lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_PHRASE, &folded)
    {
        return None;
    }
    let payload = free_text_payload(prompt)?;
    let summary = summarize_free_text(&payload, crate::language::detect(&payload).slug())?;
    for (index, node) in summary.statements.iter().enumerate() {
        let verdict = if summary.kept.contains(&index) {
            "kept"
        } else {
            "dropped"
        };
        log.append(
            "summarization_statement",
            format!("{verdict} {}", node.entry.statement.text),
        );
    }
    for entry in &summary.removed {
        log.append("summarization_duplicate", entry.statement.text.clone());
    }
    log.append(
        "summarization_bound",
        format!("{}/{}", summary.kept.len(), summary.statements.len()),
    );
    let selected: Vec<&str> = summary
        .kept
        .iter()
        .map(|index| summary.statements[*index].entry.statement.text.as_str())
        .collect();
    log.append("summarization_selected", selected.join(" | "));
    Some(finalize_simple(
        prompt,
        log,
        "summarization_free_text",
        "response:summarization_free_text",
        &summary.text,
        0.8,
    ))
}

/// Read the requested record using the ordinary project or concept lookup.
pub(super) fn derived_topic_summary(
    subject: &str,
    language: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<(String, String, String)> {
    let registry = seed::projects_registry();
    if let Some(project) = registry.by_alias(subject) {
        let (description, source) =
            super::web_requests::verified_project_description(project, offline, log)
                .unwrap_or_else(|| {
                    log.append("project:description", "unavailable".to_owned());
                    (
                        seed::localized_response("project-description-unavailable", language)
                            .unwrap_or_default(),
                        String::from("none"),
                    )
                });
        return Some((
            project.display_name_for(language).to_owned(),
            description,
            source,
        ));
    }
    let query = crate::concepts::ConceptQuery {
        term: subject.to_owned(),
        context: None,
        response_language: None,
    };
    let record = crate::concepts::lookup_concept_query(&query)?.record;
    let variant = record
        .localized_for(language)
        .filter(|variant| !variant.summary.is_empty());
    Some((
        variant.map_or(&record.term, |value| &value.term).clone(),
        variant
            .map_or(&record.summary, |value| &value.summary)
            .clone(),
        variant
            .filter(|value| !value.source.is_empty())
            .map_or(&record.source, |value| &value.source)
            .clone(),
    ))
}
