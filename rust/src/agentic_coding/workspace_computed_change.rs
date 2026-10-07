//! Workspace changes whose new bytes are computed from the file they change
//! (PR #1188 dogfooding): text added at one end of a file, quoted text
//! removed from it, a configuration setting assigned a new value.
//!
//! Each is read → compute → smallest unique edit (or the whole file) → digest
//! check → the seeded sentence that states the change, the same state machine
//! [`super::workspace_change`] runs for grounded rewrites. Before these, an
//! append (`Append the line 'third line' to notes.txt.`) reached the
//! general-change fallback and was written as the *whole* file.

use super::code_artifact::source_from_read_result;
use super::code_task::render_seeded_outcome;
use super::general_planner::compose_edit_request;
use super::intent_router::edit_arguments;
use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use super::workspace_change::{
    VerifiedChange, changed_lines_edit, identifier_tokens, plan_digest_verification,
    read_arguments, result_for_edit, result_for_path,
};
use super::write_request::{clean_path_token, looks_like_file_path, safe_relative_path, tokens};
use crate::normal_markov::{quoted_segment_spans, quoted_segments};
use crate::protocol::ChatMessage;
use crate::seed;

const MAX_ANCHOR_BYTES: usize = 4096;

/// How a computed change derives the file's new bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Computation {
    /// Quoted text placed as the last (`at_end`) or the first line.
    EndInsertion { text: String, at_end: bool },
    /// Every line that is exactly `text`, else its one in-line occurrence.
    Removal { text: String },
    /// The one line assigning `key` gets `value`, in the file's own quoting.
    Setting { key: String, value: String },
}

/// A computed change: the file, how its bytes are computed, and the seed
/// sentence (intent and slots) that states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ComputedChange {
    target: String,
    computation: Computation,
    intent: &'static str,
    slots: Vec<(&'static str, String)>,
}

/// The first computed change `task` asks for, if any.
pub(super) fn grounded_computed_change(task: &str) -> Option<ComputedChange> {
    grounded_end_insertion(task)
        .or_else(|| grounded_removal(task))
        .or_else(|| grounded_setting(task))
}

fn mentions(role: &str, task: &str) -> bool {
    seed::lexicon().mentions_role(role, &task.to_lowercase())
}

/// The one workspace path a request names and its one quoted segment that is
/// not that path.
fn quoted_payload_and_path(task: &str) -> Option<(String, String)> {
    let segments = quoted_segment_spans(task);
    let mut paths: Vec<&str> = Vec::new();
    for token in tokens(task) {
        let path = clean_path_token(token.text);
        let inside = segments.iter().any(|segment| {
            token.start >= segment.start && token.end <= segment.end && path != segment.text
        });
        if !inside
            && looks_like_file_path(path)
            && safe_relative_path(path)
            && !paths.contains(&path)
        {
            paths.push(path);
        }
    }
    let [path] = paths.as_slice() else {
        return None;
    };
    let payloads: Vec<&str> = segments
        .iter()
        .map(|segment| segment.text.as_str())
        .filter(|text| text != path && !text.trim().is_empty())
        .collect();
    let [payload] = payloads.as_slice() else {
        return None;
    };
    Some(((*path).to_owned(), (*payload).to_owned()))
}

/// `Append 'x' to notes.txt`, `Prepend "x" to a.md`: positioned by the seeded
/// `file_edit_position_end` / `file_edit_position_start` meanings.
fn grounded_end_insertion(task: &str) -> Option<ComputedChange> {
    let at_end = mentions("file_edit_position_end", task);
    if at_end == mentions("file_edit_position_start", task) {
        return None;
    }
    let (target, text) = quoted_payload_and_path(task).or_else(|| blank_line_and_path(task))?;
    let text = super::positional_edit::unescape_prose_newlines(&text);
    Some(ComputedChange {
        target,
        intent: if at_end {
            "file_edit_position_end"
        } else {
            "file_edit_position_start"
        },
        slots: vec![("{new}", text.clone())],
        computation: Computation::EndInsertion { text, at_end },
    })
}

/// `Delete the line 'x' from notes.txt`: the seeded `coding_text_remove_action`
/// with one quoted payload and one path, and no add action alongside it.
fn grounded_removal(task: &str) -> Option<ComputedChange> {
    if !mentions("coding_text_remove_action", task) || mentions("coding_member_add_action", task) {
        return None;
    }
    let (target, text) = quoted_payload_and_path(task)?;
    Some(ComputedChange {
        target,
        intent: "coding_text_remove",
        slots: vec![("{old}", text.clone())],
        computation: Computation::Removal { text },
    })
}

