//! Calendar and weekday relation reasoning.
//!
//! This handler keeps date-like questions inside the symbolic solver when the
//! answer can be derived from a stable calendar relation instead of an external
//! clock or lookup.

use super::calendar_create::try_calendar_create_event;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::rule_interpreter::handler_table_value;
use crate::seed::{
    ROLE_CALENDAR_DAY_REFERENCE, ROLE_CALENDAR_DIRECTION_NEXT, ROLE_CALENDAR_DIRECTION_PREVIOUS,
    ROLE_CALENDAR_QUESTION, ROLE_CALENDAR_TODAY, ROLE_CALENDAR_WEEKDAY, fill_template_once,
    lexicon, response_for,
};
use crate::solver_handlers::finalize_simple;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod date_weekday;
pub mod month;

/// The seed table of weekday names by language and direction case.
const WEEKDAY_LABEL_TABLE: &str = "calendar_weekday_label";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
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

    /// Shift by a signed day count. `rem_euclid` keeps negative offsets
    /// ("30 days before Friday") inside the cycle without a manual wrap, so
    /// every offset a prompt can state lands on a real weekday (issue #1176).
    const fn shifted_by(self, days: i64) -> Self {
        // `rem_euclid(7)` is always in 0..7, so each residue maps to its
        // `usize` without a cast.
        let offset: usize = match days.rem_euclid(7) {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            _ => 6,
        };
        Self::from_index(self.index() + offset)
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

    /// The weekday's name in `form`: a language slug, or a language slug and
    /// the direction case a phrase takes (`ru.next`, `ru.previous`).
    ///
    /// The names are rows of the `calendar_weekday_label` table of
    /// `data/seed/handler-rules.lino` (issue #918), so case is data as well.
    fn label(self, form: &str) -> Option<&'static str> {
        handler_table_value(WEEKDAY_LABEL_TABLE, &format!("{}.{form}", self.slug()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeekdayOperation {
    Next,
    Previous,
}

impl WeekdayOperation {
    /// The signed day count of the bare next/previous reading.
    const fn sign(self) -> i64 {
        match self {
            Self::Next => 1,
            Self::Previous => -1,
        }
    }

    const fn delta(self) -> &'static str {
        match self {
            Self::Next => "+1",
            Self::Previous => "-1",
        }
    }

    /// The direction slug the weekday-label table keys a case form by.
    const fn slug(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Previous => "previous",
        }
    }

    /// The seeded response that phrases the bare one-day shift.
    const fn relation_intent(self) -> &'static str {
        match self {
            Self::Next => "calendar_weekday_relation_next",
            Self::Previous => "calendar_weekday_relation_previous",
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
pub(super) struct CalendarDate {
    pub(super) year: i32,
    pub(super) month: u32,
    pub(super) day: u32,
    pub(super) days_since_unix_epoch: i64,
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
    // Issue #1176: a month offset ("2 months after January", "3 months after
    // Monday") is read before the weekday gate, since a month name carries no
    // day reference and a weekday plus months has no fixed day count.
    if let Some(answer) = month::try_month_offset(prompt, normalized, log) {
        return Some(answer);
    }
    // R1017: "What day of the week was 2024-02-29?" names a date, not a
    // weekday, and is answered by counting days from the epoch.
    if let Some(answer) = date_weekday::try_date_weekday(prompt, normalized, log) {
        return Some(answer);
    }
    if !mentions_weekday_context(normalized) {
        return None;
    }
    let operation = detect_operation(normalized)?;
    let source = detect_weekday(normalized)?;
    let language = detect_language(prompt).slug();
    // Issue #1176: read the offset the prompt states ("100 days after",
    // "2 weeks before") instead of always shifting by one. A bare "the day
    // after X" states no offset and keeps the original ±1 reading.
    let offset = detect_offset(normalized, language);
    let signed = offset.map_or_else(
        || operation.sign(),
        |offset| operation.sign() * offset.total(),
    );
    let result = source.shifted_by(signed);

    log.append(
        "calendar:cycle",
        "monday,tuesday,wednesday,thursday,friday,saturday,sunday",
    );
    log.append("calendar:subject_weekday", source.slug());
    log.append(operation.event_kind(), source.slug());
    if let Some(offset) = offset {
        log.append("calendar:offset", format!("{signed:+}d"));
        let total = offset.total().unsigned_abs();
        log.append(
            "calendar:offset_derivation",
            // Evidence-log notation, deliberately language-neutral: the
            // user-facing derivation comes from the localized response
            // template, not from this diagnostic line.
            format!("{total}d = {}w + {}d", total / 7, total % 7),
        );
    }
    log.append("calendar:result_weekday", result.slug());

    log.append("language", language.to_owned());
    let body = match offset {
        Some(offset) if offset.total() > 1 => {
            render_offset_answer(language, operation, source, result, offset)
        }
        _ => render_answer(language, operation, source, result),
    };
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

pub fn mentions_current_day_question(normalized: &str) -> bool {
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

pub fn detect_weekday(normalized: &str) -> Option<Weekday> {
    // Walk the `calendar_weekday` meanings in cycle order (Monday … Sunday) and
    // return the first whose surface words appear as a standalone term, mapping
    // its slug back to a position. The words live in the lexicon, per language.
    lexicon()
        .meanings_with_role(ROLE_CALENDAR_WEEKDAY)
        .filter(|meaning| meaning.words().any(|word| contains_term(normalized, word)))
        .find_map(|meaning| Weekday::from_slug(&meaning.slug))
}

/// The unit a stated offset counts: days (×1) or weeks (×7). The surfaces come
/// from the `calendar_day` / `calendar_week` meanings, so the vocabulary stays
/// in the seed; only the multiplier is calendar arithmetic (issue #1176).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OffsetUnit {
    Days,
    Weeks,
}

impl OffsetUnit {
    const fn multiplier(self) -> i64 {
        match self {
            Self::Days => 1,
            Self::Weeks => 7,
        }
    }

    /// The lexicon meaning slug that carries this unit's surfaces.
    const fn meaning_slug(self) -> &'static str {
        match self {
            Self::Days => "calendar_day",
            Self::Weeks => "calendar_week",
        }
    }
}

/// An offset a prompt states before a weekday shift: "100 days", "2 weeks".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Offset {
    count: i64,
    unit: OffsetUnit,
}

impl Offset {
    const fn total(self) -> i64 {
        self.count * self.unit.multiplier()
    }
}

/// Read the offset a prompt states as `<numeral> days/weeks` — "100 days
/// after Monday", "2 weeks before Friday" (issue #1176). Spelled-out numerals
/// resolve through the seed cardinal tables inside the quantity extractor, so
/// "two weeks" and "две недели" read the same as "2 weeks". Returns `None`
/// when the prompt states no offset, which keeps the bare "the day after X"
/// on its original ±1 reading.
fn detect_offset(normalized: &str, language: &str) -> Option<Offset> {
    let lex = lexicon();
    let mut units: Vec<(OffsetUnit, String)> = [OffsetUnit::Days, OffsetUnit::Weeks]
        .into_iter()
        .filter_map(|unit| {
            let meaning = lex.meaning(unit.meaning_slug())?;
            Some(meaning.words().map(move |word| (unit, word.to_owned())))
        })
        .flatten()
        .collect();
    // Longest surfaces first so "weeks" is not read as the tail of "week".
    units.sort_by_key(|unit| std::cmp::Reverse(unit.1.len()));
    stated_count_with_unit(normalized, language, &units)
        .map(|(count, unit, _)| Offset { count, unit })
}

/// The first stated `<numeral> <unit surface>` pair in the prompt: the count,
/// the unit tag and the surface that matched. `units` is ordered longest
/// surface first by the caller. Shared by the day/week reader above and the
/// month reader in `calendar/month.rs` (issue #1176).
fn stated_count_with_unit<'units, U: Copy>(
    normalized: &str,
    language: &str,
    units: &'units [(U, String)],
) -> Option<(i64, U, &'units str)> {
    for quantity in crate::verifiable_task::quantities::extract_quantities(normalized, language) {
        let Ok(count) = quantity.value.parse::<i64>() else {
            continue;
        };
        // A numeral far past any calendar horizon is not an offset; skip it
        // rather than answer a nonsense magnitude (12 digits ≈ 27 million years).
        if count.unsigned_abs() > 999_999_999_999 {
            continue;
        }
        // The quantity value is normalized ("two" → "2"), so a spelled token
        // can be longer than its digits; walk to the end of the actual token.
        // CJK unit characters end a numeral token, so "100天" reads as the
        // numeral 100 followed by the day surface 天.
        let token_end = normalized[quantity.offset..]
            .find(|character: char| !is_numeral_token_character(character))
            .map_or(normalized.len(), |gap| quantity.offset + gap);
        let tail = normalized[token_end..].trim_start();
        for (unit, surface) in units {
            if starts_with_term(tail, surface) {
                return Some((count, *unit, surface.as_str()));
            }
        }
    }
    None
}

/// Numeral tokens are letters and digits, except CJK unit characters, which
/// start the unit word instead of continuing the numeral.
fn is_numeral_token_character(character: char) -> bool {
    character.is_alphanumeric() && !is_cjk_character(character)
}

/// Does `text` begin with `needle` as a standalone term? CJK surfaces match as
/// a prefix (they carry no word boundaries); other scripts require the next
/// character after the match to end the word.
fn starts_with_term(text: &str, needle: &str) -> bool {
    if needle.chars().any(is_cjk_character) {
        return text.starts_with(needle);
    }
    if !text.starts_with(needle) {
        return false;
    }
    text[needle.len()..]
        .chars()
        .next()
        .is_none_or(|character| !is_word_character(character))
}

pub(super) fn contains_term(haystack: &str, needle: &str) -> bool {
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

/// Byte offset of the earliest occurrence of `needle` in `haystack` — the
/// positional sibling of [`contains_term`]. Presence is decided by
/// `contains_term` (word boundaries, CJK substrings); the offset is the first
/// raw occurrence, which is close enough for the proximity questions callers
/// ask ("which number stands next to 'each'?", issue #1176).
pub(super) fn term_position(haystack: &str, needle: &str) -> Option<usize> {
    if !contains_term(haystack, needle) {
        return None;
    }
    haystack
        .match_indices(needle)
        .next()
        .map(|(start, _)| start)
}

fn is_cjk_character(character: char) -> bool {
    (0x4E00..=0x9FFF).contains(&u32::from(character))
}

pub(super) fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

pub(super) fn current_utc_date() -> Option<CalendarDate> {
    let seconds_since_epoch =
        i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs()).ok()?;
    Some(date_from_unix_days(seconds_since_epoch.div_euclid(86_400)))
}

pub(super) fn date_from_unix_days(days_since_unix_epoch: i64) -> CalendarDate {
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

/// The seeded template of `intent` in `language`, or the English one when the
/// seed carries none for it, paired with the language its weekday names take.
fn template_for<'language>(intent: &str, language: &'language str) -> (&'language str, String) {
    response_for(intent, language).map_or_else(
        || ("en", response_for(intent, "en").unwrap_or_default()),
        |template| (language, template),
    )
}

fn render_current_day_answer(
    language: &str,
    weekday: Weekday,
    iso_date: &str,
    time_zone: &str,
) -> String {
    let (language, template) = template_for("calendar_current_day", language);
    fill_template_once(
        &template,
        &[
            ("weekday", weekday_label(language, weekday)),
            ("date", iso_date),
            ("time_zone", time_zone),
        ],
    )
}

/// Render the bare one-day shift from the seeded
/// `calendar_weekday_relation_*` responses (issue #918).
fn render_answer(
    language: &str,
    operation: WeekdayOperation,
    source: Weekday,
    result: Weekday,
) -> String {
    let (language, template) = template_for(operation.relation_intent(), language);
    fill_template_once(
        &template,
        &[
            (
                "source",
                weekday_direction_label(language, operation, source),
            ),
            ("result", weekday_label(language, result)),
            ("source_plain", weekday_label(language, source)),
            ("delta", operation.delta()),
        ],
    )
}

/// The weekday's plain name for answer prose: the seeded name in `language`,
/// else the English one (issue #918).
fn weekday_label(language: &str, weekday: Weekday) -> &'static str {
    weekday
        .label(language)
        .or_else(|| weekday.label("en"))
        .unwrap_or(weekday.slug())
}

