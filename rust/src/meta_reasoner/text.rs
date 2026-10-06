//! Request reading.
//!
//! Words, value literals, examples and the definitions a request gives itself.
//! JavaScript's regular expressions are written out as scanners here (the crate
//! carries no regex engine), each following the backtracking order of the
//! pattern it mirrors.

use core::cmp::Ordering;

use unicode_general_category::{GeneralCategory, get_general_category};

use super::seed::{MetaSeed, meta_seed};
use super::value::Value;

/// ECMAScript `WhiteSpace` and `LineTerminator`, what `String.prototype.trim`
/// removes.
///
/// Mirrors `trim()` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub const fn is_js_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{9}' | '\u{a}' | '\u{b}' | '\u{c}' | '\u{d}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// `String.prototype.trim`.
///
/// Mirrors `trim()` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn js_trim(text: &str) -> &str {
    text.trim_matches(is_js_whitespace)
}

/// The length of a string in UTF-16 code units, JavaScript's `length`.
///
/// Mirrors `String.prototype.length` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// `\p{L}`: a letter of any script.
#[must_use]
pub fn is_letter(character: char) -> bool {
    matches!(
        get_general_category(character),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}

/// `\p{N}`: a number of any script.
#[must_use]
pub fn is_number(character: char) -> bool {
    matches!(
        get_general_category(character),
        GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
    )
}

fn is_letter_or_number(character: char) -> bool {
    is_letter(character) || is_number(character)
}

/// Lowercased word tokens of a text: maximal runs of `[\p{L}\p{N}_]`.
///
/// Mirrors `metaWords` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_words(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase();
    let mut out = Vec::new();
    let mut current = String::new();
    for character in lowered.chars() {
        if is_letter_or_number(character) || character == '_' {
            current.push(character);
        } else if !current.is_empty() {
            out.push(core::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// True when a word is digits only (`/^\p{N}+$/u`).
///
/// Mirrors the numeral test in `metaIsGrammatical`
/// (`js/worker/formal_ai_worker_meta_reasoner.js`).
#[must_use]
pub fn is_numeral(word: &str) -> bool {
    !word.is_empty() && word.chars().all(is_number)
}

/// What a value literal of the request is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    /// A quoted string, a list or a number.
    Value,
    /// A slash-separated path.
    Path,
}

/// One value literal of a request with its byte span.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    /// Byte offset where the literal starts.
    pub start: usize,
    /// Byte offset just past the literal.
    pub end: usize,
    /// The literal's value.
    pub value: Value,
    /// Whether it was written as a path.
    pub kind: LiteralKind,
}

/// One input/output example of a request.
#[derive(Debug, Clone, PartialEq)]
pub struct Example {
    /// The example's input value.
    pub input: Value,
    /// The output the request expects for it.
    pub output: Value,
    /// True when an unambiguous symbol (an arrow) joined the pair; a word
    /// marker ("into", "в") may be prose.
    pub symbolic: bool,
}

/// A definition the request gives itself: `<term> is <definition>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    /// The defined word.
    pub term: String,
    /// The text that defines it.
    pub definition: String,
}

enum Scanned {
    Found(usize, Value, LiteralKind),
    Skipped(usize),
}

struct Scanner<'a> {
    chars: &'a [(usize, char)],
}

