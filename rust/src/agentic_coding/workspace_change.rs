//! Grounded, verified workspace transformations learned from coding-task runs.
//!
//! The planner never edits request prose. It compiles a bounded transformation,
//! reads the client-owned bytes, executes the transformation in memory, applies
//! a compact edit or bounded replace-all operation, and accepts success only
//! after an exact content-digest observation. The same state machine composes
//! source creation with a second module-registration edit, so a multi-file
//! request cannot stop after its first observable effect.

mod digest_verification;
pub(super) use digest_verification::{plan_digest_verification, plan_write_digest_verification};

use serde_json::{Value, json};
use std::path::Path;

use super::code_artifact::source_from_read_result;
use super::code_task::{
    render_rust_template, render_seeded_change, render_seeded_outcome, verified_source_description,
};
use super::final_result::{FinalDisposition, FinalResult, record};
use super::general_planner::compose_edit_request;
use super::intent_router::edit_arguments;
use super::planner::{
    AgenticPlan, Capability, plan_one, tool_capability, tool_for, write_arguments,
};
use super::positional_edit::PositionalInsert;
use crate::normal_markov::{quoted_segments, unwrap_transport_quotes};
use crate::protocol::ChatMessage;
use crate::seed;
use crate::workspace_change_learning::{
    RewriteScope, execute_scoped_workspace_rewrite, is_identifier_word, word_scoped_matches,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroundedRewrite {
    target: String,
    pattern: String,
    replacement: String,
    /// How the pattern is allowed to match. Renaming an identifier is a
    /// word-scoped edit; replacing a quoted literal is a substring one -- or a
    /// word-scoped one when the new word contains the old (`smal` → `small`).
    scope: RewriteScope,
    /// Whether the request renamed a name (a word-scoped rename).
    renaming: bool,
    /// Whether the pattern must occur exactly once (a positional insertion's
    /// anchor), replaced once rather than everywhere.
    unique: bool,
    /// Whether the replacement holds the pattern, so it is applied in one
    /// pass rather than rewritten until no pattern is left (PR #1188 G78).
    single_pass: bool,
    /// The seed sentence that states the change, when it is not the
    /// rename/replace one (a positional insertion's).
    stated: Option<(&'static str, Vec<(&'static str, String)>)>,
    /// The pattern and replacement as the request wrote them, when it wrote
    /// `\n` / `\t` escapes the rewrite unescaped (see [`as_written`]).
    verbatim: Option<(String, String)>,
}

/// The template slots the seed sentences that state a rewrite fill.
const OLD_SLOT: &str = concat!("{", "old", "}");
const NEW_SLOT: &str = concat!("{", "new", "}");
const ANCHOR_SLOT: &str = concat!("{", "anchor", "}");

impl GroundedRewrite {
    /// The intent whose seed sentence states this change rather than merely
    /// reporting that the file was touched.
    const fn stated_intent(&self) -> &'static str {
        match self.stated {
            Some((intent, _)) => intent,
            None if self.renaming => "coding_identifier_renamed",
            None => "coding_text_replaced",
        }
    }

    /// The operands the seed sentence names, as template slots.
    fn stated_slots(&self) -> Vec<(&str, &str)> {
        match &self.stated {
            Some((_, slots)) => slots
                .iter()
                .map(|(slot, value)| (*slot, value.as_str()))
                .collect(),
            None => vec![
                (OLD_SLOT, self.pattern.as_str()),
                (NEW_SLOT, self.replacement.as_str()),
            ],
        }
    }
}

/// A change whose bytes are settled, awaiting the observation that confirms it.
///
/// `expected` is what the workspace should now hold; `intent` and `slots` are
/// the seed sentence that states the change once it does.
pub(super) struct VerifiedChange<'a> {
    pub(super) target: &'a str,
    pub(super) expected: &'a str,
    pub(super) intent: &'a str,
    pub(super) slots: &'a [(&'a str, &'a str)],
    pub(super) list_slots: &'a [(&'a str, &'a [String])],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompositeModuleChange {
    source_path: String,
    source: String,
    registration_path: String,
    registration: String,
}

