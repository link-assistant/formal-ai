//! The part of a file a Replace is scoped to (PR #1188 G79).
//!
//! `In languages.rs, in the row whose slug is rust, replace 'x' with 'y'`:
//! the edit composer reads the path, the old text and the new text; a further
//! clause that a seeded target cue leads (`in the row …`) narrows where the
//! old text is meant. Where the old text occurs once the clause changes
//! nothing; where it occurs more than once, no part of the file is named that
//! the planner can find, so nothing is changed and the answer names the
//! clause. The Rust original of `js/agentic/edit_scope.mjs`.

use super::code_task::render_seeded_change;
use super::workspace_setting::line_block_starts;
use super::write_request::{bare_surfaces, clean_cue_token, compose_edit_clauses, tokens};
use crate::seed;

/// The marks that end a sentence when whitespace follows them.
const SENTENCE_ENDS: &[char] = &['.', '!', '?', '。', '！', '？'];
/// The marks that close a clause on the word they end.
const CLAUSE_MARKS: &[char] = &[',', ';', ':'];

/// The words of a clause that narrows an edit request to a part of its file.
///
/// `the row whose slug is rust`: the clause stands outside the edit's own
/// clauses -- between them, or before them on the request's line -- and a
/// seeded target cue leads it (mirrors `editScope`).
fn edit_scope(request: &str) -> Option<String> {
    let mut spans = compose_edit_clauses(request)?.spans?;
    spans.sort_unstable();
    let [first, second] = spans;
    // Each gap counts from its line's start and past its last sentence end: a
    // clause of another sentence (`Open f.txt in the editor.`) is no scope.
    let line_start = request.get(..first.0)?.rfind('\n').map_or(0, |at| at + 1);
    let quoted = crate::normal_markov::quoted_segment_spans(request);
    let cues = bare_surfaces(seed::ROLE_FILE_EDIT_TARGET_CUE);
    let is_cue = |text: &str| cues.contains(&clean_cue_token(text));
    let toks: Vec<_> = tokens(request)
        .into_iter()
        .filter(|token| {
            !quoted
                .iter()
                .any(|segment| token.start < segment.end && token.end > segment.start)
        })
        .collect();
    [(line_start, first.0), (first.1, second.0)]
        .into_iter()
        .find_map(|(start, to)| {
            let from = sentence_start(request, start, to);
            let words: Vec<_> = toks
                .iter()
                .filter(|token| token.start >= from && token.end <= to)
                .collect();
            let lead = words.iter().position(|token| is_cue(token.text))?;
            let mut clause = Vec::new();
            for token in &words[lead + 1..] {
                clause.push(*token);
                if token.text.ends_with(CLAUSE_MARKS) {
                    break;
                }
            }
            let named = clause
                .iter()
                .any(|token| token.text.chars().any(char::is_alphabetic) && !is_cue(token.text));
            let (head, tail) = (clause.first()?, clause.last()?);
            named.then(|| {
                request[head.start..tail.end]
                    .trim_matches(|character: char| !character.is_alphanumeric())
                    .to_owned()
            })
        })
}

/// Where the sentence holding `to` starts within `from..to`.
///
/// Past the last sentence end in that gap, or at `from`.
fn sentence_start(request: &str, from: usize, to: usize) -> usize {
    let gap = request.get(from..to).unwrap_or_default();
    let mut start = from;
    let mut characters = gap.char_indices().peekable();
    while let Some((at, character)) = characters.next() {
        if !SENTENCE_ENDS.contains(&character)
            || !characters
                .peek()
                .is_some_and(|(_, next)| next.is_whitespace())
        {
            continue;
        }
        let mut end = at + character.len_utf8();
        while let Some((next_at, next)) = characters.next_if(|(_, next)| next.is_whitespace()) {
            end = next_at + next.len_utf8();
        }
        start = from + end;
    }
    start
}

/// The scope, old text and count of a scoped request's repeated old text.
///
/// `None` unless the old text occurs more than once in `source`, as text or
/// as one run of whole lines (mirrors `scopedOccurrences`).
fn scoped_occurrences(request: &str, source: &str) -> Option<(String, String, usize)> {
    let scope = edit_scope(request)?;
    let (_, old, _) = compose_edit_clauses(request)?.edit;
    if old.is_empty() {
        return None;
    }
    let blocks = if old.contains('\n') {
        line_block_starts(source, &old, 0).len()
    } else {
        0
    };
    let count = source.matches(old.as_str()).count().max(blocks);
    (count > 1).then_some((scope, old, count))
}

/// The answer declining a scoped edit whose old text repeats.
///
/// The request narrows its edit to a part of the file where its old text
/// occurs more than once; the answer names the part (PR #1188 G79; mirrors
/// `scopedDecline`).
pub(super) fn scoped_decline(request: &str, target: &str, source: &str) -> Option<String> {
    let (part, needle, times) = scoped_occurrences(request, source)?;
    render_seeded_change(
        "file_edit_scope_unapplied",
        request,
        target,
        &[
            ("{old}", &needle.replace('\n', "`, `")),
            ("{scope}", &part),
            ("{count}", &times.to_string()),
        ],
    )
}
