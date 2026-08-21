//! The report *format*: title convention and record-safe truncation (#839).
//!
//! Both rules are stated in the issue against real artifacts. §4 fixes the
//! title convention against the titles of issues #826, #827 and #838, and §2.3
//! names `tail -c 12000 | sed '1d'` as the reason #838's attached context could
//! not be read. Each test below fails against the code that filed #838.

use formal_ai::issue_report::{ReportTurn, TitleSettings, issue_title, truncate_records};
use formal_ai::json_lino::lino_to_json;
use serde_json::json;

fn settings() -> TitleSettings {
    TitleSettings {
        prefix: String::from("Formal AI: "),
        default_title: String::from("Formal AI agentic session report"),
    }
}

/// Build a conversation whose final user turn is the one asking for a report.
fn conversation(user_turns: &[&str]) -> Vec<ReportTurn> {
    let last = user_turns.len().saturating_sub(1);
    user_turns
        .iter()
        .enumerate()
        .flat_map(|(index, text)| {
            [
                ReportTurn {
                    report_invoking: index == last,
                    ..ReportTurn::new("user", *text)
                },
                ReportTurn::new("assistant", "…"),
            ]
        })
        .collect()
}

/// Issue #827: two distinct subjects, both quoted, in first + last order.
#[test]
fn two_subjects_are_titled_first_plus_last() {
    let turns = conversation(&[
        "Что такое фуфломицин?",
        "Так что это такое то?",
        "Зарепорти баг",
    ]);
    assert_eq!(
        issue_title(&turns, &settings()),
        "Formal AI: `Что такое фуфломицин?` + `Так что это такое то?`"
    );
}

/// Issue #826: an earlier report request the agent already answered is part of
/// the story being reported, so it may still be the closing subject.
#[test]
fn an_answered_report_request_can_still_be_a_subject() {
    let turns = conversation(&["ФБС vs ФБО", "Зарепорти баг", "Report"]);
    assert_eq!(
        issue_title(&turns, &settings()),
        "Formal AI: `ФБС vs ФБО` + `Зарепорти баг`"
    );
}

/// Issue #838: one subject, quoted. The filed title was
/// `Formal AI: Find hive-mind on my desktop` — unquoted, and the convention
/// requires the backticks that make the quoted turn unambiguous.
#[test]
fn a_single_subject_is_quoted_on_its_own() {
    let turns = conversation(&["Find hive-mind on my desktop", "report issue"]);
    assert_eq!(
        issue_title(&turns, &settings()),
        "Formal AI: `Find hive-mind on my desktop`"
    );
}

/// Rule 4: the bare default title is never used while any user turn exists.
#[test]
fn the_default_title_is_never_used_when_a_user_turn_exists() {
    let turns = conversation(&["report issue"]);
    let title = issue_title(&turns, &settings());
    assert_eq!(title, "Formal AI: `report issue`");
    assert_ne!(title, settings().default_title);

    // With no conversation at all there is nothing to quote.
    assert_eq!(issue_title(&[], &settings()), settings().default_title);
}

/// A long first turn is cut on a word boundary, never mid-word.
#[test]
fn an_oversize_subject_is_truncated_on_a_word_boundary() {
    let long = "Find the hive mind control center folder somewhere on my desktop "
        .repeat(4)
        .trim()
        .to_owned();
    let title = issue_title(&conversation(&[&long, "report issue"]), &settings());
    assert!(title.chars().count() <= 120, "{title}");
    assert!(title.ends_with("…`"), "{title}");
    let quoted = title
        .trim_start_matches("Formal AI: `")
        .trim_end_matches("…`");
    assert!(long.starts_with(quoted), "{title}");
    assert!(
        quoted.ends_with(|character: char| character != ' '),
        "{title}"
    );
    // The cut lands between words: the kept prefix is followed by a space.
    assert!(long[quoted.len()..].starts_with(' '), "{title}");
}

/// Every language this agent supports, with a subject, the turn that asks for
/// the report, and the separator that language writes between phrases — all
/// written the way a speaker of that language would type them.
///
/// The set matches the `supported_languages` field of `data/seed/agent-info.lino`
/// (English, Russian, Hindi, Chinese) — the same list
/// `tests/e2e/scripts/check-language-test-coverage.mjs` reads. Chinese has an
/// empty separator because it does not space its words.
const SUPPORTED_LANGUAGE_CASES: [(&str, &str, &str, &str); 4] = [
    (
        "English",
        "Find hive-mind on my desktop",
        "report issue",
        " ",
    ),
    ("Russian", "Что такое фуфломицин?", "Зарепорти баг", " "),
    (
        "Hindi",
        "मेरे डेस्कटॉप पर हाइव-माइंड खोजें",
        "समस्या रिपोर्ट करें",
        " ",
    ),
    ("Chinese", "在我的桌面上找到蜂巢思维", "报告问题", ""),
];

