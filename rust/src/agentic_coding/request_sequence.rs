//! A file operation followed by edits of the file it makes (PR #1188 G82).
//!
//! `Copy a.lino to b.lino. In b.lino replace 'x' with 'y', replace 'p' with
//! 'q'.` was written as one literal file. When the first sentence is a seeded
//! shell intent (copy, move) and every later sentence is an edit request whose
//! file that sentence names, the sentences are planned one after another, each
//! as the request it would be alone, over the same tool history; the answer
//! states each in turn. Edits joined by a seeded sequence cue (`, then`, `and
//! then`) that opens a clause are steps too (PR #1188 G99): `Insert the line
//! «b2» after the line «b» in f.txt, then delete the line «c» from f.txt.`
//! Each step sees only the tool calls made for it, so a later step reads the
//! file as the step before left it. The Rust original of
//! `js/agentic/request_sequence.mjs`.

use super::final_result::{FinalDisposition, FinalResult, ResolvedPlan, record};
use super::planner::AgenticPlan;
use super::write_request::{
    bare_surfaces, clean_cue_token, clean_path_token, looks_like_file_path, safe_relative_path,
    tokens,
};
use crate::coding::catalog::contains_cjk;
use crate::normal_markov::quoted_segment_spans;
use crate::protocol::{ChatMessage, MessageContent};

/// The marks that close a clause before a sequence cue.
const CLAUSE_MARKS: &[char] = &[',', ';', '.', '，', '；', '。'];
/// The seeded cues that start the next step (`then`, `затем`, `然后`).
const SEQUENCE_CUE_ROLE: &str = "file-edit-sequence-cue";
/// The seeded joiner words (`and`).
const JOINER_ROLE: &str = "file_edit_joiner_cue";

/// The request's sentences when the first is a seeded shell intent and every
/// later one edits a file the first names (mirrors `requestSequence`).
fn request_sequence(task: &str) -> Option<Vec<String>> {
    let parts: Vec<String> = super::shell_command_policy::sentences(task)
        .iter()
        .map(|sentence| sentence.text.to_owned())
        .collect();
    let (first, rest) = parts.split_first()?;
    if rest.is_empty() || super::shell_command::semantic_shell_command_for_task(first).is_none() {
        return None;
    }
    let named = super::module_function::paths_in(first);
    for part in rest {
        let target = super::replace_list::replace_list(part)
            .map(|(target, _)| target)
            .or_else(|| {
                super::write_request::compose_edit_request(part).map(|(target, _, _)| target)
            })?;
        if !named.contains(&target) {
            return None;
        }
    }
    Some(parts)
}

/// Where the step before a sequence cue at byte `at` ends: at the clause mark
/// or the seeded joiner word (`and`) that opens the cue's clause; `None` when
/// the cue does not open a clause (mirrors `sequenceCut`).
fn sequence_cut(task: &str, at: usize) -> Option<usize> {
    let lead = task[..at].trim_end();
    let last = lead.chars().next_back()?;
    if CLAUSE_MARKS.contains(&last) {
        return Some(lead.len() - last.len_utf8());
    }
    let word = lead.split_whitespace().next_back()?;
    bare_surfaces(JOINER_ROLE)
        .contains(&clean_cue_token(word))
        .then_some(lead.len() - word.len())
}

/// Each unquoted seeded sequence cue of the instruction that opens a clause,
/// as `(step_end, next_start)` (mirrors `sequenceCues`).
fn sequence_cues(task: &str) -> Vec<(usize, usize)> {
    let end = super::positional_edit::instruction_end(task);
    let segments = quoted_segment_spans(task);
    let mut cues = bare_surfaces(SEQUENCE_CUE_ROLE);
    cues.sort_by_key(|surface| std::cmp::Reverse(surface.len()));
    let mut out = Vec::new();
    let mut at = 0;
    while at < end {
        let quoted = segments
            .iter()
            .any(|segment| at >= segment.start && at < segment.end);
        let cue = if quoted {
            None
        } else {
            cues.iter().find(|surface| cue_at(task, at, surface))
        };
        let cut = cue.and_then(|_| sequence_cut(task, at));
        let (Some(cue), Some(cut)) = (cue, cut) else {
            at += task[at..].chars().next().map_or(1, char::len_utf8);
            continue;
        };
        let rest = &task[at + cue.len()..];
        let skipped = rest.len()
            - rest
                .trim_start_matches(|c: char| c.is_whitespace() || CLAUSE_MARKS.contains(&c))
                .len();
        let next = at + cue.len() + skipped;
        out.push((cut, next));
        at = next;
    }
    out
}

/// Whether the seeded cue `surface` (lowercase) stands at byte `at` of `task`
/// as a whole word (any CJK cue stands anywhere).
fn cue_at(task: &str, at: usize, surface: &str) -> bool {
    let Some(found) = task.get(at..at + surface.len()) else {
        return false;
    };
    if found.to_lowercase() != surface {
        return false;
    }
    let before = task[..at].chars().next_back();
    let after = task[at + surface.len()..].chars().next();
    contains_cjk(surface)
        || (!before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric))
}

