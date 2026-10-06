//! The weekday a stated calendar date falls on (R1017).
//!
//! "What day of the week was 2024-02-29?" is pure calendar arithmetic: the
//! date is day 19 782 counted from 1970-01-01 (a Thursday), 19 782 = 7 × 2 826
//! + 0, so it is a Thursday. The question is recognised by the
//! `calendar_weekday_query` role ("day of the week", "weekday", "какой день",
//! 星期几…) and the month names by `calendar_month_name`, both in
//! `data/seed/meanings-calendar.lino`; the prose is the `calendar_date_weekday`
//! template in `data/seed/multilingual-responses-quantities.lino`. A prompt
//! that also names a direction ("3 days after 2024-02-29") is date offset
//! arithmetic, not this question, and is declined.
//!
//! Two date shapes are read from the raw prompt (the dispatch-normalized form
//! folds the ISO separators away): the ISO-like `YYYY-MM-DD` (also with `/` or
//! `.` as the one separator), and a month name with a one- or two-digit day
//! and a four-digit year in either order ("25 December 2025", "December 25,
//! 2025"). Any other number in the prompt declines the question.

use super::month::{days_from_civil, detect_month};
use super::{contains_term, weekday_for_unix_days, weekday_label};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    ROLE_CALENDAR_DIRECTION_NEXT, ROLE_CALENDAR_DIRECTION_PREVIOUS, ROLE_CALENDAR_WEEKDAY_QUERY,
    lexicon, localized_response,
};
use crate::solver_handlers::finalize_simple;

/// A run of ASCII digits in the prompt: its byte span and value.
#[derive(Debug, Clone, Copy)]
struct DigitRun {
    start: usize,
    end: usize,
    value: i64,
}

impl DigitRun {
    const fn width(self) -> usize {
        self.end - self.start
    }
}

/// Answer "which weekday is <date>?" or decline with `None`.
pub(super) fn try_date_weekday(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lex = lexicon();
    let asks_weekday = lex
        .words_for_role(ROLE_CALENDAR_WEEKDAY_QUERY)
        .iter()
        .any(|word| contains_term(normalized, word));
    if !asks_weekday || mentions_direction(normalized) {
        return None;
    }
    let (year, month, day) = stated_date(prompt, normalized)?;
    let days = days_from_civil(year, month, day);
    let weekday = weekday_for_unix_days(days);
    let epoch_weekday = weekday_for_unix_days(0);
    let date = format!("{year:04}-{month:02}-{day:02}");
    let weeks = days.div_euclid(7);
    let rest = days.rem_euclid(7);
    let language = detect_language(prompt).slug();

    log.append("calendar:date", date.clone());
    log.append("calendar:epoch", "1970-01-01".to_owned());
    log.append("calendar:days_since_epoch", days.to_string());
    log.append(
        "calendar:offset_derivation",
        format!("{days} = 7 × {weeks} + {rest}"),
    );
    log.append("calendar:result_weekday", weekday.slug());
    log.append("language", language.to_owned());

    let template = localized_response("calendar_date_weekday", language)?;
    let body = template
        .replace(concat!("{", "date}"), &date)
        .replace(
            concat!("{", "epoch_weekday}"),
            weekday_label(language, epoch_weekday),
        )
        .replace(concat!("{", "epoch}"), "1970-01-01")
        .replace(concat!("{", "weekday}"), weekday_label(language, weekday))
        .replace(concat!("{", "days}"), &days.to_string())
        .replace(concat!("{", "weeks}"), &weeks.to_string())
        .replace(concat!("{", "rest}"), &rest.to_string());
    Some(finalize_simple(
        prompt,
        log,
        "calendar_date_weekday",
        "response:calendar_date_weekday",
        &body,
        1.0,
    ))
}

/// Does the prompt name a next/previous direction ("after", "до", "之前")? The
/// direction surfaces match as raw substrings, exactly as the weekday-relation
/// reader matches them.
fn mentions_direction(normalized: &str) -> bool {
    let lex = lexicon();
    [
        ROLE_CALENDAR_DIRECTION_NEXT,
        ROLE_CALENDAR_DIRECTION_PREVIOUS,
    ]
    .into_iter()
    .any(|role| {
        lex.words_for_role(role)
            .iter()
            .any(|marker| normalized.contains(marker.as_str()))
    })
}

/// Every run of ASCII digits in `text`, in order.
fn digit_runs(text: &str) -> Vec<DigitRun> {
    let bytes = text.as_bytes();
    let mut runs = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        // A run too long to be a date part is kept with a sentinel value so it
        // still counts as "another number" and declines the question.
        let value = text[start..index].parse::<i64>().unwrap_or(-1);
        runs.push(DigitRun {
            start,
            end: index,
            value,
        });
    }
    runs
}

/// The date the prompt states as `(year, month, day)`, validated against the
/// proleptic Gregorian calendar.
fn stated_date(prompt: &str, normalized: &str) -> Option<(i64, i64, i64)> {
    let runs = digit_runs(prompt);
    let (year, month, day) = if let [first, second, third] = runs.as_slice() {
        iso_date(prompt, *first, *second, *third)?
    } else {
        named_month_date(normalized, &runs)?
    };
    let valid = (1..=9999).contains(&year)
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day);
    valid.then_some((year, month, day))
}

/// `YYYY-MM-DD` with one separator (`-`, `/` or `.`) used twice.
fn iso_date(
    prompt: &str,
    year: DigitRun,
    month: DigitRun,
    day: DigitRun,
) -> Option<(i64, i64, i64)> {
    if year.width() != 4 || !(1..=2).contains(&month.width()) || !(1..=2).contains(&day.width()) {
        return None;
    }
    let first = prompt.get(year.end..month.start)?;
    let second = prompt.get(month.end..day.start)?;
    let separator_ok = first == second && matches!(first, "-" | "/" | ".");
    separator_ok.then_some((year.value, month.value, day.value))
}

/// A month name with exactly one day run (one or two digits) and one year run
/// (four digits), in either order.
fn named_month_date(normalized: &str, runs: &[DigitRun]) -> Option<(i64, i64, i64)> {
    let [first, second] = runs else {
        return None;
    };
    let month = detect_month(normalized)?;
    let (year, day) = match (first.width(), second.width()) {
        (4, 1 | 2) => (first.value, second.value),
        (1 | 2, 4) => (second.value, first.value),
        _ => return None,
    };
    Some((year, month, day))
}

/// Days in `month` of `year`: the distance to the first of the next month.
const fn days_in_month(year: i64, month: i64) -> i64 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    days_from_civil(next_year, next_month, 1) - days_from_civil(year, month, 1)
}
