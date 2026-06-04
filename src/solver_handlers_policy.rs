//! Policy and edge-case handlers extracted from `solver_handlers.rs` to keep
//! that module under the 1000-line cap enforced by
//! `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::solver_handlers::finalize_simple;

/// Physical-action questions directed at the AI (issue #39).
///
/// When a user asks whether formal-ai performed a physical action (e.g.
/// «Сосал?» — "Did you suck?"), the AI can answer factually: it has no
/// physical body and therefore never performed any such action.  Treating
/// these as inappropriate content and refusing would be both unhelpful and
/// technically wrong.  A short, factual "No." (in the surface language) is
/// the correct response.
///
/// Recognition is data-driven: the trigger surfaces live in
/// `data/seed/meanings-policy.lino` under the
/// [`physical_action_trigger`](seed::ROLE_PHYSICAL_ACTION_TRIGGER) role, matched
/// as raw substrings so inflected forms in any supported language are caught
/// without listing words in code. Content-policy screening runs before this
/// handler, so a surface that is also flagged as vulgar is refused first. The
/// reply is localized through [`seed::response_for`], falling back to Russian.
pub fn try_physical_action_question(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !seed::lexicon().mentions_role_raw(seed::ROLE_PHYSICAL_ACTION_TRIGGER, normalized) {
        return None;
    }

    let language = detect_language(prompt);
    let body = seed::response_for("physical_action_question", language.slug())
        .or_else(|| seed::response_for("physical_action_question", "ru"))
        .unwrap_or_else(|| String::from("Нет. У меня нет физического тела."));
    Some(finalize_simple(
        prompt,
        log,
        "physical_action_question",
        "response:physical_action_question",
        &body,
        1.0,
    ))
}

/// «Купи слона» — Russian circular-joke idiom (issue #41).
///
/// The canonical opening line of a children's folk game: the listener is
/// supposed to refuse, then the requester re-uses their words to keep the
/// cycle going. Recognising it as a known idiom prevents the solver from
/// falling through to the generic "unknown" catch-all.
///
/// Recognition is data-driven: the idiom surfaces live in
/// `data/seed/meanings-policy.lino` under the
/// [`circular_joke_phrase`](seed::ROLE_CIRCULAR_JOKE_PHRASE) role (the calque
/// "buy an elephant" in every supported language), matched as raw substrings.
/// The reply is localized through [`seed::response_for`], falling back to the
/// canonical Russian explanation.
pub fn try_kupi_slona(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !seed::lexicon().mentions_role_raw(seed::ROLE_CIRCULAR_JOKE_PHRASE, normalized) {
        return None;
    }

    let language = detect_language(prompt);
    let body = seed::response_for("kupi_slona", language.slug())
        .or_else(|| seed::response_for("kupi_slona", "ru"))
        .unwrap_or_else(|| {
            String::from(
                "«Купи слона» — это известная русская детская фраза-игра. \
                 На любой ответ следует продолжение: «Все так говорят, а ты купи слона!» \
                 Правильный ответ по правилам игры: «У всех есть слон, а у меня нет».",
            )
        });
    Some(finalize_simple(
        prompt,
        log,
        "kupi_slona",
        "response:kupi_slona",
        &body,
        1.0,
    ))
}
