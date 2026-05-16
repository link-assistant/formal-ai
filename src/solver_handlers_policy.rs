//! Policy and edge-case handlers extracted from `solver_handlers.rs` to keep
//! that module under the 1000-line cap enforced by
//! `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;

/// Physical-action questions directed at the AI (issue #39).
///
/// When a user asks whether formal-ai performed a physical action (e.g.
/// «Сосал?» — "Did you suck?"), the AI can answer factually: it has no
/// physical body and therefore never performed any such action.  Treating
/// these as inappropriate content and refusing would be both unhelpful and
/// technically wrong.  A short, factual "No." (in the surface language) is
/// the correct response.
pub fn try_physical_action_question(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // Russian past-tense verbs describing physical oral actions asked in
    // second-person / interrogative form.  The question is about the AI's
    // body, not an insult — formal-ai simply has no physical body, so "No."
    // is factually correct.
    let ru_physical_triggers: &[&str] = &["сосал", "сосала", "сосёшь", "соси", "сосать"];
    let is_russian_physical = ru_physical_triggers.iter().any(|w| normalized.contains(w));

    if !is_russian_physical {
        return None;
    }

    let body = "Нет. У меня нет физического тела.";
    Some(finalize_simple(
        prompt,
        log,
        "physical_action_question",
        "response:physical_action_question",
        body,
        1.0,
    ))
}

/// «Купи слона» — Russian circular-joke idiom (issue #41).
///
/// The canonical opening line of a children's folk game: the listener is
/// supposed to refuse, then the requester re-uses their words to keep the
/// cycle going. Recognising it as a known idiom prevents the solver from
/// falling through to the generic "unknown" catch-all.
pub fn try_kupi_slona(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("купи слона") {
        return None;
    }
    let body = String::from(
        "«Купи слона» — это известная русская детская фраза-игра. \
         На любой ответ следует продолжение: «Все так говорят, а ты купи слона!» \
         Правильный ответ по правилам игры: «У всех есть слон, а у меня нет».",
    );
    Some(finalize_simple(
        prompt,
        log,
        "kupi_slona",
        "response:kupi_slona",
        &body,
        1.0,
    ))
}
