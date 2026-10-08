//! Workspace changes whose new bytes are computed from the file they change
//! (PR #1188 dogfooding): text added at one end of a file, quoted text
//! removed from it, a configuration setting assigned a new value, a named
//! function removed whole.
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
use super::write_request::{
    bare_surfaces, clean_cue_token, clean_path_token, looks_like_file_path, safe_relative_path,
    tokens,
};
use crate::normal_markov::{quoted_segment_spans, quoted_segments};
use crate::protocol::ChatMessage;
use crate::seed;
use crate::workspace_change_learning::{
    RewriteScope, execute_scoped_workspace_rewrite, is_identifier_word,
};

const MAX_ANCHOR_BYTES: usize = 4096;

/// How a computed change derives the file's new bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Computation {
    /// Quoted text placed as the last (`at_end`) or the first line.
    EndInsertion { text: String, at_end: bool },
    /// Every line that is exactly `text`, else its one in-line occurrence.
    Removal { text: String },
    /// Every line that contains `text`, whole (the request names a line).
    LineRemoval { text: String },
    /// The one line assigning `key` gets `value`, in the file's own quoting.
    Setting { key: String, value: String },
    /// Every whole-word `word` becomes its discovered `correction`.
    TypoFix { word: String, correction: String },
    /// The functions the file declares under these names go whole, with the
    /// comment and attribute lines directly above them.
    DeclarationRemoval { names: Vec<String> },
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
        .or_else(|| grounded_declaration_removal(task))
        .or_else(|| grounded_setting(task))
        .or_else(|| grounded_typo_fix(task))
}

/// `Fix the typo 'smal' in README.md`: the seeded `typo_fix_lead` with one
/// quoted word and one path, and no stated correction. The correction is
/// discovered ([`super::spelling::corrected_spelling`]), and the change is the
/// word-scoped replacement a stated correction would have made.
fn grounded_typo_fix(task: &str) -> Option<ComputedChange> {
    if !mentions("typo_fix_lead", task) {
        return None;
    }
    let (target, word) = quoted_payload_and_path(task)?;
    if !is_identifier_word(&word) {
        return None;
    }
    let correction = super::spelling::corrected_spelling(&word)?;
    Some(ComputedChange {
        target,
        intent: "coding_text_replaced",
        slots: vec![("{old}", word.clone()), ("{new}", correction.clone())],
        computation: Computation::TypoFix { word, correction },
    })
}

fn mentions(role: &str, task: &str) -> bool {
    seed::lexicon().mentions_role(role, &sentence_words(task))
}

/// The marks that end a sentence or a clause.
const SENTENCE_MARKS: [char; 11] = [
    '.', '!', '?', ';', ':', ',', '\u{0964}', '\u{0965}', '\u{3002}', '\u{ff01}', '\u{ff1f}',
];

/// The lowered request with the marks that end a sentence or clause set off
/// as spaces, so a verb that closes its sentence (`… पंक्ति हटाओ।`, `…
/// удали.`) is still a whole word. A dot inside a token (`t.md`) is not
/// followed by a space and stays.
fn sentence_words(task: &str) -> String {
    let lowered = task.to_lowercase();
    let characters: Vec<char> = lowered.chars().collect();
    let mut out = String::with_capacity(lowered.len());
    let mut index = 0;
    while index < characters.len() {
        let mut end = index;
        while end < characters.len() && SENTENCE_MARKS.contains(&characters[end]) {
            end += 1;
        }
        if end > index && characters.get(end).is_none_or(|next| next.is_whitespace()) {
            out.push(' ');
            index = end;
        } else {
            out.push(characters[index]);
            index += 1;
        }
    }
    out
}

/// Whether the request is about text inside a file: it quotes a payload that
/// is not a path, or names a line (the seeded `line` meaning) or another unit
/// of text (`file_text_unit`: word, phrase, occurrence, text, in every
/// registered language) outside its quotes. A destructive shell intent never
/// reads such a request as deleting the file (PR #1188).
pub(super) fn edits_inside_a_file(prompt: &str) -> bool {
    if quoted_segment_spans(prompt)
        .iter()
        .map(|segment| segment.text.trim())
        .any(|text| !text.is_empty() && !super::shell_command::looks_like_a_path(text))
    {
        return true;
    }
    let outside = crate::engine::normalize_prompt(
        &crate::solver_handlers::text_outside_quoted_segments(prompt),
    )
    .to_lowercase();
    seed::lexicon()
        .meaning("line")
        .is_some_and(|meaning| meaning.evidenced_in(&outside))
        || seed::lexicon().mentions_role("file_text_unit", &outside)
}

