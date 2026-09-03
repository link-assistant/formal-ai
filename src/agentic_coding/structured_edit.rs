//! Grounded structural edits over client-supplied workspace bytes.
//!
//! Unlike literal replacement, member insertion has no complete old value in the
//! request. The planner must read the target, derive the replacement from those
//! bytes, write the full updated source, and observe the result.
//!
//! Everything about *where* the insertion goes is discovered from the target's
//! own bytes rather than from the request's wording: which member list is meant,
//! which delimiters enclose it, which separator joins its members, and which of
//! the request's values are already there. The request only has to name a file,
//! say that a member list is involved, and quote the members. That is why one
//! recipe covers a bracketed array, a `matches!` alternation and a parenthesised
//! group without a branch per shape (#1069).

use std::collections::HashMap;

use serde_json::json;

use super::code_artifact::{latest_result, source_from_read_result};
use super::code_task::render_seeded_outcome;
use super::planner::{plan_one, tool_for, write_arguments, AgenticPlan, Capability};
use crate::normal_markov::{quoted_segment_spans, unwrap_transport_quotes};
use crate::protocol::ChatMessage;
use crate::seed;

/// Longest member literal the route will write.
///
/// Member literals are short by nature. The bound is what keeps a quoted
/// *sentence* — prose the caller happened to quote — from being written into
/// source as if it were a member.
const MAX_MEMBER_LENGTH: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemberInsertion {
    target: String,
    /// Quoted member literals, in request order. Some are usually already in
    /// the file: those are the evidence that locates the list, and the rest are
    /// what gets inserted. Which is which is decided by reading the file.
    values: Vec<String>,
    /// Identifiers the request offered as names for the list, most explicit
    /// first. Only consulted when no quoted value occurs in the file.
    named: Vec<String>,
}

pub(super) fn plan_structured_edit_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(task);
    let edit = member_insertion(task)?;
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let current_turn = &messages[latest_user + 1..];
    let source = latest_result(current_turn, Capability::Read)
        .as_deref()
        .map(source_from_read_result)
        .filter(|source| !source.is_empty());

    let Some(source) = source else {
        let read_tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(read_tool, read_arguments(&edit.target)));
    };
    let updated = insert_members(&source, &edit)?;

    if latest_result(current_turn, Capability::Write).is_none() {
        let write_tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(
            write_tool,
            write_arguments(&edit.target, &updated),
        ));
    }
    if let Some(observed) = latest_result(current_turn, Capability::Run) {
        if observed == updated {
            return Some(AgenticPlan::Final(render_seeded_outcome(
                "coding_workspace_effect_observed",
                task,
                &edit.target,
            )?));
        }
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_verification_failed",
            task,
            &edit.target,
        )?));
    }
    let run_tool = tool_for(tool_names, Capability::Run)?;
    Some(plan_one(
        run_tool,
        json!({"command": format!("cat {}", edit.target)}).to_string(),
    ))
}

