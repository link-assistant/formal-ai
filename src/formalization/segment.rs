//! Script-aware sentence and clause segmentation with exact character spans
//! (issue #1138, plan 04 L2).
//!
//! Terminators are declared in the seed, never as literals in Rust, and a
//! segment's span selects exactly its own text — the defect recorded at
//! `docs/case-studies/issue-710/plans/07:371-377` (a span that swallowed the
//! preceding separator whitespace) is fixed here rather than documented again.
//!
//! What the seed declares, and why each part of it has to be data:
//!
//! - **`terminator`** — `.` `!` `?` in Latin and Cyrillic, `।` and `॥` in
//!   Devanagari, `。` `！` `？` `；` in Han. Before this module the formalizer
//!   knew one terminator, so a Chinese or Hindi requirement was one undivided
//!   blob. A Rust `match` over those characters would be a list of writing
//!   systems in Rust, which is the thing `scripts/check-hardcoded-language.rs`
//!   exists to keep out of the runtime: adding a sixth language would then be a
//!   code change rather than a seed row.
//! - **`opening`** — `¿` and `¡`. They mark where a sentence *begins*, so they
//!   must never be read as a break; declaring them is what keeps
//!   `¿La palabra…?` one sentence instead of two.
//! - **`range`** — the Unicode blocks a script is written in, so
//!   [`Script`] is decided from the characters a segment actually contains and
//!   never from a language flag a caller passed in.
//! - **`clause_separator`** — where one sentence carries more than one
//!   obligation, which is what plan 04 L4's need emission splits on.

use std::sync::OnceLock;

use crate::seed::SENTENCE_PUNCTUATION_LINO;
use crate::seed::parser::parse_lino;

/// One segmented unit of source text with exact character offsets into the
/// original string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub script: Script,
}

/// The writing system a segment is in, decided from its characters, never from
/// a language flag supplied by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Latin,
    Cyrillic,
    Devanagari,
    Han,
    Other,
}

impl Script {
    /// The seed slug for this script.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Latin => "latin",
            Self::Cyrillic => "cyrillic",
            Self::Devanagari => "devanagari",
            Self::Han => "han",
            Self::Other => "other",
        }
    }

    fn from_slug(slug: &str) -> Self {
        match slug {
            "latin" => Self::Latin,
            "cyrillic" => Self::Cyrillic,
            "devanagari" => Self::Devanagari,
            "han" => Self::Han,
            _ => Self::Other,
        }
    }
}

/// One script's declared punctuation and the blocks it is written in.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptPunctuation {
    script: Script,
    ranges: Vec<(u32, u32)>,
    terminators: Vec<char>,
    openings: Vec<char>,
    clause_separators: Vec<char>,
}

