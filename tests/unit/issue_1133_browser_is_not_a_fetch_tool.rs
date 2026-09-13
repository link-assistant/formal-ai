//! Issue #1133: the browser is not a fetch tool.
//!
//! Hive Mind's Kotlin run advertised every playwright tool beside `WebFetch`.
//! `mcp__playwright__browser_click` carries the substring `browse`, the
//! compatibility fallback in `classify_tool` read that as a fetch capability,
//! and `tool_for` preferred any `mcp__` name over the client's own alias. The
//! run then called the click tool 547 times with an empty selector.
//!
//! Authored by Formal AI through `scripts/author-change-with-formal-ai.sh`.

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};

/// The tool set the Kotlin run advertised, shortened to the pair that decides
/// the routing: the client's fetch tool and one browser-automation tool.
const TOOLS: [&str; 4] = [
    "Bash",
    "Write",
    "WebFetch",
    "mcp__playwright__browser_click",
];

/// A work item is read with the client's fetch tool, never with the browser.
#[test]
fn a_work_item_is_read_with_webfetch_and_not_with_a_browser_click() {
    let messages = vec![ChatMessage::user(
        "Resolve the GitHub issue at https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/issues/1 in this repository.",
    )];
    match plan_chat_step(&messages, &TOOLS).expect("a work item has a plan") {
        AgenticPlan::ToolCalls(calls) => {
            assert_eq!(calls[0].tool, "WebFetch", "{calls:?}");
        }
        AgenticPlan::Final(answer) => panic!("expected the work item to be read, got {answer:?}"),
    }
}
