//! Issue #1131: a rewrite is not an insertion, however the prose is worded.
//!
//! A request to rewrite `configure-dockerhub-publishing.sh` in full was handled
//! as a request to add members to an enumeration inside it. Nothing in the
//! request asked for that. Two accidents combined: the prose carried the bare
//! word `set` -- from `set -euo pipefail`, a recognised surface for an
//! unordered member grouping -- and the prose double-quoted two values, which
//! made them look like member literals. The two were spliced into the nearest
//! bracketed construct, a shell condition:
//!
//! ```sh
//! if [ -z "$DOCKERHUB_USERNAME" ] || [ -z "$DOCKERHUB_TOKEN", "konard/formal-ai", "konard" ]; then
//! ```
//!
//! That parses under `bash -n` and is always true, so the release would have
//! failed loudly on every run. Removing either accident alone makes the defect
//! disappear, so the guard keeps both present.
//!
//! Authored by Formal AI through `scripts/author-change-with-formal-ai.sh`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

/// The script as it stands on disk, which the planner reads before it writes.
const SOURCE: &str = "\
set -euo pipefail

DOCKERHUB_IMAGE=\"${DOCKERHUB_IMAGE:-}\"
DOCKERHUB_USERNAME=\"${DOCKERHUB_USERNAME:-}\"
DOCKERHUB_TOKEN=\"${DOCKERHUB_TOKEN:-}\"

if [ -z \"$DOCKERHUB_USERNAME\" ] || [ -z \"$DOCKERHUB_TOKEN\" ]; then
  exit 1
fi
";

/// A whole-file rewrite must not carry quoted prose values into the file.
#[test]
fn a_rewrite_request_does_not_add_quoted_prose_values() {
    const TOOLS: [&str; 3] = ["read", "write", "bash"];
    let messages = vec![ChatMessage::user(
        "Rewrite the file configure-dockerhub-publishing.sh in the current \
         directory. That must change, because DOCKERHUB_IMAGE and \
         DOCKERHUB_USERNAME are about to get defaults (\"konard/formal-ai\" and \
         \"konard\"), so they will never be empty. Keep the existing shell \
         style: a bash script beginning with the shebang line for \
         /usr/bin/env bash, then set -euo pipefail, reading the three \
         variables so an unset variable is not an error.",
    )];

    // The splice happens on the turn *after* the read, once the planner has the
    // file's own bytes to graft onto, so one planning step cannot see it.
    let mut messages = messages;
    for turn in 0..3 {
        let planned = plan_chat_step(&messages, &TOOLS);
        let Some(AgenticPlan::ToolCalls(calls)) = planned else {
            break;
        };
        let call = calls.first().expect("a planned step calls a tool").clone();
        assert!(
            !call.arguments.contains("konard"),
            "a rewrite request must not add quoted prose values into an enumeration: {call:?}"
        );
        let id = format!("call_{turn}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        let result = if call.tool == "read" { SOURCE } else { "ok" };
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }
}
