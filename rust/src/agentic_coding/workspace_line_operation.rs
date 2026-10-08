//! Whole-line operations a request states by position, adjacency or exchange.
//!
//! PR #1188 dogfooding: `Delete lines 266-267 from f.lino.` only read the
//! file (T30), `Insert the line '…' after line 12 in f.lino.` planned nothing
//! (T33), `delete the line 'A' and the 'B' line directly above it` was routed
//! to a whole-file write of a fragment (T29), `Move the line 'x' to the top
//! of f` duplicated the line and `Swap the lines 'a' and 'b' in f` planned
//! nothing. Each is now a computed change over the file's lines (read,
//! compute, the smallest unique edit, the digest check, the seeded sentence),
//! run by [`super::workspace_computed_change`]. Every word that carries the
//! operation is seeded (`numbered_line_noun`, `numbered_line_lead`,
//! `line_range_connector`, `line_adjacent_above`, `line_adjacent_below`,
//! `line_move_action`, `line_swap_action`) in every registered language.
//! Mirrors `js/agentic/workspace_line_operation.mjs`.

use super::positional_edit::unescape_prose_newlines;
use super::workspace_computed_change::sentence_words;
use super::write_request::{clean_path_token, looks_like_file_path, safe_relative_path, tokens};
use crate::normal_markov::quoted_segment_spans;
use crate::seed;
use crate::solver_handlers::text_outside_quoted_segments;

/// A digit run longer than this is not a line number.
const MAX_NUMBER_DIGITS: usize = 9;
/// A file below this many lines is too short for "most of it" to mean anything.
const MIN_GUARDED_LINES: usize = 8;
const DASHES: [char; 3] = ['-', '\u{2013}', '\u{2014}'];
/// Punctuation beyond ASCII that separates the pieces of a request.
const WIDE_PUNCTUATION: [char; 15] = [
    '\u{ab}', '\u{bb}', '\u{964}', '\u{965}', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}',
    '\u{3001}', '\u{3002}', '\u{ff01}', '\u{ff0c}', '\u{ff1a}', '\u{ff1b}', '\u{ff1f}',
];

/// Where a moved line goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Destination {
    Start,
    End,
    After,
    Before,
}

/// A whole-line operation and the operands it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LineOperation {
    /// Lines `first..=last` (one-based) go.
    RangeRemoval { first: usize, last: usize },
    /// `text` goes in as a line after (or before) line `line`.
    NumberedInsertion {
        line: usize,
        text: String,
        after: bool,
    },
    /// The line `anchor` names and the line directly above (or below) it go;
    /// a named `neighbour` must be that line.
    AdjacentRemoval {
        anchor: String,
        neighbour: Option<String>,
        above: bool,
    },
    /// The line `text` names moves to `destination` (beside `anchor`).
    Move {
        text: String,
        destination: Destination,
        anchor: Option<String>,
    },
    /// The two named lines exchange places.
    Swap { first: String, second: String },
}

/// A line operation on one file, with the seed sentence that states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LineChange {
    pub(super) target: String,
    pub(super) operation: LineOperation,
    pub(super) intent: &'static str,
    pub(super) slots: Slots,
}

/// The slots of a seed sentence.
type Slots = Vec<(&'static str, String)>;

/// One piece of a request outside its quotes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    Path,
    Word(String),
    Number(usize),
    Dash,
}

/// What a request character contributes to a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Word,
    Number,
    Dash,
    Han,
}

