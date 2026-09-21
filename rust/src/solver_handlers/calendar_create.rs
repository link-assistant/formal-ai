//! Natural-language calendar event creation (issue #404) and the routed
//! create entry the shared capability-routing table selects.
//!
//! Split from `calendar.rs` so both stay under the 1000-line gate enforced
//! by `scripts/check-file-size.rs`; the weekday-relation and current-day
//! reasoning stayed behind. All recognition surfaces stay in
//! `data/seed/meanings-calendar.lino`; this module knows only the roles and
//! stable English slugs.

use super::calendar::{
    CalendarDate, contains_term, current_utc_date, date_from_unix_days, is_word_character,
};
use crate::engine::SymbolicAnswer;
use crate::entity_resolution::edit_budget;
use crate::event_log::EventLog;
use crate::fuzzy::typo_distance;
use crate::language::detect as detect_language;
use crate::seed::{
    ROLE_CALENDAR_DAY_REFERENCE, ROLE_CALENDAR_EVENT, ROLE_CALENDAR_RELATIVE_DATE,
    ROLE_CALENDAR_SCHEDULE_ACTION, ROLE_CALENDAR_TIME, lexicon,
};
use crate::solver_handlers::calendar_ics::ScheduledEvent;
use crate::solver_handlers::finalize_simple;

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
    // A software-authoring request (authoring verb plus software artifact)
    // that mentions a schedule word is a build plan, not an event: "Build a
    // bot ... that sends weekly notifications" schedules nothing. The claim
    // declines here so the software-project handler answers it (issue #1138
    // software-project corpus).
    if super::software_project_claims(normalized) {
        return None;
    }
    if !mentions_calendar_create_request(normalized) {
        return None;
    }

    try_routed_calendar_create_event(prompt, normalized, log)
}

/// Build the event selected by the shared capability-routing table.
///
/// The ordinary handler above retains its standalone recognizer for legacy
/// callers.  Solver dispatch calls this entry point only after
/// `(time_expression, schedule, dialogue)` has selected
/// `calendar_create_event`, so asking the old verb list a second time would
/// undo the generalized decision for held-out paraphrases.
pub fn try_routed_calendar_create_event(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // Same software-project decline as the standalone recognizer above: a
    // build request that mentions a schedule word is a plan, not an event,
    // whichever entry point the table selected it through.
    if super::software_project_claims(normalized) {
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
    let place = resolve_timezone(normalized);
    let tz = match &place {
        Some(place) => place.zone.as_str(),
        None => "UTC",
    };
    log.append(
        "calendar:timezone_origin",
        match &place {
            Some(place) => format!("entity:{}:{}", place.slug, place.surface),
            None => String::from("default_utc_no_grounded_place"),
        },
    );
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
        time_zone: tz.to_owned(),
        duration_minutes: 60,
    };

    // Rich diagnostic trace (exactly the style used by the weekday paths).
    log.append("calendar:parsed_date", event.iso_date());
    log.append(
        "calendar:parsed_time",
        format!("{:02}:{:02}", event.hour, event.minute),
    );
    log.append("calendar:parsed_time_zone", event.time_zone.clone());
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

/// Whether the calendar handler's own gate accepts this prompt: a date
/// signal backed by entities (clock hour, timezone, participant) or an
/// explicit scheduling act. The capability table asks before it answers a
/// gap with the request (issue #595).
pub fn calendar_claims(normalized: &str) -> bool {
    mentions_calendar_create_request(normalized)
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
    let has_timezone = resolve_timezone(normalized).is_some();
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

    [
        "забей",
        "поставь",
        "создай",
        "добавь",
        "schedule",
        "book",
        "add to",
    ]
    .iter()
    .any(|verb| contains_term(normalized, verb))
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
            if let Ok(n) = digits.parse::<u32>()
                && (1..=31).contains(&n)
            {
                return Some(n);
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
    if let Ok(n) = num.parse::<u32>()
        && (1..=31).contains(&n)
    {
        return Some(n);
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
        if let Ok(h) = num.parse::<u32>()
            && h <= 23
        {
            return Some((h, 0));
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
            if let Ok(hour) = prefix[start..].parse::<u32>()
                && hour <= 23
            {
                return Some((hour, 0));
            }
        }
    }
    None
}

/// One timezone-bearing place the prompt names: the IANA zone its grounding
/// resolves to, the entity slug, and the surface that named it.
struct PlaceZone {
    zone: String,
    slug: String,
    surface: String,
}

/// Resolve the prompt's place mention to an IANA time zone through the entity
/// registry.
///
/// Plan 10 leaf 16 (issue #869): the zone is not a memorized alias table. The
/// registry's entities are grounded in Wikidata, their `timezone` field is the
/// IANA zone the grounding script resolves through the entity's country and
/// the tz database, and a mention is any surface of such an entity the prompt
/// names — exactly, as part of a CJK token (`柏林时间` contains `柏林`), or
/// within the same per-eight-characters edit budget `entity_resolution`
/// corrects remembered names with, so an inflected form (`по Грузии`)
/// resolves without its own seed row. The longest matching surface wins. A
/// place whose grounding yields no single zone anchors nothing, and the
/// caller records which zone it used instead.
fn resolve_timezone(normalized: &str) -> Option<PlaceZone> {
    let prompt = normalize_zone_text(normalized);
    let tokens: Vec<&str> = prompt.split_whitespace().collect();
    let mut best: Option<PlaceZone> = None;
    for entity in crate::seed::entity_names() {
        let Some(zone) = entity.timezone.as_deref() else {
            continue;
        };
        for surface in &entity.surfaces {
            let candidate = normalize_zone_text(surface);
            if candidate.is_empty() {
                continue;
            }
            let mentioned = if candidate.contains(' ') {
                prompt.contains(candidate.as_str())
            } else {
                tokens.iter().any(|token| {
                    **token == *candidate
                        || token.contains(candidate.as_str())
                        || typo_distance(token, &candidate) <= edit_budget(&candidate)
                })
            };
            if mentioned
                && best.as_ref().is_none_or(|current| {
                    surface.chars().count() > current.surface.chars().count()
                })
            {
                best = Some(PlaceZone {
                    zone: zone.to_owned(),
                    slug: entity.slug.clone(),
                    surface: surface.clone(),
                });
            }
        }
    }
    best
}

/// Compare a place surface and the prompt on letters and digits alone, the
/// same shape `entity_resolution` corrects remembered names with.
fn normalize_zone_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut pending_space = false;
    for character in text.to_lowercase().chars() {
        if character.is_alphanumeric() {
            if pending_space && !normalized.is_empty() {
                normalized.push(' ');
            }
            pending_space = false;
            normalized.push(character);
        } else {
            pending_space = true;
        }
    }
    normalized
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
            if let Some(title) = tidy_title(after)
                && title.chars().count() < 60
            {
                return Some(title);
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
    let tz = event.time_zone.as_str();
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