pub(super) fn plan_workspace_change_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(task);
    // A continuation cue resumes this run rather than opening a new request,
    // so the read and write evidence gathered for it must survive the ping
    // (issue #1138): slicing at the last user message -- the cue itself --
    // restarted the read--write pair on every ping.
    if super::general_planner::has_additive_position(task)
        && (super::workspace_computed_change::grounded_end_insertion(task).is_some()
            || super::contents_source::contents_source(task).is_some()
            || super::general_planner::declared_addition_contract(task).is_some())
        && !super::general_planner::owns_additive_scope(task)
    {
        return Some(record(
            AgenticPlan::Final(
                super::code_task::render_seeded_outcome("file-addition-unverified", task, "")
                    .unwrap_or_else(|| task.to_owned()),
            ),
            FinalDisposition::Gap,
            "literal-addition-unowned-request",
            result,
        ));
    }
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    if let Some(addition) =
        super::literal_addition::plan_literal_addition_step(task, current_turn, tool_names, result)
    {
        return Some(addition);
    }
    // `Append the contents of a.txt to b.txt`: the source is read, and the
    // request is restated with its lines as the block placed (PR #1188 G50).
    let sourced = match sourced_request(task, current_turn) {
        Some(Sourced::Read(path)) => {
            let tool = tool_for(tool_names, Capability::Read)?;
            return Some(plan_one(tool, read_arguments(&path)));
        }
        Some(Sourced::Task(sourced)) => Some(sourced),
        None => None,
    };
    let task = sourced.as_deref().unwrap_or(task);

    if let Some(change) = composite_module_change(task) {
        return plan_composite_step(task, current_turn, tool_names, &change, result);
    }
    // Numbered lines moved to another file: placed there, then removed (G85).
    if let Some(order) = super::line_range_move::line_range_move(task) {
        return super::line_range_move::plan_line_range_move_step(
            task,
            current_turn,
            tool_names,
            &order,
            result,
        );
    }
    if let Some(change) = super::workspace_computed_change::grounded_line_change(task) {
        return super::workspace_computed_change::plan_computed_change_step(
            task,
            current_turn,
            tool_names,
            &change,
            result,
        );
    }
    if let Some(inserts) = insert_sequence(task) {
        return super::positional_edit::plan_insert_sequence_step(
            task,
            current_turn,
            tool_names,
            &inserts,
            result,
        );
    }
    // Several replacements asked in one sentence are made in order (G82).
    if let Some(change) = super::workspace_computed_change::grounded_replace_list(task) {
        return super::workspace_computed_change::plan_computed_change_step(
            task,
            current_turn,
            tool_names,
            &change,
            result,
        );
    }
    if let Some(rewrite) = grounded_rewrite(task) {
        return plan_rewrite_step(task, current_turn, tool_names, &rewrite, result);
    }
    let change = super::workspace_computed_change::grounded_computed_change(task)?;
    super::workspace_computed_change::plan_computed_change_step(
        task,
        current_turn,
        tool_names,
        &change,
        result,
    )
}

/// A request whose additive edit places another file's contents: the source
/// to read first, or the request restated with its lines.
enum Sourced {
    Read(String),
    Task(String),
}

/// `None` unless the request names a contents source and, restated, is an
/// additive edit the composers place.
fn sourced_request(task: &str, current_turn: &[ChatMessage]) -> Option<Sourced> {
    use super::contents_source::{contents_source, with_contents};
    let source = contents_source(task)?;
    let probe = with_contents(task, &source, "x");
    if insert_sequence(&probe).is_none()
        && super::workspace_computed_change::grounded_end_insertion(&probe).is_none()
    {
        return None;
    }
    let Some(read) = result_for_path(current_turn, Capability::Read, &source.path, None) else {
        return Some(Sourced::Read(source.path));
    };
    if super::code_artifact::source_from_agent_read_result(&read).is_none()
        && super::tool_result::failure_message(&read, false, true).is_some()
    {
        return None;
    }
    let contents = source_from_read_result(&read);
    Some(Sourced::Task(with_contents(task, &source, &contents)))
}