fn member_insertion(task: &str) -> Option<MemberInsertion> {
    // Seed matching is token-bounded. Collapse punctuation to token separators
    // so a concept at the end of a sentence (for example `array.`) remains
    // evidence for the same meaning without adding language-specific syntax.
    let normalized = task
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let lexicon = seed::lexicon();
    let changes_a_file = lexicon.mentions_role(seed::ROLE_FILE_WRITE_ACTION_CUE, &normalized)
        || lexicon.mentions_role(seed::ROLE_FILE_EDIT_ACTION_CUE, &normalized);
    if !changes_a_file
        || !lexicon.mentions_role(seed::ROLE_CODING_MEMBER_LIST_KIND, &normalized)
    {
        return None;
    }

    // A backtick slot is prose markup; a quotation slot is a literal. The
    // delimiter says what the slot is *for*, and the slot's own text says what
    // it *is*: a workspace path, a declared name, or a member value. Reading
    // both keeps `matches!` out of the values and a quoted `"target"` out of
    // the names, while letting a path be written either way -- which delimiter
    // a request wraps its target in is a spelling, not a different task.
    let mut values = Vec::new();
    let mut named = Vec::new();
    let mut candidates = Vec::new();
    let mut literal_target = None;
    let mut literal_spans = Vec::new();
    for segment in quoted_segment_spans(task) {
        let marked_up = task[segment.start..].starts_with('`');
        let path = is_workspace_path(&segment.text).then(|| segment.text.clone());
        if marked_up {
            if let Some(path) = path {
                candidates.push((segment.start, path));
            } else if valid_identifier(&segment.text) && !named.contains(&segment.text) {
                named.push(segment.text.clone());
            }
        } else if is_member_literal(&segment.text) {
            values.push(segment.text.clone());
            if literal_target.is_none() {
                literal_target = path;
            }
        }
        literal_spans.push((segment.start, segment.end));
    }
    if values.is_empty() {
        return None;
    }

    // The path and any bare identifier hints are read from the prose that is
    // left once the delimited slots are removed, so a quoted value can never be
    // mistaken for either. A quoted path is not a candidate at all, only a last
    // resort, because a quoted path is a member value first.
    let prose = without_spans(task, &literal_spans);
    candidates.extend(bare_source_paths(task, &literal_spans));

    // A request may name several files and change only one of them. The ladder
    // leaf prompts name three -- the source to edit, an effects record, and a
    // proof note -- and an earlier rule that read the first *undelimited* path
    // picked the proof note, because the file to edit was the one the prompt had
    // bothered to mark up. Preferring markup instead only moves the failure: a
    // request is free to mark up an incidental file and leave its target bare.
    //
    // What separates them is not the delimiter but the shape and the position.
    // A token carrying a `/` locates a file in the workspace; a bare basename
    // only mentions one, and a request that means to be acted on says where.
    // Among equals, the earliest wins, because a request states what it is
    // acting on before it says what to do afterwards.
    //
    // Ranked against labelled data by
    // `experiments/issue_1069_paths_in_prose/order-survey.py`, which replays
    // this repository's own commit messages -- change requests written after
    // the fact -- against the files each commit actually touched. Over the 418
    // messages that name two or more paths, taking the earliest names a changed
    // file 68.42% of the time, and preferring a workspace-relative one first
    // raises that to 84.67%; 29.19% of those messages open with a bare basename
    // before giving the path (`release.yml`, then
    // `.github/workflows/release.yml`). Taking the *latest* path is worse than
    // either at 63.64%. The proxy is loose -- a commit touches many files --
    // so it is used to rank the rules against each other, not as an accuracy
    // claim; the exact obligation is pinned instead by
    // `issue_1069_every_ladder_leaf_reaches_a_real_change`, which requires all
    // 32 ladder leaves to reach the file their contract names.
    candidates.sort_by_key(|(offset, path)| (!path.contains('/'), *offset));
    let target = candidates
        .into_iter()
        .next()
        .map(|(_, path)| path)
        .or(literal_target)?;
    for token in prose
        .replacen(&target, " ", 1)
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| looks_like_a_declared_name(token))
    {
        let token = token.to_owned();
        if !named.contains(&token) {
            named.push(token);
        }
    }
    Some(MemberInsertion {
        target,
        values,
        named,
    })
}

/// A byte range of `source` enclosed by one matched delimiter pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Region {
    open: usize,
    close: usize,
}

/// A string literal in `source`, with the innermost region that encloses it.
#[derive(Debug, Clone)]
struct Literal {
    start: usize,
    end: usize,
    value: String,
    enclosing: Option<usize>,
}