/// The one workspace path the request names and its quoted payloads.
///
/// A path the request leaves unquoted is the file; only when it names none
/// does a quoted literal that is exactly one path name it, so `Delete the
/// lines containing 'src/x.rs' from list.txt` edits `list.txt` (PR #1188 T35).
pub(super) fn named_target_and_payloads(task: &str) -> Option<(String, Vec<String>)> {
    let segments = quoted_segment_spans(task);
    let is_path = |path: &str| looks_like_file_path(path) && safe_relative_path(path);
    let mut unquoted: Vec<&str> = Vec::new();
    for token in tokens(task) {
        if segments
            .iter()
            .any(|segment| token.start < segment.end && token.end > segment.start)
        {
            continue;
        }
        let path = clean_path_token(token.text);
        if is_path(path) && !unquoted.contains(&path) {
            unquoted.push(path);
        }
    }
    let mut quoted: Vec<&str> = Vec::new();
    for segment in &segments {
        if is_path(&segment.text) && !quoted.contains(&segment.text.as_str()) {
            quoted.push(&segment.text);
        }
    }
    let target = match (unquoted.as_slice(), quoted.as_slice()) {
        ([path], _) | ([], [path]) => (*path).to_owned(),
        _ => return None,
    };
    let payloads = segments
        .iter()
        .map(|segment| segment.text.clone())
        .filter(|text| *text != target && !text.trim().is_empty())
        .collect();
    Some((target, payloads))
}

/// One Han ideograph, a word of its own.
const fn is_han(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
    )
}

fn is_ascii_printable(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
}

/// The request outside its quotes as words, numbers and dashes. A path is one
/// opaque piece; a Han ideograph is a word of its own.
fn line_pieces(task: &str) -> Vec<Piece> {
    let outside = text_outside_quoted_segments(task);
    let mut pieces = Vec::new();
    for token in tokens(&outside) {
        let path = clean_path_token(token.text);
        if is_ascii_printable(path) && looks_like_file_path(path) {
            pieces.push(Piece::Path);
            continue;
        }
        let mut run = String::new();
        let mut kind: Option<Run> = None;
        for character in token.text.chars() {
            let next = if character.is_ascii_digit() {
                Some(Run::Number)
            } else if DASHES.contains(&character) {
                Some(Run::Dash)
            } else if is_han(character) {
                Some(Run::Han)
            } else if character.is_ascii_punctuation() || WIDE_PUNCTUATION.contains(&character) {
                None
            } else {
                Some(Run::Word)
            };
            if next != kind || matches!(next, Some(Run::Dash | Run::Han)) {
                flush_piece(&mut pieces, &mut run, kind);
            }
            if let Some(next) = next {
                kind = Some(next);
                run.push(character);
            } else {
                kind = None;
            }
        }
        flush_piece(&mut pieces, &mut run, kind);
    }
    pieces
}

fn flush_piece(pieces: &mut Vec<Piece>, run: &mut String, kind: Option<Run>) {
    if run.is_empty() {
        return;
    }
    let piece = match kind {
        Some(Run::Dash) => Piece::Dash,
        Some(Run::Number) if run.len() <= MAX_NUMBER_DIGITS => run
            .parse()
            .map_or_else(|_| Piece::Word(run.clone()), Piece::Number),
        _ => Piece::Word(run.to_lowercase()),
    };
    pieces.push(piece);
    run.clear();
}

/// Whether the piece at `index` is one of `words`.
fn is_word_in(pieces: &[Piece], index: usize, words: &[String]) -> bool {
    matches!(pieces.get(index), Some(Piece::Word(word)) if words.contains(word))
}

fn role_words(role: &str) -> Vec<String> {
    seed::lexicon()
        .words_for_role(role)
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect()
}

/// Every line number or range the request names, one-based and inclusive.
///
/// A seeded `numbered_line_noun` comes before the number (`line 12`, `lines
/// 3-5`, `lines from 3 to 5`, `строки с 3 по 5`), or, after a seeded lead,
/// after it (`第3到5行`).
fn numbered_lines(task: &str) -> Vec<(usize, usize)> {
    let pieces = line_pieces(task);
    let nouns = role_words("numbered_line_noun");
    let leads = role_words("numbered_line_lead");
    let connectors = role_words("line_range_connector");
    let word_in = |index: usize, words: &[String]| is_word_in(&pieces, index, words);
    let number_at = |index: usize| match pieces.get(index) {
        Some(Piece::Number(number)) => Some(*number),
        _ => None,
    };
    let mut found = Vec::new();
    let mut index = 0;
    while index < pieces.len() {
        let Some(first) = number_at(index) else {
            index += 1;
            continue;
        };
        let mut end = index;
        let joint = index + 1;
        if matches!(pieces.get(joint), Some(Piece::Dash)) || word_in(joint, &connectors) {
            let second = if word_in(joint + 1, &leads) {
                joint + 2
            } else {
                joint + 1
            };
            if number_at(second).is_some() {
                end = second;
            }
        }
        let mut led = 0;
        while led < 2 && index > led && word_in(index - 1 - led, &leads) {
            led += 1;
        }
        let noun_before = index
            .checked_sub(1 + led)
            .is_some_and(|noun| word_in(noun, &nouns));
        if noun_before || (led > 0 && word_in(end + 1, &nouns)) {
            found.push((first, number_at(end).unwrap_or(first)));
            index = end + 1;
        } else {
            index += 1;
        }
    }
    found
}

