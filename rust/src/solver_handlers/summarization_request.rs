//! Free-text summarization handler (issue #1174, E139 text-transform family).
//!
//! Recognizes requests that carry the text to summarize themselves — a
//! paragraph after a command colon, a line break, or quotes — and runs it
//! through the shared formalize → summarize → deformalize pipeline
//! (`crate::summarization`) under a ~30% selection bound (at least two
//! statements, always at least one short of the input so the summary can
//! never echo the source). Requests that carry no text (for example
//! "summarize the rust language", a seeded topic) stay with the handlers
//! that own them: this handler returns `None` when no payload follows the
//! command, and when the payload is a single sentence that cannot be
//! shortened without being echoed back.
//!
//! Recognition vocabulary lives in `data/seed/meanings-summarization.lino`
//! (`text_summarization_action`, in all five supported languages); the
//! handler only computes the transform and logs every ranking decision.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use super::finalize_simple;
use super::text_rewrite::free_text_payload;

/// The role naming the act of summarizing a text the request itself
/// carries. Surfaces live in `data/seed/meanings-summarization.lino`.
const ROLE_TEXT_SUMMARIZATION_ACTION: &str = "text_summarization_action";

/// Recognize a summarization request that carries its own text and answer
/// with a weight-ranked selection of its statements. Returns `None` when
/// the prompt names no text to summarize — the seeded-topic and
/// conversation-summary handlers keep those prompts.
pub fn handle_summarization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !crate::seed::lexicon().mentions_role(ROLE_TEXT_SUMMARIZATION_ACTION, normalized) {
        return None;
    }
    let payload = free_text_payload(prompt)?;
    let statements = crate::summarization::formalize(&payload);
    if statements.len() < 2 {
        // A single statement cannot be shortened without echoing it back.
        return None;
    }
    for statement in &statements {
        log.append(
            "summarization_statement",
            format!("{:?} weight {}", statement.kind, statement.weight),
        );
    }
    // A ~30% selection bound, floored at two statements and always at least
    // one short of the input.
    let bound = (((statements.len() * 3) + 5) / 10)
        .max(2)
        .min(statements.len() - 1)
        .max(1);
    log.append(
        "summarization_bound",
        format!("{}/{}", bound, statements.len()),
    );
    let config = crate::summarization::SummarizationConfig::default()
        .with_max_statements(bound)
        .with_language(crate::language::detect(prompt).slug());
    let selected = crate::summarization::summarize(&statements, &config);
    if selected.is_empty() {
        return None;
    }
    log.append(
        "summarization_selected",
        selected
            .iter()
            .map(|statement| statement.text.as_str())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    let body = crate::summarization::deformalize(&selected);
    Some(finalize_simple(
        prompt,
        log,
        "summarization_free_text",
        "response:summarization_free_text",
        &body,
        0.8,
    ))
}
