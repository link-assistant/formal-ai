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

/// The verb is required in every registered language, not just English.
///
/// `coding_member_add` carries surfaces for english, russian, hindi, chinese
/// and spanish, so the noun-only prose that issue #1131 mis-routed has to stay
/// mis-routing-proof in each of them. Each case pairs a member-list noun with a
/// quoted value and no add verb: the planner must decline. The same sentence
/// with its add verb restored must still reach the insertion, or the guard
/// would be passing by refusing everything.
///
/// Hindi and spanish are checked for the refusal only. Both already returned no
/// plan for the positive case before `coding_member_add` existed -- verified
/// against the parent commit -- because `data/seed/meanings-file-edit.lino`
/// carries no spanish lexemes at all, so `Edita` is not yet an edit cue.
/// Requiring the insertion here would pin gaps this change did not create and
/// does not close. What it does close for spanish is the noun: the three
/// `coding_member_list_*` concepts had no spanish surfaces either, so
/// `a la lista` now reads as a member list and `add ... a la lista NAMES`
/// reaches the insertion.
#[test]
fn every_registered_language_needs_the_verb_before_a_member_list_grows() {
    const TOOLS: [&str; 3] = ["read", "write", "bash"];
    const SOURCE: &str = "const NAMES: &[&str] = &[\"alpha\"];\n";

    // (language, noun-only request, the same request with an add verb)
    let cases = [
        (
            "english",
            "Edit names.rs so the set is documented. It quotes \"beta\" today.",
            "Edit names.rs: add \"beta\" to the NAMES list.",
        ),
        (
            "russian",
            "Отредактируй names.rs, чтобы множество было описано. Там указано \"beta\".",
            "Отредактируй names.rs: добавь \"beta\" в список NAMES.",
        ),
        (
            "hindi",
            "names.rs को संपादित करें ताकि समुच्चय प्रलेखित हो। वहाँ \"beta\" लिखा है।",
            "",
        ),
        (
            "chinese",
            "编辑 names.rs 使集合被记录。那里写着 \"beta\"。",
            "编辑 names.rs：在 NAMES 列表中添加 \"beta\"。",
        ),
        (
            "spanish",
            "Edita names.rs para que el conjunto quede documentado. Allí dice \"beta\".",
            "",
        ),
        // The spanish noun is reachable today even though the spanish verb is
        // not, so the half that this change added is pinned on its own.
        (
            "spanish (noun only)",
            "Edit names.rs so the conjunto is documented. It quotes \"beta\" today.",
            "Edit names.rs: add \"beta\" a la lista NAMES.",
        ),
    ];

    for (language, noun_only, with_verb) in cases {
        let messages = vec![ChatMessage::user(noun_only)];
        let planned = plan_chat_step(&messages, &TOOLS);
        let writes_beta = match &planned {
            Some(AgenticPlan::ToolCalls(calls)) => calls
                .iter()
                .any(|call| call.tool == "write" && call.arguments.contains("beta")),
            _ => false,
        };
        assert!(
            !writes_beta,
            "{language}: a member-list noun without an add verb is not an insertion: {planned:?}"
        );

        // The verb restored: the route must still be reachable, or requiring it
        // would have fixed the false positive by breaking the true one.
        if with_verb.is_empty() {
            continue;
        }
        let mut messages = vec![ChatMessage::user(with_verb)];
        let mut reached_write = false;
        for turn in 0..3 {
            let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
                break;
            };
            let call = calls.first().expect("a planned step calls a tool").clone();
            if call.tool == "write" {
                assert!(
                    call.arguments.contains("beta"),
                    "{language}: the requested member is written: {call:?}"
                );
                reached_write = true;
                break;
            }
            let id = format!("call_{turn}");
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                id.clone(),
                call.tool.clone(),
                call.arguments.clone(),
            )]));
            messages.push(ChatMessage::tool_result(id, &call.tool, SOURCE));
        }
        assert!(
            reached_write,
            "{language}: an explicit add verb still reaches the insertion"
        );
    }
}
