//! Issue #1487 — the proposition structure a coding requirement holds that no
//! discovered coding part reads.
//!
//! Program synthesis composes a candidate from the coding-structure meanings a
//! requirement evidences. A structural word no such meaning claims — a
//! negation (`not vowels`), an explicit count (`twice the number`), a
//! disjunction (`a or b`), an ordered comparison or operator (`x < 10`) —
//! would otherwise be dropped in silence, and `Return twice the number of
//! vowels` would receive the program for `Return the number of vowels`, then
//! pass the examples derived from the same reading. The residual structure is
//! the obligation the candidate leaves unresolved; composition withholds the
//! candidate rather than claiming it.
//!
//! The surfaces a discovered meaning claims (`below zero`, `less than or
//! equal`, `if not`) are blanked before the requirement's surface structure is
//! read, so structure that is read by a part is not reported. Quoted literal
//! text (`prints "a or b"`) is data the program reproduces, not structure, so
//! it is blanked too, and so is a code identifier (`below_zero`), whose
//! underscore joins words into a name rather than a phrase. A hyphen between
//! two words of two or more letters (`reverse-sorted`, `kebab-case`) joins a
//! compound word; between single letters (`a-b`) it stays an operator. The
//! JavaScript twin is
//! `js/agentic/crate/unread_structure.mjs`.

use crate::formalization::proposition_structure::surface_structure;

/// Lower-cased surfaces of the coding meanings `ids`, longest first.
#[must_use]
pub fn claimed_surfaces<S: AsRef<str>>(ids: &[S]) -> Vec<String> {
    let lexicon = crate::seed::lexicon();
    let mut out: Vec<String> = Vec::new();
    for id in ids {
        let Some(found) = lexicon.meaning(id.as_ref()) else {
            continue;
        };
        for word in found.words() {
            let surface = word.to_lowercase().trim().to_owned();
            if !surface.is_empty() && !out.contains(&surface) {
                out.push(surface);
            }
        }
    }
    out.sort_by(|left, right| {
        right
            .chars()
            .count()
            .cmp(&left.chars().count())
            .then_with(|| left.cmp(right))
    });
    out
}

/// A surface occurrence at byte `start` that begins and ends on a word
/// boundary (any occurrence, for a CJK surface).
fn bounded_at(text: &str, start: usize, surface: &str) -> bool {
    if crate::coding::contains_cjk(surface) {
        return true;
    }
    let before = text[..start].chars().next_back();
    let after = text[start + surface.len()..].chars().next();
    before.is_none_or(|character| !character.is_alphanumeric())
        && after.is_none_or(|character| !character.is_alphanumeric())
}

/// Literal delimiters that need no word boundary.
pub const QUOTE_PAIRS: [(char, char); 6] = [
    ('"', '"'),
    ('`', '`'),
    ('\u{201c}', '\u{201d}'),
    ('\u{ab}', '\u{bb}'),
    ('\u{300c}', '\u{300d}'),
    ('\u{300e}', '\u{300f}'),
];

/// Every closed quoted span of `text` replaced by one space.
#[must_use]
pub fn blank_quoted(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    loop {
        let Some((opener, open, close)) = rest.char_indices().find_map(|(index, character)| {
            QUOTE_PAIRS
                .iter()
                .find(|(candidate, _)| *candidate == character)
                .map(|(open, close)| (index, *open, *close))
        }) else {
            out.push_str(rest);
            return out;
        };
        let body = &rest[opener + open.len_utf8()..];
        let Some(end) = body.find(close) else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..opener]);
        out.push(' ');
        rest = &body[end + close.len_utf8()..];
    }
}

/// Every word run joined by an underscore replaced by one space.
#[must_use]
pub fn blank_identifiers(text: &str) -> String {
    let mut out = String::new();
    let mut run = String::new();
    for character in text.chars() {
        if character == '_' || character.is_alphanumeric() {
            run.push(character);
        } else {
            flush_run(&mut out, &mut run);
            out.push(character);
        }
    }
    flush_run(&mut out, &mut run);
    out
}

fn flush_run(out: &mut String, run: &mut String) {
    if run.contains('_') {
        out.push(' ');
    } else {
        out.push_str(run);
    }
    run.clear();
}

/// A hyphen joining two words of two or more letters becomes a space.
#[must_use]
pub fn split_compounds(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let before = |index: usize| {
        characters[..index]
            .iter()
            .rev()
            .take_while(|character| character.is_alphabetic())
            .count()
    };
    let after = |index: usize| {
        characters[index + 1..]
            .iter()
            .take_while(|character| character.is_alphabetic())
            .count()
    };
    characters
        .iter()
        .enumerate()
        .map(|(index, &character)| {
            if character == '-' && before(index) >= 2 && after(index) >= 2 {
                ' '
            } else {
                character
            }
        })
        .collect()
}

/// `text` lower-cased, quoted spans and identifiers blanked, compounds split, and every claimed surface blanked.
#[must_use]
pub fn mask_claimed(text: &str, surfaces: &[String]) -> String {
    let mut masked = split_compounds(&blank_identifiers(&blank_quoted(text))).to_lowercase();
    for surface in surfaces {
        let mut from = 0;
        while let Some(offset) = masked
            .get(from..)
            .and_then(|rest| rest.find(surface.as_str()))
        {
            let start = from + offset;
            if bounded_at(&masked, start, surface) {
                masked.replace_range(start..start + surface.len(), " ");
                from = start + 1;
            } else {
                from = start + surface.len();
            }
        }
    }
    masked
}

/// The structure of `text` left once the surfaces of the discovered coding
/// meanings `ids` are blanked — empty when every structural word is read.
#[must_use]
pub fn unread_structure<S: AsRef<str>>(text: &str, ids: &[S]) -> String {
    let read = surface_structure(&mask_claimed(text, &claimed_surfaces(ids)));
    let mut parts = Vec::new();
    if !read.structure.is_empty() {
        parts.push(read.structure);
    }
    if !read.scope.is_empty() {
        parts.push(format!("scope={}", read.scope));
    }
    parts.join(";")
}
