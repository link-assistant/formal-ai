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
use super::line_removal::{
    bare_line, removed_declarations, removed_line_indices, removed_lines, removed_literal,
};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use super::workspace_change::{
    VerifiedChange, changed_lines_edit, identifier_tokens, plan_digest_verification,
    read_arguments, result_for_edit, result_for_path,
};
use super::workspace_line_operation::{
    LineOperation, drops_most_of_file, grounded_line_operation, named_target_and_payloads,
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
    /// Quoted text placed as the last (`at_end`) or the first line; lines
    /// given under the request keep their shared `indentation` when the file
    /// has lines indented exactly so (PR #1188 G16).
    EndInsertion {
        text: String,
        at_end: bool,
        indentation: Option<String>,
    },
    /// Every line that is exactly `text`, else its one in-line occurrence.
    Removal { text: String },
    /// Whole lines (the request names a line): see `removed_line_indices`.
    LineRemoval { text: String, containing: bool },
    /// The one line assigning `key` (holding `held`, when stated) gets
    /// `value`, in the file's own quoting.
    Setting {
        key: String,
        value: String,
        held: Option<String>,
    },
    /// Every line that is `old` becomes `new` (the request names a line), or
    /// only the first one after `context`.
    LineReplacement {
        old: String,
        new: String,
        context: Option<String>,
        rebase: bool,
    },
    /// Every whole-word `word` becomes its discovered `correction`.
    TypoFix { word: String, correction: String },
    /// The functions the file declares under these names go whole, with the
    /// comment and attribute lines directly above them.
    DeclarationRemoval { names: Vec<String> },
    /// A whole-line operation: numbered, adjacent, moved or swapped lines.
    Line(LineOperation),
    /// `text` placed at one end of the Markdown section under `heading`
    /// (PR #1188 G35).
    Section {
        heading: String,
        text: String,
        at_end: bool,
    },
    /// Each `(old, new)` pair replaced everywhere, in order (PR #1188 G82).
    ReplaceList { pairs: Vec<(String, String)> },
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
        .or_else(|| grounded_line_replacement(task))
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
pub(super) fn sentence_words(task: &str) -> String {
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
pub(super) fn names_line(task: &str) -> bool {
    // A path's own words are no line: `x-line.lino` names a file (PR #1188 G83).
    seed::lexicon()
        .meaning("line")
        .is_some_and(|meaning| meaning.evidenced_in(&outside_quotes(&without_path_words(task))))
}

/// `task` with every unquoted path token blanked.
///
/// A path's own words (`x-line.lino`, `line-budget.lino`) never read as the
/// words of the request (PR #1188 G83; mirrors `withoutPathWords`).
pub(super) fn without_path_words(task: &str) -> String {
    let segments = quoted_segment_spans(task);
    let mut out = task.to_owned();
    for token in super::write_request::tokens(task).iter().rev() {
        if segments
            .iter()
            .any(|segment| token.start < segment.end && token.end > segment.start)
        {
            continue;
        }
        let path = super::write_request::clean_path_token(token.text);
        if super::write_request::looks_like_file_path(path)
            && super::write_request::safe_relative_path(path)
        {
            out.replace_range(token.start..token.end, &" ".repeat(token.end - token.start));
        }
    }
    out
}

/// The lowered request outside its quotes.
fn outside_quotes(task: &str) -> String {
    let outside = crate::solver_handlers::text_outside_quoted_segments(task);
    crate::engine::normalize_prompt(&outside).to_lowercase()
}

/// The one workspace path a request names and its one quoted segment that is
/// not that path. An unquoted path outranks a quoted one, which is then the
/// payload (PR #1188 T35).
fn quoted_payload_and_path(task: &str) -> Option<(String, String)> {
    let (target, payloads) = named_target_and_payloads(task)?;
    let [payload] = payloads.as_slice() else {
        return None;
    };
    Some((target, payload.clone()))
}

/// A whole-line operation (numbered, adjacent, moved or swapped lines) as a
/// computed change. It is read before a rewrite: `move 'x' after 'y'` is not
/// an insertion (PR #1188).
/// Several quoted replacements in one file, asked in one sentence, as one
/// computed change (PR #1188 G82; mirrors `groundedReplaceList`).
pub(super) fn grounded_replace_list(task: &str) -> Option<ComputedChange> {
    let (target, pairs) = super::replace_list::replace_list(task)?;
    let listed: Vec<String> = pairs
        .iter()
        .map(|(old, new)| format!("`{old}` → `{new}`"))
        .collect();
    Some(ComputedChange {
        target,
        intent: "coding_texts_replaced",
        slots: vec![
            (concat!("{", "pairs", "}"), listed.join(", ")),
            (concat!("{", "count", "}"), pairs.len().to_string()),
        ],
        computation: Computation::ReplaceList { pairs },
    })
}

pub(super) fn grounded_line_change(task: &str) -> Option<ComputedChange> {
    let change = grounded_line_operation(task)?;
    Some(ComputedChange {
        target: change.target,
        computation: Computation::Line(change.operation),
        intent: change.intent,
        slots: change.slots,
    })
}

/// `Append 'x' to notes.txt`, `Prepend "x" to a.md`: positioned by the seeded
/// `file_edit_position_end` / `file_edit_position_start` meanings.
pub(super) fn grounded_end_insertion(task: &str) -> Option<ComputedChange> {
    // `Append these lines to f:` followed by lines: the first line is the
    // request and the lines under it are the text added, verbatim (PR #1188).
    let block = super::positional_edit::introduced_block(task);
    // Position cues are read outside the quoted payload: a line saying "at the
    // start of x" is text being appended (PR #1188 G64).
    let cues = block
        .as_ref()
        .map_or_else(|| outside_quotes(task), |block| sentence_words(block.head));
    let lexicon = seed::lexicon();
    let at_end = lexicon.mentions_role("file_edit_position_end", &cues);
    if at_end == lexicon.mentions_role("file_edit_position_start", &cues) {
        return None;
    }
    // `… at the end of the section '## Usage' in README.md` (PR #1188 G35).
    if block.is_none()
        && let Some(section) = super::markdown_section::section_scope(task, &outside_quotes(task))
    {
        let text = super::positional_edit::unescape_prose_newlines(&section.text);
        return Some(ComputedChange {
            target: section.target,
            intent: "file_edit_position_after",
            slots: vec![
                ("{new}", text.clone()),
                ("{anchor}", section.heading.clone()),
            ],
            computation: Computation::Section {
                heading: section.heading,
                text,
                at_end,
            },
        });
    }
    let (target, text, indentation) = if let Some(block) = block {
        let indentation = (!block.verbatim).then(|| block.indentation.to_owned());
        (block_target(block.head)?, block.text, indentation)
    } else if let Some((target, text)) = quoted_lines_and_path(task) {
        (target, text, None)
    } else {
        let (target, text) = quoted_payload_and_path(task)
            .or_else(|| blank_line_and_path(task))
            .or_else(|| line_payload_and_path(task))?;
        (
            target,
            super::positional_edit::unescape_prose_newlines(&text),
            None,
        )
    };
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
        computation: Computation::EndInsertion {
            text,
            at_end,
            indentation,
        },
    })
}

