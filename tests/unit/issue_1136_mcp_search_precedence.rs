//! Issue #1136: a wired-up MCP search outranks the client's own alias.
//!
//! Claude Code advertises its built-in `WebSearch` beside the
//! `mcp__issue781__websearch` the four-client harness wires up, but grants
//! permission only for the MCP one. Ranking the client's own alias first ended
//! every run at "Claude requested permissions to use WebSearch, but you haven't
//! granted it yet", and the run recorded no search at all.
//!
//! Authored by Formal AI through `scripts/author-change-with-formal-ai.sh`.

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};

/// The tool set Claude Code advertised, shortened to the pair that decides the
/// routing: the client's own search alias and the wired-up MCP search tool.
const TOOLS: [&str; 6] = [
    "Bash",
    "Edit",
    "Read",
    "WebSearch",
    "mcp__issue781__websearch",
    "Write",
];

/// Research is planned with the MCP tool the client was configured to permit.
#[test]
fn a_wired_up_mcp_search_is_planned_over_the_clients_own_alias() {
    let messages = vec![ChatMessage::user(
        "Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in",
    )];
    match plan_chat_step(&messages, &TOOLS).expect("a research request has a plan") {
        AgenticPlan::ToolCalls(calls) => {
            assert_eq!(calls[0].tool, "mcp__issue781__websearch", "{calls:?}");
        }
        AgenticPlan::Final(answer) => panic!("expected a search, got {answer:?}"),
    }
}