/// The file's lines without their `\n`, and whether it ended in one.
fn file_lines(source: &str) -> (Vec<&str>, bool) {
    let trailing = source.ends_with('\n');
    let body = source.strip_suffix('\n').unwrap_or(source);
    let lines = if source.is_empty() {
        Vec::new()
    } else {
        body.split('\n').collect()
    };
    (lines, trailing)
}

fn joined_lines(lines: &[&str], trailing: bool) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut out = lines.join("\n");
    if trailing {
        out.push('\n');
    }
    out
}

fn bare(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

/// The one line that is exactly `text`, else the one line containing it;
/// `None` when there are none or several.
fn unique_line(lines: &[&str], text: &str) -> Option<usize> {
    let exact: Vec<usize> = (0..lines.len())
        .filter(|index| bare(lines[*index]) == text)
        .collect();
    if !exact.is_empty() {
        return only(&exact);
    }
    let containing: Vec<usize> = (0..lines.len())
        .filter(|index| bare(lines[*index]).contains(text))
        .collect();
    only(&containing)
}

const fn only(indices: &[usize]) -> Option<usize> {
    match indices {
        [index] => Some(*index),
        _ => None,
    }
}

/// Lines `first..=last` (one-based) gone.
fn removed_range(source: &str, first: usize, last: usize) -> Option<String> {
    let (lines, trailing) = file_lines(source);
    if first < 1 || last < first || last > lines.len() {
        return None;
    }
    let kept: Vec<&str> = [&lines[..first - 1], &lines[last..]].concat();
    Some(joined_lines(&kept, trailing))
}

/// `text` as a line after (or before) line `line`.
fn inserted_at_line(source: &str, line: usize, text: &str, after: bool) -> Option<String> {
    let (lines, trailing) = file_lines(source);
    if line < 1 || line > lines.len() {
        return None;
    }
    let at = if after { line } else { line - 1 };
    let inserted: Vec<&str> = text.split('\n').collect();
    let out: Vec<&str> = [&lines[..at], &inserted[..], &lines[at..]].concat();
    Some(joined_lines(&out, trailing))
}

/// The line `anchor` names and the line directly above (or below) it, when
/// exactly one such pair exists and the neighbour, if named, is that line.
fn adjacent_pair(
    lines: &[&str],
    anchor: &str,
    neighbour: Option<&str>,
    above: bool,
) -> Option<(usize, usize)> {
    let fits = |index: usize| {
        let line = bare(lines[index]);
        neighbour.is_none_or(|neighbour| {
            line == neighbour || line.trim() == neighbour.trim() || line.contains(neighbour)
        })
    };
    let pairs = |exact: bool| -> Vec<(usize, usize)> {
        (0..lines.len())
            .filter_map(|index| {
                let line = bare(lines[index]);
                let matches = if exact {
                    line == anchor
                } else {
                    line.contains(anchor)
                };
                let other = if above {
                    index.checked_sub(1)?
                } else {
                    index + 1
                };
                (matches && other < lines.len() && fits(other)).then_some((index, other))
            })
            .collect()
    };
    let mut found = pairs(true);
    if found.is_empty() {
        found = pairs(false);
    }
    match found.as_slice() {
        [pair] => Some(*pair),
        _ => None,
    }
}

fn removed_adjacent(
    source: &str,
    anchor: &str,
    neighbour: Option<&str>,
    above: bool,
) -> Option<String> {
    let (lines, trailing) = file_lines(source);
    let (one, two) = adjacent_pair(&lines, anchor, neighbour, above)?;
    let kept: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != one && *index != two)
        .map(|(_, line)| *line)
        .collect();
    Some(joined_lines(&kept, trailing))
}

