use formal_ai::FormalAiEngine;
use std::time::{SystemTime, UNIX_EPOCH};

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn utc_date(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    (
        i32::try_from(year + i64::from(month <= 2)).expect("year"),
        u32::try_from(month).expect("month"),
        u32::try_from(day).expect("day"),
    )
}

fn expected_relative_call_answer(timestamp: u64) -> String {
    let seconds = i64::try_from(timestamp).expect("test clock fits i64");
    let today = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (stamp_year, stamp_month, stamp_day) = utc_date(today);
    let (year, month, day) = utc_date(today + 1);
    let stamp = format!(
        "{stamp_year:04}{stamp_month:02}{stamp_day:02}T{:02}{:02}{:02}Z",
        second_of_day / 3_600,
        (second_of_day % 3_600) / 60,
        second_of_day % 60,
    );
    let iso = format!("{year:04}-{month:02}-{day:02}");
    let start = format!("{year:04}{month:02}{day:02}T170000");
    let end = format!("{year:04}{month:02}{day:02}T180000");
    format!(
        "Создать событие «Созвон» на {day} число ({iso}). Время: 17:00, часовой пояс: UTC. Длительность 60 минут.\n\
         Импортируйте этот файл .ics в любой календарь:\n\
         BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//formal-ai//calendar//EN\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nBEGIN:VEVENT\r\nUID:{start}-UTC@formal-ai\r\nDTSTAMP:{stamp}\r\nDTSTART;TZID=UTC:{start}\r\nDTEND;TZID=UTC:{end}\r\nSUMMARY:Созвон\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n\n\
         Или откройте в Google Календаре (вход не требуется):\n\
         https://calendar.google.com/calendar/render?action=TEMPLATE&text=%D0%A1%D0%BE%D0%B7%D0%B2%D0%BE%D0%BD&dates={start}/{end}&ctz=UTC\n\
         Ответьте «да», чтобы подтвердить."
    )
}

// Issue #435: "Можешь поставить мне созвон в кальндарь на завтра?" used to fall
// through to the `unknown` intent. The prompt carries no day number and no clock
// time — only a relative-date word ("на завтра") and an event noun ("созвон").
// The calendar create path now recognizes relative-date words as a date anchor,
// resolves "завтра" to tomorrow, and titles the event from the matched event
// noun, so the request produces a structured `calendar_create_event` with a real
// importable `.ics` VEVENT and a no-login Google Calendar render URL.
#[test]
fn issue_435_relative_tomorrow_call_is_scheduled() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_secs();
    let response = answer("Можешь поставить мне созвон в кальндарь на завтра?");
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_secs();
    assert_ne!(
        response.intent, "unknown",
        "relative-date scheduling prompt must not return unknown; got intent={}, answer={}",
        response.intent, response.answer
    );
    assert_eq!(
        response.intent, "calendar_create_event",
        "expected calendar_create_event intent, got {}",
        response.intent
    );
    assert!(
        (before..=after)
            .map(expected_relative_call_answer)
            .any(|answer| answer == response.answer),
        "complete calendar answer differed from the independently rendered clock-bounded fixtures: {}",
        response.answer
    );
    // The event noun "созвон" becomes the title, never the localized default.
    assert!(
        response.answer.contains("Созвон"),
        "title should be derived from the event noun «созвон»; got: {}",
        response.answer
    );
    // Portable artifacts: an RFC 5545 VEVENT plus a no-login render URL.
    assert!(
        response.answer.contains("BEGIN:VCALENDAR") && response.answer.contains("BEGIN:VEVENT"),
        "answer must embed an importable .ics VEVENT; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("calendar.google.com/calendar/render"),
        "answer must offer a no-login Google Calendar render URL; got: {}",
        response.answer
    );
    // The relative offset is traced as evidence (+1 day for "завтра").
    assert!(
        response
            .evidence_links
            .iter()
            .any(|l| l.contains("parsed_relative_offset")),
        "must record the parsed relative-date offset; links={:?}",
        response.evidence_links
    );
    // Rich parsed_* evidence, like the day-number create path.
    let parsed_keys = response
        .evidence_links
        .iter()
        .filter(|l| l.starts_with("calendar:parsed_"))
        .count();
    assert!(
        parsed_keys >= 4,
        "must emit multiple calendar:parsed_* evidence keys; links={:?}",
        response.evidence_links
    );
}

// The same relative-date support must work without an explicit calendar word and
// across every supported language: a bare relative-date word plus a schedule/event
// cue is enough. Coverage spans all four locales so a fix never lands with a
// single-language regression:
//   - English ("en"): "tomorrow" + "schedule a call"
//   - Russian ("ru"): "на завтра" + "созвон"
//   - Hindi ("hi"):   "कल" + "मीटिंग शेड्यूल करें"
//   - Chinese ("zh"): "明天" + "安排...通话"
#[test]
fn issue_435_relative_tomorrow_multilingual() {
    for (prompt, title) in [
        ("поставь созвон на завтра", "Созвон"),
        ("schedule a call for tomorrow", "Call"),
        ("कल मीटिंग शेड्यूल करें", "मीटिंग"),
        ("明天安排一个通话", "通话"),
    ] {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_secs();
        let response = answer(prompt);
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_secs();
        if prompt == "поставь созвон на завтра" {
            assert!(
                (before..=after)
                    .map(expected_relative_call_answer)
                    .any(|answer| answer == response.answer)
            );
        }
        assert_eq!(
            response.intent, "calendar_create_event",
            "{prompt:?} should schedule a create event, got {} -> {}",
            response.intent, response.answer
        );
        assert!(
            response.answer.contains(title),
            "{prompt:?} should title the event {title:?}; got: {}",
            response.answer
        );
        assert!(
            response.answer.contains("BEGIN:VEVENT")
                && response
                    .answer
                    .contains("calendar.google.com/calendar/render"),
            "{prompt:?} must export a .ics + Google Calendar URL; got: {}",
            response.answer
        );
    }
}

// Guard: a relative-date word alone, with no schedule verb and no event noun,
// must not be hijacked into a create request (e.g. a plain question about
// tomorrow's weekday stays on its own path / unknown rather than scheduling).
#[test]
fn issue_435_relative_word_without_action_does_not_schedule() {
    let response = answer("что будет завтра");
    assert_ne!(
        response.intent, "calendar_create_event",
        "a bare relative-date mention with no schedule/event cue must not create an event; got {} -> {}",
        response.intent, response.answer
    );
}