/// The weekday in the case its direction phrase needs (Russian takes the
/// genitive after "после" and the instrumental after "перед"); a language
/// whose seed rows carry no case form uses the plain name.
fn weekday_direction_label(
    language: &str,
    operation: WeekdayOperation,
    weekday: Weekday,
) -> &'static str {
    weekday
        .label(&format!("{language}.{}", operation.slug()))
        .unwrap_or_else(|| weekday_label(language, weekday))
}

/// Render the answer for a stated offset (issue #1176). The prose lives in
/// `data/seed/multilingual-responses-quantities.lino` as
/// `calendar_weekday_offset_*` templates, so the derivation sentence —
/// "100 days = 14 weeks + 2 days, and Monday + 2 days = Wednesday" — ships in
/// every supported language from the seed instead of Rust string literals.
fn render_offset_answer(
    language: &str,
    operation: WeekdayOperation,
    source: Weekday,
    result: Weekday,
    offset: Offset,
) -> String {
    let total = offset.total().unsigned_abs();
    let weeks = total / 7;
    let days = total % 7;
    // A whole number of weeks cannot move the weekday, and the answer says so
    // instead of deriving a shift of zero days; "1 week" gets its own template
    // so the English stays grammatical (issue #1176).
    let intent = match (operation, days) {
        (WeekdayOperation::Next, 0) if weeks == 1 => "calendar_weekday_offset_exact_one_next",
        (WeekdayOperation::Next, 0) => "calendar_weekday_offset_exact_next",
        (WeekdayOperation::Previous, 0) if weeks == 1 => {
            "calendar_weekday_offset_exact_one_previous"
        }
        (WeekdayOperation::Previous, 0) => "calendar_weekday_offset_exact_previous",
        (WeekdayOperation::Next, _) => "calendar_weekday_offset_split_next",
        (WeekdayOperation::Previous, _) => "calendar_weekday_offset_split_previous",
    };
    let Some(template) = crate::seed::localized_response(intent, language) else {
        // A missing template falls back to the bare ±1 prose rather than an
        // empty answer; the derivation still lives in the evidence log.
        return render_answer(language, operation, source, result);
    };
    template
        .replace("{n}", &total.to_string())
        .replace(concat!("{", "weeks}"), &weeks.to_string())
        .replace(concat!("{", "days}"), &days.to_string())
        .replace(
            concat!("{", "source}"),
            weekday_direction_label(language, operation, source),
        )
        .replace("{source_plain}", weekday_label(language, source))
        .replace(concat!("{", "result}"), weekday_label(language, result))
}
