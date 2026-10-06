//! Month offsets for the calendar relation handler (issue #1176).
//!
//! "2 months after January" is exact arithmetic in the twelve-month cycle:
//! January is month 1, 1 + 2 = 3, and month 3 is March. "3 months after
//! Monday" is not: a calendar month runs 28 to 31 days, so a month offset is
//! no fixed number of days and the weekday it lands on depends on the starting
//! date. That case is answered with a clarification that names why, showing
//! the day span the stated months can cover, instead of a guessed weekday.
//!
//! The unit surfaces come from the `calendar_month` meaning, the month names
//! from the `calendar_month_name` meanings (both in
//! `data/seed/meanings-calendar.lino`), and the prose from the
//! `calendar_month_offset_*` and `calendar_weekday_month_offset_unresolved_*`
//! templates in `data/seed/multilingual-responses-quantities.lino`.

use super::{
    Weekday, WeekdayOperation, contains_term, detect_operation, detect_weekday,
    stated_count_with_unit, weekday_direction_label,
};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{ROLE_CALENDAR_MONTH_NAME, lexicon, localized_response};
use crate::solver_handlers::finalize_simple;

/// The lexicon meaning carrying the month unit surfaces (month, месяц, 个月…).
const MONTH_UNIT_SLUG: &str = "calendar_month";

/// The `calendar_month_name` meaning slugs in calendar order; a slug's
/// position plus one is its month number.
const MONTH_SLUGS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// Months in the calendar cycle.
const MONTHS_PER_YEAR: i64 = 12;

/// Answer a stated month offset: modulo-12 arithmetic from a month name, or a
/// clarification from a weekday. Returns `None` when the prompt states no
/// month offset, no direction, or neither a weekday nor a month name.
pub(super) fn try_month_offset(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lex = lexicon();
    let mut units: Vec<((), String)> = lex
        .meaning(MONTH_UNIT_SLUG)?
        .words()
        .map(|word| ((), word.to_owned()))
        .collect();
    // Longest surfaces first so "months" is not read as the tail of "month".
    units.sort_by_key(|unit| std::cmp::Reverse(unit.1.len()));
    let language = detect_language(prompt).slug();
    let (count, (), unit) = stated_count_with_unit(normalized, language, &units)?;
    let operation = detect_operation(normalized)?;
    let signed = operation.sign() * count;
    // A weekday plus months has no single answer; checked first so a stray
    // month-like word ("may") never turns that question into month arithmetic.
    if let Some(weekday) = detect_weekday(normalized) {
        return Some(render_unresolved(
            prompt, log, language, operation, weekday, count, unit,
        ));
    }
    let source = detect_month(normalized)?;
    let sum = source + signed;
    let result = (sum - 1).rem_euclid(MONTHS_PER_YEAR) + 1;
    let source_slug = month_slug(source)?;
    let result_slug = month_slug(result)?;
    let sign = if operation.sign() < 0 { "-" } else { "+" };

    log.append("calendar:subject_month", source_slug);
    log.append(operation.event_kind(), source_slug);
    log.append("calendar:offset", format!("{signed:+}m"));
    log.append(
        "calendar:offset_derivation",
        // Evidence-log notation, deliberately language-neutral.
        format!("{source} {sign} {count} = {sum} ≡ {result} (mod 12)"),
    );
    log.append("calendar:result_month", result_slug);
    log.append("language", language.to_owned());

    let intent = match operation {
        WeekdayOperation::Next => "calendar_month_offset_next",
        WeekdayOperation::Previous => "calendar_month_offset_previous",
    };
    let template = localized_response(intent, language)?;
    let body = template
        .replace("{n}", &count.to_string())
        .replace(concat!("{", "unit}"), unit)
        .replace(concat!("{", "source}"), month_label(source_slug, language))
        .replace(concat!("{", "source_index}"), &source.to_string())
        .replace(concat!("{", "sum}"), &sum.to_string())
        .replace(concat!("{", "result_index}"), &result.to_string())
        .replace(concat!("{", "result}"), month_label(result_slug, language));
    Some(finalize_simple(
        prompt,
        log,
        "calendar_month_relation",
        "response:calendar_month_relation",
        &body,
        1.0,
    ))
}

