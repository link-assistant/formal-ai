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
pub fn detect_month(normalized: &str) -> Option<i64> {
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
pub(super) fn month_slug(month: i64) -> Option<&'static str> {
    let index = usize::try_from(month - 1).ok()?;
    MONTH_SLUGS.get(index).copied()
}

/// The month's name in the answer language: the first surface of its seed
/// lexeme (falling back to English), so month names stay seed data.
pub(super) fn month_label(slug: &str, language: &str) -> &'static str {
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
/// years from 1 on (the month-span scan from 2000, the date-weekday reader in
/// `date_weekday.rs` from year 1), so every division here is on non-negative
/// values.
pub(super) const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = shifted_year / 400;
    let year_of_era = shifted_year - era * 400;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Calendar duration sums and completed anniversaries over seeded vocabulary.
pub(crate) mod duration {
    //! Calendar-month arithmetic, mirrored by the JavaScript duration reader.
    //! Quantities bind adjacent units. Completed periods compare anniversaries,
    //! never a fixed average length of a month. Month-only dates have day 1;
    //! mixed date precision is deliberately unresolved.

    use super::super::{contains_term, date_weekday::stated_date};
    use super::detect_month;
    use crate::engine::SymbolicAnswer;
    use crate::event_log::EventLog;
    use crate::language::detect;
    use crate::rule_interpreter::{handler_table_rows, handler_table_value};
    use crate::seed::{lexicon, localized_response};
    use crate::solver_handlers::finalize_simple;

    pub(crate) struct DurationRequest {
        kind: &'static str,
        result: String,
        factor: i64,
        derivation: String,
    }

    fn words(role: &str) -> Vec<String> {
        let mut out: Vec<String> = handler_table_rows(role)
            .iter()
            .flat_map(|(_, value)| value.split('|').map(str::to_owned))
            .collect();
        out.sort_by_key(|word| std::cmp::Reverse(word.len()));
        out
    }

    fn starts(text: &str, word: &str) -> bool {
        text.strip_prefix(word).is_some_and(|tail| {
            tail.chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_')
        })
    }

    fn offsets(text: &str, word: &str) -> Vec<usize> {
        text.match_indices(word)
            .filter_map(|(at, _)| {
                let before = text[..at].chars().next_back();
                (before.is_none_or(|c| !c.is_alphanumeric() && c != '_')
                    && starts(&text[at..], word))
                .then_some(at)
            })
            .collect()
    }

    fn trim(text: &str) -> &str {
        text.trim().trim_end_matches(['.', '?', '!']).trim()
    }

    fn unit(text: &str) -> Option<(String, i64)> {
        let mut units = Vec::new();
        for (slug, factor) in [("calendar_month", 1), ("calendar_year", 12)] {
            for word in if slug == "calendar_year" {
                words("calendar-duration-year")
            } else {
                lexicon()
                    .meaning(slug)?
                    .words()
                    .map(str::to_owned)
                    .collect()
            } {
                units.push((word.to_owned(), factor));
            }
        }
        units.sort_by_key(|(word, _)| std::cmp::Reverse(word.len()));
        units.into_iter().find(|(word, _)| starts(text, word))
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Date {
        year: i64,
        month: i64,
        day: i64,
        precision: bool,
    }

    impl Date {
        fn iso(self) -> String {
            format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
        }
    }

    fn date(text: &str) -> Option<Date> {
        let text = trim(text);
        let months = lexicon()
            .meanings_with_role("calendar_month_name")
            .filter(|meaning| meaning.words().any(|word| contains_term(text, word)))
            .count();
        if months > 1 {
            return None;
        }
        let runs: Vec<&str> = text
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .collect();
        if runs.len() == 1 && runs[0].len() == 4 {
            let year = runs[0].parse().ok()?;
            if !(1..=9999).contains(&year) {
                return None;
            }
            return Some(Date {
                year,
                month: detect_month(text)?,
                day: 1,
                precision: false,
            });
        }
        let (year, month, day) = stated_date(text, text)?;
        Some(Date {
            year,
            month,
            day,
            precision: true,
        })
    }

    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 { a } else { gcd(b, a % b) }
    }