fn plan_rewrite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    rewrite: &GroundedRewrite,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let Some(read) = result_for_path(current_turn, Capability::Read, &rewrite.target, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&rewrite.target)));
    };
    let source = source_from_read_result(&read);
    let written = as_written(&source, rewrite);
    let rewrite = written.as_ref().unwrap_or(rewrite);
    let Some(updated) = rewritten_source(&source, rewrite) else {
        // A replace asked again finds its old text gone: that is the answer (G87).
        let absent = super::replace_list::absent_text_answer(
            task,
            &rewrite.target,
            &source,
            &rewrite.pattern,
            Some(&rewrite.replacement),
        );
        return Some(AgenticPlan::Final(match absent {
            Some(answer) => answer,
            None => render_seeded_outcome(
                "coding_workspace_verification_failed",
                task,
                &rewrite.target,
            )?,
        }));
    };

    let occurrences = match rewrite.scope {
        RewriteScope::Substring => source.match_indices(&rewrite.pattern).count(),
        RewriteScope::Word => word_scoped_matches(&source, &rewrite.pattern).len(),
    };
    if occurrences > 1
        && let Some(answer) = super::edit_scope::scoped_decline(task, &rewrite.target, &source)
    {
        return Some(AgenticPlan::Final(answer));
    }
    if occurrences == 1 {
        // The bare pattern when it is unique in the file; otherwise the
        // smallest unique run of changed lines -- the edit tool refuses an
        // `oldString` it finds twice, and `smal` also sits inside `small`.
        let edit = if rewrite.unique {
            anchored_lines(&source, rewrite).map(|(_, old, new)| (old, new))
        } else if source.matches(rewrite.pattern.as_str()).count() == 1 {
            Some((rewrite.pattern.clone(), rewrite.replacement.clone()))
        } else {
            changed_lines_edit(&source, &updated)
        };
        if let Some(tool) = tool_for(tool_names, Capability::Edit)
            && let Some((old, new)) = edit
        {
            if result_for_edit(current_turn, &rewrite.target, &old, &new).is_none() {
                return Some(plan_one(tool, edit_arguments(&rewrite.target, &old, &new)));
            }
            return plan_digest_verification(
                task,
                current_turn,
                tool_names,
                &VerifiedChange {
                    target: &rewrite.target,
                    expected: &updated,
                    intent: rewrite.stated_intent(),
                    slots: &rewrite.stated_slots(),
                    list_slots: &[],
                },
                result,
            );
        }
    } else if tool_for(tool_names, Capability::Edit).is_some()
        && let Some(command) = repeated_identifier_rewrite_command(rewrite)
        && let Some(tool) = tool_for(tool_names, Capability::Run)
    {
        if result_for_command(current_turn, &command).is_none() {
            return Some(plan_one(tool, json!({"command": command}).to_string()));
        }
        return plan_digest_verification(
            task,
            current_turn,
            tool_names,
            &VerifiedChange {
                target: &rewrite.target,
                expected: &updated,
                intent: rewrite.stated_intent(),
                slots: &rewrite.stated_slots(),
                list_slots: &[],
            },
            result,
        );
    }

    if result_for_path(
        current_turn,
        Capability::Write,
        &rewrite.target,
        Some(&updated),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(tool, write_arguments(&rewrite.target, &updated)));
    }
    plan_write_digest_verification(
        task,
        current_turn,
        tool_names,
        &VerifiedChange {
            target: &rewrite.target,
            expected: &updated,
            intent: rewrite.stated_intent(),
            slots: &rewrite.stated_slots(),
            list_slots: &[],
        },
        result,
    )
}

fn plan_composite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &CompositeModuleChange,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if result_for_path(
        current_turn,
        Capability::Write,
        &change.source_path,
        Some(&change.source),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(
            tool,
            write_arguments(&change.source_path, &change.source),
        ));
    }

    let source_command = format!("cat {}", change.source_path);
    let Some(observed_source) = result_for_command(current_turn, &source_command) else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(
            tool,
            json!({"command": source_command}).to_string(),
        ));
    };
    if !super::tool_result::observed_bytes_match(&observed_source, &change.source) {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &change.source_path,
        )?));
    }

    let Some(read) = result_for_path(
        current_turn,
        Capability::Read,
        &change.registration_path,
        None,
    ) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&change.registration_path)));
    };
    let current = source_from_read_result(&read);
    let updated = insert_registration(&current, &change.registration);
    let registered = ["`", change.registration.trim(), "`"].concat();
    if updated == current {
        return Some(record(
            AgenticPlan::Final(render_seeded_change(
                "coding_member_already_present",
                task,
                &change.registration_path,
                &[("{members}", &registered)],
            )?),
            FinalDisposition::Finding,
            "workspace_change_observed",
            result,
        ));
    }

    if let Some((old, new)) = compact_registration_edit(&current, &change.registration)
        && let Some(tool) = tool_for(tool_names, Capability::Edit)
    {
        if result_for_edit(current_turn, &change.registration_path, &old, &new).is_none() {
            return Some(plan_one(
                tool,
                edit_arguments(&change.registration_path, &old, &new),
            ));
        }
        return plan_digest_verification(
            task,
            current_turn,
            tool_names,
            &VerifiedChange {
                target: &change.registration_path,
                expected: &updated,
                intent: "coding_member_inserted",
                slots: &[("{members}", &registered)],
                list_slots: &[],
            },
            result,
        );
    }

    if result_for_path(
        current_turn,
        Capability::Write,
        &change.registration_path,
        Some(&updated),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(
            tool,
            write_arguments(&change.registration_path, &updated),
        ));
    }

    let registration_command = format!("cat {}", change.registration_path);
    let Some(observed_registration) = result_for_command(current_turn, &registration_command)
    else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(
            tool,
            json!({"command": registration_command}).to_string(),
        ));
    };
    if !super::tool_result::observed_bytes_match(&observed_registration, &updated) {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &change.registration_path,
        )?));
    }
    Some(record(
        AgenticPlan::Final(render_seeded_change(
            "coding_member_inserted",
            task,
            &change.registration_path,
            &[("{members}", &registered)],
        )?),
        FinalDisposition::Finding,
        "workspace_change_observed",
        result,
    ))
}

