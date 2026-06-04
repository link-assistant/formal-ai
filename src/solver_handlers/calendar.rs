//! Calendar and weekday relation reasoning.
//!
//! This handler keeps date-like questions inside the symbolic solver when the
//! answer can be derived from a stable calendar relation instead of an external
//! clock or lookup.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    lexicon, ROLE_CALENDAR_DAY_REFERENCE, ROLE_CALENDAR_DIRECTION_NEXT,
    ROLE_CALENDAR_DIRECTION_PREVIOUS, ROLE_CALENDAR_QUESTION, ROLE_CALENDAR_TODAY,
    ROLE_CALENDAR_WEEKDAY,
};
use crate::solver_handlers::finalize_simple;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    const fn index(self) -> usize {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    const fn from_index(index: usize) -> Self {
        match index % 7 {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            _ => Self::Sunday,
        }
    }

    const fn shifted(self, operation: WeekdayOperation) -> Self {
        match operation {
            WeekdayOperation::Next => Self::from_index(self.index() + 1),
            WeekdayOperation::Previous => Self::from_index(self.index() + 6),
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Monday => "monday",
            Self::Tuesday => "tuesday",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
            Self::Saturday => "saturday",
            Self::Sunday => "sunday",
        }
    }

    /// Resolve a `calendar_weekday` meaning slug (its English name) back to a
    /// position in the cycle. The lexicon owns the surface words; this only maps
    /// the stable slug, so adding a language never touches this code.
    fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "monday" => Some(Self::Monday),
            "tuesday" => Some(Self::Tuesday),
            "wednesday" => Some(Self::Wednesday),
            "thursday" => Some(Self::Thursday),
            "friday" => Some(Self::Friday),
            "saturday" => Some(Self::Saturday),
            "sunday" => Some(Self::Sunday),
            _ => None,
        }
    }

    const fn en(self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    const fn ru(self) -> &'static str {
        match self {
            Self::Monday => "понедельник",
            Self::Tuesday => "вторник",
            Self::Wednesday => "среда",
            Self::Thursday => "четверг",
            Self::Friday => "пятница",
            Self::Saturday => "суббота",
            Self::Sunday => "воскресенье",
        }
    }

    const fn ru_genitive(self) -> &'static str {
        match self {
            Self::Monday => "понедельника",
            Self::Tuesday => "вторника",
            Self::Wednesday => "среды",
            Self::Thursday => "четверга",
            Self::Friday => "пятницы",
            Self::Saturday => "субботы",
            Self::Sunday => "воскресенья",
        }
    }

    const fn ru_instrumental(self) -> &'static str {
        match self {
            Self::Monday => "понедельником",
            Self::Tuesday => "вторником",
            Self::Wednesday => "средой",
            Self::Thursday => "четвергом",
            Self::Friday => "пятницей",
            Self::Saturday => "субботой",
            Self::Sunday => "воскресеньем",
        }
    }

    const fn hi(self) -> &'static str {
        match self {
            Self::Monday => "सोमवार",
            Self::Tuesday => "मंगलवार",
            Self::Wednesday => "बुधवार",
            Self::Thursday => "गुरुवार",
            Self::Friday => "शुक्रवार",
            Self::Saturday => "शनिवार",
            Self::Sunday => "रविवार",
        }
    }

    const fn zh(self) -> &'static str {
        match self {
            Self::Monday => "星期一",
            Self::Tuesday => "星期二",
            Self::Wednesday => "星期三",
            Self::Thursday => "星期四",
            Self::Friday => "星期五",
            Self::Saturday => "星期六",
            Self::Sunday => "星期日",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeekdayOperation {
    Next,
    Previous,
}

impl WeekdayOperation {
    const fn delta(self) -> &'static str {
        match self {
            Self::Next => "+1",
            Self::Previous => "-1",
        }
    }

    const fn event_kind(self) -> &'static str {
        match self {
            Self::Next => "calendar:operation:next",
            Self::Previous => "calendar:operation:previous",
        }
    }
}

// Issue #386: the calendar recognition vocabulary is no longer hardcoded here.
// The weekday names, the next/previous direction relations, the today marker,
// day/date/week references, and interrogatives all live as self-describing
// meanings in `data/seed/meanings-calendar.lino`, tagged with the
// `calendar_*` semantic roles. The matching functions below ask the lexicon
// which surface words evidence each role, so the words live once, in the data,
// and translate to every supported language; this code knows only the concepts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CalendarDate {
    year: i32,
    month: u32,
    day: u32,
    days_since_unix_epoch: i64,
}