/// The line `text` names taken out and put back at the start, the end, or
/// beside the line `anchor` names among the rest.
fn moved_line(
    source: &str,
    text: &str,
    destination: Destination,
    anchor: Option<&str>,
) -> Option<String> {
    let (lines, trailing) = file_lines(source);
    let from = unique_line(&lines, text)?;
    let mut rest = lines;
    let moved = rest.remove(from);
    let at = match destination {
        Destination::Start => 0,
        Destination::End => rest.len(),
        Destination::After => unique_line(&rest, anchor?)? + 1,
        Destination::Before => unique_line(&rest, anchor?)?,
    };
    rest.insert(at, moved);
    Some(joined_lines(&rest, trailing))
}

/// The two named lines exchanged.
fn swapped_lines(source: &str, first: &str, second: &str) -> Option<String> {
    let (mut lines, trailing) = file_lines(source);
    let one = unique_line(&lines, first)?;
    let two = unique_line(&lines, second)?;
    if one == two {
        return None;
    }
    lines.swap(one, two);
    Some(joined_lines(&lines, trailing))
}

impl LineOperation {
    /// The file after the operation, or `None` when it cannot apply.
    pub(super) fn compute(&self, source: &str) -> Option<String> {
        match self {
            Self::RangeRemoval { first, last } => removed_range(source, *first, *last),
            Self::NumberedInsertion { line, text, after } => {
                inserted_at_line(source, *line, text, *after)
            }
            Self::AdjacentRemoval {
                anchor,
                neighbour,
                above,
            } => removed_adjacent(source, anchor, neighbour.as_deref(), *above),
            Self::Move {
                text,
                destination,
                anchor,
            } => moved_line(source, text, *destination, anchor.as_deref()),
            Self::Swap { first, second } => swapped_lines(source, first, second),
        }
    }

    /// Whether the request states the operation's whole extent (a numbered
    /// range), so it may remove most of a file.
    pub(super) const fn bounded(&self) -> bool {
        matches!(self, Self::RangeRemoval { .. })
    }

    /// The seed sentence for the lines actually removed, when the operation
    /// reports them from the file rather than from the request.
    pub(super) fn reported(&self, source: &str) -> Option<(&'static str, Slots)> {
        let Self::AdjacentRemoval {
            anchor,
            neighbour,
            above,
        } = self
        else {
            return None;
        };
        let (lines, _) = file_lines(source);
        let (one, two) = adjacent_pair(&lines, anchor, neighbour.as_deref(), *above)?;
        let removed = [bare(lines[one.min(two)]), bare(lines[one.max(two)])];
        Some(("coding_text_remove", vec![("{old}", removed.join("`, `"))]))
    }
}

/// Exactly one of the two seeded meanings: `Some(true)` for the first.
fn either_of(outside: &str, first: &str, second: &str) -> Option<bool> {
    let lexicon = seed::lexicon();
    let one = lexicon.mentions_role(first, outside);
    (one != lexicon.mentions_role(second, outside)).then_some(one)
}