/// The file after `rewrite`, or `None` when it cannot apply. A positional
/// insertion replaces its one anchor occurrence; an anchor found zero or
/// several times does not say where the line goes.
/// The rewrite as the request wrote it, when the file holds no unescaped
/// pattern but holds the pattern as written (source code quoting `'\n'`,
/// PR #1188 G65).
fn as_written(source: &str, rewrite: &GroundedRewrite) -> Option<GroundedRewrite> {
    let (pattern, replacement) = rewrite.verbatim.clone()?;
    if (rewrite.pattern != pattern && source.contains(&rewrite.pattern))
        || !source.contains(&pattern)
    {
        return None;
    }
    Some(GroundedRewrite {
        pattern,
        replacement,
        verbatim: None,
        ..rewrite.clone()
    })
}

fn rewritten_source(source: &str, rewrite: &GroundedRewrite) -> Option<String> {
    if rewrite.single_pass {
        return source
            .contains(rewrite.pattern.as_str())
            .then(|| source.replace(rewrite.pattern.as_str(), &rewrite.replacement));
    }
    if rewrite.unique {
        let (start, old, new) = anchored_lines(source, rewrite)?;
        return Some([&source[..start], new.as_str(), &source[start + old.len()..]].concat());
    }
    execute_scoped_workspace_rewrite(
        source,
        &rewrite.pattern,
        &rewrite.replacement,
        rewrite.scope,
    )
    .ok()
    .map(|execution| execution.output)
}

/// A positional insertion's one anchor occurrence widened to the whole lines
/// it sits on, as `(start, old_lines, new_lines)`, or `None` unless the anchor
/// occurs exactly once. The inserted text is a line, so an anchor that is only
/// part of a line (`after the line containing '| T20 |'`) must not split it.
fn anchored_lines(source: &str, rewrite: &GroundedRewrite) -> Option<(usize, String, String)> {
    let (pattern, replacement) = (rewrite.pattern.as_str(), rewrite.replacement.as_str());
    let mut found = source.match_indices(pattern).map(|(at, _)| at);
    let (Some(at), None) = (found.next(), found.next()) else {
        return None;
    };
    let start = source[..at].rfind('\n').map_or(0, |newline| newline + 1);
    let tail = at + pattern.len();
    let end = if pattern.ends_with('\n') {
        tail - 1
    } else {
        source[tail..]
            .find('\n')
            .map_or(source.len(), |newline| tail + newline)
    };
    let old = &source[start..end];
    let new = replacement.strip_prefix(pattern).map_or_else(
        || [&replacement[..replacement.len() - pattern.len()], old].concat(),
        |rest| [old, rest].concat(),
    );
    Some((start, old.to_owned(), new))
}

