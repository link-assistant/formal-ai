use formal_ai::FormalAiEngine;

fn answer(prompt: &str) -> formal_ai::SymbolicAnswer {
    FormalAiEngine.answer(prompt)
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
    let response = answer("Можешь поставить мне созвон в кальндарь на завтра?");
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
// across languages: a bare "tomorrow"/"明天" plus a schedule/event cue is enough.
#[test]
fn issue_435_relative_tomorrow_multilingual() {
    for (prompt, title) in [
        ("поставь созвон на завтра", "Созвон"),
        ("schedule a call for tomorrow", "Call"),
        ("明天安排一个通话", "通话"),
    ] {
        let response = answer(prompt);
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
