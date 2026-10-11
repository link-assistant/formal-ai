//! Several replacements in one file, asked in one sentence (PR #1188 G82).
//!
//! `In b.lino replace 'x' with 'y', replace 'p' with 'q', and replace 'm'
//! with 'n'.` The sentence is cut before every repeated seeded edit action
//! outside its quotes; a clause that names no file takes the file another
//! one names, and each clause is the edit request one replacement alone would
//! be. The replacements are applied in order to the file as read. The Rust
//! original of `js/agentic/replace_list.mjs`.

use super::positional_edit::{instruction_end, unescape_prose_newlines};
use super::write_request::{
    Token, bare_surfaces, clean_cue_token, clean_path_token, compose_edit_request,
    cued_write_targets, looks_like_file_path, safe_relative_path, tokens,
};
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
    let end = instruction_end(request);
    let cuts: Vec<usize> = tokens(request)
        .iter()
        .filter(|token| {
            token.start < end
                && !segments
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
    // The file may be named in any clause: `In m.js, replace ...` names it
    // first and `Replace ... and replace ... in m.js` last (PR #1188 G89).
    let (target, _, _) = clauses
        .iter()
        .find_map(|clause| compose_edit_request(clause))?;
    // A quoted text may spell a newline as an escape, which the composed edit
    // has already turned into the character (PR #1188 G88).
    let quoted: Vec<String> = quoted_segments(request)
        .into_iter()
        .flat_map(|segment| {
            let unescaped = unescape_prose_newlines(&segment);
            [segment, unescaped]
        })
        .collect();
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

/// The marks that end a sentence, dropped from a clause the answer names.
const SENTENCE_END: &[char] = &['.', '!', '?', '。', '！', '？'];

/// The first clause of a request asking for several edits that is no
/// replacement of quoted text in the request's file, when the first clause is
/// one (PR #1188 G106; mirrors `unplannedEditClause`).
///
/// Such a request would otherwise make the first edit and drop the rest
/// without a word, so it is declined, naming the clause. A clause is an edit
/// of its own only when a clause mark or a seeded joiner opens it (`, replace`,
/// `and replace`); `None` when there is no such clause or every one is a
/// replacement.
pub(super) fn unplanned_edit_clause(request: &str) -> Option<String> {
    let clauses = replace_clauses(request)?;
    if replace_list(request).is_some() {
        return None;
    }
    let (target, _, _) = compose_edit_request(&clauses[0])?;
    let joiners = bare_surfaces("file_edit_joiner_cue");
    let mut from = 0;
    for clause in &clauses[1..] {
        let at = from + request.get(from..)?.find(clause.as_str())?;
        from = at + clause.len();
        let lead = request[..at].trim_end();
        let opened = lead.ends_with(CLAUSE_TAIL)
            || lead
                .split_whitespace()
                .last()
                .is_some_and(|word| joiners.contains(&clean_cue_token(word)));
        if !opened {
            continue;
        }
        let edit = if clause.contains(target.as_str()) {
            compose_edit_request(clause)
        } else {
            compose_edit_request(&format!("{clause} in {target}"))
        };
        let segments = quoted_segments(clause);
        let replaces = edit.is_some_and(|(file, old, new)| {
            file == target && segments.contains(&old) && segments.contains(&new)
        });
        if !replaces {
            return Some(clause.trim_end_matches(SENTENCE_END).to_owned());
        }
    }
    None
}

/// The distinct files an edit request names outside its quotes, when it names
/// more than one (PR #1188 G91).
///
/// One edit request changes one file, so such a request is declined with the
/// files named rather than applied to the first alone (mirrors
/// `severalEditTargets`).
pub(super) fn several_edit_targets(request: &str) -> Option<Vec<String>> {
    let composed = super::write_request::compose_edit_clauses(request)?;
    let segments = quoted_segment_spans(request);
    let toks = tokens(request);
    let (from, end) = edit_region(request, composed.spans);
    let counted = |index: usize| {
        let token = &toks[index];
        token.start >= from
            && token.start < end
            && !segments
                .iter()
                .any(|segment| token.start < segment.end && token.end > segment.start)
    };
    let mut targets: Vec<(usize, String)> = cued_write_targets(&toks)
        .into_iter()
        .filter(|(index, _)| counted(*index))
        .collect();
    // A file listed after a cued one shares its cue (PR #1188 G104):
    // `In a.mjs and b.mjs, replace ...` names b.mjs as much as a.mjs.
    let cued: Vec<usize> = targets.iter().map(|(index, _)| *index).collect();
    for (index, path) in coordinated_paths(&toks, &cued) {
        if counted(index) {
            targets.push((index, path));
        }
    }
    targets.sort_by_key(|(index, _)| *index);
    let mut paths: Vec<String> = Vec::new();
    for (_, path) in targets {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    (paths.len() > 1).then_some(paths)
}

/// The sentences that hold the composed edit's clauses (`spans`), or the
/// whole instruction when the edit has none (mirrors `editRegion`).
///
/// A path in a later sentence (`Leave supporting evidence in proof.md.`) is
/// no target of the edit (PR #1188 ladder regression of G91).
fn edit_region(request: &str, spans: Option<[(usize, usize); 2]>) -> (usize, usize) {
    let end = instruction_end(request);
    let Some(spans) = spans else {
        return (0, end);
    };
    let first = spans[0].0.min(spans[1].0);
    let last = spans[0].1.max(spans[1].1);
    let parts = super::shell_command_policy::sentences(request);
    let opening = parts
        .iter()
        .find(|sentence| first >= sentence.span.start && first < sentence.span.end)
        .map_or(0, |sentence| sentence.span.start);
    let closing = parts
        .iter()
        .find(|sentence| last > sentence.span.start && last <= sentence.span.end)
        .map_or(end, |sentence| sentence.span.end);
    (opening, end.min(closing))
}

/// The paths listed right after one of the `cued` path tokens, each with its
/// token index (PR #1188 G104).
///
/// A path is listed when a comma closes the token before it, or when a seeded
/// joiner word (`and`) stands between them (mirrors `coordinatedPaths`).
pub(super) fn coordinated_paths(toks: &[Token<'_>], cued: &[usize]) -> Vec<(usize, String)> {
    let joiners = bare_surfaces("file_edit_joiner_cue");
    let path_at = |index: usize| -> Option<String> {
        let cleaned = clean_path_token(toks.get(index)?.text);
        (looks_like_file_path(cleaned) && safe_relative_path(cleaned)).then(|| cleaned.to_owned())
    };
    let mut out = Vec::new();
    for &start in cued {
        let mut index = start;
        loop {
            let listed = toks[index].text.ends_with(',');
            let mut next = index + 1;
            let joined = toks
                .get(next)
                .is_some_and(|token| joiners.contains(&clean_cue_token(token.text)));
            if joined {
                next += 1;
            } else if !listed {
                break;
            }
            let Some(path) = path_at(next) else {
                break;
            };
            out.push((next, path));
            index = next;
        }
    }
    out
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

/// The honest answer when the file no longer holds `replaced` (PR #1188 G87).
///
/// The replacement is already made when the file holds `replacement`, else
/// the text does not occur -- never a failed verification of an effect
/// nothing planned. `None` while `replaced` (any name a listed slot gives) is
/// still there.
pub(super) fn absent_text_answer(
    task: &str,
    target: &str,
    source: &str,
    replaced: &str,
    replacement: Option<&str>,
) -> Option<String> {
    if replaced.is_empty() || replaced.split(LISTED).any(|name| source.contains(name)) {
        return None;
    }
    replacement
        .filter(|text| !text.is_empty() && source.contains(*text))
        .map_or_else(
            || {
                super::code_task::render_seeded_change(
                    "coding_text_not_found",
                    task,
                    target,
                    &[("{old}", replaced)],
                )
            },
            |text| {
                super::code_task::render_seeded_change(
                    "coding_text_already_replaced",
                    task,
                    target,
                    &[("{old}", replaced), ("{new}", text)],
                )
            },
        )
}