/// The smallest run of whole lines holding every change from `source` to
/// `updated`, as an edit pair, when that run occurs once in `source`.
pub(super) fn changed_lines_edit(source: &str, updated: &str) -> Option<(String, String)> {
    const MAX_EDIT_BYTES: usize = 4096;
    let prefix = source
        .bytes()
        .zip(updated.bytes())
        .take_while(|(left, right)| left == right)
        .count();
    let limit = source.len().min(updated.len()) - prefix;
    let suffix = source
        .bytes()
        .rev()
        .zip(updated.bytes().rev())
        .take(limit)
        .take_while(|(left, right)| left == right)
        .count();
    // Byte positions only meet `\n`, so `start` and `end` fall on line -- and
    // therefore character -- boundaries even when the first or last differing
    // byte sits inside a multi-byte character.
    let bytes = source.as_bytes();
    let line_start = |before: usize| {
        bytes[..before]
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |at| at + 1)
    };
    let mut start = line_start(prefix);
    let end = bytes[source.len() - suffix..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(source.len(), |at| source.len() - suffix + at);
    // Lines inserted between two lines change none: the line above carries
    // them.
    if start == end && start > 0 {
        start = line_start(start - 1);
    }
    let old = &source[start..end];
    let new = &updated[start..updated.len() - (source.len() - end)];
    (!old.is_empty() && old.len() <= MAX_EDIT_BYTES && source.matches(old).count() == 1)
        .then(|| (old.to_owned(), new.to_owned()))
}

/// `Insert 'x' after the line 'y' in f`: the anchor line, replaced once by the
/// anchor beside the new line, stated by the position meaning that placed it.
fn grounded_positional_insert(task: &str) -> Option<GroundedRewrite> {
    let (target, anchor, replacement) = super::positional_edit::compose_positional_insert(task)?;
    let after_anchor = [anchor.as_str(), "\n"].concat();
    let (after, inserted) = if let Some(inserted) = replacement.strip_prefix(&after_anchor) {
        (true, inserted.to_owned())
    } else {
        let before_anchor = ["\n", anchor.as_str()].concat();
        (false, replacement.strip_suffix(&before_anchor)?.to_owned())
    };
    let intent = super::positional_edit::insert_intent(after, &inserted);
    Some(GroundedRewrite {
        target,
        stated: Some((
            intent,
            vec![(NEW_SLOT, inserted), (ANCHOR_SLOT, anchor.clone())],
        )),
        pattern: anchor,
        replacement,
        scope: RewriteScope::Substring,
        renaming: false,
        unique: true,
        single_pass: false,
        verbatim: None,
    })
}

/// Every insert the request asks for, each its own anchored edit.
///
/// Several clauses (`…, and insert …`), an anchor named by the line it
/// follows (T31, T34), lines rebased on the anchor (G16), or one insert whose
/// anchor is a whole line also found inside others (`the line 'b'`).
fn insert_sequence(task: &str) -> Option<Vec<PositionalInsert>> {
    super::positional_edit::positional_inserts(task)
}

