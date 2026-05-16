//! Policy and edge-case handlers extracted from `solver_handlers.rs` to keep
//! that module under the 1000-line cap enforced by
//! `scripts/check-file-size.rs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;

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
