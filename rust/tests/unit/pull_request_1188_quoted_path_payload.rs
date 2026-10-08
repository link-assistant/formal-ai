//! PR #1188 dogfooding (T21): `Replace 'rust/src/x.rs' with
//! 'data/seed/x.lino' in check.mjs.` planned nothing. The edit reader took the
//! quoted new text as the file to edit, because a literal that is exactly a
//! path may name the file and the cue after it ("in") read as the target cue.
//! A path the request leaves unquoted now names the file first; a quoted path
//! still does when nothing else can. The browser twin is pinned by
//! `rust/tests/web/pull-request-1188-quoted-path-payload.test.mjs`.

use formal_ai::agentic_coding::general_planner::compose_edit_request;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::capability_routing::first_path;
use formal_ai::protocol::{ChatMessage, ToolCall};

fn edit(request: &str) -> Option<(String, String, String)> {
    compose_edit_request(request)
}

#[test]
fn a_quoted_path_payload_is_the_new_text_not_the_target() {
    assert_eq!(
        edit("Replace 'abc' with 'data/seed/x.lino' in check.mjs."),
        Some((
            "check.mjs".to_owned(),
            "abc".to_owned(),
            "data/seed/x.lino".to_owned()
        ))
    );
    assert_eq!(
        edit("Replace 'feature_capability.rs' with 'features.lino' in check.mjs."),
        Some((
            "check.mjs".to_owned(),
            "feature_capability.rs".to_owned(),
            "features.lino".to_owned()
        ))
    );
}

#[test]
fn a_quoted_path_still_names_the_file_when_nothing_else_does() {
    assert_eq!(
        edit("Replace 'a' with 'b' in 'notes.txt'."),
        Some(("notes.txt".to_owned(), "a".to_owned(), "b".to_owned()))
    );
}

/// T25: `Replace the heading '# Title' with '# Project' in README.md.` sent the
/// whole clause as the old text; a literal led only by the words that say what
/// it is stands for the literal.
#[test]
fn a_described_literal_stands_for_its_quoted_text() {
    assert_eq!(
        edit("Replace the heading '# Title' with '# Project' in README.md."),
        Some((
            "README.md".to_owned(),
            "# Title".to_owned(),
            "# Project".to_owned()
        ))
    );
    assert_eq!(
        edit("Replace the word 'cat' with 'dog' in a.txt."),
        Some(("a.txt".to_owned(), "cat".to_owned(), "dog".to_owned()))
    );
}

/// T26: `Create a file new.txt containing 'hello'.` wrote `'hello'.`; the
/// sentence's closing mark after one quoted literal is the sentence's.
#[test]
fn a_sentence_mark_after_one_quoted_literal_is_not_content() {
    let mut messages = vec![ChatMessage::user(
        "Create a file new.txt containing 'hello'.",
    )];
    let mut written = None;
    for index in 0..4 {
        let Some(AgenticPlan::ToolCalls(calls)) =
            plan_chat_step(&messages, &["read", "write", "edit", "bash"])
        else {
            panic!("a file creation plans tool calls until the file is written");
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        if call.tool == "write" && arguments["filePath"] == "new.txt" {
            written = arguments["content"].as_str().map(str::to_owned);
            break;
        }
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(
            id,
            call.tool.clone(),
            String::new(),
        ));
    }
    assert_eq!(written.as_deref(), Some("hello"));
}

/// T36: a double-quoted payload holding double quotes split apart, and the
/// read went to `/v1/x/learn` -- a path inside the payload -- instead of the
/// file the request names, which only the sentence's period hid.
#[test]
fn a_path_inside_a_quoted_payload_never_outranks_the_named_file() {
    assert_eq!(
        first_path(
            r#"Insert the line "    body '{"token":"p"}'" after the line "    path '/v1/x/learn'" in r.lino."#
        )
        .as_deref(),
        Some("r.lino")
    );
    assert_eq!(
        first_path("Read 'notes.txt'.").as_deref(),
        Some("notes.txt")
    );
    assert_eq!(
        first_path("Open (docs/x.md).").as_deref(),
        Some("docs/x.md")
    );
}

/// T37: `… with '… the named file.' in ledger.md.` cut the new text before
/// "file", because the target clause's walk back read the quoted word as its
/// cue; a cue word inside a quoted literal is payload.
#[test]
fn a_target_cue_inside_the_quoted_new_text_is_payload() {
    assert_eq!(
        edit("Replace 'works.' with 'works. It reads the named file.' in ledger.md."),
        Some((
            "ledger.md".to_owned(),
            "works.".to_owned(),
            "works. It reads the named file.".to_owned()
        ))
    );
}