/// The paths `step` names outside its quotes.
fn unquoted_paths(step: &str) -> Vec<String> {
    let segments = quoted_segment_spans(step);
    tokens(step)
        .iter()
        .filter(|token| {
            !segments
                .iter()
                .any(|segment| token.start < segment.end && token.end > segment.start)
        })
        .map(|token| clean_path_token(token.text))
        .filter(|path| looks_like_file_path(path) && safe_relative_path(path))
        .map(str::to_owned)
        .collect()
}

/// The request cut at its sequence cues into steps that each quote a text and
/// name a file (a step naming none takes the one file the others name), or
/// `None` (PR #1188 G99; mirrors `sequenceSteps`).
pub(super) fn sequence_steps(task: &str) -> Option<Vec<String>> {
    let cues = sequence_cues(task);
    if cues.is_empty() {
        return None;
    }
    let mut bounds = vec![0];
    for (step_end, next_start) in cues {
        bounds.push(step_end);
        bounds.push(next_start);
    }
    bounds.push(task.len());
    let steps: Vec<&str> = bounds
        .chunks(2)
        .map(|pair| {
            task.get(pair[0]..pair[1])
                .unwrap_or_default()
                .trim()
                .trim_end_matches(|c: char| c.is_whitespace() || STEP_TAIL.contains(&c))
        })
        .collect();
    if steps.iter().enumerate().any(|(index, step)| {
        step.is_empty()
            || (quoted_segment_spans(step).is_empty()
                && (index != 0
                    || super::shell_command::semantic_shell_command_for_task(step).is_none()))
    }) {
        return None;
    }
    let mut named: Vec<String> = Vec::new();
    for path in steps.iter().flat_map(|step| unquoted_paths(step)) {
        if !named.contains(&path) {
            named.push(path);
        }
    }
    steps
        .iter()
        .map(|step| {
            if !unquoted_paths(step).is_empty() {
                return Some((*step).to_owned());
            }
            // A step naming no path, not even inside its quotes, edits the one
            // file the others name.
            if named.len() != 1 || !super::module_function::paths_in(step).is_empty() {
                return None;
            }
            Some(format!("{step} in {}", named[0]))
        })
        .collect()
}

/// The marks a step loses at its end.
const STEP_TAIL: &[char] = &[',', ';', '，', '；', '.', '。'];

/// The conversation with its latest user turn asking `part` alone.
fn with_request(messages: &[ChatMessage], part: &str) -> Vec<ChatMessage> {
    let latest = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"));
    messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let mut message = message.clone();
            if Some(index) == latest {
                message.content = MessageContent::Text(part.to_owned());
            }
            message
        })
        .collect()
}

/// The next step of the first sentence not yet answered, or every answer.
///
/// Each sentence is planned by `plan_for` as that sentence alone; once each is
/// answered, the answers in order (mirrors `planRequestSequenceStep`).
pub(super) fn plan_request_sequence_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan_for: fn(&[ChatMessage], &[&str]) -> Option<ResolvedPlan>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let parts = request_sequence(task).or_else(|| sequence_steps(task))?;
    let (base, exchanges) = turn_exchanges(messages);
    let mut taken = 0;
    let mut answers = Vec::new();
    for part in &parts {
        // A step is planned over the tool calls made for it alone: replayed
        // one exchange at a time until it answers or asks for its next call.
        let mut own: Vec<ChatMessage> = base.to_vec();
        loop {
            let resolved = plan_for(&with_request(&own, part), tool_names)?;
            if matches!(&resolved.plan, AgenticPlan::Final(_)) && !resolved.can_deliver() {
                return Some(resolved.into_plan(result));
            }
            match resolved.plan {
                AgenticPlan::Final(answer) => {
                    answers.push(answer);
                    break;
                }
                plan @ AgenticPlan::ToolCalls(_) => {
                    let Some(exchange) = exchanges.get(taken) else {
                        return Some(plan);
                    };
                    own.extend_from_slice(exchange);
                    taken += 1;
                }
            }
        }
    }
    Some(record(
        AgenticPlan::Final(answers.join("\n\n")),
        FinalDisposition::Finding,
        "request_sequence_verified",
        result,
    ))
}

/// The conversation up to its latest user turn, and the tool exchanges after
/// it, each an assistant message with the messages that answer it (mirrors
/// `turnExchanges`).
fn turn_exchanges(messages: &[ChatMessage]) -> (&[ChatMessage], Vec<&[ChatMessage]>) {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |latest| latest + 1);
    let mut exchanges: Vec<&[ChatMessage]> = Vec::new();
    let mut from = start;
    for (index, message) in messages.iter().enumerate().skip(start) {
        let opens = message.role.eq_ignore_ascii_case("assistant");
        if opens && index > from {
            exchanges.push(&messages[from..index]);
            from = index;
        }
    }
    if from < messages.len() {
        exchanges.push(&messages[from..]);
    }
    (&messages[..start], exchanges)
}
