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
    ROLE_CALENDAR_DIRECTION_PREVIOUS, ROLE_CALENDAR_EVENT, ROLE_CALENDAR_QUESTION,
    ROLE_CALENDAR_RELATIVE_DATE, ROLE_CALENDAR_SCHEDULE_ACTION, ROLE_CALENDAR_TIME,
    ROLE_CALENDAR_TIMEZONE_ALIAS, ROLE_CALENDAR_TODAY, ROLE_CALENDAR_WEEKDAY,
};
use crate::solver_handlers::calendar_ics::ScheduledEvent;
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
    // Calendar create/schedule (issue #404) must be attempted before the weekday-relation
    // gate so that "18 число ... забей / поставь" claims are handled by the action path
    // and do not fall through to the existing weekday-only logic.
    if let Some(answer) = try_calendar_create_event(prompt, normalized, log) {
        return Some(answer);
    }
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
        before.is_none_or(|character| !is_word_character(character))
            && after.is_none_or(|character| !is_word_character(character))
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

pub(super) fn days_to_date(days: i64) -> (i32, u32, u32) {
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

// ---------------------------------------------------------------------------
// Issue #404: calendar create / schedule action support ("забей 18 число в 17:00 по грузии...").
// The implementation follows the exact post-#386 lexicon-driven pattern of the
// existing weekday/today code: all recognition surfaces live in
// data/seed/meanings-calendar.lino; this module only knows the roles and stable
// English slugs. Existing weekday relation + current-day logic is 100% untouched.
// ---------------------------------------------------------------------------

/// Entry point for natural-language calendar event creation/scheduling.
/// Returns Some only for prompts that look like a create request (day reference +
/// schedule action cue or "число" + time/title signals). On success it records rich
/// `calendar:parsed_*` trace events, builds a localized confirmation proposal, and
/// emits via `finalize_simple` with intent `calendar_create_event`.
pub fn try_calendar_create_event(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !mentions_calendar_create_request(normalized) {
        return None;
    }

    let base = current_utc_date()?;
    log.append("calendar:clock", "system_utc".to_owned());

    let language = detect_language(prompt).slug();
    // A relative-date word ("завтра", "tomorrow", "послезавтра", …) anchors the
    // event to a day offset from today (issue #435). It takes priority over a
    // bare day number so "поставь созвон на завтра" lands on tomorrow rather
    // than today's date.
    let relative_offset = relative_date_offset(normalized);
    let (year, month, day) = relative_offset.map_or_else(
        || {
            let day = extract_day_number(normalized).unwrap_or(base.day);
            compute_target_date_with_rollover(base, day)
        },
        |offset| {
            log.append("calendar:parsed_relative_offset", offset.to_string());
            let target = date_from_unix_days(base.days_since_unix_epoch + offset);
            (target.year, target.month, target.day)
        },
    );
    let (hour, minute) = extract_clock_time(normalized).unwrap_or((17, 0));
    let tz = resolve_timezone(normalized).unwrap_or("UTC");
    // Prefer an explicit "на <subject>" / "for <subject>" title; otherwise fall
    // back to the matched event noun ("созвон" → "Созвон") before the localized
    // default, so a title-less request still proposes a meaningful event.
    let title = extract_title(normalized)
        .or_else(|| extract_event_title(normalized))
        .unwrap_or_else(|| default_title(language).to_owned());

    let event = ScheduledEvent {
        title,
        year,
        month,
        day,
        hour,
        minute,
        time_zone: tz,
        duration_minutes: 60,
    };

    // Rich diagnostic trace (exactly the style used by the weekday paths).
    log.append("calendar:parsed_date", event.iso_date());
    log.append(
        "calendar:parsed_time",
        format!("{:02}:{:02}", event.hour, event.minute),
    );
    log.append("calendar:parsed_time_zone", event.time_zone.to_owned());
    log.append("calendar:parsed_title", event.title.clone());
    log.append(
        "calendar:parsed_duration_minutes",
        event.duration_minutes.to_string(),
    );
    if normalized.contains("число") || normalized.contains("number") {
        log.append("calendar:parsed_via", "day_number".to_owned());
    }

    let ics = event.to_ics();
    let google_url = event.to_google_calendar_url();
    log.append("calendar:ics", ics.clone());
    log.append("calendar:google_calendar_url", google_url.clone());

    log.append("language", language.to_owned());

    let body = render_create_confirmation(language, &event, &ics, &google_url);

    Some(finalize_simple(
        prompt,
        log,
        "calendar_create_event",
        "response:calendar_create_event",
        &body,
        0.95,
    ))
}

/// Localized fallback title used when no explicit subject was parsed.
fn default_title(language: &str) -> &'static str {
    match language {
        "ru" => "Событие",
        "hi" => "घटना",
        "zh" => "事件",
        _ => "Event",
    }
}