/// Several quoted literals, joined only by seeded joiners and commas, are the
/// lines added, each its own line, as written (PR #1188 G53, G54).
fn quoted_lines_and_path(task: &str) -> Option<(String, String)> {
    let (target, payloads) = named_target_and_payloads(task)?;
    if payloads.len() < 2 {
        return None;
    }
    let text = super::positional_edit::joined_literal_lines(task, &target)?;
    Some((target, text))
}

/// `Delete the line 'x' from notes.txt`: the seeded `coding_text_remove_action`
/// with one quoted payload and one path, and no add action alongside it.
fn grounded_removal(task: &str) -> Option<ComputedChange> {
    if !mentions("coding_text_remove_action", task) || mentions("coding_member_add_action", task) {
        return None;
    }
    let (target, text) = quoted_payload_and_path(task)?;
    // A request that names a line removes whole lines, never only the
    // payload inside one (PR #1188 G60).
    let computation = if names_line(task) {
        let containing =
            seed::lexicon().mentions_role("line_containment_cue", &outside_quotes(task));
        Computation::LineRemoval {
            text: text.clone(),
            containing,
        }
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
/// identifiers; `removed_declarations` keeps those the file itself declares
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
///
/// `Change MAXIMUM_RATIO from 8 to 16 in p.rs` states the value the key holds
/// now (the seeded `file_edit_old_lead_cue`): the line assigning the key that
/// value is the one changed, and a key assigned nowhere leaves the stated
/// value itself, when it occurs once as a word (PR #1188 G2).
fn grounded_setting(task: &str) -> Option<ComputedChange> {
    let (target, old_clause, value) = compose_edit_request(task)?;
    let stated = super::workspace_setting::stated_old_value(&old_clause);
    if stated.is_none() && !mentions("config_value_lead", task) {
        return None;
    }
    // `set the contents of note.txt to hello` writes the file; its contents are
    // not a key (PR #1188 G68, issue #745).
    if stated.is_none() && mentions("file_contents_source_cue", task) {
        return None;
    }
    let key_clause = stated
        .as_ref()
        .map_or(old_clause.as_str(), |(key, _)| key.as_str());
    let quoted = quoted_segments(key_clause);
    let key = match quoted.as_slice() {
        [key] => key.clone(),
        _ => identifier_tokens(key_clause).next_back()?.to_owned(),
    };
    let value = value.trim().to_owned();
    if value.is_empty() {
        return None;
    }
    Some(ComputedChange {
        target,
        intent: "setting",
        slots: vec![("{old}", key.clone()), ("{new}", value.clone())],
        computation: Computation::Setting {
            key,
            value,
            held: stated.map(|(_, held)| held),
        },
    })
}

/// `Replace the line 'x' with 'y' in f`: the seeded `line` meaning outside
/// the quotes, both clauses quoted. A line replacement never touches text
/// inside a longer line (T32: `text pays` became `text "pay"s`). With a seeded
/// anchor context (`the line 'x' that follows the line 'z'`), the first such
/// line after `z` is the one replaced (T62).
fn grounded_line_replacement(task: &str) -> Option<ComputedChange> {
    if !names_line(task)
        || seed::lexicon().mentions_role(
            seed::ROLE_CODING_IDENTIFIER_RENAME_ACTION,
            &task.to_lowercase(),
        )
    {
        return None;
    }
    let context = super::positional_edit::anchor_context(task);
    let request = context.as_ref().map_or_else(
        || task.to_owned(),
        |(start, end, _)| [&task[..*start], " ", &task[*end..]].concat(),
    );
    let (target, original, replacement) = compose_edit_request(&request)?;
    if super::positional_edit::compose_positional_insert(task).is_some() {
        return None;
    }
    let quoted: Vec<String> = quoted_segments(&request)
        .iter()
        .map(String::as_str)
        .map(super::positional_edit::unescape_prose_newlines)
        .collect();
    // Lines under the request (G22) replace the line as its siblings, unless
    // they were fenced or quoted.
    let block =
        super::positional_edit::introduced_block(task).filter(|block| block.text == replacement);
    // Listed lines (`the three lines 'a', 'b' and 'c'`) are quoted one by one (G80).
    let listed = !quoted.contains(&original);
    if (listed
        && !original
            .split('\n')
            .all(|line| quoted.iter().any(|text| text == line)))
        || (block.is_none() && !quoted.contains(&replacement))
        || original.is_empty()
        || original == replacement
    {
        return None;
    }
    Some(ComputedChange {
        target,
        intent: "coding_text_replaced",
        // Listed lines are named one by one, not as one span holding line breaks.
        slots: vec![
            (
                "{old}",
                if listed {
                    original.replace('\n', "`, `")
                } else {
                    original.clone()
                },
            ),
            ("{new}", replacement.clone()),
        ],
        computation: Computation::LineReplacement {
            old: original,
            new: replacement,
            context: context.map(|(_, _, text)| text),
            rebase: block.is_some_and(|block| !block.verbatim),
        },
    })
}

/// The smallest unique changed run of lines, or the lines from the context's
/// line through the change.
///
/// When the changed run repeats, the run is widened up to the context's line:
/// the context occurs once, so the widened run does not (mirrors
/// `contextLinesEdit`).
fn context_lines_edit(
    source: &str,
    updated: &str,
    context: Option<&str>,
) -> Option<(String, String)> {
    let compact = changed_lines_edit(source, updated);
    let Some(context) = context.filter(|_| compact.is_none()) else {
        return compact;
    };
    let mut found = source.match_indices(context).map(|(at, _)| at);
    let (Some(at), None) = (found.next(), found.next()) else {
        return None;
    };
    let start = source[..at].rfind('\n').map_or(0, |newline| newline + 1);
    let suffix = source[start..]
        .bytes()
        .rev()
        .zip(updated[start..].bytes().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let end = source[source.len() - suffix..]
        .find('\n')
        .map_or(source.len(), |newline| source.len() - suffix + newline);
    Some((
        source[start..end].to_owned(),
        updated[start..updated.len() - (source.len() - end)].to_owned(),
    ))
}

impl ComputedChange {
    /// The file after the change, or `None` when it cannot apply.
    fn compute(&self, source: &str, missing: bool) -> Option<String> {
        match &self.computation {
            Computation::EndInsertion {
                text,
                at_end,
                indentation,
            } => Some(inserted_at_end(
                source,
                &appended_block(source, text, indentation.as_deref()),
                *at_end,
            )),
            Computation::Removal { text } => {
                (!missing).then(|| removed_literal(source, text)).flatten()
            }
            Computation::LineRemoval { text, containing } => (!missing)
                .then(|| removed_lines(source, text, *containing))
                .flatten(),
            Computation::Setting { key, value, held } => (!missing)
                .then(|| {
                    super::workspace_setting::assigned_setting(
                        source,
                        key,
                        value,
                        &self.target,
                        held.as_deref(),
                    )
                })
                .flatten(),
            Computation::LineReplacement {
                old,
                new,
                context,
                rebase,
            } => (!missing)
                .then(|| {
                    let new = if *rebase {
                        super::positional_edit::rebased_block(new, line_indentation(source, old))
                    } else {
                        new.clone()
                    };
                    super::workspace_setting::replaced_lines(source, old, &new, context.as_deref())
                })
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
            Computation::Line(operation) => (!missing).then(|| operation.compute(source)).flatten(),
            Computation::Section {
                heading,
                text,
                at_end,
            } => (!missing)
                .then(|| {
                    super::markdown_section::inserted_in_section(source, heading, text, *at_end)
                })
                .flatten()
                .map(|(updated, _)| updated),
            Computation::ReplaceList { pairs } => (!missing)
                .then(|| super::replace_list::replaced_in_order(source, pairs))
                .flatten(),
        }
    }

    /// Whether the request states the change's whole extent (named
    /// declarations, a numbered range), so it may remove most of a file.
    const fn bounded(&self) -> bool {
        match &self.computation {
            Computation::DeclarationRemoval { .. } => true,
            Computation::Line(operation) => operation.bounded(),
            _ => false,
        }
    }

    /// The smallest unique edit carrying the change, when there is one.
    fn edit(&self, source: &str, updated: &str) -> Option<(String, String)> {
        match &self.computation {
            Computation::EndInsertion { .. } if source.is_empty() => None,
            Computation::EndInsertion { at_end, .. } => compact_end_edit(source, updated, *at_end),
            Computation::LineReplacement { context, .. } => {
                context_lines_edit(source, updated, context.as_deref())
            }
            Computation::Removal { .. }
            | Computation::LineRemoval { .. }
            | Computation::Setting { .. }
            | Computation::TypoFix { .. }
            | Computation::DeclarationRemoval { .. }
            | Computation::Line(_)
            | Computation::Section { .. }
            | Computation::ReplaceList { .. } => changed_lines_edit(source, updated),
        }
    }
}

/// The file after the insertion; a file without a final newline keeps none.
/// Lines under an append request, at the indentation they were given.
///
/// Only when the file already has lines indented exactly so (a `.lino` table
/// appended among its siblings); else at the file's top level, since the
/// indentation of lines written under a request is otherwise only its layout
/// (PR #1188 G16).
fn appended_block(source: &str, text: &str, indentation: Option<&str>) -> String {
    match indentation {
        Some(indentation)
            if !indentation.is_empty()
                && source.split('\n').any(|line| {
                    !line.trim().is_empty()
                        && super::positional_edit::leading_indentation(line) == indentation
                }) =>
        {
            super::positional_edit::rebased_block(text, indentation)
        }
        _ => text.to_owned(),
    }
}

/// The indentation of the first line that is `text` but for its indentation.
fn line_indentation<'a>(source: &'a str, text: &str) -> &'a str {
    source
        .split('\n')
        .find(|line| line.trim() == text.trim())
        .map_or("", super::positional_edit::leading_indentation)
}

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
        if let Computation::LineRemoval { text, containing } = &self.computation {
            let lines: Vec<&str> = source.split_inclusive('\n').map(bare_line).collect();
            let removed: Vec<&str> = removed_line_indices(source, text, *containing)
                .into_iter()
                .map(|index| lines[index])
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
        if let Computation::Section {
            heading,
            text,
            at_end,
        } = &self.computation
            && let Some((_, follows)) =
                super::markdown_section::inserted_in_section(source, heading, text, *at_end)
        {
            return (
                self.intent,
                vec![("{new}", text.clone()), ("{anchor}", follows)],
            );
        }
        if let Computation::DeclarationRemoval { names } = &self.computation
            && let Some((_, declared)) = removed_declarations(source, names)
        {
            return (self.intent, vec![("{old}", declared.join("`, `"))]);
        }
        if let Computation::Line(operation) = &self.computation
            && let Some(reported) = operation.reported(source)
        {
            return reported;
        }
        (self.intent, self.slots.clone())
    }
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
    if let Some(answer) = super::edit_scope::scoped_decline(task, target, &source) {
        return Some(AgenticPlan::Final(answer));
    }
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
    // A change that would drop most of the file is refused unless its request
    // states that extent (PR #1188 T29).
    if !change.bounded() && drops_most_of_file(&source, &updated) {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_fragment_refused",
            task,
            target,
        )?));
    }
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

/// The one path a colon-ended request line names, with nothing else quoted
/// (mirrors `blockPayloadAndPath`).
fn block_target(head: &str) -> Option<String> {
    let mut paths: Vec<&str> = super::positional_edit::unquoted_path_tokens(head)
        .iter()
        .map(|token| clean_path_token(token.text))
        .filter(|path| looks_like_file_path(path) && safe_relative_path(path))
        .collect();
    paths.sort_unstable();
    paths.dedup();
    let [path] = paths.as_slice() else {
        return None;
    };
    quoted_segment_spans(head)
        .iter()
        .all(|segment| segment.text == *path)
        .then(|| (*path).to_owned())
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