fn grounded_rewrite(task: &str) -> Option<GroundedRewrite> {
    if let Some(positional) = grounded_positional_insert(task) {
        return Some(positional);
    }
    let (target, old_clause, new_clause) = compose_edit_request(task)?;
    let renaming = seed::lexicon().mentions_role(
        seed::ROLE_CODING_IDENTIFIER_RENAME_ACTION,
        &task.to_lowercase(),
    );
    // A request that names a line replaces whole lines
    // (`grounded_line_replacement`).
    if !renaming && super::workspace_computed_change::names_line(task) {
        return None;
    }
    // Both operands have to be literals the request actually quoted, so prose
    // ("replace the old header with something clearer") is never rewritten as
    // bytes. What that cannot be is a count: requiring the request to hold
    // *exactly* two quoted segments made every other pair of delimiters in the
    // prompt disqualify the edit, and this repository's own prose backticks the
    // file it names. The issue #1028 ladder node "In the file
    // `src/protocol_memory.rs`, replace \"request_history\" with
    // \"conversation_history\"" quotes three things and states one replacement.
    // Asking whether each clause *is* quoted keeps the guarantee and drops the
    // arithmetic.
    // The edit request unescapes `\n` / `\t` in its literals, so the quoted
    // segments are compared unescaped too: a multi-line replacement written
    // with `\n` was otherwise not "quoted", and the general fallback wrote its
    // new text over the whole file (PR #1188 sub-agent gap 3).
    let quoted: Vec<String> = quoted_segments(task)
        .iter()
        .map(String::as_str)
        .map(super::positional_edit::unescape_prose_newlines)
        .collect();
    // A literal quoted whole counts even when backtick spans inside it pair
    // among themselves (PR #1188 G63).
    let is_quoted =
        |value: &String| quoted.contains(value) || crate::normal_markov::quotes_whole(task, value);
    let operands = if is_quoted(&old_clause) && is_quoted(&new_clause) {
        Some((old_clause, new_clause))
    } else if renaming {
        Some((
            identifier_tokens(&old_clause).next_back()?.to_owned(),
            identifier_tokens(&new_clause).next()?.to_owned(),
        ))
    } else {
        None
    }?;
    let (old, new) = operands;
    let as_quoted = |text: &str| {
        quoted_segments(task)
            .into_iter()
            .find(|raw| raw != text && super::positional_edit::unescape_prose_newlines(raw) == text)
            .unwrap_or_else(|| text.to_owned())
    };
    let verbatim = (as_quoted(&old) != old || as_quoted(&new) != new)
        .then(|| (as_quoted(&old), as_quoted(&new)));
    // A rename names a *word*, so the edit is word-scoped whichever way the
    // request spelled its operands. Without that scope the most ordinary rename
    // there is -- giving a name a prefix or a suffix -- has to be refused, since
    // an unanchored substring rewrite that reintroduces its own pattern never
    // terminates.
    //
    // A word replaced by a longer word that contains it (`smal` → `small`, a
    // typo fix) is only safe word by word for the same reason.
    let words = is_identifier_word(&old) && is_identifier_word(&new);
    let scope = if words && (renaming || new.contains(&old)) {
        RewriteScope::Word
    } else {
        RewriteScope::Substring
    };
    if old.is_empty() || old == new {
        return None;
    }
    // A new text holding the old one (`key 3` → `key 33`) is replaced in one
    // pass over the file, never rewritten again inside its own result (PR
    // #1188 G78).
    let single_pass = scope == RewriteScope::Substring && new.contains(&old);
    Some(GroundedRewrite {
        target,
        pattern: old,
        replacement: new,
        scope,
        renaming: renaming && scope == RewriteScope::Word,
        unique: false,
        single_pass,
        stated: None,
        verbatim,
    })
}

// This is a seed-template placeholder, not a Rust formatting argument.
#[allow(clippy::literal_string_with_formatting_args)]
fn composite_module_change(task: &str) -> Option<CompositeModuleChange> {
    let contract = verified_source_description(task)?;
    if contract["wholeRequestConsumed"].as_bool() != Some(true) {
        return None;
    }
    let generated = &contract["declaration"]["output"];
    let module = contract["registration"]["module"].as_str()?;
    let registration =
        render_rust_template("coding_source_module_registration", &[("{module}", module)])?;
    Some(CompositeModuleChange {
        source_path: generated["path"].as_str()?.to_owned(),
        source: generated["content"].as_str()?.to_owned(),
        registration_path: contract["registration"]["target"]["text"]
            .as_str()?
            .to_owned(),
        registration,
    })
}

fn insert_registration(source: &str, registration: &str) -> String {
    if source
        .lines()
        .any(|line| line.trim() == registration.trim())
    {
        return source.to_owned();
    }
    let mut updated = source.to_owned();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(registration);
    updated
}

fn compact_registration_edit(source: &str, registration: &str) -> Option<(String, String)> {
    const MAX_SUFFIX_BYTES: usize = 4096;
    let mut starts = source
        .match_indices('\n')
        .map(|(index, _)| index + 1)
        .filter(|start| *start < source.len())
        .collect::<Vec<_>>();
    starts.insert(0, 0);
    for start in starts.into_iter().rev() {
        let suffix = &source[start..];
        if suffix.len() > MAX_SUFFIX_BYTES {
            break;
        }
        if source.match_indices(suffix).count() != 1 {
            continue;
        }
        let mut replacement = suffix.to_owned();
        if !replacement.ends_with('\n') {
            replacement.push('\n');
        }
        replacement.push_str(registration);
        return Some((suffix.to_owned(), replacement));
    }
    None
}

