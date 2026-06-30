use formal_ai::FormalAiEngine;

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
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
        let response = answer(prompt);
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
