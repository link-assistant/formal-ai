//! Calendar event request tests for issue #404.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn has_evidence(response: &SymbolicAnswer, expected: &str) -> bool {
    response
        .evidence_links
        .iter()
        .any(|link| link.starts_with(expected))
}

#[test]
fn calendar_reasoning_drafts_reported_event_request() {
    let response = answer("Забей мне 18 число в 17:00 по грузии на встречу с Леваном");
    assert_eq!(
        response.intent, "calendar_event_request",
        "reported prompt must route to calendar event planning, got {}: {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("17:00"),
        "calendar event draft must preserve the requested time: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("Asia/Tbilisi"),
        "calendar event draft must resolve Georgia time to Asia/Tbilisi: {}",
        response.answer,
    );
    assert!(
        response.answer.to_lowercase().contains("встречу с леваном"),
        "calendar event draft must extract the event title: {}",
        response.answer,
    );
    assert!(
        has_evidence(&response, "calendar:export:ics"),
        "calendar event planning must expose the offline calendar-file path: {:?}",
        response.evidence_links,
    );
    assert!(
        has_evidence(&response, "calendar:confirmation_required"),
        "calendar event planning must require confirmation before writing a calendar: {:?}",
        response.evidence_links,
    );
}

#[test]
fn calendar_reasoning_drafts_event_requests_across_supported_languages() {
    struct Case {
        prompt: &'static str,
        language: &'static str,
    }

    let cases = [
        Case {
            prompt: "Add to calendar on the 18th at 17:00 Georgia time for meeting with Levan",
            language: "en",
        },
        Case {
            prompt: "Забей мне 18 число в 17:00 по грузии на встречу с Леваном",
            language: "ru",
        },
        Case {
            prompt: "18 तारीख को 17:00 बजे जॉर्जिया समय पर लेवान के साथ बैठक कैलेंडर में जोड़ो",
            language: "hi",
        },
        Case {
            prompt: "把18号17:00格鲁吉亚时间和Levan的会议加到日历",
            language: "zh",
        },
    ];

    for case in cases {
        let prompt = case.prompt;
        let language_tag = format!("language:{}", case.language);
        let response = answer(prompt);
        assert_eq!(
            response.intent, "calendar_event_request",
            "prompt {prompt:?} must draft a calendar event, got {}: {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.contains("17:00"),
            "calendar event draft must preserve time for {prompt:?}: {}",
            response.answer,
        );
        assert!(
            response.answer.contains("Asia/Tbilisi"),
            "calendar event draft must resolve Georgia time for {prompt:?}: {}",
            response.answer,
        );
        assert!(
            response.answer.contains(".ics"),
            "calendar event draft must mention portable calendar export for {prompt:?}: {}",
            response.answer,
        );
        assert!(
            has_evidence(&response, "calendar:confirmation_required"),
            "calendar event draft must require confirmation for {prompt:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &language_tag),
            "calendar event draft must record {language_tag} for {prompt:?}: {:?}",
            response.evidence_links,
        );
    }
}
