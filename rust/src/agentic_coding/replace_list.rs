//! Several replacements in one file, asked in one sentence (PR #1188 G82).
//!
//! `In b.lino replace 'x' with 'y', replace 'p' with 'q', and replace 'm'
//! with 'n'.` The sentence is cut before every repeated seeded edit action
//! outside its quotes; a clause that names no file takes the file the first
//! one names, and each clause is the edit request one replacement alone would
//! be. The replacements are applied in order to the file as read. The Rust
//! original of `js/agentic/replace_list.mjs`.

use super::write_request::{bare_surfaces, clean_cue_token, compose_edit_request, tokens};
use crate::normal_markov::{quoted_segment_spans, quoted_segments};
use crate::seed;

/// How a slot lists several names: `a`, `b`.
const LISTED: &str = "`, `";

/// The marks that end a clause before the next edit action.
const CLAUSE_TAIL: &[char] = &[',', ';', '，', '；'];

/// A clause without the whitespace and clause marks that close it.
fn without_tail(text: &str) -> &str {
    text.trim_end_matches(|character: char| {
        character.is_whitespace() || CLAUSE_TAIL.contains(&character)
    })
}

/// The request cut before every repeated unquoted edit action.
///
/// Each clause loses the commas and seeded joiners that led into the next;
/// `None` unless there are at least two (mirrors `replaceClauses`).
fn replace_clauses(request: &str) -> Option<Vec<String>> {
    let segments = quoted_segment_spans(request);
    let actions = bare_surfaces(seed::ROLE_FILE_EDIT_ACTION_CUE);
    let joiners = bare_surfaces("file_edit_joiner_cue");
    let cuts: Vec<usize> = tokens(request)
        .iter()
        .filter(|token| {
            !segments
                .iter()
                .any(|segment| token.start < segment.end && token.end > segment.start)
                && actions.contains(&clean_cue_token(token.text))
        })
        .map(|token| token.start)
        .collect();
    if cuts.len() < 2 {
        return None;
    }
    let mut pieces = Vec::new();
    for (index, cut) in cuts.iter().enumerate() {
        let from = if index == 0 { 0 } else { *cut };
        let to = cuts.get(index + 1).copied().unwrap_or(request.len());
        let mut piece = without_tail(request.get(from..to)?);
        loop {
            let words: Vec<&str> = piece.split_whitespace().collect();
            let Some(last) = words.last().filter(|_| words.len() >= 2) else {
                break;
            };
            if !joiners.contains(&clean_cue_token(last)) {
                break;
            }
            piece = without_tail(&piece[..piece.len() - last.len()]);
        }
        pieces.push(piece.to_owned());
    }
    Some(pieces)
}

/// The one file and the quoted `(old, new)` pairs a request replaces, in order.
///
/// `None` unless every clause is one replacement of quoted text in that file
/// (mirrors `replaceList`).
pub(super) fn replace_list(request: &str) -> Option<(String, Vec<(String, String)>)> {
    let clauses = replace_clauses(request)?;
    let (target, _, _) = compose_edit_request(clauses.first()?)?;
    let quoted = quoted_segments(request);
    let mut pairs = Vec::new();
    for clause in &clauses {
        let (file, old, new) = if clause.contains(target.as_str()) {
            compose_edit_request(clause)?
        } else {
            compose_edit_request(&format!("{clause} in {target}"))?
        };
        if file != target
            || !quoted.contains(&old)
            || !quoted.contains(&new)
            || old.is_empty()
            || old == new
        {
            return None;
        }
        pairs.push((old, new));
    }
    Some((target, pairs))
}

/// `source` with each pair's old text replaced everywhere, in order.
///
/// `None` when an old text is missing by the time its turn comes (mirrors
/// `replacedInOrder`).
pub(super) fn replaced_in_order(source: &str, pairs: &[(String, String)]) -> Option<String> {
    let mut text = source.to_owned();
    for (old, new) in pairs {
        if !text.contains(old.as_str()) {
            return None;
        }
        text = text.replace(old.as_str(), new);
    }
    Some(text)
}

/// The honest answer when the file no longer holds `old` (PR #1188 G87).
///
/// The replacement is already made when the file holds `new`, else the text
/// does not occur -- never a failed verification of an effect nothing
/// planned. `None` while `old` (any name a listed slot gives) is still there.
pub(super) fn absent_text_answer(
    task: &str,
    target: &str,
    source: &str,
    old: &str,
    new: Option<&str>,
) -> Option<String> {
    if old.is_empty() || old.split(LISTED).any(|name| source.contains(name)) {
        return None;
    }
    match new.filter(|new| !new.is_empty() && source.contains(*new)) {
        Some(new) => super::code_task::render_seeded_change(
            "coding_text_already_replaced",
            task,
            target,
            &[("{old}", old), ("{new}", new)],
        ),
        None => super::code_task::render_seeded_change(
            "coding_text_not_found",
            task,
            target,
            &[("{old}", old)],
        ),
    }
}
