//! Seed-driven recognition and acknowledgement of dialog-control turns.

use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::solver_handlers::finalize_simple;

/// Whether a prompt establishes a dialog preference or corrects an
/// unauthorized mutation. These turns stay local instead of becoming plans.
#[must_use]
pub fn is_conversation_control_prompt(prompt: &str) -> bool {
    let normalized = normalize_prompt(prompt);
    conversation_preference_term(prompt, &normalized).is_some()
        || is_unauthorized_mutation_correction(&normalized)
}

pub fn try_conversation_control(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(normalized);
    if let Some(term) = conversation_preference_term(prompt, &normalized) {
        log.append("conversation_preference:avoid_term", term.clone());
        let body = seed::render_response(
            "conversation_preference",
            detect_language(prompt).slug(),
            &[("term", &term)],
        )?;
        return Some(finalize_simple(
            prompt,
            log,
            "conversation_preference",
            "response:conversation_preference",
            &body,
            1.0,
        ));
    }
    if is_unauthorized_mutation_correction(&normalized) {
        log.append("action_correction:unauthorized_mutation", prompt.to_owned());
        let body = seed::localized_response("action_correction", detect_language(prompt).slug())?;
        return Some(finalize_simple(
            prompt,
            log,
            "action_correction",
            "response:action_correction",
            &body,
            1.0,
        ));
    }
    None
}

fn conversation_preference_term(prompt: &str, normalized: &str) -> Option<String> {
    let asks_to_avoid =
        direct_role_surface_present(seed::ROLE_CONVERSATION_PREFERENCE_AVOID, normalized);
    if !asks_to_avoid {
        return None;
    }
    prompt
        .split('`')
        .nth(1)
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(str::to_owned)
}

fn is_unauthorized_mutation_correction(normalized: &str) -> bool {
    direct_role_surface_present(seed::ROLE_UNAUTHORIZED_MUTATION_CORRECTION, normalized)
}

fn direct_role_surface_present(role: &str, normalized: &str) -> bool {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .any(|form| !form.text.is_empty() && normalized.contains(&form.text))
}
