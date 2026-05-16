//! "How it works" follow-up handler extracted from `solver_handlers` to keep
//! that module under the 1000-line cap enforced by `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver_handlers::{finalize_simple, try_concept_lookup};
use crate::solver_helpers::last_assistant_turn;

/// Handles follow-up elaboration prompts such as "how it works?", "how does
/// it work?", or "how does X work?". When the prior assistant turn mentioned a
/// named concept the solver re-runs a concept lookup for that topic; when no
/// prior context is present it redirects to the meta-explanation handler.
pub fn try_how_it_works(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_how_it_works = normalized == "how it works?"
        || normalized == "how it works"
        || normalized == "how does it work?"
        || normalized == "how does it work"
        || normalized.starts_with("how does it work")
        || normalized.starts_with("how it works")
        || normalized.starts_with("how does ")
            && (normalized.ends_with(" work?") || normalized.ends_with(" work"));
    if !is_how_it_works {
        return None;
    }
    log.append("followup:how_it_works", normalized.to_owned());

    // Try to extract the subject from the prompt itself ("how does Curve25519 work?").
    let subject = extract_how_it_works_subject(normalized);

    // When a subject was explicit in the prompt, do a direct concept lookup.
    if let Some(ref term) = subject {
        use crate::concepts::{extract_concept_query, lookup_concept_query};
        if let Some(query) = extract_concept_query(&format!("what is {term}")) {
            if lookup_concept_query(&query).is_some() {
                log.append("followup:subject", format!("inline:{term}"));
                // Delegate to try_concept_lookup by synthesising a standard prompt.
                return try_concept_lookup(&format!("what is {term}"), log);
            }
        }
    }

    // No inline subject — look for the topic in the prior assistant reply.
    if let Some(prior) = last_assistant_turn(log).map(str::to_owned) {
        log.append("followup:prior_turn", "assistant".to_owned());
        // Extract the first capitalised noun phrase from the prior reply
        // (typically the term in "Term (category): …" format).
        if let Some(term) = extract_topic_from_prior_reply(&prior) {
            use crate::concepts::{extract_concept_query, lookup_concept_query};
            if let Some(query) = extract_concept_query(&format!("what is {term}")) {
                if lookup_concept_query(&query).is_some() {
                    log.append("followup:subject", format!("prior_reply:{term}"));
                    return try_concept_lookup(&format!("what is {term}"), log);
                }
            }
            // Topic is known from history but not in the concept corpus —
            // return a helpful explanation that names the topic.
            let body = format!(
                "To explain how {term} works: I know the term from the prior conversation \
                 but do not have a detailed symbolic rule for it yet. Add a Links Notation \
                 fact with the mechanism description, then ask again."
            );
            log.append("followup:subject", format!("prior_reply_no_record:{term}"));
            return Some(finalize_simple(
                prompt,
                log,
                "concept_elaboration_missing",
                "response:concept_elaboration_missing",
                &body,
                0.3,
            ));
        }
    }

    // No context at all — route to meta_explanation.
    let body = String::from(
        "I answered that way because the prompt matched a deterministic Links Notation rule. \
         To ask about a specific topic, try \"how does X work?\" where X is a concept I know \
         (e.g. \"how does Wikipedia work?\"). The evidence and trace events are appended to \
         the log; see the trace link for the full chain.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "meta_explanation",
        "response:meta_explanation",
        &body,
        0.5,
    ))
}

/// Extract the explicit subject from a "how does X work?" prompt.
/// Returns `None` when the prompt is the bare "how it works?" form.
fn extract_how_it_works_subject(normalized: &str) -> Option<String> {
    // "how does X work" / "how does X work?"
    if let Some(rest) = normalized.strip_prefix("how does ") {
        let term = rest.trim_end_matches('?').trim_end_matches(" work").trim();
        if !term.is_empty() && term != "it" {
            return Some(term.to_owned());
        }
    }
    None
}

/// Extract the first meaningful topic word/phrase from a prior assistant reply.
/// Looks for "Term (category):" patterns first, then the first capitalised token.
fn extract_topic_from_prior_reply(reply: &str) -> Option<String> {
    // Match "Term (category): description" — common in concept_lookup answers.
    let first_line = reply.lines().next().unwrap_or("").trim();
    if let Some(paren_pos) = first_line.find('(') {
        let candidate = first_line[..paren_pos].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_lowercase());
        }
    }
    // Fallback: first capitalised word that is not a stop word.
    let stop_words = [
        "I", "The", "A", "An", "In", "To", "For", "Of", "And", "Or", "Source",
    ];
    for word in reply.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.len() >= 2
            && clean.chars().next().is_some_and(char::is_uppercase)
            && !stop_words.contains(&clean)
        {
            return Some(clean.to_lowercase());
        }
    }
    None
}
