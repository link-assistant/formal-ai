//! Inserts placed by unquoted line words (PR #1188 G51).
//!
//! `Insert the contents of rows.txt after the line row a that follows the line
//! table u in f.lino.`: the words after the position cue, up to a seeded
//! context cue or the target clause, are the anchor span, and the words after
//! the context cue the context span. Each span is resolved against the file's
//! lines once it is read. Split out of `positional_edit.rs` (the 1000-line
//! Rust ceiling), as is the reading of a quoted anchor the file holds only as
//! written ([`written_escapes`]).

use super::{CueOccurrence, PositionalInsert};
use crate::agentic_coding::write_request::bare_surfaces;
use crate::seed;

/// The block insert a clause with no quoted literal asks for.
///
/// A postpositional cue is left to quoted anchors.
pub(super) fn unquoted_anchor_insert(
    sentence: &str,
    positions: &[CueOccurrence],
    contexts: &[CueOccurrence],
    target: Option<String>,
    text: &str,
    after: bool,
) -> Option<(Option<String>, PositionalInsert)> {
    let cue = earliest(positions.iter().copied())?;
    if cue.2 {
        return None;
    }
    let context = earliest(
        contexts
            .iter()
            .copied()
            .filter(|&(start, _, postpositional)| start >= cue.1 && !postpositional),
    );
    let end = target
        .as_deref()
        .and_then(|path| sentence.rfind(path))
        .filter(|&at| at >= cue.1)
        .unwrap_or(sentence.len());
    let anchor = without_target_cue(sentence.get(cue.1..context.map_or(end, |found| found.0))?);
    let context = match context {
        Some(found) => Some(without_target_cue(sentence.get(found.1..end)?)),
        None => None,
    };
    if anchor.is_empty() || context.as_deref() == Some("") {
        return None;
    }
    let insert = PositionalInsert {
        target: String::new(),
        anchor,
        inserted: text.to_owned(),
        after,
        context,
        rebase: None,
        spans: true,
    };
    Some((target, insert))
}

/// The cue that starts first, the longest of those that start there.
fn earliest(cues: impl Iterator<Item = CueOccurrence>) -> Option<CueOccurrence> {
    cues.min_by_key(|&(start, end, _)| (start, usize::MAX - end))
}

/// A span without its closing mark and a trailing seeded target cue.
fn without_target_cue(span: &str) -> String {
    let trimmed = span.trim();
    let trimmed = trimmed.strip_suffix([':', '：']).unwrap_or(trimmed);
    let mut words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() > 1
        && words.last().is_some_and(|word| {
            bare_surfaces(seed::ROLE_FILE_EDIT_TARGET_CUE).contains(&word.to_lowercase())
        })
    {
        words.pop();
    }
    words.join(" ")
}

/// The insert with its unquoted spans resolved to lines of `source`.
///
/// Each span becomes the longest line it ends with (`the line table u` ends
/// with the line `table u`); `None` when a span names no line.
pub(super) fn resolved_spans(source: &str, insert: &PositionalInsert) -> Option<PositionalInsert> {
    let mut lines: Vec<&str> = Vec::new();
    for line in source.split('\n').map(str::trim) {
        if !line.is_empty() && !lines.contains(&line) {
            lines.push(line);
        }
    }
    let resolve = |span: &str| {
        lines
            .iter()
            .filter(|line| span == **line || span.ends_with(&[" ", line].concat()))
            .fold(None, |best: Option<&str>, line| match best {
                Some(found) if found.len() >= line.len() => Some(found),
                _ => Some(*line),
            })
            .map(str::to_owned)
    };
    let anchor = resolve(&insert.anchor)?;
    let context = match insert.context.as_deref() {
        Some(span) => Some(resolve(span)?),
        None => None,
    };
    Some(PositionalInsert {
        anchor,
        context,
        spans: false,
        ..insert.clone()
    })
}

/// The insert with an anchor or context the file holds only as written, or
/// `None` when nothing changes.
///
/// A quoted `\n` or `\t` reads as a line break or a tab where the file holds
/// it so, and as its two characters where the file holds those inside a line
/// of code (`split_inclusive('\n')`, PR #1188 G96). Mirrored by
/// `writtenEscapes` in js/agentic/workspace_change.mjs.
pub(super) fn written_escapes(source: &str, insert: &PositionalInsert) -> Option<PositionalInsert> {
    let written = |text: &str| {
        let raw = text.replace('\n', "\\n").replace('\t', "\\t");
        (!source.contains(text) && raw != text && source.contains(&raw)).then_some(raw)
    };
    let anchor = written(&insert.anchor);
    let context = insert.context.as_deref().and_then(written);
    if anchor.is_none() && context.is_none() {
        return None;
    }
    Some(PositionalInsert {
        anchor: anchor.unwrap_or_else(|| insert.anchor.clone()),
        context: context.or_else(|| insert.context.clone()),
        ..insert.clone()
    })
}