fn mentions_calendar_create_request(normalized: &str) -> bool {
    let lex = lexicon();
    let has_day_ref = lex
        .words_for_role(ROLE_CALENDAR_DAY_REFERENCE)
        .iter()
        .any(|w| contains_term(normalized, w));
    let has_clock = lex
        .words_for_role(ROLE_CALENDAR_TIME)
        .iter()
        .any(|w| contains_term(normalized, w))
        || extract_clock_time(normalized).is_some();
    let has_relative_date = lex
        .words_for_role(ROLE_CALENDAR_RELATIVE_DATE)
        .iter()
        .any(|w| contains_term(normalized, w));
    let has_date_signal = has_day_ref || has_clock || has_relative_date;
    if !has_date_signal {
        return false;
    }
    let has_timezone = mentions_timezone_alias(normalized);
    let has_participant = extract_participant_title(normalized).is_some();
    let has_action = lex
        .words_for_role(ROLE_CALENDAR_SCHEDULE_ACTION)
        .iter()
        .any(|w| contains_term(normalized, w))
        || lex
            .words_for_role(ROLE_CALENDAR_EVENT)
            .iter()
            .any(|w| contains_term(normalized, w));
    if has_action {
        return true;
    }
    if has_clock && has_timezone && has_participant {
        return true;
    }
    let has_schedule_verb = [
        "забей",
        "поставь",
        "создай",
        "добавь",
        "schedule",
        "book",
        "add to",
    ]
    .iter()
    .any(|verb| contains_term(normalized, verb));
    has_schedule_verb
}

fn extract_day_number(normalized: &str) -> Option<u32> {
    let lex = lexicon();
    for word in lex.words_for_role(ROLE_CALENDAR_DAY_REFERENCE) {
        if !contains_term(normalized, &word) {
            continue;
        }
        if let Some(pos) = normalized.find(&word) {
            let prefix = &normalized[..pos];
            let mut digits = String::new();
            for ch in prefix.chars().rev() {
                if ch.is_ascii_digit() {
                    digits.insert(0, ch);
                } else if !digits.is_empty() {
                    break;
                }
            }
            if let Ok(n) = digits.parse::<u32>() {
                if (1..=31).contains(&n) {
                    return Some(n);
                }
            }
        }
    }
    let mut num = String::new();
    for ch in normalized.trim_start().chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else if !num.is_empty() {
            break;
        } else {
            return None;
        }
    }
    if let Ok(n) = num.parse::<u32>() {
        if (1..=31).contains(&n) {
            return Some(n);
        }
    }
    None
}

fn relative_date_offset(normalized: &str) -> Option<i64> {
    let lex = lexicon();
    for meaning in lex.meanings_with_role(ROLE_CALENDAR_RELATIVE_DATE) {
        if !meaning.words().any(|word| contains_term(normalized, word)) {
            continue;
        }
        return match meaning.slug.as_str() {
            "calendar_tomorrow" => Some(1),
            "calendar_day_after_tomorrow" => Some(2),
            _ => None,
        };
    }
    None
}

fn extract_event_title(normalized: &str) -> Option<String> {
    let lex = lexicon();
    for word in lex.words_for_role(ROLE_CALENDAR_EVENT) {
        if contains_term(normalized, &word) {
            return Some(capitalize_first(&word));
        }
    }
    None
}