impl CalendarDate {
    fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

pub fn try_calendar_reasoning(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if mentions_current_day_question(normalized) {
        return try_current_day_reasoning(prompt, log);
    }
    if !mentions_weekday_context(normalized) {
        return None;
    }
    let operation = detect_operation(normalized)?;
    let source = detect_weekday(normalized)?;
    let result = source.shifted(operation);

    log.append(
        "calendar:cycle",
        "monday,tuesday,wednesday,thursday,friday,saturday,sunday",
    );
    log.append("calendar:subject_weekday", source.slug());
    log.append(operation.event_kind(), source.slug());
    log.append("calendar:result_weekday", result.slug());

    let language = detect_language(prompt).slug();
    log.append("language", language.to_owned());
    let body = render_answer(language, operation, source, result);
    Some(finalize_simple(
        prompt,
        log,
        "calendar_weekday_relation",
        "response:calendar_weekday_relation",
        &body,
        1.0,
    ))
}

fn try_current_day_reasoning(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let date = current_utc_date()?;
    let weekday = weekday_for_unix_days(date.days_since_unix_epoch);
    let iso = date.iso();
    log.append("calendar:clock", "system_utc".to_owned());
    log.append("calendar:today", iso.clone());
    log.append("calendar:weekday", weekday.slug());
    log.append("calendar:time_zone", "UTC".to_owned());

    let language = detect_language(prompt).slug();
    log.append("language", language.to_owned());
    let body = render_current_day_answer(language, weekday, &iso, "UTC");
    Some(finalize_simple(
        prompt,
        log,
        "calendar_current_day",
        "response:calendar_current_day",
        &body,
        1.0,
    ))
}

fn mentions_current_day_question(normalized: &str) -> bool {
    let lex = lexicon();
    // A today marker must be present as a standalone word (CJK substring).
    let mentions_today = lex
        .words_for_role(ROLE_CALENDAR_TODAY)
        .iter()
        .any(|word| contains_term(normalized, word));
    if !mentions_today {
        return false;
    }

    // …referring to a day/date/week…
    let asks_for_day = lex
        .words_for_role(ROLE_CALENDAR_DAY_REFERENCE)
        .iter()
        .any(|word| contains_term(normalized, word));
    // …phrased as a question. Interrogatives match as a raw substring (so a
    // trailing "?" or "что" inside a longer word still counts), matching the
    // original behaviour.
    let question_like = lex
        .words_for_role(ROLE_CALENDAR_QUESTION)
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    asks_for_day && question_like
}

fn mentions_weekday_context(normalized: &str) -> bool {
    lexicon()
        .words_for_role(ROLE_CALENDAR_DAY_REFERENCE)
        .iter()
        .any(|word| contains_term(normalized, word))
}

fn detect_operation(normalized: &str) -> Option<WeekdayOperation> {
    let lex = lexicon();
    // Direction markers match as raw substrings: many are multi-word phrases
    // ("comes after", "наступает после") and inflected forms that should match
    // inside a larger run, exactly as the previous hardcoded lists did.
    let has_next = lex
        .words_for_role(ROLE_CALENDAR_DIRECTION_NEXT)
        .iter()
        .any(|marker| normalized.contains(marker.as_str()));
    let has_previous = lex
        .words_for_role(ROLE_CALENDAR_DIRECTION_PREVIOUS)
        .iter()
        .any(|marker| normalized.contains(marker.as_str()));
    match (has_next, has_previous) {
        (true, false) => Some(WeekdayOperation::Next),
        (false, true) => Some(WeekdayOperation::Previous),
        _ => None,
    }
}

fn detect_weekday(normalized: &str) -> Option<Weekday> {
    // Walk the `calendar_weekday` meanings in cycle order (Monday … Sunday) and
    // return the first whose surface words appear as a standalone term, mapping
    // its slug back to a position. The words live in the lexicon, per language.
    lexicon()
        .meanings_with_role(ROLE_CALENDAR_WEEKDAY)
        .filter(|meaning| meaning.words().any(|word| contains_term(normalized, word)))
        .find_map(|meaning| Weekday::from_slug(&meaning.slug))
}

fn contains_term(haystack: &str, needle: &str) -> bool {
    if needle.chars().any(is_cjk_character) {
        return haystack.contains(needle);
    }
    haystack.match_indices(needle).any(|(start, _)| {
        let before = haystack[..start].chars().next_back();
        let after = haystack[start + needle.len()..].chars().next();
        before.map_or(true, |character| !is_word_character(character))
            && after.map_or(true, |character| !is_word_character(character))
    })
}

fn is_cjk_character(character: char) -> bool {
    (0x4E00..=0x9FFF).contains(&u32::from(character))
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn current_utc_date() -> Option<CalendarDate> {
    let seconds_since_epoch =
        i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs()).ok()?;
    Some(date_from_unix_days(seconds_since_epoch.div_euclid(86_400)))
}

fn date_from_unix_days(days_since_unix_epoch: i64) -> CalendarDate {
    let (year, month, day) = days_to_date(days_since_unix_epoch);
    CalendarDate {
        year,
        month,
        day,
        days_since_unix_epoch,
    }
}

fn days_to_date(days: i64) -> (i32, u32, u32) {
    // Algorithm adapted from civil-from-days (Howard Hinnant, public domain).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(month <= 2);
    (
        i32::try_from(year).expect("civil date year fits i32"),
        u32::try_from(month).expect("civil date month fits u32"),
        u32::try_from(day).expect("civil date day fits u32"),
    )
}

const fn weekday_for_unix_days(days_since_unix_epoch: i64) -> Weekday {
    match (days_since_unix_epoch + 3).rem_euclid(7) {
        0 => Weekday::Monday,
        1 => Weekday::Tuesday,
        2 => Weekday::Wednesday,
        3 => Weekday::Thursday,
        4 => Weekday::Friday,
        5 => Weekday::Saturday,
        _ => Weekday::Sunday,
    }
}

fn render_current_day_answer(
    language: &str,
    weekday: Weekday,
    iso_date: &str,
    time_zone: &str,
) -> String {
    match language {
        "ru" => format!("Сегодня {}, {iso_date} ({time_zone}).", weekday.ru()),
        "hi" => format!("आज {} है, {iso_date} ({time_zone}).", weekday.hi()),
        "zh" => format!("今天是{}，{iso_date}（{time_zone}）。", weekday.zh()),
        _ => format!("Today is {}, {iso_date} ({time_zone}).", weekday.en()),
    }
}

fn render_answer(
    language: &str,
    operation: WeekdayOperation,
    source: Weekday,
    result: Weekday,
) -> String {
    match language {
        "ru" => match operation {
            WeekdayOperation::Next => format!(
                "После {} наступает {}. Я сдвинул {} на {} в семидневном календарном цикле.",
                source.ru_genitive(),
                result.ru(),
                source.ru(),
                operation.delta(),
            ),
            WeekdayOperation::Previous => format!(
                "Перед {} идёт {}. Я сдвинул {} на {} в семидневном календарном цикле.",
                source.ru_instrumental(),
                result.ru(),
                source.ru(),
                operation.delta(),
            ),
        },
        "hi" => match operation {
            WeekdayOperation::Next => format!(
                "{} के बाद {} आता है। मैं सात दिनों के कैलेंडर चक्र में {} को {} दिन सरकाता हूँ।",
                source.hi(),
                result.hi(),
                source.hi(),
                operation.delta(),
            ),
            WeekdayOperation::Previous => format!(
                "{} से पहले {} आता है। मैं सात दिनों के कैलेंडर चक्र में {} को {} दिन सरकाता हूँ।",
                source.hi(),
                result.hi(),
                source.hi(),
                operation.delta(),
            ),
        },
        "zh" => match operation {
            WeekdayOperation::Next => format!(
                "{}之后是{}。我在七天的日历循环中将{}移动{}天。",
                source.zh(),
                result.zh(),
                source.zh(),
                operation.delta(),
            ),
            WeekdayOperation::Previous => format!(
                "{}之前是{}。我在七天的日历循环中将{}移动{}天。",
                source.zh(),
                result.zh(),
                source.zh(),
                operation.delta(),
            ),
        },
        _ => match operation {
            WeekdayOperation::Next => format!(
                "The day after {} is {}. I move {} by {} in the seven-day calendar cycle.",
                source.en(),
                result.en(),
                source.en(),
                operation.delta(),
            ),
            WeekdayOperation::Previous => format!(
                "The day before {} is {}. I move {} by {} in the seven-day calendar cycle.",
                source.en(),
                result.en(),
                source.en(),
                operation.delta(),
            ),
        },
    }
}