/// `Change the value of "debug" to true in config.json`: the seeded
/// `config_value_lead` names a setting; the edit request's old clause names its
/// key (quoted, or its last identifier) and the new clause its value.
fn grounded_setting(task: &str) -> Option<ComputedChange> {
    if !mentions("config_value_lead", task) {
        return None;
    }
    let (target, old_clause, value) = compose_edit_request(task)?;
    let quoted = quoted_segments(&old_clause);
    let key = match quoted.as_slice() {
        [key] => key.clone(),
        _ => identifier_tokens(&old_clause).next_back()?.to_owned(),
    };
    let value = value.trim().to_owned();
    if value.is_empty() {
        return None;
    }
    Some(ComputedChange {
        target,
        intent: "setting",
        slots: vec![("{old}", key.clone()), ("{new}", value.clone())],
        computation: Computation::Setting { key, value },
    })
}

impl ComputedChange {
    /// The file after the change, or `None` when it cannot apply.
    fn compute(&self, source: &str, missing: bool) -> Option<String> {
        match &self.computation {
            Computation::EndInsertion { text, at_end } => {
                Some(inserted_at_end(source, text, *at_end))
            }
            Computation::Removal { text } => {
                (!missing).then(|| removed_literal(source, text)).flatten()
            }
            Computation::Setting { key, value } => (!missing)
                .then(|| assigned_setting(source, key, value, &self.target))
                .flatten(),
        }
    }

    /// The smallest unique edit carrying the change, when there is one.
    fn edit(&self, source: &str, updated: &str) -> Option<(String, String)> {
        match &self.computation {
            Computation::EndInsertion { .. } if source.is_empty() => None,
            Computation::EndInsertion { at_end, .. } => compact_end_edit(source, updated, *at_end),
            Computation::Removal { .. } | Computation::Setting { .. } => {
                changed_lines_edit(source, updated)
            }
        }
    }
}

/// The file after the insertion; a file without a final newline keeps none.
fn inserted_at_end(source: &str, text: &str, at_end: bool) -> String {
    if source.is_empty() {
        [text, "\n"].concat()
    } else if at_end && source.ends_with('\n') {
        [source, text, "\n"].concat()
    } else if at_end {
        [source, "\n", text].concat()
    } else {
        [text, "\n", source].concat()
    }
}

/// The smallest unique run of whole lines at the insertion end, and what it
/// becomes, so the edit tool carries the change rather than the whole file.
fn compact_end_edit(source: &str, updated: &str, at_end: bool) -> Option<(String, String)> {
    let starts = source
        .match_indices('\n')
        .map(|(index, _)| index + 1)
        .filter(|start| *start < source.len());
    let cuts: Vec<usize> = if at_end {
        std::iter::once(0).chain(starts).rev().collect()
    } else {
        starts.chain(std::iter::once(source.len())).collect()
    };
    for cut in cuts {
        let anchor = if at_end {
            &source[cut..]
        } else {
            &source[..cut]
        };
        if anchor.trim().is_empty() {
            continue;
        }
        if anchor.len() > MAX_ANCHOR_BYTES {
            break;
        }
        if source.matches(anchor).count() != 1 {
            continue;
        }
        let replacement = if at_end {
            &updated[cut..]
        } else {
            &updated[..updated.len() - (source.len() - cut)]
        };
        return Some((anchor.to_owned(), replacement.to_owned()));
    }
    None
}

fn removed_literal(source: &str, text: &str) -> Option<String> {
    let kept: String = source
        .split_inclusive('\n')
        .filter(|line| {
            let bare = line
                .strip_suffix('\n')
                .map_or(*line, |body| body.strip_suffix('\r').unwrap_or(body));
            bare != text
        })
        .collect();
    if kept != source {
        return Some(kept);
    }
    (source.matches(text).count() == 1).then(|| source.replacen(text, "", 1))
}

/// A value every config format writes bare: a boolean, null or a number.
fn is_bare_literal(value: &str) -> bool {
    if matches!(value, "true" | "false" | "null") {
        return true;
    }
    let digits = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = digits
        .split_once('.')
        .map_or((digits, None), |(whole, fraction)| (whole, Some(fraction)));
    let all_digits =
        |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    all_digits(whole) && fraction.is_none_or(all_digits)
}

