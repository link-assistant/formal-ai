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
pub(in crate::agentic_coding) fn is_continuation_cue(text: &str) -> bool {
    crate::rule_interpreter::handler_matches("agentic_continuation", text)
}

/// Where the evidence window of the request being served opens.
///
/// A continuation cue resumes the run already under way (issue #1095): it
/// opens no route of its own, and it opens no evidence window of its own. The
/// tool results gathered while serving the standing task stay visible to the
/// state machine that continues it -- slicing the window at the cue is what
/// restarted the write--verify chain from its read on every ping and kept
/// every binary-tree ladder leaf from reaching a completion (issue #1138,
/// 0/32). A user message that is not a cue still opens a fresh window:
/// results from an earlier request must not answer a new one.
pub(in crate::agentic_coding) fn evidence_window_start(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .rposition(|message| {
            message.role.eq_ignore_ascii_case("user")
                && !is_continuation_cue(&message.content.plain_text())
        })
        .map_or(0, |index| index + 1)
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
///
/// Agent may compact an already compacted conversation, and each compaction
/// summarizes the previous summary turn, so envelopes nest; every
/// `Conversation summary:` envelope is read, the latest first, until one names
/// a standing task (PR #1188 dogfooding: the second compaction listed "What did
/// we do so far?" first and the task was lost).
pub(super) fn compacted_agent_task(earlier: &[ChatMessage]) -> Option<String> {
    earlier.iter().rev().find_map(|message| {
        if !message.role.eq_ignore_ascii_case("assistant") {
            return None;
        }
        let text = message.content.plain_text();
        text.split("Conversation summary:")
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .find_map(|envelope| {
                let summary = envelope.trim_start();
                preserved_first_user_turn(summary)
                    .or_else(|| {
                        summary
                            .split_once("\n\nTitle:")
                            .and_then(|(head, _)| standing_sentences(head.trim()))
                    })
                    .map(repair_compacted_dot_paths)
            })
    })
}

/// The summary head's sentences that state work, joined: a re-summarized head
/// trails the client's own residue (`… What did we do so far? Title: … User
/// turns:.`) after the task.
fn standing_sentences(head: &str) -> Option<String> {
    let kept: Vec<&str> = crate::agentic_coding::shell_command_policy::prose_sentences(head)
        .into_iter()
        .filter(|sentence| {
            is_standing_task(sentence.text)
                && !mentions_envelope_marker(sentence.text, "compaction_title_marker")
                && !mentions_envelope_marker(sentence.text, "compaction_turns_marker")
        })
        .map(|sentence| head[sentence.span].trim())
        .collect();
    (!kept.is_empty()).then(|| kept.join(" "))
}

/// A turn that states work, not the client's own protocol: a continuation cue,
/// a seeded request to summarize the conversation, or a nested summary
/// envelope.
fn is_standing_task(text: &str) -> bool {
    !is_continuation_cue(text)
        && !crate::seed::lexicon().mentions_role(
            crate::seed::ROLE_CONVERSATION_SUMMARY_PHRASE,
            &crate::engine::normalize_prompt(text),
        )
        && !mentions_envelope_marker(text, "compaction_summary_marker")
}

/// Whether `text` carries one of the compaction envelope's labels, read from
/// `data/seed/agent-info.lino` (an absent or empty label never matches).
fn mentions_envelope_marker(text: &str, key: &str) -> bool {
    crate::seed::agent_info()
        .get(key)
        .is_some_and(|marker| !marker.is_empty() && text.contains(marker.as_str()))
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
    let mut rest = summary.split_once("\n\nUser turns:\n")?.1;
    let mut number = 1;
    loop {
        let body = rest.strip_prefix(format!("  {number}. ").as_str())?;
        let next = body.find(format!("\n  {}.", number + 1).as_str());
        let end = next.unwrap_or(body.len());
        let task = body[..end].trim();
        if !task.is_empty() && is_standing_task(task) {
            return Some(task.to_owned());
        }
        rest = &body[next? + 1..];
        number += 1;
    }
}

/// Emit a `route=value` planner-routing trace line to stderr when
/// `FORMAL_AI_TRACE_REQUESTS` is set to a true spelling (issue #1181).
///
/// Mirrors the request tracing in
/// `crate::protocol`. Off by default; issue #956 asked for visibility into how
/// a received task was routed.
pub fn trace_route(route: &str, value: &str) {
    if crate::cli_env::flag_enabled("FORMAL_AI_TRACE_REQUESTS") {
        eprintln!("[trace] {route}={value}");
    }
}
