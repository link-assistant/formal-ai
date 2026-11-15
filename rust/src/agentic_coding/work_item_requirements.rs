//! A work item's text is split into requirements before any evidence is
//! collected, so a refusal or a gap can name the requirement it concerns
//! (issue #1560). The text is read as Markdown blocks: a paragraph, a list
//! item (without its marker) or a fenced block is one candidate; a heading or a
//! rule closes a block and is not a requirement. Each block is then split at
//! enumeration cues by the shared clause splitter. Spans are UTF-8 byte offsets
//! into the whole text.
//! JavaScript twin: js/agentic/work_item_requirements.mjs.

use crate::obligation_ledger::clauses_with_spans;

/// A fence opens with three backticks or three tildes.
fn is_fence(line: &str) -> bool {
    ['`', '~']
        .into_iter()
        .any(|mark| line.chars().take(3).eq(std::iter::repeat_n(mark, 3)))
}

fn is_heading(line: &str) -> bool {
    let hashes = line.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6).contains(&hashes)
        && line[hashes..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
}

fn is_rule(line: &str) -> bool {
    ['-', '*', '_'].into_iter().any(|mark| {
        line.chars().filter(|character| *character == mark).count() >= 3
            && line.starts_with(mark)
            && line
                .chars()
                .all(|character| character == mark || character.is_whitespace())
    })
}

/// The byte length of a list item's leading marker and the blanks around it.
fn list_marker(raw: &str) -> Option<usize> {
    let indent = raw.len() - raw.trim_start_matches([' ', '\t']).len();
    let rest = &raw[indent..];
    let marker = if rest.starts_with(['-', '*', '+']) {
        1
    } else {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if !(1..=9).contains(&digits) || !rest[digits..].starts_with(['.', ')']) {
            return None;
        }
        digits + 1
    };
    let blanks = rest[marker..].len() - rest[marker..].trim_start_matches([' ', '\t']).len();
    (blanks > 0).then_some(indent + marker + blanks)
}

/// `(start, end)` UTF-8 byte ranges of the Markdown blocks.
fn work_item_blocks(text: &str) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let mut fenced = false;
    let mut offset = 0_usize;
    for raw in text.split('\n') {
        let start = offset;
        let end = start + raw.len();
        offset = end + 1;
        let line = raw.trim();
        if fenced {
            if let Some(block) = open.as_mut() {
                block.1 = end;
            }
            if is_fence(line) {
                fenced = false;
            }
            continue;
        }
        if is_fence(line) {
            open = Some((open.map_or(start, |block| block.0), end));
            fenced = true;
            continue;
        }
        if line.is_empty() || is_heading(line) || is_rule(line) {
            blocks.extend(open.take());
            continue;
        }
        // A list item opens its own block after its marker, so `1. First` is
        // not cut at the cue that follows the marker's full stop.
        if let Some(marker) = list_marker(raw) {
            blocks.extend(open.take());
            open = Some((start + marker, end));
        }
        open = Some((open.map_or(start, |block| block.0), end));
    }
    blocks.extend(open);
    blocks
}

/// `(text, (start, end))` in UTF-8 bytes, the same shape as
/// `clauses_with_spans`.
pub(super) fn work_item_requirements(text: &str) -> Vec<(String, (usize, usize))> {
    let mut out = Vec::new();
    for (start, end) in work_item_blocks(text) {
        for (clause, (from, to)) in clauses_with_spans(&text[start..end]) {
            if !clause.is_empty() {
                out.push((clause, (start + from, start + to)));
            }
        }
    }
    if out.is_empty() {
        out.push((text.trim().to_owned(), (0, text.len())));
    }
    out
}

/// The index of the first requirement whose text names `path`.
pub(super) fn requirement_naming(requirements: &[serde_json::Value], path: &str) -> Option<usize> {
    if path.is_empty() {
        return None;
    }
    requirements.iter().position(|requirement| {
        requirement["text"]
            .as_str()
            .is_some_and(|text| text.contains(path))
    })
}