/// The convention quotes the user's own words, so it has to hold in every
/// supported language and not only in the Latin and Cyrillic titles the issue
/// was filed with.
///
/// Rule 1 (drop the report-invoking turn) is a role-and-flag decision, not a
/// text match, so it must not depend on the script the request is written in:
/// a Hindi or Chinese `report issue` turn is dropped exactly like the English
/// one, leaving the real subject quoted verbatim.
#[test]
fn the_title_quotes_the_subject_in_every_supported_language() {
    for (language, subject, request, _) in SUPPORTED_LANGUAGE_CASES {
        let turns = conversation(&[subject, request]);
        assert_eq!(
            issue_title(&turns, &settings()),
            format!("Formal AI: `{subject}`"),
            "the {language} subject should be quoted on its own"
        );
    }
}

/// An oversize subject is cut by characters in every script.
///
/// A byte cut would split a Cyrillic, Devanagari or Han character in half and
/// leave the title invalid; a character cut cannot. Scripts that do not
/// separate words with spaces (Chinese) have no word boundary to fall back on,
/// so the cut lands mid-phrase — but still on a character, and still inside the
/// 120-character budget the title convention allows.
#[test]
fn an_oversize_subject_is_cut_by_characters_in_every_supported_language() {
    for (language, subject, request, separator) in SUPPORTED_LANGUAGE_CASES {
        let long = format!("{subject}{separator}").repeat(12).trim().to_owned();
        let title = issue_title(&conversation(&[&long, request]), &settings());
        assert!(
            title.chars().count() <= 120,
            "{language}: {} characters in {title}",
            title.chars().count()
        );
        assert!(title.ends_with("…`"), "{language}: {title}");
        let quoted = title
            .trim_start_matches("Formal AI: `")
            .trim_end_matches("…`");
        // What is kept is a prefix of the original text: no character was cut
        // in half, in any script.
        assert!(long.starts_with(quoted), "{language}: {title}");
        assert!(!quoted.is_empty(), "{language}: {title}");
    }
}

/// A Links Notation export with `count` message records.
fn exported_context(count: usize) -> String {
    let messages: Vec<_> = (0..count)
        .map(|index| {
            json!({
                "role": if index % 2 == 0 { "user" } else { "assistant" },
                "content": format!("turn {index} {}", "payload ".repeat(20)),
            })
        })
        .collect();
    formal_ai::conversation_context::conversation_context_to_lino(
        "ses_truncation",
        &json!({"metadata": {"dialog_id": "ses_truncation"}, "messages": messages}),
    )
}

/// The excerpt is still a readable document, and it says what it dropped.
///
/// This is the assertion #838 could not pass: its attached context began in the
/// middle of a base64 request body, so no reader — human or parser — could tell
/// what the conversation had been.
#[test]
fn truncation_never_splits_a_links_notation_record() {
    let text = exported_context(60);
    let label = "... omitted {count} records ...";
    let excerpt = truncate_records(&text, 4_000, label);

    assert!(excerpt.omitted > 0, "the fixture must overflow the budget");
    assert!(excerpt.text.len() <= 4_000, "{}", excerpt.text.len());
    assert!(
        excerpt
            .text
            .contains(&format!("... omitted {} records ...", excerpt.omitted)),
        "{}",
        excerpt.text
    );
    lino_to_json(&excerpt.text).expect("the excerpt is still Links Notation");

    // Every kept line is a whole line of the original, in order.
    let original: Vec<&str> = text.lines().collect();
    let mut position = 0;
    for line in excerpt.text.lines() {
        if line.trim_start().starts_with("... omitted ") {
            continue;
        }
        position = position
            + original[position..]
                .iter()
                .position(|candidate| *candidate == line)
                .unwrap_or_else(|| panic!("{line:?} is not a whole line of the export"));
    }
}

/// The method #838 used, on the same fixture, for contrast.
///
/// `tail -c` starts wherever the byte offset lands — inside a record, usually
/// inside a line — and it drops the head of the document, so the session the
/// context belongs to is gone. The excerpt above keeps both.
#[test]
fn a_byte_slice_of_the_same_export_loses_the_record_boundaries() {
    let text = exported_context(60);
    let budget = 4_000;
    let sliced = &text[text.len() - budget..];
    // `tail -c` lands inside a line …
    assert!(
        !text
            .lines()
            .any(|line| line == sliced.lines().next().unwrap()),
        "the byte cut is expected to start inside a line"
    );
    // … and `sed '1d'` only hides that by dropping the damaged line.
    let tail = sliced.split_once('\n').map_or(sliced, |(_, rest)| rest);
    assert!(!tail.contains("conversation ses_truncation"), "{tail}");
    assert!(
        !tail.contains("... omitted"),
        "a byte cut says nothing about what it dropped"
    );

    let excerpt = truncate_records(&text, budget, "... omitted {count} records ...");
    assert!(
        excerpt.text.contains("conversation ses_truncation"),
        "{}",
        excerpt.text
    );
    assert!(
        excerpt.text.contains("dialog_id ses_truncation"),
        "{}",
        excerpt.text
    );
    assert!(excerpt.text.contains("turn 0 "), "{}", excerpt.text);
}
