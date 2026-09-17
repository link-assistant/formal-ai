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

fn documented_calendar_answer(
    timestamp: u64,
    language: &str,
    title: &str,
    encoded_title: &str,
    hour: u32,
    timezone: &str,
    encoded_timezone: &str,
) -> String {
    let seconds = i64::try_from(timestamp).expect("test clock fits i64");
    let today = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = utc_date(today);
    let stamp = format!(
        "{year:04}{month:02}{day:02}T{:02}{:02}{:02}Z",
        second_of_day / 3_600,
        (second_of_day % 3_600) / 60,
        second_of_day % 60,
    );
    let iso = format!("{year:04}-{month:02}-{day:02}");
    let start = format!("{year:04}{month:02}{day:02}T{hour:02}0000");
    let end = format!("{year:04}{month:02}{day:02}T{:02}0000", hour + 1);
    let ics = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//formal-ai//calendar//EN\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nBEGIN:VEVENT\r\nUID:{start}-{timezone}@formal-ai\r\nDTSTAMP:{stamp}\r\nDTSTART;TZID={timezone}:{start}\r\nDTEND;TZID={timezone}:{end}\r\nSUMMARY:{title}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    );
    let google_url = format!(
        "https://calendar.google.com/calendar/render?action=TEMPLATE&text={encoded_title}&dates={start}/{end}&ctz={encoded_timezone}"
    );
    match language {
        "ru" => format!(
            "Создать событие «{title}» на {day} число ({iso}). Время: {hour:02}:00, часовой пояс: {timezone}. Длительность 60 минут.\nИмпортируйте этот файл .ics в любой календарь:\n{ics}\nИли откройте в Google Календаре (вход не требуется):\n{google_url}\nОтветьте «да», чтобы подтвердить."
        ),
        "en" => format!(
            "Create event «{title}» on {iso}. Time: {hour:02}:00, timezone: {timezone}. Duration 60 minutes.\nImport this .ics file into any calendar:\n{ics}\nOr open it in Google Calendar (no login required):\n{google_url}\nReply 'yes' to confirm."
        ),
        _ => unreachable!("these exact examples document Russian and English"),
    }
}

#[test]
fn issue_595_russian_spoken_hour_calendar_prompts_are_scheduled() {
    let cases = [
        (
            "А можешь на 10 часов по Грузии с Марией?",
            "С марией",
            "10:00",
            "Asia/Tbilisi",
        ),
        (
            "Создай встречу на 10 часов с Марией",
            "С марией",
            "10:00",
            "UTC",
        ),
        ("Встречу с Марией на 10 часов", "С марией", "10:00", "UTC"),
        (
            "Поставь мне встречу с Леваном на 5 часов по Грузии",
            "С леваном",
            "05:00",
            "Asia/Tbilisi",
        ),
    ];

    for (prompt, title, time, timezone) in cases {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_secs();
        let response = answer(prompt);
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_secs();
        if prompt == "А можешь на 10 часов по Грузии с Марией?" {
            let documented_answers = (before..=after)
                .map(|timestamp| {
                    documented_calendar_answer(
                        timestamp,
                        "ru",
                        "С марией",
                        "%D0%A1%20%D0%BC%D0%B0%D1%80%D0%B8%D0%B5%D0%B9",
                        10,
                        "Asia/Tbilisi",
                        "Asia%2FTbilisi",
                    )
                })
                .collect::<Vec<_>>();
            assert!(documented_answers.contains(&response.answer));
        }
        assert_eq!(
            response.intent, "calendar_create_event",
            "{prompt:?} should create a calendar event, got {} -> {}",
            response.intent, response.answer
        );
        assert!(
            response.answer.contains(title),
            "{prompt:?} should title the event {title:?}; got: {}",
            response.answer
        );
        assert!(
            response.answer.contains(time),
            "{prompt:?} should preserve spoken hour as {time}; got: {}",
            response.answer
        );
        assert!(
            response.answer.contains(timezone),
            "{prompt:?} should use timezone {timezone}; got: {}",
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

#[test]
fn issue_595_calendar_create_path_keeps_supported_language_coverage() {
    let cases = [
        (
            "English clock create",
            "schedule a meeting with Maria at 10:00",
            "Maria",
            "10:00",
        ),
        (
            "Hindi clock create",
            "कल शाम 5 बजे मीटिंग शेड्यूल करें",
            "मीटिंग",
            "17:00",
        ),
        (
            "Chinese clock create",
            "明天下午5点安排会议",
            "会议",
            "17:00",
        ),
    ];

    for (label, prompt, title, time) in cases {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_secs();
        let response = answer(prompt);
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_secs();
        if label == "English clock create" {
            let documented_answers = (before..=after)
                .map(|timestamp| {
                    documented_calendar_answer(timestamp, "en", "Maria", "Maria", 10, "UTC", "UTC")
                })
                .collect::<Vec<_>>();
            assert!(documented_answers.contains(&response.answer));
        }
        assert_eq!(
            response.intent, "calendar_create_event",
            "{label}: {prompt:?} should create a calendar event, got {} -> {}",
            response.intent, response.answer
        );
        assert!(
            response.answer.contains(title),
            "{label}: {prompt:?} should title the event {title:?}; got: {}",
            response.answer
        );
        assert!(
            response.answer.contains(time),
            "{label}: {prompt:?} should use time {time}; got: {}",
            response.answer
        );
    }
}
