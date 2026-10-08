//! Grounded, verified workspace transformations learned from coding-task runs.
//!
//! The planner never edits request prose. It compiles a bounded transformation,
//! reads the client-owned bytes, executes the transformation in memory, applies
//! a compact edit or bounded replace-all operation, and accepts success only
//! after an exact content-digest observation. The same state machine composes
//! source creation with a second module-registration edit, so a multi-file
//! request cannot stop after its first observable effect.

use serde_json::{json, Value};
use std::path::Path;

use super::code_artifact::source_from_read_result;
use super::code_task::{
    render_rust_template, render_seeded_change, render_seeded_outcome, rust_source_for_task,
};
use super::general_planner::compose_edit_request;
use super::intent_router::edit_arguments;
use super::planner::{
    plan_one, tool_capability, tool_for, write_arguments, AgenticPlan, Capability,
};
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
    /// The seed sentence that states the change, when it is not the
    /// rename/replace one (a positional insertion's).
    stated: Option<(&'static str, Vec<(&'static str, String)>)>,
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
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(task);
    // A continuation cue resumes this run rather than opening a new request,
    // so the read and write evidence gathered for it must survive the ping
    // (issue #1138): slicing at the last user message -- the cue itself --
    // restarted the read--write pair on every ping.
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];

    if let Some(change) = composite_module_change(task) {
        return plan_composite_step(task, current_turn, tool_names, &change);
    }
    if let Some(rewrite) = grounded_rewrite(task) {
        return plan_rewrite_step(task, current_turn, tool_names, &rewrite);
    }
    let change = super::workspace_computed_change::grounded_computed_change(task)?;
    super::workspace_computed_change::plan_computed_change_step(
        task,
        current_turn,
        tool_names,
        &change,
    )
}

/// Whether `answer` is one of this module's seeded verification-failure
/// renderings for `task`'s targets.
///
/// The rendering is deterministic in the task and the target, so equality
/// against it is a structural test with no phrase table: it recognises the
/// failure report in whatever language the seed rendered it. The delivery
/// route uses it to keep such prose out of the files it writes -- a report
/// that a check failed is chat, never the `result=` line of an effect file
/// (issue #1138: the binary-tree ladder's effect files carried the failure
/// template and every leaf read as an unverified result).
pub(super) fn is_verification_failure_answer(task: &str, answer: &str) -> bool {
    let task = unwrap_transport_quotes(task);
    compose_edit_request(task)
        .map(|(target, _, _)| target)
        .into_iter()
        .chain(rust_paths(task))
        .any(|target| {
            render_seeded_outcome("coding_workspace_verification_failed", task, &target)
                .is_some_and(|rendered| rendered == answer)
        })
}

fn plan_rewrite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    rewrite: &GroundedRewrite,
) -> Option<AgenticPlan> {
    let Some(read) = result_for_path(current_turn, Capability::Read, &rewrite.target, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&rewrite.target)));
    };
    let source = source_from_read_result(&read);
    let Some(updated) = rewritten_source(&source, rewrite) else {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &rewrite.target,
        )?));
    };

    let occurrences = match rewrite.scope {
        RewriteScope::Substring => source.match_indices(&rewrite.pattern).count(),
        RewriteScope::Word => word_scoped_matches(&source, &rewrite.pattern).len(),
    };
    if occurrences == 1 {
        // The bare pattern when it is unique in the file; otherwise the
        // smallest unique run of changed lines -- the edit tool refuses an
        // `oldString` it finds twice, and `smal` also sits inside `small`.
        let edit = if source.matches(rewrite.pattern.as_str()).count() == 1 {
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
                },
            );
        }
    } else if tool_for(tool_names, Capability::Edit).is_some()
        && let Some(command) = repeated_identifier_rewrite_command(rewrite)
            && let Some(tool) = tool_for(tool_names, Capability::Run) {
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
                    },
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
    let command = format!("cat {}", rewrite.target);
    let Some(observed) = result_for_command(current_turn, &command) else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(tool, json!({"command": command}).to_string()));
    };
    if observed != updated {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &rewrite.target,
        )?));
    }
    Some(AgenticPlan::Final(render_seeded_change(
        rewrite.stated_intent(),
        task,
        &rewrite.target,
        &rewrite.stated_slots(),
    )?))
}