    pub(crate) fn request(prompt: &str) -> Option<DurationRequest> {
        let text = prompt.to_lowercase();
        let text = text.trim();
        if let Some(cue) = words("calendar-duration-sum")
            .into_iter()
            .find(|word| starts(text, word))
        {
            let body = text[cue.len()..].trim();
            for marker in words("calendar-duration-output") {
                for position in offsets(body, &marker) {
                    let target_text = trim(&body[position + marker.len()..]);
                    let Some((surface, factor)) = unit(target_text) else {
                        continue;
                    };
                    if surface != target_text {
                        continue;
                    }
                    let mut rest = trim(&body[..position]);
                    let mut total: i64 = 0;
                    let mut terms = Vec::new();
                    while !rest.is_empty() && terms.len() < 32 {
                        let Some((number, tail)) = rest.split_once(char::is_whitespace) else {
                            break;
                        };
                        let digits = number.trim_start_matches(['+', '-']);
                        if digits.is_empty()
                            || digits.len() > 12
                            || !digits.bytes().all(|c| c.is_ascii_digit())
                        {
                            break;
                        }
                        let count: i64 = number.parse().ok()?;
                        let tail = tail.trim_start();
                        let Some((surface, source)) = unit(tail) else {
                            break;
                        };
                        total = total.checked_add(count.checked_mul(source)?)?;
                        terms.push(format!("{count} × {source}"));
                        rest = tail[surface.len()..].trim();
                        if rest.is_empty() {
                            break;
                        }
                        let mut separated = false;
                        if let Some(tail) = rest.strip_prefix(',') {
                            separated = true;
                            rest = tail.trim_start();
                        }
                        if let Some(join) = words("calendar-duration-join")
                            .into_iter()
                            .find(|word| starts(rest, word))
                        {
                            separated = true;
                            rest = rest[join.len()..].trim_start();
                        }
                        if !separated || rest.is_empty() {
                            rest = "!";
                            break;
                        }
                    }
                    if !rest.is_empty() || terms.len() < 2 {
                        continue;
                    }
                    let common = gcd(total.abs(), factor);
                    let result = if common == factor {
                        (total / factor).to_string()
                    } else {
                        format!("{}/{}", total / common, factor / common)
                    };
                    let derivation = format!(
                        "{} = {total} months; {total} / {factor} = {result}",
                        terms.join(" + ")
                    );
                    return Some(DurationRequest {
                        kind: "calendar-duration-sum",
                        result,
                        factor,
                        derivation,
                    });
                }
            }
            return None;
        }
        if !words("calendar-full-query")
            .iter()
            .any(|word| starts(text, word))
        {
            return None;
        }
        let factor = if words("calendar-full-years")
            .iter()
            .any(|word| contains_term(text, word))
        {
            12
        } else if words("calendar-full-months")
            .iter()
            .any(|word| contains_term(text, word))
        {
            1
        } else {
            return None;
        };
        let mut candidates = Vec::new();
        for start in words("calendar-interval-start") {
            for at in offsets(text, &start) {
                let tail = &text[at + start.len()..];
                for end in words("calendar-interval-end") {
                    for split in offsets(tail, &end) {
                        let (Some(left), Some(right)) =
                            (date(&tail[..split]), date(&tail[split + end.len()..]))
                        else {
                            continue;
                        };
                        if left.precision != right.precision {
                            continue;
                        }
                        let (first, last) = if left <= right {
                            (left, right)
                        } else {
                            (right, left)
                        };
                        let raw = (last.year - first.year) * 12 + last.month - first.month;
                        let incomplete = i64::from(last.day < first.day);
                        let months = raw - incomplete;
                        let result = (months / factor).to_string();
                        let derivation = format!(
                            "{} → {}: {raw} - {incomplete} = {months} months; ⌊{months} / {factor}⌋ = {result}",
                            first.iso(),
                            last.iso()
                        );
                        candidates.push(DurationRequest {
                            kind: "calendar-full-periods",
                            result,
                            factor,
                            derivation,
                        });
                    }
                }
            }
        }
        if candidates.len() == 1 {
            candidates.pop()
        } else {
            None
        }
    }

    fn label(result: &str, factor: i64, language: &str) -> Option<&'static str> {
        let number = result.parse::<i64>().ok().map(i64::abs);
        let class = match number {
            None => "fraction",
            Some(n)
                if if language == "ru" {
                    n % 10 == 1 && n % 100 != 11
                } else {
                    n == 1
                } =>
            {
                "one"
            }
            Some(n) if (2..=4).contains(&(n % 10)) && !(12..=14).contains(&(n % 100)) => "few",
            _ => "other",
        };
        let unit = if factor == 12 { "year" } else { "month" };
        handler_table_value(
            "calendar-duration-label",
            &format!("{unit}.{language}.{class}"),
        )
        .or_else(|| handler_table_value("calendar-duration-label", &format!("{unit}.en.{class}")))
    }

    pub(crate) fn duration(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
        let request = request(prompt)?;
        let language = detect(prompt).slug();
        let body = localized_response(request.kind, language)?
            .replace("{result}", &request.result)
            .replace("{unit}", label(&request.result, request.factor, language)?)
            .replace("{derivation}", &request.derivation);
        log.append("calendar:duration_derivation", request.derivation);
        log.append("calendar:duration_result", request.result);
        Some(finalize_simple(
            prompt,
            log,
            request.kind,
            &format!("response:{}", request.kind),
            &body,
            1.0,
        ))
    }
}
