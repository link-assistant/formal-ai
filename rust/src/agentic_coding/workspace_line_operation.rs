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
/// The seed frames' slots, spelled so no literal reads as a format argument
/// beside a binding of the same name.
const LINES_SLOT: &str = concat!("{", "lines", "}");
const OLD_SLOT: &str = concat!("{", "old", "}");
const ANCHOR_SLOT: &str = concat!("{", "anchor", "}");
const NEW_SLOT: &str = concat!("{", "new", "}");
const LINE_SLOT: &str = concat!("{", "line", "}");
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
    /// Lines `first..=last` (one-based) go; `from_end` counts them from the
    /// last line (1), so `first` is then the earlier line's count.
    RangeRemoval {
        first: usize,
        last: usize,
        from_end: bool,
    },
    /// `text` goes in as a line after (or before) line `line`, counted from
    /// the last line when `from_end`.
    NumberedInsertion {
        line: usize,
        text: String,
        after: bool,
        from_end: bool,
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
pub(super) fn numbered_lines(task: &str) -> Vec<(usize, usize)> {
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

/// A line named by a seeded ordinal right before a seeded line noun.
struct Ordinal {
    /// The one-based count, from the start or (`from_end`) from the end.
    value: usize,
    from_end: bool,
    /// The byte span of the ordinal and its noun in the lowered request.
    start: usize,
    end: usize,
}

/// A letter or digit outside the Han block, which a word boundary may not split.
fn joins_word(character: Option<char>) -> bool {
    character.is_some_and(|character| character.is_alphanumeric() && !is_han(character))
}

/// The one-based number a seeded line ordinal counts to: the digits its
/// cardinal meaning (`defined-by one`) spells.
fn ordinal_value(meaning: &seed::Meaning) -> Option<usize> {
    let lexicon = seed::lexicon();
    meaning.defined_by.iter().find_map(|slug| {
        let cardinal = lexicon.meaning(slug)?;
        if !cardinal.has_role("cardinal_number_word") {
            return None;
        }
        cardinal
            .words()
            .filter(|text| {
                !text.is_empty()
                    && text.len() <= MAX_NUMBER_DIGITS
                    && text.bytes().all(|byte| byte.is_ascii_digit())
            })
            .find_map(|text| text.parse().ok())
    })
}

/// Every line the request names by a seeded ordinal before a line noun.
///
/// `the first line`, `последнюю строку`, `第一行`, `la última línea`; a
/// `line_ordinal_last` ordinal counts from the end (PR #1188 G19).
fn ordinal_lines(lowered: &str) -> Vec<Ordinal> {
    let nouns: Vec<String> = role_words("numbered_line_noun")
        .into_iter()
        .filter(|noun| !noun.is_empty())
        .collect();
    let mut found = Vec::new();
    for (role, from_end) in [("line_ordinal", false), ("line_ordinal_last", true)] {
        for meaning in seed::lexicon().meanings_with_role(role) {
            let Some(value) = ordinal_value(meaning) else {
                continue;
            };
            for surface in meaning.words().map(str::to_lowercase) {
                if surface.is_empty() {
                    continue;
                }
                for (at, _) in lowered.match_indices(surface.as_str()) {
                    if joins_word(lowered[..at].chars().next_back()) {
                        continue;
                    }
                    let rest = &lowered[at + surface.len()..];
                    let gap = rest.len() - rest.trim_start().len();
                    if gap == 0 && joins_word(rest.chars().next()) {
                        continue;
                    }
                    let noun = nouns.iter().find(|noun| {
                        rest[gap..].starts_with(noun.as_str())
                            && !joins_word(rest[gap + noun.len()..].chars().next())
                    });
                    if let Some(noun) = noun {
                        found.push(Ordinal {
                            value,
                            from_end,
                            start: at,
                            end: at + surface.len() + gap + noun.len(),
                        });
                    }
                }
            }
        }
    }
    found
}

/// Line `line` of a file of `count` lines, counted from its end when `from_end`.
const fn resolved_line(line: usize, count: usize, from_end: bool) -> Option<usize> {
    if from_end {
        (count + 1).checked_sub(line)
    } else {
        Some(line)
    }
}

/// The one run of lines a read request names (PR #1188 G33).
///
/// One-based and inclusive, counted from the end when `from_end`: a numbered
/// line or range (`lines 2-3`), a seeded ordinal line (`the last line`), or a
/// count between a seeded head or tail cue and a line noun (`the first 2
/// lines`, `последние 2 строки`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LineSlice {
    pub(super) from: usize,
    pub(super) to: usize,
    pub(super) from_end: bool,
}

/// The [`LineSlice`] `task` names, if exactly one.
pub(super) fn line_slice(task: &str) -> Option<LineSlice> {
    let numbered = numbered_lines(task);
    if let [(from, to)] = numbered.as_slice() {
        return Some(LineSlice {
            from: *from,
            to: *to,
            from_end: false,
        });
    }
    if numbered.len() > 1 {
        return None;
    }
    let outside = text_outside_quoted_segments(task).to_lowercase();
    if let [ordinal] = ordinal_lines(&outside).as_slice() {
        return Some(LineSlice {
            from: ordinal.value,
            to: ordinal.value,
            from_end: ordinal.from_end,
        });
    }
    let pieces = line_pieces(task);
    let nouns = role_words("numbered_line_noun");
    let (heads, tails) = (
        role_words("line_slice_head_cue"),
        role_words("line_slice_tail_cue"),
    );
    let mut slices = Vec::new();
    for (index, piece) in pieces.iter().enumerate().skip(1) {
        let Piece::Number(count) = *piece else {
            continue;
        };
        if count == 0 || !is_word_in(&pieces, index + 1, &nouns) {
            continue;
        }
        if is_word_in(&pieces, index - 1, &heads) {
            slices.push(LineSlice {
                from: 1,
                to: count,
                from_end: false,
            });
        } else if is_word_in(&pieces, index - 1, &tails) {
            slices.push(LineSlice {
                from: count,
                to: 1,
                from_end: true,
            });
        }
    }
    match slices.as_slice() {
        [slice] => Some(*slice),
        _ => None,
    }
}

/// The lines of `source` a [`LineSlice`] names, with the file's own one-based
/// numbers of the first and last, or `None` when the file has none of them.
pub(super) fn sliced_lines(source: &str, slice: LineSlice) -> Option<(usize, usize, String)> {
    let (lines, _) = file_lines(source);
    let count = lines.len();
    let first = if slice.from_end {
        (count + 1).saturating_sub(slice.from).max(1)
    } else {
        slice.from
    };
    let last = count.min(if slice.from_end {
        (count + 1).saturating_sub(slice.to)
    } else {
        slice.to
    });
    if first == 0 || first > last {
        return None;
    }
    Some((first, last, lines[first - 1..last].join("\n")))
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
            Self::RangeRemoval { .. } | Self::NumberedInsertion { .. } => {
                let (first, last) = self.resolved_lines(source)?;
                match self {
                    Self::NumberedInsertion { text, after, .. } => {
                        inserted_at_line(source, first, text, *after)
                    }
                    _ => removed_range(source, first, last),
                }
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
        match self {
            Self::RangeRemoval { from_end: true, .. } => {
                let (first, last) = self.resolved_lines(source)?;
                let lines = if first == last {
                    first.to_string()
                } else {
                    format!("{first}-{last}")
                };
                return Some(("numbered_lines_removed", vec![(LINES_SLOT, lines)]));
            }
            Self::NumberedInsertion {
                text,
                after,
                from_end: true,
                ..
            } => {
                let (line, _) = self.resolved_lines(source)?;
                let intent = if *after {
                    "numbered_line_insert_after"
                } else {
                    "numbered_line_insert_before"
                };
                return Some((
                    intent,
                    vec![(NEW_SLOT, text.clone()), (LINE_SLOT, line.to_string())],
                ));
            }
            _ => {}
        }
        let removed = self.reported_values(source)?;
        Some(("coding_text_remove", vec![(OLD_SLOT, removed.join("\n"))]))
    }

    /// Original removed lines, retained as list values rather than presentation text.
    pub(super) fn reported_values(&self, source: &str) -> Option<Vec<String>> {
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
        Some(
            [one.min(two), one.max(two)]
                .map(|index| bare(lines[index]).to_owned())
                .to_vec(),
        )
    }
}

impl LineOperation {
    /// The one-based lines a numbered operation names in `source`, earlier
    /// first; `None` for any other operation or a count past the file.
    fn resolved_lines(&self, source: &str) -> Option<(usize, usize)> {
        let count = file_lines(source).0.len();
        let (first, last, from_end) = match self {
            Self::RangeRemoval {
                first,
                last,
                from_end,
            } => (*first, *last, *from_end),
            Self::NumberedInsertion { line, from_end, .. } => (*line, *line, *from_end),
            _ => return None,
        };
        Some((
            resolved_line(first, count, from_end)?,
            resolved_line(last, count, from_end)?,
        ))
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
            vec![(OLD_SLOT, first.clone()), (NEW_SLOT, second.clone())],
        );
    }
    if lexicon.mentions_role("line_move_action", &outside) {
        return moved(&payloads, &outside)
            .and_then(|(operation, intent, slots)| change(operation, intent, slots));
    }
    let removing = lexicon.mentions_role("coding_text_remove_action", &outside)
        && !lexicon.mentions_role("coding_member_add_action", &outside);
    // An ordinal's words are not an adjacency (`最后一行` holds `后一行`).
    let mut unordinal = outside_text.to_lowercase();
    let ordinals = ordinal_lines(&unordinal);
    for ordinal in ordinals.iter().rev() {
        unordinal.replace_range(ordinal.start..ordinal.end, " ");
    }
    let above = either_of(
        &sentence_words(&unordinal),
        "line_adjacent_above",
        "line_adjacent_below",
    );
    let mut numbered: Vec<(usize, usize, bool)> = numbered_lines(task)
        .into_iter()
        .map(|(first, last)| (first, last, false))
        .collect();
    numbered.extend(
        ordinals
            .iter()
            .map(|ordinal| (ordinal.value, ordinal.value, ordinal.from_end)),
    );
    match numbered.as_slice() {
        [] => {}
        [(first, last, from_end)] => {
            let from_end = *from_end;
            if removing && payloads.is_empty() {
                // Counted from the end, the earlier line has the larger count.
                let (earlier, later) = if from_end {
                    (*last, *first)
                } else {
                    (*first, *last)
                };
                let from = match (above == Some(true), from_end) {
                    (false, _) => earlier,
                    (true, false) => earlier.saturating_sub(1),
                    (true, true) => earlier + 1,
                };
                let to = match (above == Some(false), from_end) {
                    (false, _) => later,
                    (true, false) => later + 1,
                    (true, true) => later.saturating_sub(1),
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
                        from_end,
                    },
                    "numbered_lines_removed",
                    vec![(LINES_SLOT, lines)],
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
                    from_end,
                },
                if after {
                    "numbered_line_insert_after"
                } else {
                    "numbered_line_insert_before"
                },
                vec![(NEW_SLOT, text), (LINE_SLOT, first.to_string())],
            );
        }
        _ => return None,
    }
    let unpathed = crate::solver_handlers::text_outside_quoted_segments(
        &super::workspace_computed_change::without_path_words(task),
    );
    let names_line = lexicon.meaning("line").is_some_and(|meaning| {
        meaning.evidenced_in(&crate::engine::normalize_prompt(&unpathed).to_lowercase())
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
        vec![(OLD_SLOT, anchor)],
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
                vec![(OLD_SLOT, text.clone())],
            ))
        }
        [text, anchor] => {
            // `so that it follows the line …` states the place by the line it
            // ends up beside (PR #1188 G81).
            let after = either_of(
                outside,
                "file_edit_position_after",
                "file_edit_position_before",
            )
            .or_else(|| either_of(outside, "line_move_after_cue", "line_move_before_cue"))?;
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
                vec![(OLD_SLOT, text.clone()), (ANCHOR_SLOT, anchor.clone())],
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
