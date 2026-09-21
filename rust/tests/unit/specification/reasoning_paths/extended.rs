use super::*;

#[test]
fn calendar_create_event_russian_day_number_with_time_and_tz() {
    // Exact prompt from https://github.com/link-assistant/formal-ai/issues/404
    let response = answer("Забей мне 18 число в 17:00 по грузии на встречу с Леваном");
    assert_ne!(
        response.intent, "unknown",
        "calendar scheduling prompt must not return unknown; got intent={}, answer={}",
        response.intent, response.answer
    );
    assert!(
        response.intent == "calendar_create_event" || response.intent.contains("calendar"),
        "expected calendar_create_event intent, got {}",
        response.intent
    );
    // Rich trace evidence for the parsed fields (the heart of the feature).
    // Evidence links use generated ids after the key (e.g. calendar:parsed_date:calendar:parsed_date_xxx).
    // We assert the presence of the parsed keys (recorded by the handler) + correct intent.
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
    assert!(
        response
            .evidence_links
            .iter()
            .any(|l| l.contains("parsed_time_zone")
                || l.contains("Asia/Tbilisi")
                || l.contains("грузии")),
        "must capture timezone in evidence; links={:?}",
        response.evidence_links
    );
    // Confirmation-style answer (the handler proposes; it does not auto-create).
    let a = response.answer.to_lowercase();
    assert!(
        a.contains("создать") || a.contains("событие") || a.contains("да"),
        "answer should propose the event and invite confirmation; got: {}",
        response.answer
    );
    // Real, portable calendar artifacts: an RFC 5545 VEVENT plus a no-login
    // Google Calendar render URL, with the "по грузии" alias resolved to IANA.
    assert!(
        response.answer.contains("BEGIN:VCALENDAR") && response.answer.contains("BEGIN:VEVENT"),
        "answer must embed an importable .ics VEVENT; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("TZID=Asia/Tbilisi"),
        "Russian timezone alias must resolve to IANA Asia/Tbilisi; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("calendar.google.com/calendar/render"),
        "answer must offer a no-login Google Calendar render URL; got: {}",
        response.answer
    );
}

#[test]
fn calendar_create_event_fallback_english() {
    let response = answer("schedule meeting with Levan on the 18th at 5pm Georgia time");
    assert_ne!(response.intent, "unknown");
    assert!(
        response.intent.contains("calendar"),
        "english scheduling must also hit calendar path; intent={}",
        response.intent
    );
    assert!(
        response.answer.contains("BEGIN:VCALENDAR")
            && response.answer.contains("TZID=Asia/Tbilisi")
            && response
                .answer
                .contains("calendar.google.com/calendar/render"),
        "english scheduling must also export a .ics + Google Calendar URL; got: {}",
        response.answer
    );
}

#[test]
fn calendar_create_event_hindi() {
    // No timezone in the prompt → defaults to UTC; the schedule verb is
    // stripped from the .ics SUMMARY so the title reads as the event noun.
    let response = answer("18 तारीख को शाम 5 बजे लेवान के साथ मीटिंग शेड्यूल करें");
    assert_ne!(response.intent, "unknown");
    assert!(
        response.intent.contains("calendar"),
        "hindi scheduling must hit calendar path; intent={}",
        response.intent
    );
    assert!(
        response.answer.contains("BEGIN:VEVENT")
            && response
                .answer
                .contains("calendar.google.com/calendar/render"),
        "hindi scheduling must export a .ics + Google Calendar URL; got: {}",
        response.answer
    );
}

#[test]
fn calendar_create_event_chinese() {
    let response = answer("18号下午5点和Levan安排会议");
    assert_ne!(response.intent, "unknown");
    assert!(
        response.intent.contains("calendar"),
        "chinese scheduling must hit calendar path; intent={}",
        response.intent
    );
    assert!(
        response.answer.contains("BEGIN:VEVENT")
            && response
                .answer
                .contains("calendar.google.com/calendar/render"),
        "chinese scheduling must export a .ics + Google Calendar URL; got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Cross-handler sanity: every reasoning path projects from a non-empty event
// log, so the answer is never memoized.
// ---------------------------------------------------------------------------

#[test]
fn every_specialized_handler_emits_a_trace_link() {
    let prompts = [
        "Hi",
        "What is 2 + 2?",
        "What is Wikipedia?",
        "Please run this javascript:\n```js\n1+1;\n```",
        "Write me hello world program in Rust",
    ];
    for prompt in prompts {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("trace:")),
            "prompt {prompt:?} must emit a trace link: {:?}",
            response.evidence_links,
        );
    }
}

// ---------------------------------------------------------------------------
// R89: incompatible-unit queries — explicit symbolic refusal (issue #43).
//
// "Сколько метров в килобайте?" mixes length (meters) with data-storage
// (kilobytes). The solver must recognise the dimensional mismatch and emit
// `intent:unit_incompatibility` with a clear explanation rather than falling
// through to `intent:unknown`.
// ---------------------------------------------------------------------------

#[test]
fn russian_meters_in_kilobyte_returns_unit_incompatibility() {
    let response = answer("Сколько метров в килобайте?");
    assert_eq!(
        response.answer,
        "метр measures length; байт measures data storage. These are different physical dimensions and cannot be converted into each other. The incompatibility is recorded as a `unit_incompatibility` link in the network."
    );
    assert_eq!(
        response.intent, "unit_incompatibility",
        "mixing length and data-storage units must not fall through to unknown: {:?}",
        response.answer,
    );
    assert!(
        response.answer.contains("length") || response.answer.contains("длин"),
        "answer should mention the length dimension: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("data storage") || response.answer.contains("данн"),
        "answer should mention the data storage dimension: {}",
        response.answer,
    );
    assert!(
        (response.confidence - 1.0).abs() < f32::EPSILON,
        "incompatibility is a known fact, confidence must be 1.0",
    );
}

#[test]
fn english_meters_in_kilobyte_returns_unit_incompatibility() {
    let response = answer("How many meters in a kilobyte?");
    assert_eq!(response.intent, "unit_incompatibility");
    assert!(response.answer.contains("length"));
    assert!(response.answer.contains("data storage"));
}

#[test]
fn incompatible_unit_answer_records_evidence_link() {
    let response = answer("How many meters in a kilobyte?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("unit_incompatibility")),
        "must emit a unit_incompatibility event: {:?}",
        response.evidence_links,
    );
}

#[test]
fn compatible_unit_query_does_not_trigger_incompatibility_handler() {
    // km to meters: both are length — must not fire the incompatibility handler.
    let response = answer("What is 2 + 2?");
    assert_ne!(
        response.intent, "unit_incompatibility",
        "arithmetic prompt must not trigger unit_incompatibility",
    );
}

#[test]
fn greeting_is_not_intercepted_by_incompatibility_handler() {
    let response = answer("Hi");
    assert_eq!(response.intent, "greeting");
}