impl Scanner<'_> {
    fn at(&self, index: usize) -> Option<char> {
        self.chars.get(index).map(|(_, character)| *character)
    }

    fn is(&self, index: usize, wanted: char) -> bool {
        self.at(index) == Some(wanted)
    }

    /// `X([^X\n]*)Y`: the index of the closing delimiter.
    fn closing(&self, open: usize, close: char) -> Option<usize> {
        let mut index = open + 1;
        while let Some(character) = self.at(index) {
            if character == close {
                return Some(index);
            }
            if character == '\n' {
                return None;
            }
            index += 1;
        }
        None
    }

    fn run_of(&self, start: usize, member: fn(char) -> bool) -> usize {
        let mut index = start;
        while self.at(index).is_some_and(member) {
            index += 1;
        }
        index - start
    }

    fn path_lookahead(&self, index: usize) -> bool {
        self.at(index).is_none_or(|character| {
            !(is_letter_or_number(character) || character == '_' || character == '/')
        })
    }

    fn path_tail(&self, position: usize) -> Option<usize> {
        if self.is(position, '/') && self.path_lookahead(position + 1) {
            return Some(position + 1);
        }
        self.path_lookahead(position).then_some(position)
    }

    fn path_repetitions(&self, position: usize, count: usize) -> Option<usize> {
        if self.is(position, '/') {
            let run = self.run_of(position + 1, is_path_character);
            for length in (1..=run).rev() {
                if let Some(end) = self.path_repetitions(position + 1 + length, count + 1) {
                    return Some(end);
                }
            }
        }
        if count >= 1 {
            self.path_tail(position)
        } else {
            None
        }
    }

    /// `(?:\.{1,2}\/|\/)?[\p{L}\p{N}_.-]+(?:\/[\p{L}\p{N}_.-]+)+\/?(?![\p{L}\p{N}_/])`
    /// in backtracking order.
    fn path(&self, start: usize) -> Option<usize> {
        let mut prefixes = Vec::new();
        if self.is(start, '.') && self.is(start + 1, '.') && self.is(start + 2, '/') {
            prefixes.push(3);
        }
        if self.is(start, '.') && self.is(start + 1, '/') {
            prefixes.push(2);
        }
        if self.is(start, '/') {
            prefixes.push(1);
        }
        prefixes.push(0);
        for prefix in prefixes {
            let first = start + prefix;
            let run = self.run_of(first, is_path_character);
            for length in (1..=run).rev() {
                if let Some(end) = self.path_repetitions(first + length, 0) {
                    return Some(end);
                }
            }
        }
        None
    }

    /// `-?\d+(?:\.\d+)?(?![\p{L}\p{N}])` in backtracking order.
    fn number(&self, start: usize) -> Option<usize> {
        let digits_start = if self.is(start, '-') {
            start + 1
        } else {
            start
        };
        let whole = self.run_of(digits_start, |character| character.is_ascii_digit());
        if whole == 0 {
            return None;
        }
        let whole_end = digits_start + whole;
        let mut candidates = Vec::new();
        if self.is(whole_end, '.') {
            let fraction = self.run_of(whole_end + 1, |character| character.is_ascii_digit());
            if fraction > 0 {
                candidates.push(whole_end + 1 + fraction);
            }
        }
        candidates.push(whole_end);
        candidates.into_iter().find(|end| {
            self.at(*end)
                .is_none_or(|character| !is_letter_or_number(character))
        })
    }

    fn text_between(&self, prompt: &str, from: usize, to: usize) -> String {
        let start = self.byte(prompt, from);
        let end = self.byte(prompt, to);
        prompt[start..end].to_owned()
    }

    fn byte(&self, prompt: &str, index: usize) -> usize {
        self.chars
            .get(index)
            .map_or(prompt.len(), |(byte, _)| *byte)
    }

    fn scan_at(&self, prompt: &str, seed: &MetaSeed, index: usize) -> Option<Scanned> {
        let character = self.at(index)?;
        let previous = index.checked_sub(1).and_then(|before| self.at(before));
        let quoted = |close: char| {
            self.closing(index, close).map(|end| {
                Scanned::Found(
                    end + 1,
                    Value::Text(self.text_between(prompt, index + 1, end)),
                    LiteralKind::Value,
                )
            })
        };
        match character {
            '\'' => {
                if previous.is_some_and(is_letter_or_number) {
                    return None;
                }
                let end = self.closing(index, '\'')?;
                if self.at(end + 1).is_some_and(is_letter_or_number) {
                    return None;
                }
                Some(Scanned::Found(
                    end + 1,
                    Value::Text(self.text_between(prompt, index + 1, end)),
                    LiteralKind::Value,
                ))
            }
            '"' => quoted('"'),
            '\u{201c}' => quoted('\u{201d}'),
            '\u{ab}' => quoted('\u{bb}'),
            '[' => {
                let end = self.closing(index, ']')?;
                let raw = self
                    .text_between(prompt, index, end + 1)
                    .replace('\'', "\"");
                Some(
                    Value::parse_json(&raw).map_or(Scanned::Skipped(end + 1), |value| {
                        Scanned::Found(end + 1, value, LiteralKind::Value)
                    }),
                )
            }
            _ => {
                let path_allowed = !previous.is_some_and(|before| {
                    is_letter_or_number(before) || matches!(before, '_' | '.' | '/' | '-')
                });
                if path_allowed && let Some(end) = self.path(index) {
                    let text = self.text_between(prompt, index, end);
                    let grammatical_only = text
                        .split('/')
                        .filter(|segment| !segment.is_empty())
                        .all(|segment| seed.is_grammatical(&segment.to_lowercase()));
                    if grammatical_only {
                        return Some(Scanned::Skipped(end));
                    }
                    return Some(Scanned::Found(end, Value::Path(text), LiteralKind::Path));
                }
                let number_allowed =
                    !previous.is_some_and(|before| is_letter_or_number(before) || before == '.');
                if !number_allowed {
                    return None;
                }
                let end = self.number(index)?;
                let text = self.text_between(prompt, index, end);
                Some(Scanned::Found(
                    end,
                    Value::Number(text.parse::<f64>().unwrap_or(f64::NAN)),
                    LiteralKind::Value,
                ))
            }
        }
    }
}

