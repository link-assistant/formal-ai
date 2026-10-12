//! The line arithmetic of an anchored insert.
//!
//! How often an anchor repeats (PR #1188 G72), the run of whole lines an edit
//! needs to be unique (G73), and the empty line an insert adds beside its
//! lines (G74), and lines listed one by one (G80). Split out of `positional_edit.rs` (the 1000-line Rust
//! ceiling).

use super::{PositionalInsert, QuotedLiteral, joined_only, lone_line_occurrences, quoted_literals};

/// `the three lines 'a', 'b' and 'c'` as one block of those lines (G80).
///
/// Several quoted literals led by words that name lines (the seeded `line`
/// meaning) and joined only by commas and seeded joiners (mirrors
/// `listedLines`).
pub(super) fn listed_lines(span: &str) -> Option<String> {
    let literals = quoted_literals(span);
    let (first, last) = (literals.first()?, literals.last()?);
    let joined: Vec<&QuotedLiteral> = literals.iter().collect();
    if literals.len() < 2 || !joined_only(span, &joined) || !span.get(last.end..)?.trim().is_empty()
    {
        return None;
    }
    let lead = crate::engine::normalize_prompt(span.get(..first.start)?).to_lowercase();
    crate::seed::lexicon()
        .meaning("line")
        .is_some_and(|meaning| meaning.evidenced_in(&lead))
        .then(|| {
            let lines: Vec<&str> = literals
                .iter()
                .map(|literal| literal.text.as_str())
                .collect();
            lines.join("\n")
        })
}

/// How many places a repeated anchor of `insert` could mean, or `None` when it
/// names one place (PR #1188 G72).
pub(super) fn repeated_anchor_count(source: &str, insert: &PositionalInsert) -> Option<usize> {
    let lead = insert.context.as_deref().unwrap_or(&insert.anchor);
    if lead.is_empty() {
        return None;
    }
    let lone = lone_line_occurrences(source, lead).len();
    let count = if lone == 0 {
        source.matches(lead).count()
    } else {
        lone
    };
    (count > 1).then_some(count)
}

/// The lines without the one empty line an insert adds beside them.
pub(super) fn blank_beside(inserted: &str) -> &str {
    if inserted.len() > 1 {
        if let Some(rest) = inserted.strip_prefix('\n') {
            return rest;
        }
        if let Some(rest) = inserted.strip_suffix('\n') {
            return rest;
        }
    }
    inserted
}

/// The start of the shortest run of whole lines ending at `end` whose text
/// occurs once in `source`.
pub(super) fn unique_from(source: &str, start: usize, end: usize) -> Option<usize> {
    let mut from = start;
    while end > from && source.matches(&source[from..end]).count() > 1 {
        if from == 0 {
            return None;
        }
        from = source[..from - 1]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
    }
    Some(from)
}