/// Whether the request names a line outside its quotes: the seeded `line`
/// meaning (`the line containing '…'`, `lines`, `строку`, `पंक्ति`, `的行`).
fn names_line(task: &str) -> bool {
    let outside = crate::solver_handlers::text_outside_quoted_segments(task);
    seed::lexicon().meaning("line").is_some_and(|meaning| {
        meaning.evidenced_in(&crate::engine::normalize_prompt(&outside).to_lowercase())
    })
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
    let (target, text) = quoted_payload_and_path(task)
        .or_else(|| blank_line_and_path(task))
        .or_else(|| line_payload_and_path(task))?;
    let text = super::positional_edit::unescape_prose_newlines(&text);
    Some(ComputedChange {
        target,
        // An empty line has no text to quote back; it is stated in its own words.
        intent: if text.is_empty() {
            "file_edit_blank_line"
        } else if at_end {
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
    // A request that names a line removes whole lines: every line containing
    // the payload goes, never only the payload inside it (PR #1188).
    let computation = if names_line(task) {
        Computation::LineRemoval { text: text.clone() }
    } else {
        Computation::Removal { text: text.clone() }
    };
    Some(ComputedChange {
        target,
        intent: "coding_text_remove",
        slots: vec![("{old}", text)],
        computation,
    })
}

/// `Delete the functions a and b from util.rs`: the seeded
/// `coding_text_remove_action` and `coding_declaration_noun` with no add
/// action, no quoted text and one named path. The names are the request's
/// identifiers; [`removed_declarations`] keeps those the file itself declares
/// as functions (mirrors `groundedDeclarationRemoval`, PR #1188 dogfooding).
fn grounded_declaration_removal(task: &str) -> Option<ComputedChange> {
    if !mentions("coding_text_remove_action", task)
        || mentions("coding_member_add_action", task)
        || !mentions("coding_declaration_noun", task)
        || !quoted_segment_spans(task).is_empty()
    {
        return None;
    }
    let mut paths: Vec<String> = super::positional_edit::unquoted_path_tokens(task)
        .iter()
        .map(|token| clean_path_token(token.text).to_owned())
        .filter(|path| looks_like_file_path(path) && safe_relative_path(path))
        .collect();
    paths.sort();
    paths.dedup();
    let [target] = paths.as_slice() else {
        return None;
    };
    let mut names: Vec<String> = Vec::new();
    for token in tokens(task) {
        if clean_path_token(token.text) == target {
            continue;
        }
        for name in identifier_tokens(token.text) {
            if !names.iter().any(|known| known == name) {
                names.push(name.to_owned());
            }
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(ComputedChange {
        target: target.clone(),
        intent: "coding_text_remove",
        slots: vec![("{old}", names.join("`, `"))],
        computation: Computation::DeclarationRemoval { names },
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
            Computation::LineRemoval { text } => {
                (!missing).then(|| removed_lines(source, text)).flatten()
            }
            Computation::Setting { key, value } => (!missing)
                .then(|| assigned_setting(source, key, value, &self.target))
                .flatten(),
            Computation::TypoFix { word, correction } => (!missing)
                .then(|| {
                    execute_scoped_workspace_rewrite(source, word, correction, RewriteScope::Word)
                        .ok()
                        .map(|execution| execution.output)
                })
                .flatten(),
            Computation::DeclarationRemoval { names } => (!missing)
                .then(|| removed_declarations(source, names).map(|(updated, _)| updated))
                .flatten(),
        }
    }

    /// The smallest unique edit carrying the change, when there is one.
    fn edit(&self, source: &str, updated: &str) -> Option<(String, String)> {
        match &self.computation {
            Computation::EndInsertion { .. } if source.is_empty() => None,
            Computation::EndInsertion { at_end, .. } => compact_end_edit(source, updated, *at_end),
            Computation::Removal { .. }
            | Computation::LineRemoval { .. }
            | Computation::Setting { .. }
            | Computation::TypoFix { .. }
            | Computation::DeclarationRemoval { .. } => changed_lines_edit(source, updated),
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

impl ComputedChange {
    /// The seed sentence that states the change made to `source`: lines that
    /// only contained the payload are reported as lines, with their count;
    /// every other change keeps its own intent and slots.
    fn reported(&self, source: &str) -> (&'static str, Vec<(&'static str, String)>) {
        if let Computation::LineRemoval { text } = &self.computation {
            let removed: Vec<&str> = source
                .split_inclusive('\n')
                .map(bare_line)
                .filter(|line| line.contains(text.as_str()))
                .collect();
            if removed.iter().any(|line| line != text) {
                return (
                    "line",
                    vec![
                        ("{old}", text.clone()),
                        ("{count}", removed.len().to_string()),
                    ],
                );
            }
        }
        if let Computation::DeclarationRemoval { names } = &self.computation
            && let Some((_, declared)) = removed_declarations(source, names)
        {
            return (self.intent, vec![("{old}", declared.join("`, `"))]);
        }
        (self.intent, self.slots.clone())
    }
}

/// `source` without the function each of `names` declares, and the names that
/// were declared; `None` when none is. A name declared zero or several times
/// is left alone; a block left between two blank lines takes one of them with
/// it, and a block that ended the file the blank line before it (mirrors
/// `removedDeclarations`).
fn removed_declarations(source: &str, names: &[String]) -> Option<(String, Vec<String>)> {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let keywords = seed::lexicon().words_for_role("function_declaration_keyword");
    let mut removed = std::collections::BTreeSet::new();
    let mut declared = Vec::new();
    for name in names {
        let headers: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| declares_function(line, name, &keywords))
            .map(|(index, _)| index)
            .collect();
        let [header] = headers.as_slice() else {
            continue;
        };
        let Some((start, end)) = declaration_span(&lines, *header) else {
            continue;
        };
        removed.extend(start..=end);
        declared.push(name.clone());
    }
    if declared.is_empty() {
        return None;
    }
    let blank = |index: usize| lines.get(index).is_some_and(|line| line.trim().is_empty());
    for index in removed.clone() {
        let next = index + 1;
        if removed.contains(&next) {
            continue;
        }
        let mut start = index;
        while start > 0 && removed.contains(&(start - 1)) {
            start -= 1;
        }
        if next == lines.len() {
            // A block that ended the file takes the blank line before it.
            if start > 0 && blank(start - 1) {
                removed.insert(start - 1);
            }
        } else if blank(next) && (start == 0 || blank(start - 1)) {
            removed.insert(next);
        }
    }
    let kept: String = lines
        .iter()
        .enumerate()
        .filter(|(index, _)| !removed.contains(index))
        .map(|(_, line)| *line)
        .collect();
    Some((kept, declared))
}

/// A seeded `function_declaration_keyword`, then `name`, then `(` or `<`
/// (mirrors `declaresFunction`). The line is read as identifier runs and
/// single other characters, so `fn name (` and `function name<T>(` both count.
fn declares_function(line: &str, name: &str, keywords: &[String]) -> bool {
    let is_identifier =
        |character: char| character.is_ascii_alphanumeric() || "_$".contains(character);
    let mut words: Vec<&str> = Vec::new();
    let mut run: Option<usize> = None;
    for (index, character) in line.char_indices() {
        if is_identifier(character) {
            if run.is_none() {
                run = Some(index);
            }
            continue;
        }
        if let Some(from) = run.take() {
            words.push(&line[from..index]);
        }
        words.push(&line[index..index + character.len_utf8()]);
    }
    if let Some(from) = run {
        words.push(&line[from..]);
    }
    let skip_blank = |mut at: usize| {
        while words.get(at).is_some_and(|word| word.trim().is_empty()) {
            at += 1;
        }
        at
    };
    words.iter().enumerate().any(|(index, word)| {
        if !keywords.iter().any(|keyword| keyword == word) {
            return false;
        }
        let at = skip_blank(index + 1);
        if words.get(at) != Some(&name) {
            return false;
        }
        matches!(words.get(skip_blank(at + 1)), Some(&("(" | "<")))
    })
}

/// The first and last line of the declaration whose header is line `header`,
/// with the comment and attribute lines directly above it (mirrors
/// `declarationSpan`). A braced body ends at the first later line that closes
/// at the header's indentation; an indented body (a header ending in `:`) at
/// the last line indented deeper.
fn declaration_span(lines: &[&str], header: usize) -> Option<(usize, usize)> {
    let indent = |line: &str| line.len() - line.trim_start_matches([' ', '\t']).len();
    let header_line = bare_line(lines[header]);
    let depth = indent(header_line);
    let closing = header_line.trim_end();
    let end = if closing.ends_with(['}', ';']) {
        header
    } else if closing.ends_with(':') {
        let mut end = header;
        for (index, line) in lines.iter().copied().enumerate().skip(header + 1) {
            if line.trim().is_empty() {
                continue;
            }
            if indent(line) <= depth {
                break;
            }
            end = index;
        }
        end
    } else {
        (header + 1..lines.len())
            .find(|index| indent(lines[*index]) == depth && lines[*index].trim().starts_with('}'))?
    };
    let opens_comment = |line: &str| line.trim().starts_with("/*");
    let mut start = header;
    while start > 0 {
        let above = lines[start - 1].trim();
        if above.ends_with("*/") {
            let mut open = start - 1;
            while open > 0 && !opens_comment(lines[open]) {
                open -= 1;
            }
            if !opens_comment(lines[open]) {
                break;
            }
            start = open;
        } else if ["//", "#[", "@"].iter().any(|lead| above.starts_with(lead)) {
            start -= 1;
        } else {
            break;
        }
    }
    Some((start, end))
}

/// A line without its `\n` or `\r\n` ending.
fn bare_line(line: &str) -> &str {
    line.strip_suffix('\n')
        .map_or(line, |body| body.strip_suffix('\r').unwrap_or(body))
}

/// `source` without every line containing `text`; `None` when none does.
fn removed_lines(source: &str, text: &str) -> Option<String> {
    let kept: String = source
        .split_inclusive('\n')
        .filter(|line| !line.contains(text))
        .collect();
    (kept != source).then_some(kept)
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
        None if std::path::Path::new(target)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
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
    // A read that came back as the client's file block is the file, whatever
    // its text says: a ledger that quotes error lines is not a missing file.
    let missing = super::code_artifact::source_from_agent_read_result(&read).is_none()
        && super::tool_result::failure_message(&read, false, true).is_some();
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
    let (intent, reported) = change.reported(&source);
    let slots: Vec<(&str, &str)> = reported
        .iter()
        .map(|(slot, value)| (*slot, value.as_str()))
        .collect();
    let verified = VerifiedChange {
        target,
        expected: &updated,
        intent,
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

/// `Append the line third to notes.txt`: no quoted text, one path, and the
/// seeded `file_edit_line_lead`; the line is the words after the lead up to
/// the last destination cue before the path (`go to bed to notes.txt` keeps
/// `go to bed`). A request without a lead is declined, never guessed.
fn line_payload_and_path(task: &str) -> Option<(String, String)> {
    if !quoted_segment_spans(task).is_empty() {
        return None;
    }
    let tokens = tokens(task);
    let paths: Vec<(usize, &str)> = tokens
        .iter()
        .enumerate()
        .map(|(index, token)| (index, clean_path_token(token.text)))
        .filter(|(_, path)| looks_like_file_path(path) && safe_relative_path(path))
        .collect();
    let (path_index, target) = *paths.first()?;
    if paths.iter().any(|(_, path)| *path != target) {
        return None;
    }
    let mut leads: Vec<Vec<String>> = seed::lexicon()
        .words_for_role("file_edit_line_lead")
        .iter()
        .map(|lead| {
            lead.to_lowercase()
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .collect();
    leads.sort_by_key(|words| std::cmp::Reverse(words.len()));
    let lead_end = (0..path_index).find_map(|index| {
        leads
            .iter()
            .find(|words| {
                words.iter().enumerate().all(|(offset, word)| {
                    index + offset < path_index
                        && clean_cue_token(tokens[index + offset].text) == *word
                })
            })
            .map(|words| index + words.len())
    })?;
    let destinations = bare_surfaces("file_write_destination_cue");
    let end = (lead_end..path_index)
        .rev()
        .find(|index| destinations.contains(&clean_cue_token(tokens[*index].text)))?;
    if end <= lead_end {
        return None;
    }
    Some((
        target.to_owned(),
        task[tokens[lead_end].start..tokens[end - 1].end].to_owned(),
    ))
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