fn insert_members(source: &str, edit: &MemberInsertion) -> Option<String> {
    let (regions, literals) = scan(source);
    let anchor = anchor_by_members(&regions, &literals, &edit.values)
        .or_else(|| anchor_by_name(source, &regions, &literals, &edit.named))?;
    let members = direct_members(&literals, anchor.open);
    let absent = edit
        .values
        .iter()
        .filter(|value| !members.iter().any(|member| &member.value == *value))
        .collect::<Vec<_>>();
    if absent.is_empty() {
        // Everything asked for is already a member. Converging on the current
        // bytes keeps a retried task from duplicating its own earlier effect.
        return Some(source.to_owned());
    }
    let last = members.last()?;
    let separator = members
        .windows(2)
        .next_back()
        .and_then(|pair| source.get(pair[0].end..pair[1].start))
        .filter(|gap| !gap.is_empty())
        .unwrap_or(", ");
    let mut addition = String::new();
    for value in absent {
        addition.push_str(separator);
        addition.push('"');
        addition.push_str(value);
        addition.push('"');
    }
    let mut updated = String::with_capacity(source.len() + addition.len());
    updated.push_str(source.get(..last.end)?);
    updated.push_str(&addition);
    updated.push_str(source.get(last.end..)?);
    Some(updated)
}

/// The list the request is about is the one that already holds the members the
/// request named. The smallest such region wins, so a `matches!(subject, Some(
/// "a" | "b"))` resolves to the alternation rather than to the macro call that
/// contains it.
fn anchor_by_members(
    regions: &[Region],
    literals: &[Literal],
    values: &[String],
) -> Option<Region> {
    let mut best: Option<(usize, usize, Region)> = None;
    for region in regions {
        let members = direct_members(literals, region.open);
        let held = values
            .iter()
            .filter(|value| members.iter().any(|member| &member.value == *value))
            .count();
        if held == 0 {
            continue;
        }
        best = better(best, held, *region);
    }
    best.map(|(_, _, region)| region)
}

/// Nothing the request quoted is in the file yet, so fall back to the names it
/// offered. Within the statement that declares the name, the region holding the
/// most string literals is the value list — which is how `&[&str]` loses to
/// `&["a", "b"]` in `const NAMES: &[&str] = &["a", "b"];`.
fn anchor_by_name(
    source: &str,
    regions: &[Region],
    literals: &[Literal],
    named: &[String],
) -> Option<Region> {
    for name in named {
        let mut best: Option<(usize, usize, Region)> = None;
        for declaration in occurrences(source, name) {
            let statement = statement_end(source, declaration);
            for region in regions
                .iter()
                .filter(|region| region.open >= declaration && region.close <= statement)
            {
                let held = direct_members(literals, region.open).len();
                if held == 0 {
                    continue;
                }
                best = better(best, held, *region);
            }
        }
        if let Some((_, _, region)) = best {
            return Some(region);
        }
    }
    None
}

/// Prefer more matched members; break ties toward the tighter region.
const fn better(
    best: Option<(usize, usize, Region)>,
    held: usize,
    region: Region,
) -> Option<(usize, usize, Region)> {
    let span = region.close.saturating_sub(region.open);
    match best {
        Some((seen, tightest, _)) if seen > held || (seen == held && tightest <= span) => best,
        _ => Some((held, span, region)),
    }
}

fn direct_members(literals: &[Literal], open: usize) -> Vec<&Literal> {
    literals
        .iter()
        .filter(|literal| literal.enclosing == Some(open))
        .collect()
}

/// Byte offsets just past each whole-word occurrence of `name`.
fn occurrences(source: &str, name: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = source.get(cursor..).and_then(|rest| rest.find(name)) {
        let at = cursor + relative;
        let end = at + name.len();
        let before_is_word = source[..at]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        let after_is_word = source[end..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        if !before_is_word && !after_is_word {
            found.push(end);
        }
        cursor = end;
    }
    found
}

/// Where the declaration that starts at `from` stops: the first `;` written
/// outside any delimiter pair, or the first blank line, whichever comes first.
fn statement_end(source: &str, from: usize) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = from;
    while index < bytes.len() {
        match bytes[index] {
            b'[' | b'(' | b'{' => depth += 1,
            b']' | b')' | b'}' => depth = depth.saturating_sub(1),
            b';' if depth == 0 => return index,
            b'\n' if depth == 0 && source[index + 1..].starts_with('\n') => return index,
            _ => {}
        }
        index += 1;
    }
    bytes.len()
}