fn punctuation() -> &'static [ScriptPunctuation] {
    static PUNCTUATION: OnceLock<Vec<ScriptPunctuation>> = OnceLock::new();
    PUNCTUATION.get_or_init(|| {
        let root = parse_lino(SENTENCE_PUNCTUATION_LINO);
        root.children
            .iter()
            .find(|node| node.name == "sentence_punctuation")
            .map(|container| {
                container
                    .children
                    .iter()
                    .filter(|node| node.name == "script")
                    .map(|node| ScriptPunctuation {
                        script: Script::from_slug(&node.id),
                        ranges: values(node, "range")
                            .filter_map(|value| block(&value))
                            .collect(),
                        terminators: values(node, "terminator")
                            .filter_map(|value| value.chars().next())
                            .collect(),
                        openings: values(node, "opening")
                            .filter_map(|value| value.chars().next())
                            .collect(),
                        clause_separators: values(node, "clause_separator")
                            .filter_map(|value| value.chars().next())
                            .collect(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

fn values<'a>(
    node: &'a crate::seed::parser::LinoNode,
    name: &'a str,
) -> impl Iterator<Item = String> + 'a {
    node.children
        .iter()
        .filter(move |child| child.name == name)
        .map(|child| child.id.trim_matches('"').to_owned())
}

/// `"0041-024F"` as an inclusive code-point range.
fn block(value: &str) -> Option<(u32, u32)> {
    let (low, high) = value.split_once('-')?;
    Some((
        u32::from_str_radix(low.trim(), 16).ok()?,
        u32::from_str_radix(high.trim(), 16).ok()?,
    ))
}

/// The script a character is written in, from the seed's declared blocks.
#[must_use]
pub fn script_of(character: char) -> Script {
    let code = u32::from(character);
    punctuation()
        .iter()
        .find(|declared| {
            declared
                .ranges
                .iter()
                .any(|(low, high)| code >= *low && code <= *high)
        })
        .map_or(Script::Other, |declared| declared.script)
}

/// Whether any declared script ends a sentence with this character.
fn is_terminator(character: char) -> bool {
    punctuation()
        .iter()
        .any(|declared| declared.terminators.contains(&character))
}

/// Whether any declared script uses this character to *open* a sentence.
fn is_opening(character: char) -> bool {
    punctuation()
        .iter()
        .any(|declared| declared.openings.contains(&character))
}

/// The script a stretch of text is written in: the script most of its
/// script-bearing characters belong to, and [`Script::Other`] when it has none.
fn dominant_script(text: &str) -> Script {
    let mut counts: Vec<(Script, usize)> = Vec::new();
    for character in text.chars() {
        let script = script_of(character);
        if script == Script::Other {
            continue;
        }
        if let Some(entry) = counts.iter_mut().find(|(seen, _)| *seen == script) {
            entry.1 += 1;
        } else {
            counts.push((script, 1));
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map_or(Script::Other, |(script, _)| script)
}

/// Segment `text` into sentences at the terminators the seed declares for each
/// script, keeping exact spans.
///
/// A segment starts at its first non-separator character and ends after the
/// terminator that closed it, so `&text[segment.start..segment.end]` is the
/// segment's own text and nothing else.
#[must_use]
pub fn sentences(text: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let characters: Vec<(usize, char)> = text.char_indices().collect();
    for (index, (offset, character)) in characters.iter().copied().enumerate() {
        if start.is_none() {
            if character.is_whitespace() {
                continue;
            }
            start = Some(offset);
        }
        if !is_terminator(character) || is_opening(character) {
            continue;
        }
        // A decimal point stands between two digits; it ends a number, not a
        // sentence. This is the one context rule the terminator table cannot
        // carry, because it is about the neighbours rather than the character.
        let previous = index
            .checked_sub(1)
            .and_then(|previous| characters.get(previous))
            .map(|(_, character)| *character);
        let next = characters.get(index + 1).map(|(_, character)| *character);
        if previous.is_some_and(char::is_numeric) && next.is_some_and(char::is_numeric) {
            continue;
        }
        let end = offset + character.len_utf8();
        let begin = start.take().unwrap_or(offset);
        push_segment(text, begin, end, &mut out);
    }
    if let Some(begin) = start {
        push_segment(text, begin, text.len(), &mut out);
    }
    out
}

fn push_segment(text: &str, start: usize, end: usize, out: &mut Vec<Segment>) {
    let slice = &text[start..end];
    let trimmed = slice.trim_end();
    if trimmed.is_empty() {
        return;
    }
    let end = start + trimmed.len();
    out.push(Segment {
        text: trimmed.to_owned(),
        start,
        end,
        script: dominant_script(trimmed),
    });
}

/// Segment one sentence into clauses at the seeded clause separators, so a
/// requirement carrying several obligations emits several needs.
///
/// Spans stay absolute: a clause's `start` and `end` index the original text the
/// sentence was cut from, so a need raised by a clause can be traced back to the
/// exact bytes of the document that raised it.
#[must_use]
pub fn clauses(sentence: &Segment) -> Vec<Segment> {
    let separators: Vec<char> = punctuation()
        .iter()
        .filter(|declared| declared.script == sentence.script)
        .flat_map(|declared| declared.clause_separators.clone())
        .collect();
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (offset, character) in sentence.text.char_indices() {
        if start.is_none() {
            if character.is_whitespace() {
                continue;
            }
            start = Some(offset);
        }
        if !separators.contains(&character) {
            continue;
        }
        let begin = start.take().unwrap_or(offset);
        push_clause(sentence, begin, offset, &mut out);
    }
    if let Some(begin) = start {
        push_clause(sentence, begin, sentence.text.len(), &mut out);
    }
    if out.is_empty() {
        out.push(sentence.clone());
    }
    out
}

fn push_clause(sentence: &Segment, start: usize, end: usize, out: &mut Vec<Segment>) {
    let slice = &sentence.text[start..end];
    let trimmed = slice.trim_end();
    if trimmed.is_empty() {
        return;
    }
    out.push(Segment {
        text: trimmed.to_owned(),
        start: sentence.start + start,
        end: sentence.start + start + trimmed.len(),
        script: sentence.script,
    });
}