fn is_path_character(character: char) -> bool {
    is_letter_or_number(character) || matches!(character, '_' | '.' | '-')
}

/// The value literals of a request with their positions: quoted strings,
/// bracketed lists, paths and bare numbers.
///
/// Mirrors `metaValueLiterals` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_value_literals(prompt: &str) -> Vec<Literal> {
    let seed = meta_seed();
    let chars: Vec<(usize, char)> = prompt.char_indices().collect();
    let scanner = Scanner { chars: &chars };
    let mut out = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        match scanner.scan_at(prompt, seed, index) {
            Some(Scanned::Found(end, value, kind)) => {
                out.push(Literal {
                    start: scanner.byte(prompt, index),
                    end: scanner.byte(prompt, end),
                    value,
                    kind,
                });
                index = end.max(index + 1);
            }
            Some(Scanned::Skipped(end)) => index = end.max(index + 1),
            None => index += 1,
        }
    }
    out
}

/// Input/output examples: two adjacent values joined by an example marker.
///
/// Mirrors `metaExamples` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_examples(prompt: &str, literals: &[Literal]) -> Vec<Example> {
    let markers = meta_seed().cue_markers("example");
    let mut examples = Vec::new();
    let mut index = 0;
    while index + 1 < literals.len() {
        let gap = &prompt[literals[index].end..literals[index + 1].start];
        let between = [" ", &gap.to_lowercase(), " "].concat();
        let marker = markers
            .iter()
            .find(|marker| between.contains(marker.as_str()));
        if utf16_len(&between) <= 28
            && let Some(marker) = marker
        {
            examples.push(Example {
                input: literals[index].value.clone(),
                output: literals[index + 1].value.clone(),
                symbolic: !marker.chars().any(is_letter),
            });
            index += 1;
        }
        index += 1;
    }
    examples
}

/// The text of `text` from the `from`-th character on, or up to it.
fn chars_from(text: &str, from: usize) -> &str {
    text.char_indices()
        .nth(from)
        .map_or("", |(byte, _)| &text[byte..])
}

fn chars_until(text: &str, until: usize) -> &str {
    text.char_indices()
        .nth(until)
        .map_or(text, |(byte, _)| &text[..byte])
}

/// True when a word is a question word: a surface of the seed lexicon's
/// `interrogative_opener` meaning, in any language.
///
/// Mirrors `metaIsInterrogative` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn is_interrogative(word: &str) -> bool {
    crate::seed::lexicon()
        .words_for_role(crate::seed::ROLE_INTERROGATIVE_OPENER)
        .iter()
        .any(|surface| surface == word)
}

/// Definitions the request gives itself: `<term> is <definition>`.
///
/// Mirrors `metaRequestDefinitions` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_request_definitions(prompt: &str) -> Vec<Definition> {
    let seed = meta_seed();
    let markers = seed.cue_markers("definition");
    let mut out = Vec::new();
    for sentence in prompt.split(['.', ';', '!', '?', '\n']) {
        let padded = [" ", js_trim(sentence), " "].concat();
        let lowered = padded.to_lowercase();
        for marker in &markers {
            let Some(at) = lowered.find(marker.as_str()) else {
                continue;
            };
            let at_chars = lowered[..at].chars().count();
            let head: Vec<String> = meta_words(chars_until(&padded, at_chars))
                .into_iter()
                .filter(|word| !seed.is_grammatical(word))
                .collect();
            let rest = chars_from(&padded, at_chars + marker.chars().count());
            let definition = js_trim(rest);
            // A question word ("what is ...") asks for the unknown; it is
            // never a term the request defines.
            if head.len() == 1 && !definition.is_empty() && !is_interrogative(&head[0]) {
                out.push(Definition {
                    term: head[0].clone(),
                    definition: definition.to_owned(),
                });
            }
            break;
        }
    }
    out
}

fn collation_class(character: char) -> u8 {
    if is_letter(character) {
        2
    } else {
        u8::from(is_number(character))
    }
}

/// `String.prototype.localeCompare` for identifiers and words: punctuation
/// before digits before letters, letters case-insensitively first, then by
/// code point.
///
/// Mirrors `localeCompare` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn locale_compare(left: &str, right: &str) -> Ordering {
    let key = |text: &str| -> Vec<(u8, String)> {
        text.chars()
            .map(|character| {
                (
                    collation_class(character),
                    character.to_lowercase().collect(),
                )
            })
            .collect()
    };
    key(left).cmp(&key(right)).then_with(|| left.cmp(right))
}
