//! Additive edits and the words that place them (issues #1115, #1116, #1133).
//!
//! "Add a line X directly after the line Y in F" names no old→new pair, so the
//! replacement composer in `general_planner` answered it with nothing and the
//! request went to web search. The anchor line is the text an edit tool can
//! match exactly, and the replacement is the anchor with the new line beside
//! it -- so an insertion is an edit after all, and no insert primitive is
//! needed. Which side of the position cue the anchor sits on is a fact of the
//! cue's language (`languages.lino`, `adposition`), not of the sentence.

use super::write_request::{
    clean_content, clean_path_token, looks_like_file_path, safe_relative_path,
    tokens,
};
use crate::seed;

/// The bytes a quoted or clause-led span of edit prose stands for.
pub(super) fn literal_text(span: &str) -> Option<String> {
    quoted_verbatim(span)
        .or_else(|| clean_content(span))
        .map(|text| unescape_prose_newlines(&text))
}

/// An additive edit: "add a line X directly after the line Y in F", "insert X
/// before Y in F" (issue #1115). The anchor line is the text replaced, and the
/// replacement is the anchor with the new line beside it, so the edit tool's
/// exact-match contract holds without a separate insert primitive.
pub(super) fn compose_positional_insert(request: &str) -> Option<(String, String, String)> {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(request);
    let after = lexicon.mentions_role(seed::ROLE_FILE_EDIT_POSITION_AFTER, &normalized);
    let before = lexicon.mentions_role(seed::ROLE_FILE_EDIT_POSITION_BEFORE, &normalized);
    if after == before || !lexicon.mentions_role(seed::ROLE_CODING_MEMBER_ADD_ACTION, &normalized) {
        return None;
    }
    let literals = quoted_literals(request);
    let [first, second] = literals.as_slice() else {
        return None;
    };
    let target = tokens(request)
        .iter()
        .map(|token| clean_path_token(token.text))
        .find(|candidate| looks_like_file_path(candidate) && safe_relative_path(candidate))?
        .to_owned();
    // The anchor is the literal the position cue governs: the one after it in
    // a prepositional language ("after the line "on:""), the one before it in
    // a postpositional language (""on:" के बाद", ""on:" 行后"). Which side a
    // language uses is a fact of the language, recorded in its ledger entry.
    let role = if after {
        seed::ROLE_FILE_EDIT_POSITION_AFTER
    } else {
        seed::ROLE_FILE_EDIT_POSITION_BEFORE
    };
    let (inserted, anchor) = match cue_governed_literal(request, role, first, second) {
        1 => (first, second),
        _ => (second, first),
    };
    let inserted = unescape_prose_newlines(&inserted.text);
    let anchor = unescape_prose_newlines(&anchor.text);
    let new = if after {
        [anchor.as_str(), inserted.as_str()].join("\n")
    } else {
        [inserted.as_str(), anchor.as_str()].join("\n")
    };
    Some((target, anchor, new))
}

/// One quoted span of a request and where it sits.
struct QuotedLiteral {
    start: usize,
    end: usize,
    text: String,
}

/// The double-quoted and backtick-quoted spans of `request`, verbatim: a
/// quoted line keeps its indentation.
fn quoted_literals(request: &str) -> Vec<QuotedLiteral> {
    let mut literals = Vec::new();
    let mut offset = 0;
    while let Some(start) = request[offset..].find(['"', '`']) {
        let open = offset + start;
        let quote = &request[open..=open];
        let Some(length) = request[open + 1..].find(quote) else {
            break;
        };
        let close = open + 1 + length;
        literals.push(QuotedLiteral {
            start: open,
            end: close + 1,
            text: request[open + 1..close].to_owned(),
        });
        offset = close + 1;
    }
    literals
}

/// Which of two literals the position cue governs: `1` for the second, `0`
/// for the first.
///
/// Every occurrence of every surface of the cue is located, each with the
/// language its lexeme belongs to, because the side a cue governs is a fact
/// of *its* language and a mixed prompt (`config.yml में … "on:" के बाद`)
/// cannot be trusted to detect as one: a preposition governs the literal
/// after it, a postposition the literal before it (`languages.lino`,
/// `adposition`). The governed literal is the nearest one on that side; with
/// no occurrence found, the prepositional default stands.
fn cue_governed_literal(
    request: &str,
    role: &str,
    first: &QuotedLiteral,
    second: &QuotedLiteral,
) -> usize {
    let lowered = request.to_lowercase();
    let mut occurrences: Vec<(usize, usize, bool)> = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(role) {
        for lexeme in &meaning.lexemes {
            let postpositional = crate::language::uses_postpositions(&lexeme.language);
            for word in &lexeme.words {
                let surface = word.text.to_lowercase();
                occurrences.extend(
                    lowered
                        .match_indices(surface.as_str())
                        .map(|(start, matched)| (start, start + matched.len(), postpositional)),
                );
            }
        }
    }
    let gap = |literal: &QuotedLiteral| {
        occurrences
            .iter()
            // `checked_sub` is the guard as well as the arithmetic: a cue on
            // the wrong side of the literal yields `None` rather than a
            // subtraction that underflows. `then_some` evaluated its argument
            // whatever the condition said, which panicked in a debug build.
            .filter_map(|&(cue_start, cue_end, postpositional)| {
                if postpositional {
                    cue_start.checked_sub(literal.end)
                } else {
                    literal.start.checked_sub(cue_end)
                }
            })
            .min()
    };
    match (gap(first), gap(second)) {
        (Some(before), Some(after)) => usize::from(after <= before),
        (Some(_), None) => 0,
        // No cue occurrence governs `first`: the second literal is the anchor,
        // which is also the prepositional default when neither is governed.
        (None, _) => 1,
    }
}


/// A span that is one quoted literal, kept byte for byte: the author who
/// quoted `"  schedule:"` quoted its indentation on purpose (issue #1116).
fn quoted_verbatim(span: &str) -> Option<String> {
    let trimmed = span
        .trim()
        .trim_start_matches([':', '-', '—', '–'])
        .trim();
    let mut chars = trimmed.chars();
    let (first, last) = (chars.next()?, chars.next_back()?);
    (first == last && matches!(first, '"' | '`') && trimmed.len() >= 2)
        .then(|| trimmed[1..trimmed.len() - 1].to_owned())
        .filter(|inner| !inner.contains(first))
}

/// Whether `task` is an instruction to change a named local file: an edit or
/// insert action beside a workspace path. Such a request is never a web
/// question, whatever else the router makes of it (issues #1115, #1133).
#[must_use]
pub(super) fn names_local_edit(task: &str) -> bool {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(task);
    let edits = lexicon.mentions_role(seed::ROLE_FILE_EDIT_ACTION_CUE, &normalized)
        || lexicon.mentions_role(seed::ROLE_CODING_MEMBER_ADD_ACTION, &normalized);
    edits
        && tokens(task).iter().any(|token| {
            let candidate = clean_path_token(token.text);
            looks_like_file_path(candidate) && safe_relative_path(candidate)
        })
}

/// A newline an author spelled as `\n` inside prose means a newline in the
/// file (issue #1116); the edit tool would otherwise write the two characters.
fn unescape_prose_newlines(text: &str) -> String {
    text.replace("\\n", "\n").replace("\\t", "\t")
}
