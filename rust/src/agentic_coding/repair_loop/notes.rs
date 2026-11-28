use super::{
    FailedStep, RepairStop, attempts_from, evidence_document, formalize_diagnostic, search_query,
};
use crate::protocol::ChatMessage;

/// Fill a localized repair template's `{placeholder}` slots.
fn template(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, language).unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// The note appended to the failure report when the ladder is spent (R4).
///
/// The rung reached is named, so the bound is reported honestly rather than
/// silently truncating. Empty when the response seed is not registered, in
/// which case today's report already stands on its own.
#[must_use]
pub fn ladder_note(language: &str, rung: u8, max_rungs: u8) -> String {
    template(
        "repair_ladder_exhausted",
        language,
        &[
            ("rung", &rung.to_string()),
            ("max_rungs", &max_rungs.to_string()),
        ],
    )
}

/// The note appended when no fetched source addresses the diagnostic (R6):
/// an unresolved need in the seed's own words, never a fabricated fix.
#[must_use]
pub fn unresolved_note(language: &str, query: &str) -> String {
    template("repair_unresolved_need", language, &[("query", query)])
}

/// The closing note for a stopped loop, appended to the failure report.
///
/// A spent ladder (R4) or an unmatched diagnostic (R6) is named in the
/// request's language, and the attempt chain follows as the
/// `repair_attempts` evidence record (R5), so the answer carries what the
/// loop tried. Empty when the loop never ran a rung (no diagnostic, no
/// tools): today's failure report then stands on its own.
#[must_use]
pub fn stop_note(
    messages: &[ChatMessage],
    failure: &FailedStep,
    stop: RepairStop,
    ladder_rung: u8,
    max_rungs: u8,
) -> String {
    let language = crate::language::detect(
        &crate::protocol::latest_user_request(messages).unwrap_or_default(),
    )
    .slug();
    let note = match stop {
        RepairStop::Exhausted => ladder_note(language, ladder_rung, max_rungs),
        RepairStop::NoMatch => {
            let Some(primary) = formalize_diagnostic(&failure.language, &failure.reported)
                .into_iter()
                .next()
            else {
                return String::new();
            };
            unresolved_note(language, &search_query(&failure.language, &primary))
        }
        RepairStop::NoDiagnostic | RepairStop::NoTools => return String::new(),
    };
    let attempts = attempts_from(messages, failure);
    if attempts.is_empty() {
        return note;
    }
    format!("{note}\n\n```lino\n{}```", evidence_document(&attempts))
}
