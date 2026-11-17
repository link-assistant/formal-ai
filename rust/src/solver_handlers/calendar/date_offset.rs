//! Gregorian day offsets with a separate quantity span and date anchor.
//! JavaScript twin: tryCalendarDateOffset in formal_ai_worker_calendar_offset.js.
use super::date_weekday::stated_date;
use super::month::{days_from_civil, month_label, month_slug};
use super::{contains_term, date_from_unix_days, detect_offset};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::rule_interpreter::handler_table_value;
use crate::seed::{lexicon, localized_response};
use crate::solver_handlers::finalize_simple;

pub(crate) fn date_offset_candidate(
    prompt: &str,
    normalized: &str,
) -> Option<(i64, i64, i64, i64, i64)> {
    let lex = lexicon();
    if !lex
        .words_for_role("calendar-date-offset")
        .iter()
        .any(|word| contains_term(normalized, word))
    {
        return None;
    }
    let lowered = prompt.to_lowercase();
    let language = detect_language(prompt).slug();
    let mut candidates = Vec::new();
    for (role, sign) in [
        ("calendar_direction_next", 1_i64),
        ("calendar_direction_previous", -1_i64),
    ] {
        for word in lex.words_for_role(role) {
            let Some(at) = lowered.find(&word) else {
                continue;
            };
            if !contains_term(&lowered, &word) {
                continue;
            }
            let prefix = &lowered[..at];
            let anchor = &lowered[at + word.len()..];
            let Some(offset) = detect_offset(prefix, language) else {
                continue;
            };
            if super::super::numeric_list::parse_numbers(prefix).len() != 1 {
                continue;
            }
            let Some((year, month, day)) = stated_date(anchor, anchor) else {
                continue;
            };
            let days = days_from_civil(year, month, day);
            let Some(signed) = sign.checked_mul(offset.total()) else {
                continue;
            };
            let Some(result_days) = days.checked_add(signed) else {
                continue;
            };
            if !(days_from_civil(1, 1, 1)..days_from_civil(10000, 1, 1)).contains(&result_days) {
                continue;
            }
            candidates.push((year, month, day, signed, result_days));
        }
    }
    candidates.sort_unstable();
    candidates.dedup();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(*candidate)
}

pub(super) fn try_date_offset(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (year, month, day, signed, days) = date_offset_candidate(prompt, normalized)?;
    let language = detect_language(prompt).slug();
    let result = date_from_unix_days(days);
    let iso = result.iso();
    let source = format!("{year:04}-{month:02}-{day:02}");
    let slug = month_slug(i64::from(result.month))?;
    let label = handler_table_value("calendar-date-month-label", &format!("{slug}.{language}"))
        .unwrap_or_else(|| month_label(slug, language));
    let template = localized_response("calendar-date-offset", language)?;
    let body = template
        .replace("{iso}", &iso)
        .replace("{source}", &source)
        .replace("{signed}", &format!("{signed:+}"))
        .replace("{day}", &result.day.to_string())
        .replace("{month}", label)
        .replace("{year}", &result.year.to_string());
    log.append("calendar:date", source.clone());
    log.append("calendar:offset", format!("{signed:+}d"));
    log.append(
        "calendar:offset_derivation",
        format!("{source} {signed:+}d = {iso}"),
    );
    log.append("calendar:result_date", iso);
    Some(finalize_simple(
        prompt,
        log,
        "calendar_date_offset",
        "response:calendar-date-offset",
        &body,
        1.0,
    ))
}