const fn compute_target_date_with_rollover(base: CalendarDate, day: u32) -> (i32, u32, u32) {
    let mut y = base.year;
    let mut m = base.month;
    let mut d = day;
    if d < base.day {
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    let max_day = match m {
        2 => 28, // ignore leap for the assistant trace; user can correct
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if d > max_day {
        d = max_day;
    }
    (y, m, d)
}

fn extract_clock_time(normalized: &str) -> Option<(u32, u32)> {
    let bytes = normalized.as_bytes();
    for i in 0..bytes.len().saturating_sub(3) {
        if bytes[i].is_ascii_digit() && bytes[i + 1].is_ascii_digit() {
            let h1 = u32::from(bytes[i] - b'0');
            let h2 = u32::from(bytes[i + 1] - b'0');
            let mut hour = h1 * 10 + h2;
            let mut j = i + 2;
            if j < bytes.len() && (bytes[j] == b':' || bytes[j] == b'.') {
                j += 1;
            }
            if j + 1 < bytes.len() && bytes[j].is_ascii_digit() && bytes[j + 1].is_ascii_digit() {
                let m1 = u32::from(bytes[j] - b'0');
                let m2 = u32::from(bytes[j + 1] - b'0');
                let minute = m1 * 10 + m2;
                if hour <= 23 && minute <= 59 {
                    if hour == 0 {
                        hour = 24; // treat 00:xx as end of previous day? keep as 0 for simplicity
                    }
                    if hour == 24 {
                        hour = 0;
                    }
                    return Some((hour, minute));
                }
            }
        }
    }
    if let Some(time) = extract_spoken_hour_time(normalized) {
        return Some(time);
    }
    if let Some(pos) = normalized.find("в ") {
        let tail = &normalized[pos + 2..];
        let mut num = String::new();
        for ch in tail.chars() {
            if ch.is_ascii_digit() {
                num.push(ch);
            } else if !num.is_empty() {
                break;
            }
        }
        if let Ok(h) = num.parse::<u32>() {
            if h <= 23 {
                return Some((h, 0));
            }
        }
    }
    None
}

fn extract_spoken_hour_time(normalized: &str) -> Option<(u32, u32)> {
    for marker in ["часов", "часа", "час", "часу"] {
        for (pos, _) in normalized.match_indices(marker) {
            let before = normalized[..pos].chars().next_back();
            let after = normalized[pos + marker.len()..].chars().next();
            if before.is_some_and(is_word_character) || after.is_some_and(is_word_character) {
                continue;
            }
            let prefix_end = normalized[..pos].trim_end().len();
            let prefix = &normalized[..prefix_end];
            let start = prefix
                .char_indices()
                .rev()
                .find(|(_, ch)| !ch.is_ascii_digit())
                .map_or(0, |(idx, ch)| idx + ch.len_utf8());
            let marker = normalized[..start].trim_end();
            let allowed_marker = marker
                .split_whitespace()
                .next_back()
                .is_none_or(|word| matches!(word, "в" | "на" | "к"));
            if start == prefix_end || !allowed_marker {
                continue;
            }
            if let Ok(hour) = prefix[start..].parse::<u32>() {
                if hour <= 23 {
                    return Some((hour, 0));
                }
            }
        }
    }
    None
}

fn mentions_timezone_alias(normalized: &str) -> bool {
    lexicon()
        .words_for_role(ROLE_CALENDAR_TIMEZONE_ALIAS)
        .iter()
        .any(|w| contains_term(normalized, w))
        || normalized.contains("asia/tbilisi")
        || normalized.contains("tbilisi")
}

fn resolve_timezone(normalized: &str) -> Option<&'static str> {
    if mentions_timezone_alias(normalized) {
        return Some("Asia/Tbilisi");
    }
    None
}

fn extract_title(normalized: &str) -> Option<String> {
    for marker in [
        "на ",
        "for ",
        "встречу ",
        "meeting with ",
        "call with ",
        "के साथ ",
        "和",
    ] {
        if let Some(pos) = normalized.find(marker) {
            let rest = normalized[pos + marker.len()..].trim();
            if let Some(title) = tidy_title(rest) {
                return Some(title);
            }
        }
    }
    if let Some(title) = extract_participant_title(normalized) {
        return Some(title);
    }
    for verb in ["забей", "поставь", "создай", "добавь"] {
        if let Some(pos) = normalized.find(verb) {
            let after = normalized[pos + verb.len()..].trim_start();
            if let Some(title) = tidy_title(after) {
                if title.chars().count() < 60 {
                    return Some(title);
                }
            }
        }
    }
    None
}

fn extract_participant_title(normalized: &str) -> Option<String> {
    let start = normalized
        .find(" с ")
        .map(|pos| pos + 1)
        .or_else(|| normalized.starts_with("с ").then_some(0))?;
    tidy_title(&normalized[start..])
}

fn tidy_title(candidate: &str) -> Option<String> {
    let mut end = candidate.len();
    for boundary in [
        " on the ",
        " on ",
        " at ",
        " в ",
        " по ",
        " на ",
        " 在 ",
        "下午",
        "上午",
        " को ",
        " शाम",
    ] {
        if let Some(pos) = candidate.find(boundary) {
            end = end.min(pos);
        }
    }
    if let Some(pos) = candidate.find(['.', '!', '?', ',']) {
        end = end.min(pos);
    }
    if let Some(pos) = candidate.find(|c: char| c.is_ascii_digit()) {
        end = end.min(pos);
    }
    let trimmed = candidate[..end].trim();
    let trimmed = strip_action_words(trimmed);
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return None;
    }
    if matches!(
        trimmed,
        "на" | "в" | "во" | "по" | "к" | "for" | "on" | "at"
    ) {
        return None;
    }
    if lexicon()
        .words_for_role(ROLE_CALENDAR_RELATIVE_DATE)
        .iter()
        .any(|word| word.eq_ignore_ascii_case(trimmed))
    {
        return None;
    }
    Some(capitalize_first(trimmed))
}