/// The clarification for a month offset from a weekday: the stated months
/// cover a range of day counts, so no single weekday follows.
fn render_unresolved(
    prompt: &str,
    log: &mut EventLog,
    language: &str,
    operation: WeekdayOperation,
    weekday: Weekday,
    count: i64,
    unit: &str,
) -> SymbolicAnswer {
    let signed = operation.sign() * count;
    let (min_days, max_days) = month_span_days(count);
    log.append("calendar:subject_weekday", weekday.slug());
    log.append(operation.event_kind(), weekday.slug());
    log.append("calendar:offset", format!("{signed:+}m"));
    log.append("calendar:offset_span", format!("{min_days}d..{max_days}d"));
    log.append("calendar:offset_unresolved", MONTH_UNIT_SLUG);
    log.append("language", language.to_owned());
    let intent = match operation {
        WeekdayOperation::Next => "calendar_weekday_month_offset_unresolved_next",
        WeekdayOperation::Previous => "calendar_weekday_month_offset_unresolved_previous",
    };
    let body = localized_response(intent, language)
        .map(|template| {
            template
                .replace("{n}", &count.to_string())
                .replace(concat!("{", "unit}"), unit)
                .replace(
                    concat!("{", "source}"),
                    weekday_direction_label(language, operation, weekday),
                )
                .replace(concat!("{", "min_days}"), &min_days.to_string())
                .replace(concat!("{", "max_days}"), &max_days.to_string())
        })
        .unwrap_or_default();
    finalize_simple(
        prompt,
        log,
        "calendar_month_offset_clarification",
        "response:calendar_month_offset_clarification",
        &body,
        1.0,
    )
}

/// The month number (1–12) a prompt names. Every `calendar_month_name`
/// surface is tried and the longest match wins, so "十一月" reads as November
/// rather than as the "一月" inside it; ties keep the earlier month.
fn detect_month(normalized: &str) -> Option<i64> {
    let mut best: Option<(usize, i64)> = None;
    for meaning in lexicon().meanings_with_role(ROLE_CALENDAR_MONTH_NAME) {
        let Some(position) = MONTH_SLUGS.iter().position(|slug| *slug == meaning.slug) else {
            continue;
        };
        for word in meaning.words() {
            let longer = best.is_none_or(|(length, _)| word.len() > length);
            if longer && contains_term(normalized, word) {
                best = Some((word.len(), i64::try_from(position).ok()? + 1));
            }
        }
    }
    best.map(|(_, month)| month)
}

/// The slug of month number `month` (1–12).
fn month_slug(month: i64) -> Option<&'static str> {
    let index = usize::try_from(month - 1).ok()?;
    MONTH_SLUGS.get(index).copied()
}

/// The month's name in the answer language: the first surface of its seed
/// lexeme (falling back to English), so month names stay seed data.
fn month_label(slug: &str, language: &str) -> &'static str {
    lexicon()
        .meaning(slug)
        .and_then(|meaning| meaning.word_in(language).or_else(|| meaning.word_in("en")))
        .unwrap_or(MONTH_UNIT_SLUG)
}

/// The fewest and most days `months` consecutive calendar months can span,
/// measured from the first of every month across one full 400-year Gregorian
/// cycle (which repeats exactly, so it covers every leap-year pattern).
fn month_span_days(months: i64) -> (i64, i64) {
    const FIRST_YEAR: i64 = 2000;
    const CYCLE_YEARS: i64 = 400;
    let mut min_days = i64::MAX;
    let mut max_days = i64::MIN;
    for year in FIRST_YEAR..FIRST_YEAR + CYCLE_YEARS {
        for month in 0..MONTHS_PER_YEAR {
            let start = days_from_civil(year, month + 1, 1);
            let total = month + months;
            let end = days_from_civil(
                year + total / MONTHS_PER_YEAR,
                total % MONTHS_PER_YEAR + 1,
                1,
            );
            min_days = min_days.min(end - start);
            max_days = max_days.max(end - start);
        }
    }
    (min_days, max_days)
}

/// Days since 1970-01-01 of a proleptic Gregorian date (days-from-civil,
/// Howard Hinnant, public domain); the inverse of `days_to_date`. Callers pass
/// years from 2000 on, so every division here is on non-negative values.
const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = shifted_year / 400;
    let year_of_era = shifted_year - era * 400;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}