/// The whole-line operation `task` asks for, if any.
pub(super) fn grounded_line_operation(task: &str) -> Option<LineChange> {
    let (target, payloads) = named_target_and_payloads(task)?;
    let outside_text = text_outside_quoted_segments(task);
    let outside = sentence_words(&outside_text);
    let lexicon = seed::lexicon();
    let change = |operation, intent, slots| {
        Some(LineChange {
            target: target.clone(),
            operation,
            intent,
            slots,
        })
    };
    if lexicon.mentions_role("line_swap_action", &outside) {
        let [first, second] = payloads.as_slice() else {
            return None;
        };
        return change(
            LineOperation::Swap {
                first: first.clone(),
                second: second.clone(),
            },
            "lines_swapped",
            vec![("{old}", first.clone()), ("{new}", second.clone())],
        );
    }
    if lexicon.mentions_role("line_move_action", &outside) {
        return moved(&payloads, &outside)
            .and_then(|(operation, intent, slots)| change(operation, intent, slots));
    }
    let removing = lexicon.mentions_role("coding_text_remove_action", &outside)
        && !lexicon.mentions_role("coding_member_add_action", &outside);
    let above = either_of(&outside, "line_adjacent_above", "line_adjacent_below");
    let numbered = numbered_lines(task);
    match numbered.as_slice() {
        [] => {}
        [(first, last)] => {
            if removing && payloads.is_empty() {
                let from = if above == Some(true) {
                    first.saturating_sub(1)
                } else {
                    *first
                };
                let to = if above == Some(false) {
                    last + 1
                } else {
                    *last
                };
                let lines = if from == to {
                    from.to_string()
                } else {
                    format!("{from}-{to}")
                };
                return change(
                    LineOperation::RangeRemoval {
                        first: from,
                        last: to,
                    },
                    "numbered_lines_removed",
                    vec![("{lines}", lines)],
                );
            }
            let after = either_of(
                &outside,
                "file_edit_position_after",
                "file_edit_position_before",
            )?;
            let [payload] = payloads.as_slice() else {
                return None;
            };
            if removing || first != last {
                return None;
            }
            let text = unescape_prose_newlines(payload);
            return change(
                LineOperation::NumberedInsertion {
                    line: *first,
                    text: text.clone(),
                    after,
                },
                if after {
                    "numbered_line_insert_after"
                } else {
                    "numbered_line_insert_before"
                },
                vec![("{new}", text), ("{line}", first.to_string())],
            );
        }
        _ => return None,
    }
    let names_line = lexicon.meaning("line").is_some_and(|meaning| {
        meaning.evidenced_in(&crate::engine::normalize_prompt(&outside_text).to_lowercase())
    });
    let above = above?;
    if !removing || !names_line {
        return None;
    }
    let (anchor, neighbour) = match payloads.as_slice() {
        [anchor] => (anchor.clone(), None),
        [anchor, neighbour] => (anchor.clone(), Some(neighbour.clone())),
        _ => return None,
    };
    change(
        LineOperation::AdjacentRemoval {
            anchor: anchor.clone(),
            neighbour,
            above,
        },
        "coding_text_remove",
        vec![("{old}", anchor)],
    )
}

/// A move: one quoted line to the start or the end, or two (the line, then
/// its anchor) with an after/before position.
fn moved(payloads: &[String], outside: &str) -> Option<(LineOperation, &'static str, Slots)> {
    match payloads {
        [text] => {
            let at_start = either_of(
                outside,
                "file_edit_position_start",
                "file_edit_position_end",
            )?;
            Some((
                LineOperation::Move {
                    text: text.clone(),
                    destination: if at_start {
                        Destination::Start
                    } else {
                        Destination::End
                    },
                    anchor: None,
                },
                if at_start {
                    "line_moved_start"
                } else {
                    "line_moved_end"
                },
                vec![("{old}", text.clone())],
            ))
        }
        [text, anchor] => {
            let after = either_of(
                outside,
                "file_edit_position_after",
                "file_edit_position_before",
            )?;
            Some((
                LineOperation::Move {
                    text: text.clone(),
                    destination: if after {
                        Destination::After
                    } else {
                        Destination::Before
                    },
                    anchor: Some(anchor.clone()),
                },
                if after {
                    "line_moved_after"
                } else {
                    "line_moved_before"
                },
                vec![("{old}", text.clone()), ("{anchor}", anchor.clone())],
            ))
        }
        _ => None,
    }
}

/// Whether `updated` keeps fewer than half of the lines of a file that had at
/// least [`MIN_GUARDED_LINES`].
///
/// A computed change that does so without the request stating that extent is
/// refused (PR #1188 T29: a 380-line file was once replaced by one line).
pub(super) fn drops_most_of_file(source: &str, updated: &str) -> bool {
    let before = file_lines(source).0.len();
    before >= MIN_GUARDED_LINES && file_lines(updated).0.len() * 2 < before
}