fn repeated_identifier_rewrite_command(rewrite: &GroundedRewrite) -> Option<String> {
    if !shell_safe_identifier(&rewrite.pattern) || !shell_safe_identifier(&rewrite.replacement) {
        return None;
    }
    // The in-memory execution and the command the client runs have to agree on
    // what a match is, so a word-scoped rewrite asks for whole words too.
    let pattern = match rewrite.scope {
        RewriteScope::Substring => rewrite.pattern.clone(),
        RewriteScope::Word => format!("\\b{}\\b", rewrite.pattern),
    };
    // `perl -pi -e`, not `sed -i` (issue #1110). GNU and BSD sed disagree about
    // both halves of that command: BSD `-i` reads the script as its backup
    // suffix, and BSD sed has no `\b`. On macOS every rename therefore failed
    // with `bad flag in substitute command`, the file was left unchanged, and
    // Formal AI reported the mismatch as its own verification failure -- a
    // confident failure report instead of a rename. perl's `-pi -e` and `\b`
    // mean the same thing on both, and perl ships with macOS and the runners.
    // Both identifiers are `[A-Za-z0-9_]` by `shell_safe_identifier`, so
    // neither side can carry a regex or shell metacharacter.
    // Assembled from named parts rather than one template string: the prose
    // detector reads `perl -pi -e ...` as a multi-word phrase (two alphabetic
    // runs, one of three letters) and would have it live in `data/seed`, which
    // is the wrong home for a shell invocation. Naming the pieces says what
    // each is, and none of them reads as a sentence.
    let script = format!("'s/{pattern}/{}/g'", rewrite.replacement);
    Some(["perl", "-pi", "-e", &script, "--", &rewrite.target].join(" "))
}

fn shell_safe_identifier(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

pub(super) fn identifier_tokens(text: &str) -> impl DoubleEndedIterator<Item = &str> {
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| valid_identifier(token))
}

fn valid_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        && !seed::lexicon()
            .words_for_role(seed::ROLE_IDENTIFIER_RESERVED_WORD)
            .iter()
            .any(|reserved| reserved == identifier)
}

pub(super) fn result_for_path(
    messages: &[ChatMessage],
    capability: Capability,
    path: &str,
    expected_content: Option<&str>,
) -> Option<String> {
    matching_result(messages, |name, arguments| {
        if tool_capability(name) != Some(capability) {
            return false;
        }
        let Ok(value) = serde_json::from_str::<Value>(arguments) else {
            return false;
        };
        let matches_path = ["path", "filePath", "file_path"].iter().any(|key| {
            value
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|observed| workspace_path_matches(path, observed))
        });
        matches_path
            && expected_content.is_none_or(|expected| {
                value.get("content").and_then(Value::as_str) == Some(expected)
            })
    })
}

pub(super) fn result_for_edit(
    messages: &[ChatMessage],
    path: &str,
    old: &str,
    new: &str,
) -> Option<String> {
    matching_result(messages, |name, arguments| {
        if tool_capability(name) != Some(Capability::Edit) {
            return false;
        }
        let Ok(value) = serde_json::from_str::<Value>(arguments) else {
            return false;
        };
        argument_matches_path(&value, path)
            && argument_matches(&value, &["oldString", "old_string", "old_str", "old"], old)
            && argument_matches(&value, &["newString", "new_string", "new_str", "new"], new)
    })
}

fn argument_matches_path(value: &Value, expected: &str) -> bool {
    ["path", "filePath", "file_path"].iter().any(|key| {
        value
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(|observed| workspace_path_matches(expected, observed))
    })
}

fn argument_matches(value: &Value, keys: &[&str], expected: &str) -> bool {
    keys.iter()
        .any(|key| value.get(key).and_then(Value::as_str) == Some(expected))
}

fn workspace_path_matches(expected: &str, observed: &str) -> bool {
    if observed == expected {
        return true;
    }
    let expected = Path::new(expected);
    let observed = Path::new(observed);
    expected.is_relative() && observed.is_absolute() && observed.ends_with(expected)
}

pub(super) fn result_for_command(messages: &[ChatMessage], command: &str) -> Option<String> {
    super::code_artifact::result_for_command(messages, command)
}

fn matching_result(
    messages: &[ChatMessage],
    matches: impl Fn(&str, &str) -> bool,
) -> Option<String> {
    messages
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, message)| {
            if !message.role.eq_ignore_ascii_case("tool") {
                return None;
            }
            let id = message.tool_call_id.as_deref()?;
            let call = messages[..index]
                .iter()
                .rev()
                .flat_map(|prior| prior.tool_calls.iter().rev())
                .find(|call| call.id == id)?;
            matches(&call.function.name, &call.function.arguments)
                .then(|| message.content.plain_text())
        })
}

pub(super) fn read_arguments(path: &str) -> String {
    json!({"path": path, "filePath": path, "file_path": path}).to_string()
}
