//! Free-text summarization handler (issue #1174, E139 text-transform family;
//! R1188-U21).
//!
//! Recognizes requests that carry the text to summarize themselves — a
//! paragraph after a command colon, a line break, or quotes — and answers with
//! the dependency summary of that text
//! ([`crate::summarization::dependency::summarize_by_dependency`]): the text is
//! formalized into statements, restatements are dropped, and the statements
//! the others depend on are kept, up to one in three, in the text's own words.
//! Requests that carry no text (for example "summarize the rust language", a
//! seeded topic) stay with the handlers that own them: this handler returns
//! `None` when no payload follows the command, and when the payload holds a
//! single statement, which cannot be shortened without being echoed back.
//!
//! The evidence trace names every statement kept or dropped and every
//! duplicate removed. Recognition vocabulary lives in
//! `data/seed/meanings-summarization.lino` (`text_summarization_action`, in
//! all five supported languages). The browser twin is `trySummarizationText`
//! in `js/worker/formal_ai_worker_text_transform.js`, which runs the
//! JavaScript root's `js/agentic/crate/dependency_summarization.mjs`.

use super::finalize_simple;
use super::text_rewrite::free_text_payload;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::summarization::dependency::summarize_by_dependency;

/// The role naming the act of summarizing a text the request itself
/// carries. Surfaces live in `data/seed/meanings-summarization.lino`.
const ROLE_TEXT_SUMMARIZATION_ACTION: &str = "text_summarization_action";

/// Recognize a summarization request that carries its own text and answer
/// with the statements the rest of the text depends on.
///
/// Returns `None` when the prompt names no text to summarize — the
/// seeded-topic and conversation-summary handlers keep those prompts — or
/// when the text holds a single statement.
pub fn handle_summarization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // The role is read on the punctuation-folded prompt, as the browser twin
    // reads it, so a colon glued to the verb ("Summarize: …") does not hide it
    // (issue #1175 p275).
    let folded = crate::engine::normalize_prompt(normalized);
    if !crate::seed::lexicon().mentions_role(ROLE_TEXT_SUMMARIZATION_ACTION, &folded) {
        return None;
    }
    let payload = free_text_payload(prompt)?;
    let summary = summarize_by_dependency(&payload, crate::language::detect(&payload).slug());
    if summary.statements.len() + summary.removed.len() < 2 || summary.kept.is_empty() {
        // A single statement cannot be shortened without echoing it back.
        return None;
    }
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