/// Collect every matched delimiter pair and every string literal, recording for
/// each literal the innermost pair that encloses it.
///
/// Literal bodies, line comments and block comments are skipped so a bracket
/// written inside one cannot open a region.
fn scan(source: &str) -> (Vec<Region>, Vec<Literal>) {
    let bytes = source.as_bytes();
    let mut regions = Vec::new();
    let mut literals = Vec::new();
    let mut open_stack: Vec<(u8, usize)> = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = source[index..]
                    .find('\n')
                    .map_or(bytes.len(), |offset| index + offset);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = source[index + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |offset| index + 2 + offset + 2);
            }
            b'\'' => index = char_literal_end(bytes, index),
            b'"' => {
                let Some(closing) = string_literal_end(bytes, index) else {
                    index += 1;
                    continue;
                };
                literals.push(Literal {
                    start: index,
                    end: closing + 1,
                    value: source.get(index + 1..closing).unwrap_or_default().to_owned(),
                    enclosing: open_stack.last().map(|(_, open)| *open),
                });
                index = closing + 1;
            }
            delimiter @ (b'[' | b'(' | b'{') => {
                open_stack.push((delimiter, index));
                index += 1;
            }
            delimiter @ (b']' | b')' | b'}') => {
                if let Some((opened, open)) = open_stack.pop()
                    && closing_delimiter(opened) == delimiter
                {
                    regions.push(Region { open, close: index });
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    // Regions close innermost-first; members read better in source order.
    let mut by_open: HashMap<usize, Region> = HashMap::new();
    for region in regions {
        by_open.entry(region.open).or_insert(region);
    }
    let mut regions = by_open.into_values().collect::<Vec<_>>();
    regions.sort_by_key(|region| (region.open, region.close));
    literals.sort_by_key(|literal| literal.start);
    (regions, literals)
}

const fn closing_delimiter(opened: u8) -> u8 {
    match opened {
        b'[' => b']',
        b'(' => b')',
        _ => b'}',
    }
}

const fn string_literal_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut cursor = open + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'"' => return Some(cursor),
            // An unterminated literal is prose, not source: stop at the line.
            b'\n' => return None,
            _ => cursor += 1,
        }
    }
    None
}

/// `'a'` and `'\n'` are literals whose body must be skipped; `'static` is a
/// lifetime, and skipping it would swallow the rest of the line.
fn char_literal_end(bytes: &[u8], open: usize) -> usize {
    for width in [2, 3] {
        if bytes.get(open + width) == Some(&b'\'') {
            let escaped = bytes.get(open + 1) == Some(&b'\\');
            if (width == 2 && !escaped) || (width == 3 && escaped) {
                return open + width + 1;
            }
        }
    }
    open + 1
}

fn without_spans(text: &str, spans: &[(usize, usize)]) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut cursor = 0;
    for &(start, end) in spans {
        if start < cursor {
            continue;
        }
        kept.push_str(text.get(cursor..start).unwrap_or_default());
        kept.push(' ');
        cursor = end;
    }
    kept.push_str(text.get(cursor..).unwrap_or_default());
    kept
}

/// A member literal is a short single-line phrase.
///
/// Spaces belong inside it. `experiments/issue_1069_member_shape/survey.py`
/// counts the string literals that sit directly inside a bracketed list in this
/// repository's own sources: 523 of 1738 (30.1%) hold whitespace, and 192 of
/// those carry a space at an edge because the list is scanned against prose --
/// `TARGET_MARKERS` holds `"to "` and `"into "` exactly so a scan cannot match
/// inside a longer word. Refusing whitespace refused nearly a third of every
/// member list in the tree, which is why `"Good morning"` could not be added to
/// `GREETING_EXAMPLES` (#1069).
///
/// What keeps a quoted *sentence* out is [`MAX_MEMBER_LENGTH`]; what keeps a
/// quoted code blob out is that a member never spans lines, which the survey
/// confirms -- only 6 of the 1738 do, and all six are `write!` fragments that
/// no request would ask to insert.
fn is_member_literal(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_MEMBER_LENGTH
        && !value.contains(['"', '\\'])
        && value
            .chars()
            .all(|character| !character.is_whitespace() || character == ' ')
}

