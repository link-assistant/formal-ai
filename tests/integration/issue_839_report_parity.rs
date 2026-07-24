//! Issue #839: the web and the agentic reporter must emit the same document.
//!
//! Before #839 the web reporter (`src/web/app/main.jsx`) and the agentic
//! reporter (`src/agentic_coding/report_issue.rs`) formatted issue bodies
//! independently, and only the web path was ever asserted on. `src/issue_report.rs`
//! now owns the format for every surface, but the browser cannot link the Rust
//! core (the wasm worker is a standalone `rustc` build), so the browser mirror
//! `src/web/app/issue-report.js` is a hand-written port. This test renders the
//! same fixture through both implementations and fails the moment they drift.

use std::io::Write;
use std::process::{Command, Stdio};

use formal_ai::issue_report::{
    issue_title, truncate_records, ReportAttachment, ReportBody, ReportField, ReportLabels,
    ReportTurn, TitleSettings, SECTIONS,
};
use serde_json::{json, Value};

/// A fixture that exercises every section, both `omitted` labels and an
/// attachment, so a drift anywhere in the document is caught.
fn fixture_body() -> ReportBody {
    ReportBody {
        labels: labels(),
        environment: vec![
            ReportField::new("Version", "0.303.0 (wasm)"),
            ReportField::new("URL", "https://formal-ai.dev/"),
            ReportField::new("Timestamp", "2026-01-01T00:00:00.000Z"),
        ],
        user_context: vec![
            ReportField::new("UI languages", "en (browser: en-US)"),
            ReportField::new("Theme", ""),
        ],
        turns: vec![
            ReportTurn::new("user", "Search hive-mind on desktop"),
            ReportTurn {
                intent: String::from("unknown"),
                ..ReportTurn::new(
                    "assistant",
                    "No matching file or folder was found.\nTry again.",
                )
            },
            ReportTurn::new("tool", "```\n(no output)\n```"),
            ReportTurn {
                report_invoking: true,
                ..ReportTurn::new("user", "report issue")
            },
        ],
        earlier_omitted: 0,
        reasoning_trace: vec![
            String::from("intent: unknown"),
            String::from("evidence:"),
            String::from("- surface:desktop"),
        ],
        attachments: vec![ReportAttachment {
            heading: String::from("### Complete agentic context"),
            note: String::from(
                "The complete Links Notation context of this session is attached below.",
            ),
            language: String::from("lino"),
            content: String::from("conversation ses_1\n  message_count 1\n"),
        }],
    }
}

fn labels() -> ReportLabels {
    ReportLabels::from_seed()
}

#[test]
fn web_and_agentic_bodies_are_identical_for_the_same_conversation() {
    let Some(mirror) = render_through_mirror(&json!({"body": fixture_body()})) else {
        return;
    };
    let rust = fixture_body().render();
    assert_eq!(
        mirror["body"].as_str().expect("mirror body"),
        rust,
        "the browser mirror drifted from src/issue_report.rs"
    );
    for section in SECTIONS {
        assert!(rust.contains(section), "missing {section}:\n{rust}");
    }
}

#[test]
fn web_and_agentic_bodies_are_identical_when_earlier_turns_were_dropped() {
    let mut body = fixture_body();
    body.earlier_omitted = 1;
    let Some(mirror) = render_through_mirror(&json!({"body": body})) else {
        return;
    };
    assert_eq!(mirror["body"].as_str().expect("mirror body"), body.render());

    let mut many = fixture_body();
    many.earlier_omitted = 7;
    let Some(mirror) = render_through_mirror(&json!({"body": many})) else {
        return;
    };
    assert_eq!(mirror["body"].as_str().expect("mirror body"), many.render());
}

#[test]
fn web_and_agentic_titles_are_identical() {
    let settings = TitleSettings::from_seed();
    let conversations = [
        vec![
            ReportTurn::new("user", "ФБС vs ФБО"),
            ReportTurn::new("assistant", "…"),
            ReportTurn::new("user", "Зарепорти баг"),
            ReportTurn::new("assistant", "…"),
            ReportTurn {
                report_invoking: true,
                ..ReportTurn::new("user", "Report")
            },
        ],
        vec![ReportTurn::new(
            "user",
            "Search hive-mind on desktop and tell me every place it could possibly hide on this machine",
        )],
        vec![],
    ];
    for turns in conversations {
        let Some(mirror) = render_through_mirror(&json!({
            "title": {"turns": turns, "settings": settings}
        })) else {
            return;
        };
        assert_eq!(
            mirror["title"].as_str().expect("mirror title"),
            issue_title(&turns, &settings)
        );
    }
}

#[test]
fn web_and_agentic_record_truncation_are_identical() {
    let text = sample_export(40);
    let label = labels().omitted_records;
    let Some(mirror) = render_through_mirror(&json!({
        "truncate": {"text": text, "max_bytes": 900, "omitted_label": label}
    })) else {
        return;
    };
    let rust = truncate_records(&text, 900, &label);
    assert_eq!(
        mirror["truncate"]["text"].as_str().expect("text"),
        rust.text
    );
    assert_eq!(
        mirror["truncate"]["omitted"].as_u64().expect("omitted"),
        rust.omitted as u64
    );
}

fn sample_export(records: usize) -> String {
    let mut text = String::from("conversation ses_06ac01b87ffeW5XnFmtYE8Amil\n  messages\n");
    for index in 0..records {
        text.push_str("    message\n");
        text.push_str(&format!("      role user\n      text \"turn {index}\"\n"));
    }
    text
}

/// Render a fixture through the browser mirror, or `None` when `node` is
/// unavailable (the same skip guard the desktop artifact test uses).
fn render_through_mirror(fixture: &Value) -> Option<Value> {
    match Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {}
        _ => {
            eprintln!("skipping: node not available");
            return None;
        }
    }

    let script = format!(
        "{}/tests/support/render-issue-report.mjs",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut child = Command::new("node")
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the browser mirror harness");
    child
        .stdin
        .as_mut()
        .expect("harness stdin")
        .write_all(fixture.to_string().as_bytes())
        .expect("write fixture");
    let output = child.wait_with_output().expect("run the mirror harness");
    assert!(
        output.status.success(),
        "mirror harness failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(serde_json::from_slice(&output.stdout).expect("mirror output"))
}
