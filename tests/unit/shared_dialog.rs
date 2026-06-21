use std::fs;
use std::path::Path;

use formal_ai::{
    convert_shared_dialog_to_demo_memory, parse_memory_links_notation, parse_shared_dialog,
    SharedDialogFormat, SharedDialogMetadata,
};

const CHATGPT_SHARE_URL: &str = "https://chatgpt.com/share/6a3825b9-8de4-83ee-9c24-52fd1eb38d24";

#[test]
fn chatgpt_share_html_extracts_visible_dialog_turns() {
    let html = read_fixture("docs/case-studies/issue-552/raw-data/chatgpt-share-6a3825b9.html");
    let dialog = parse_shared_dialog(
        &html,
        SharedDialogFormat::ChatGptShareHtml,
        &SharedDialogMetadata {
            source_url: Some(String::from(CHATGPT_SHARE_URL)),
            ..SharedDialogMetadata::default()
        },
    )
    .expect("real ChatGPT share capture should parse");

    assert_eq!(dialog.title.as_deref(), Some("Infinite loop script"));
    assert_eq!(
        dialog.conversation_id.as_deref(),
        Some("6a3825b9-8de4-83ee-9c24-52fd1eb38d24")
    );
    assert_eq!(dialog.turns.len(), 4);
    assert_eq!(dialog.turns[0].role, "user");
    assert_eq!(dialog.turns[1].role, "assistant");
    assert_eq!(dialog.turns[2].role, "user");
    assert_eq!(dialog.turns[3].role, "assistant");
    assert!(dialog.turns[0].content.contains("make a loop of that"));
    assert!(dialog.turns[1]
        .content
        .contains("while true; do sleep 30m && hive-cleanup -f; done"));
    assert!(dialog.turns[3]
        .content
        .contains("screen -dmS auto-cleanup bash -c"));
}

#[test]
fn chatgpt_share_conversion_exports_demo_memory_events() {
    let html = read_fixture("docs/case-studies/issue-552/raw-data/chatgpt-share-6a3825b9.html");
    let exported = convert_shared_dialog_to_demo_memory(
        &html,
        SharedDialogFormat::ChatGptShareHtml,
        &SharedDialogMetadata {
            source_url: Some(String::from(CHATGPT_SHARE_URL)),
            demo_label: Some(String::from("issue-552-chatgpt-share")),
            ..SharedDialogMetadata::default()
        },
    )
    .expect("ChatGPT share should export to demo_memory");
    let events = parse_memory_links_notation(&exported);

    assert_eq!(events.len(), 4);
    assert_eq!(events[0].role.as_deref(), Some("user"));
    assert_eq!(
        events[0].demo_label.as_deref(),
        Some("issue-552-chatgpt-share")
    );
    assert_eq!(
        events[0].conversation_id.as_deref(),
        Some("6a3825b9-8de4-83ee-9c24-52fd1eb38d24")
    );
    assert_eq!(
        events[0].conversation_title.as_deref(),
        Some("Infinite loop script")
    );
    assert_eq!(events[0].evidence, vec![String::from(CHATGPT_SHARE_URL)]);
    assert!(events[3]
        .content
        .as_deref()
        .unwrap_or_default()
        .contains("screen -dmS auto-cleanup bash -c"));
}

#[test]
fn markdown_transcript_exports_demo_memory_events() {
    let transcript = r"U: sleep 30m && hive-cleanup -f
A: while true; do sleep 30m && hive-cleanup -f; done
";
    let exported = convert_shared_dialog_to_demo_memory(
        transcript,
        SharedDialogFormat::MarkdownTranscript,
        &SharedDialogMetadata {
            demo_label: Some(String::from("issue-552-markdown")),
            conversation_id: Some(String::from("issue-552-markdown-fixture")),
            conversation_title: Some(String::from("Loop command fixture")),
            ..SharedDialogMetadata::default()
        },
    )
    .expect("compact markdown transcript should export");
    let events = parse_memory_links_notation(&exported);

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].id, "markdown-turn-1");
    assert_eq!(events[0].role.as_deref(), Some("user"));
    assert_eq!(events[1].role.as_deref(), Some("assistant"));
    assert_eq!(
        events[1].content.as_deref(),
        Some("while true; do sleep 30m && hive-cleanup -f; done")
    );
}

fn read_fixture(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(path).expect("fixture should be readable")
}