/// The one line assigning `key` (`"k": v`, `k: v`, `k = v`) with `value`, in
/// the file's own quoting; `None` when the key is assigned zero or several
/// times, or already holds the value.
fn assigned_setting(source: &str, key: &str, value: &str, target: &str) -> Option<String> {
    let lines: Vec<&str> = source.split('\n').collect();
    let assignments: Vec<(usize, &str, &str, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let (head, old, tail) = assignment(line, key)?;
            Some((index, head, old, tail))
        })
        .collect();
    let [(index, head, old, tail)] = assignments.as_slice() else {
        return None;
    };
    let quote = old
        .chars()
        .next()
        .filter(|first| matches!(first, '"' | '\''));
    let written = match quote {
        Some(quote) if !is_bare_literal(value) => format!("{quote}{value}{quote}"),
        None if target.ends_with(".json")
            && !is_bare_literal(value)
            && !value.starts_with(['"', '\'', '[', '{']) =>
        {
            serde_json::Value::String(value.to_owned()).to_string()
        }
        _ => value.to_owned(),
    };
    if written == *old {
        return None;
    }
    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    out[*index] = [*head, written.as_str(), *tail].concat();
    Some(out.join("\n"))
}

/// `(head, value, tail)` when `line` assigns `key`: optional indentation, the
/// key bare or quoted, `:` or `=`, the value, and an optional trailing comma.
fn assignment<'a>(line: &'a str, key: &str) -> Option<(&'a str, &'a str, &'a str)> {
    let indent = line.len() - line.trim_start().len();
    let rest = &line[indent..];
    let (quote, rest) = match rest.chars().next() {
        Some(quote @ ('"' | '\'')) => (Some(quote), &rest[1..]),
        _ => (None, rest),
    };
    let rest = rest.strip_prefix(key)?;
    let rest = match quote {
        Some(quote) => rest.strip_prefix(quote)?,
        None => rest,
    };
    let after_key = rest.trim_start();
    let after_separator = after_key.strip_prefix([':', '='])?;
    let value_start = line.len() - after_separator.trim_start().len();
    let body = line[value_start..].trim_end();
    let value = body.strip_suffix(',').unwrap_or(body).trim_end();
    let value_end = value_start + value.len();
    Some((
        &line[..value_start],
        &line[value_start..value_end],
        &line[value_end..],
    ))
}

/// Read, compute, edit (or write), check the digest, state the change.
pub(super) fn plan_computed_change_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &ComputedChange,
) -> Option<AgenticPlan> {
    let target = change.target.as_str();
    let Some(read) = result_for_path(current_turn, Capability::Read, target, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(target)));
    };
    let missing = super::tool_result::failure_message(&read, false, true).is_some();
    let source = if missing {
        String::new()
    } else {
        source_from_read_result(&read)
    };
    let Some(updated) = change
        .compute(&source, missing)
        .filter(|updated| *updated != source)
    else {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            target,
        )?));
    };
    let slots: Vec<(&str, &str)> = change
        .slots
        .iter()
        .map(|(slot, value)| (*slot, value.as_str()))
        .collect();
    let verified = VerifiedChange {
        target,
        expected: &updated,
        intent: change.intent,
        slots: &slots,
    };
    if let Some((old, new)) = change.edit(&source, &updated)
        && let Some(tool) = tool_for(tool_names, Capability::Edit)
    {
        if result_for_edit(current_turn, target, &old, &new).is_none() {
            return Some(plan_one(tool, edit_arguments(target, &old, &new)));
        }
        return plan_digest_verification(task, current_turn, tool_names, &verified);
    }
    if result_for_path(current_turn, Capability::Write, target, Some(&updated)).is_none() {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(tool, write_arguments(target, &updated)));
    }
    plan_digest_verification(task, current_turn, tool_names, &verified)
}

/// The seeded `file_edit_blank_line` with one named path and no quoted text:
/// the payload is the empty string (mirrors `blankLineAndPath`).
fn blank_line_and_path(task: &str) -> Option<(String, String)> {
    if !quoted_segment_spans(task).is_empty() || !mentions("file_edit_blank_line", task) {
        return None;
    }
    let mut paths: Vec<String> = super::positional_edit::unquoted_path_tokens(task)
        .iter()
        .map(|token| clean_path_token(token.text).to_owned())
        .filter(|path| looks_like_file_path(path) && safe_relative_path(path))
        .collect();
    paths.sort();
    paths.dedup();
    match paths.as_slice() {
        [path] => Some((path.clone(), String::new())),
        _ => None,
    }
}