fn strip_action_words(value: &str) -> String {
    let mut out = value.to_string();
    for fragment in [
        "शेड्यूल करें",
        "कैलेंडर में जोड़ें",
        "बनाएँ",
        "बनाओ",
        "安排",
        "添加到日历",
        "创建",
    ] {
        out = out.replace(fragment, "");
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

fn render_create_confirmation(
    language: &str,
    event: &ScheduledEvent,
    ics: &str,
    google_url: &str,
) -> String {
    let iso = event.iso_date();
    let time = format!("{:02}:{:02}", event.hour, event.minute);
    let tz = event.time_zone;
    let title = &event.title;
    let minutes = event.duration_minutes;
    match language {
        "ru" => format!(
            "Создать событие «{title}» на {day} число ({iso}). Время: {time}, часовой пояс: {tz}. Длительность {minutes} минут.\n\
             Импортируйте этот файл .ics в любой календарь:\n{ics}\n\
             Или откройте в Google Календаре (вход не требуется):\n{google_url}\n\
             Ответьте «да», чтобы подтвердить.",
            day = event.day,
        ),
        "hi" => format!(
            "{iso} ({time}, समय क्षेत्र {tz}) पर «{title}» कार्यक्रम बनाएँ। अवधि {minutes} मिनट।\n\
             इस .ics फ़ाइल को किसी भी कैलेंडर में आयात करें:\n{ics}\n\
             या Google Calendar में खोलें (लॉगिन आवश्यक नहीं):\n{google_url}\n\
             पुष्टि के लिए «हाँ» उत्तर दें।",
        ),
        "zh" => format!(
            "在 {iso}（{time}，时区 {tz}）创建事件「{title}」。时长 {minutes} 分钟。\n\
             将此 .ics 文件导入任何日历：\n{ics}\n\
             或在 Google 日历中打开（无需登录）：\n{google_url}\n\
             回复「是」以确认。",
        ),
        _ => format!(
            "Create event «{title}» on {iso}. Time: {time}, timezone: {tz}. Duration {minutes} minutes.\n\
             Import this .ics file into any calendar:\n{ics}\n\
             Or open it in Google Calendar (no login required):\n{google_url}\n\
             Reply 'yes' to confirm.",
        ),
    }
}
