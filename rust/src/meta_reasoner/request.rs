//! The request's shape beyond its literals.
//!
//! Location operands ("any file in data"), the imperative mood ("replace every colon ..."), and where a word
//! stands, so values named by words keep the request's order.
//!
//! Originals: `metaLocationOperands`, `metaWordPosition` and the imperative
//! test of `metaReasonCore` in `js/worker/formal_ai_worker_meta_reasoner.js`.

use super::BOUNDS;
use super::seed::meta_seed;
use super::text::{is_letter, is_number, js_trim, meta_words};

/// A location operand: a bare name after a locative cue that follows a file
/// noun, with its byte span in the request whose literals are blanked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operand {
    /// Byte offset where the name starts.
    pub start: usize,
    /// Byte offset just past the name.
    pub end: usize,
    /// The name, as written.
    pub value: String,
    /// The cue it follows, trimmed.
    pub cue: String,
}

/// True when a word opens an imperative: its documentation names one or two
/// operations, not only representation changes.
///
/// Mirrors the `openingTop` test of `metaReasonCore` in
/// `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn names_operation(word: &str, language: &str) -> bool {
    let seed = meta_seed();
    if seed.is_grammatical(word) {
        return false;
    }
    let hypotheses = seed.doc_hypotheses(word, language);
    let Some(first) = hypotheses.first() else {
        return false;
    };
    let top: Vec<&str> = hypotheses
        .iter()
        .filter(|item| item.score >= first.score * 0.99)
        .map(|item| item.operation.as_str())
        .collect();
    top.len() <= BOUNDS.required_group_size && !top.iter().all(|id| seed.is_view(id))
}

/// Location operands.
///
/// The bare name right after a locative cue that follows a file noun, a word whose documented operations all take or give a path.
/// A name with no determiner is a proper name ("in data"); "in a folder"
/// names no operand.
///
/// Mirrors `metaLocationOperands` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn location_operands(text: &str, language: &str) -> Vec<Operand> {
    let seed = meta_seed();
    let lowered = text.to_lowercase();
    let mut out: Vec<Operand> = Vec::new();
    for marker in seed.cue_markers("location") {
        let step = marker.chars().next().map_or(1, char::len_utf8);
        let mut from = 0;
        while let Some(found) = lowered
            .get(from..)
            .and_then(|rest| rest.find(marker.as_str()))
        {
            let at = from + found;
            from = at + step;
            let before = meta_words(&lowered[..at]);
            let start = at + marker.len();
            let Some(noun) = before.last() else {
                continue;
            };
            let Some(name) = text
                .get(start..)
                .map(bare_name)
                .filter(|name| !name.is_empty())
            else {
                continue;
            };
            if seed.is_grammatical(&name.to_lowercase()) || !is_file_noun(noun, language) {
                continue;
            }
            if out.iter().any(|operand| operand.start == start) {
                continue;
            }
            out.push(Operand {
                start,
                end: start + name.len(),
                value: name.to_owned(),
                cue: js_trim(&marker).to_owned(),
            });
        }
    }
    out.sort_by_key(|operand| operand.start);
    out
}

/// The name at the start of a text: word characters, joined by single dots
/// or hyphens (`data`, `notes.v2`).
///
/// Mirrors `/^[\p{L}\p{N}_]+(?:[.-][\p{L}\p{N}_]+)*/u` in
/// `metaLocationOperands` (`js/worker/formal_ai_worker_meta_reasoner.js`).
fn bare_name(text: &str) -> &str {
    let word = |character: char| is_letter(character) || is_number(character) || character == '_';
    let mut end = 0;
    let mut characters = text.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if word(character) {
            end = index + character.len_utf8();
        } else if !(matches!(character, '.' | '-')
            && end == index
            && end > 0
            && characters.peek().is_some_and(|(_, next)| word(*next)))
        {
            break;
        }
    }
    &text[..end]
}

/// True when every documented operation of a word that is a primitive takes
/// or gives a path (`file`, `folder`).
///
/// Mirrors the file-noun test of `metaLocationOperands` in
/// `js/worker/formal_ai_worker_meta_reasoner.js`.
fn is_file_noun(word: &str, language: &str) -> bool {
    let seed = meta_seed();
    let primitives: Vec<_> = seed
        .doc_hypotheses(word, language)
        .iter()
        .filter_map(|item| seed.primitive(&item.operation))
        .collect();
    !primitives.is_empty()
        && primitives.iter().all(|primitive| {
            [primitive.from.as_str(), primitive.to.as_str()]
                .iter()
                .any(|kind| matches!(*kind, "path" | "list_path"))
        })
}

/// Whether the request names a file noun anywhere.
///
/// A request about files ("every file in a folder") asks for a program over
/// paths, which no fixed template renders.
///
/// Mirrors `metaNamesFileNoun` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn names_file_noun(text: &str, language: &str) -> bool {
    meta_words(text)
        .iter()
        .any(|word| !meta_seed().is_grammatical(word) && is_file_noun(word, language))
}

/// Where a word first stands in the request (a byte offset in its lower
/// case), so values named by words and values stated as literals keep the
/// request's order.
///
/// Mirrors `metaWordPosition` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn word_position(prompt: &str, word: &str) -> usize {
    let lowered = prompt.to_lowercase();
    let boundary = |character: Option<char>| {
        character.is_none_or(|character| {
            !(is_letter(character) || is_number(character) || character == '_')
        })
    };
    for (at, _) in lowered.match_indices(word) {
        let before = lowered[..at].chars().next_back();
        let after = lowered[at + word.len()..].chars().next();
        if boundary(before) && boundary(after) {
            return at;
        }
    }
    prompt.len()
}