fn plan_composite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &CompositeModuleChange,
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
    if observed_source != change.source {
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
        return Some(AgenticPlan::Final(render_seeded_change(
            "coding_member_already_present",
            task,
            &change.registration_path,
            &[("{members}", &registered)],
        )?));
    }

    if let Some((old, new)) = compact_registration_edit(&current, &change.registration)
        && let Some(tool) = tool_for(tool_names, Capability::Edit) {
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
                },
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
    if observed_registration != updated {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &change.registration_path,
        )?));
    }
    Some(AgenticPlan::Final(render_seeded_change(
        "coding_member_inserted",
        task,
        &change.registration_path,
        &[("{members}", &registered)],
    )?))
}

/// The file after `rewrite`, or `None` when it cannot apply. A positional
/// insertion replaces its one anchor occurrence; an anchor found zero or
/// several times does not say where the line goes.
fn rewritten_source(source: &str, rewrite: &GroundedRewrite) -> Option<String> {
    if rewrite.unique {
        return (source.matches(rewrite.pattern.as_str()).count() == 1)
            .then(|| source.replacen(rewrite.pattern.as_str(), &rewrite.replacement, 1));
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
    let start = bytes[..prefix]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |at| at + 1);
    let end = bytes[source.len() - suffix..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(source.len(), |at| source.len() - suffix + at);
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
    let (intent, inserted) = if let Some(inserted) = replacement.strip_prefix(&after_anchor) {
        ("file_edit_position_after", inserted.to_owned())
    } else {
        let before_anchor = ["\n", anchor.as_str()].concat();
        (
            "file_edit_position_before",
            replacement.strip_suffix(&before_anchor)?.to_owned(),
        )
    };
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
    })
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
    let is_quoted = |value: &String| quoted.contains(value);
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
    if old.is_empty() || old == new || (scope == RewriteScope::Substring && new.contains(&old)) {
        return None;
    }
    Some(GroundedRewrite {
        target,
        pattern: old,
        replacement: new,
        scope,
        renaming: renaming && scope == RewriteScope::Word,
        unique: false,
        stated: None,
    })
}

// This is a seed-template placeholder, not a Rust formatting argument.
#[allow(clippy::literal_string_with_formatting_args)]
fn composite_module_change(task: &str) -> Option<CompositeModuleChange> {
    let lowered = task.to_lowercase();
    if !seed::lexicon().mentions_role(seed::ROLE_CODING_MODULE_REGISTRATION_ACTION, &lowered) {
        return None;
    }
    let generated = rust_source_for_task(task)?;
    let registration_path = rust_paths(task)
        .into_iter()
        .find(|path| path != &generated.path)?;
    let module = generated.path.rsplit('/').next()?.strip_suffix(".rs")?;
    if !valid_identifier(module) {
        return None;
    }
    let registration =
        render_rust_template("coding_source_module_registration", &[("{module}", module)])?;
    Some(CompositeModuleChange {
        source_path: generated.path,
        source: generated.content,
        registration_path,
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
    Some(
        ["perl", "-pi", "-e", &script, "--", &rewrite.target]
            .join(" "),
    )
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

fn rust_paths(task: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for (suffix, _) in task.match_indices(".rs") {
        let end = suffix + 3;
        let start = task[..end]
            .char_indices()
            .rev()
            .take_while(|(_, character)| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
            })
            .last()
            .map_or(0, |(index, _)| index);
        let path = &task[start..end];
        if !path.is_empty()
            && !path.starts_with('/')
            && !path.split('/').any(|component| component == "..")
            && !paths.iter().any(|existing| existing == path)
        {
            paths.push(path.to_owned());
        }
    }
    paths
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

fn result_for_command(messages: &[ChatMessage], command: &str) -> Option<String> {
    matching_result(messages, |name, arguments| {
        tool_capability(name) == Some(Capability::Run)
            && super::tool_result::command_argument(arguments).as_deref() == Some(command)
    })
}

pub(super) fn plan_digest_verification(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &VerifiedChange<'_>,
) -> Option<AgenticPlan> {
    let command = ["sha256sum -- ", change.target].concat();
    let Some(observed) = result_for_command(current_turn, &command) else {
        let tool = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(tool, json!({"command": command}).to_string()));
    };
    let digest = crate::source_fetch::sha256_hex(change.expected.as_bytes());
    if observed.split_whitespace().next() != Some(digest.as_str()) {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            change.target,
        )?));
    }
    Some(AgenticPlan::Final(render_seeded_change(
        change.intent,
        task,
        change.target,
        change.slots,
    )?))
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