/// A bare prose token only counts as the name of a declaration when it is
/// spelled the way declarations are and prose is not: `snake_case`, `SCREAMING`
/// or `camelCase`.
fn looks_like_a_declared_name(token: &str) -> bool {
    valid_identifier(token)
        && token.len() > 1
        && (token.contains('_')
            || token.chars().all(|character| !character.is_ascii_lowercase())
            || (token.chars().any(|character| character.is_ascii_uppercase())
                && !token.starts_with(|character: char| character.is_ascii_uppercase())))
}

/// Every undelimited workspace path in `task`, each with its byte offset in
/// `task`, skipping the delimited slots so a quoted member value is never read
/// as a path.
fn bare_source_paths(task: &str, literal_spans: &[(usize, usize)]) -> Vec<(usize, String)> {
    let mut paths = Vec::new();
    let mut cursor = 0;
    for (start, end) in literal_spans
        .iter()
        .copied()
        .chain(std::iter::once((task.len(), task.len())))
    {
        if let Some(gap) = task.get(cursor..start) {
            paths.extend(
                prose_path_tokens(gap).map(|(offset, path)| (cursor + offset, path)),
            );
        }
        cursor = end.max(cursor);
    }
    paths
}

/// The workspace paths in a run of prose, in order, each with its byte offset.
fn prose_path_tokens(prose: &str) -> impl Iterator<Item = (usize, String)> + '_ {
    prose
        .split_inclusive(|character: char| character.is_whitespace() || character == ',')
        .scan(0_usize, |cursor, chunk| {
            let start = *cursor;
            *cursor += chunk.len();
            Some((start, chunk))
        })
        .filter_map(|(start, chunk)| {
            let trimmed = chunk.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-' | '.' | '/')
            });
            let token = trimmed.trim_end_matches('.');
            if !is_workspace_path(token) {
                return None;
            }
            let offset = start + (token.as_ptr() as usize - chunk.as_ptr() as usize);
            Some((offset, token.to_owned()))
        })
}

fn is_workspace_path(token: &str) -> bool {
    let Some((stem, extension)) = token.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && !token.starts_with('/')
        && !token.split('/').any(|component| component == "..")
        && (1..=8).contains(&extension.len())
        && extension.chars().all(|character| character.is_ascii_alphanumeric())
        // A dotted run of digits is a number, not a file. Without this the
        // recursive ladder's own node id `1.1.2.2.1` reads as a path, and a
        // leaf prompt that names both `src/engine_responses.rs` and its node
        // sends the planner to open the node: the first real Agent CLI run of
        // that leaf opened with `File not found: .../1.1.2.2.1`, then, having
        // observed nothing, built an edit out of sentences from its own prompt.
        //
        // Measured rather than guessed, by
        // `experiments/issue_1069_dotted_tokens/survey.py`: across 15 782
        // tracked files and 67 distinct extensions, **no** tracked file has an
        // all-digit extension, while 12.77% of the dotted tokens this predicate
        // accepts in committed Markdown end in one -- 1 455 distinct spellings,
        // every one of them an IP address, a licence identifier, a version or a
        // section number (`127.0.0.1`, `Apache-2.0`, `0.1.0-beta.1`, `v1.22.0`),
        // and not one of them the name of a file this repository tracks.
        && !extension.chars().all(|character| character.is_ascii_digit())
        && token.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
        })
}

fn valid_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn read_arguments(path: &str) -> String {
    json!({"path": path, "filePath": path, "file_path": path}).to_string()
}
