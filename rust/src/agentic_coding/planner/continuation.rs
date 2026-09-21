//! Continuation and compaction helpers for the agentic planner.
//!
//! Resuming a prior agent session means reading a continuation cue, carrying
//! the continued task forward, and reconstructing it from a compacted
//! transcript without losing the first user turn.

use super::ChatMessage;

/// Whether a turn is nothing but a continuation cue (issue #1095).
///
/// The rule `agentic_continuation` in `data/seed/handler-rules.lino` compares
/// the whole cleaned prompt with the role's surfaces -- "continue the migration
/// in src/queue.rs" contains the word and keeps its own route.
pub(super) fn is_continuation_cue(text: &str) -> bool {
    crate::rule_interpreter::handler_matches("agentic_continuation", text)
}

/// The task a continuation cue continues.
///
/// Two places can hold it. After Agent's compaction the objective survives only
/// inside the assistant's `Conversation summary:` envelope, and that is read
/// first because it is the more specific record. After an ordinary tool result
/// there is no envelope: the task is simply the last user turn that was not
/// itself a cue. The ladder's eight #1095 leaves were this second case -- the
/// envelope path found nothing, the bare cue became the request, and the words
/// "continue" and "next step" went to web search.
pub(super) fn continued_agent_task(messages: &[ChatMessage], latest: &str) -> Option<String> {
    if !is_continuation_cue(latest) {
        return None;
    }
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let earlier = &messages[..latest_user];
    compacted_agent_task(earlier).or_else(|| {
        earlier
            .iter()
            .rev()
            .filter(|message| message.role.eq_ignore_ascii_case("user"))
            .map(|message| message.content.plain_text())
            .find(|text| !text.trim().is_empty() && !is_continuation_cue(text))
    })
}

/// Restore the objective carried through Agent's compaction protocol from the
/// turns before the continuation.
pub(super) fn compacted_agent_task(earlier: &[ChatMessage]) -> Option<String> {
    earlier.iter().rev().find_map(|message| {
        if !message.role.eq_ignore_ascii_case("assistant") {
            return None;
        }
        let envelope = message.content.plain_text();
        // Agent may compact an already compacted conversation. In that
        // case its new summary repeats the protocol continuation before
        // embedding the prior `Conversation summary:` envelope. The
        // continuation remains trusted only as a protocol trigger; recover
        // the objective from the last summary marker in the assistant's
        // compaction response rather than requiring that marker at byte 0.
        let summary = envelope
            .rsplit_once("Conversation summary:")?
            .1
            .trim_start();
        preserved_first_user_turn(summary)
            .or_else(|| {
                summary
                    .split_once("\n\nTitle:")
                    .map(|(task, _)| task.trim())
                    .filter(|task| !task.is_empty())
                    .map(str::to_owned)
            })
            .map(repair_compacted_dot_paths)
    })
}

/// Repair a Markdown dotfile path spaced apart by Agent's prose summarizer.
///
/// A live compaction changed `.agent-ladder/node.md` into
/// `. agent-ladder/node.md`. Only a standalone dot followed by a safe
/// slash-bearing relative path has this whitespace removed; ordinary prose and
/// mathematical uses of a period remain untouched.
pub(super) fn repair_compacted_dot_paths(mut task: String) -> String {
    let mut search_from = 0;
    while let Some(relative_dot) = task[search_from..].find(". ") {
        let dot = search_from + relative_dot;
        let standalone = task[..dot]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace);
        let after_dot = dot + 1;
        let Some(non_space) = task[after_dot..].find(|character: char| !character.is_whitespace())
        else {
            break;
        };
        let path_start = after_dot + non_space;
        let path_end = task[path_start..]
            .find(char::is_whitespace)
            .map_or(task.len(), |offset| path_start + offset);
        let token = task[path_start..path_end]
            .trim_matches(|character: char| matches!(character, '`' | '"' | '\'' | ',' | ';'))
            .trim_end_matches(['.', '!', '?']);
        let candidate = format!(".{token}");
        if standalone
            && token.contains('/')
            && crate::agentic_coding::write_request::safe_relative_path(&candidate)
        {
            task.replace_range(after_dot..path_start, "");
            search_from = dot + 1;
        } else {
            search_from = path_start;
        }
    }
    task
}

/// The summarizer keeps exact user bytes after its prose summary.
///
/// Its prose is allowed to normalize Markdown and, in a live Agent run, changed
/// `.agent-ladder` into `. agent-ladder`. The numbered `User turns` appendix is
/// the lossless copy, so prefer its first task turn and fall back to the prose
/// only for older summaries that did not include the appendix.
pub(super) fn preserved_first_user_turn(summary: &str) -> Option<String> {
    let turns = summary.split_once("\n\nUser turns:\n")?.1;
    let first = turns.strip_prefix("  1. ")?;
    let end = first.find("\n  2.").unwrap_or(first.len());
    let task = first[..end].trim();
    (!task.is_empty()).then(|| task.to_owned())
}

/// Emit a `route=value` planner-routing trace line to stderr when
/// `FORMAL_AI_TRACE_REQUESTS=1`.
///
/// Mirrors the request tracing in
/// `crate::protocol`. Off by default; issue #956 asked for visibility into how
/// a received task was routed.
pub fn trace_route(route: &str, value: &str) {
    if std::env::var("FORMAL_AI_TRACE_REQUESTS").as_deref() == Ok("1") {
        eprintln!("[trace] {route}={value}");
    }
}
