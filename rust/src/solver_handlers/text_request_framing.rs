//! What a text-manipulation request says about its payload in its own words.
//!
//! R1017: a request quotes the text it works on, and that payload is content —
//! "how many words are in 'line counts'" asks for a word count whatever the
//! quoted words say. The framing is the request minus its payload, and the
//! seeded vocabulary questions (counting cues, unit nouns, operation phrases
//! that carry an argument) are asked of the framing alone.

use crate::normal_markov::QuotedSegment;
use crate::seed::OperationVocabulary;
use crate::solver_handlers::text_edit_ops::normalized_word_spans;
use crate::solver_handlers::text_manipulation::text_outside_quoted_segments;

/// The request's own words, normalized: everything outside its quoted
/// segments. Empty when the request quotes no payload, so neither a question
/// about the world ("how many words does English have?") nor a signature
/// colon (`count_vowels(text: str)`) is read as a count over some text.
#[must_use]
pub(super) fn request_framing(prompt: &str, spans: &[QuotedSegment]) -> String {
    if spans.is_empty() {
        return String::new();
    }
    crate::web_engine_core::normalize_prompt(&text_outside_quoted_segments(prompt))
}

/// The argument an operation phrase introduces ("lines that start with `x`"),
/// read from the request's framing: a quoted segment that sits where the
/// argument goes, or the bare token there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OperationArgument {
    pub text: String,
    /// Index of the quoted segment the argument was, so the payload is read
    /// from another one.
    pub quoted: Option<usize>,
}

/// The argument of the first phrase of `canonical` that the request states
/// outside its quoted payload. Each language declares whether its phrase is
/// followed (`en`, `ru`) or preceded (`argument before`: `hi`, `zh`) by it.
#[must_use]
pub(super) fn phrase_argument(
    vocabulary: &OperationVocabulary,
    canonical: &str,
    prompt: &str,
    spans: &[QuotedSegment],
) -> Option<OperationArgument> {
    let words = normalized_word_spans(prompt);
    vocabulary
        .argument_phrases(canonical)
        .into_iter()
        .find_map(|(phrase, before)| {
            let needle = normalized_word_spans(phrase)
                .into_iter()
                .map(|span| span.word)
                .collect::<Vec<_>>();
            if needle.is_empty() || words.len() < needle.len() {
                return None;
            }
            let start = (0..=words.len() - needle.len()).find(|&index| {
                !inside_quoted(words[index].start, spans)
                    && needle
                        .iter()
                        .enumerate()
                        .all(|(offset, word)| words[index + offset].word == *word)
            })?;
            if before {
                argument_before(prompt, words[start].start, spans)
            } else {
                argument_after(prompt, words[start + needle.len() - 1].end, spans)
            }
        })
}

fn inside_quoted(offset: usize, spans: &[QuotedSegment]) -> bool {
    spans
        .iter()
        .any(|span| span.start <= offset && offset < span.end)
}

fn argument_after(prompt: &str, end: usize, spans: &[QuotedSegment]) -> Option<OperationArgument> {
    let rest = &prompt[end..];
    let start = end + (rest.len() - rest.trim_start().len());
    if let Some(index) = spans.iter().position(|span| span.start == start) {
        return quoted_argument(spans, index);
    }
    let token = prompt[start..].split_whitespace().next()?;
    bare_argument(token, true)
}

fn argument_before(
    prompt: &str,
    start: usize,
    spans: &[QuotedSegment],
) -> Option<OperationArgument> {
    let head = prompt[..start].trim_end();
    if let Some(index) = spans.iter().position(|span| span.end == head.len()) {
        return quoted_argument(spans, index);
    }
    let token = head.split_whitespace().next_back()?;
    bare_argument(token, false)
}

fn quoted_argument(spans: &[QuotedSegment], index: usize) -> Option<OperationArgument> {
    let text = spans[index].text.clone();
    (!text.is_empty()).then_some(OperationArgument {
        text,
        quoted: Some(index),
    })
}

/// Punctuation that ends a clause or opens a quote, and so ends a bare
/// argument token.
const ARGUMENT_ENDS: [char; 20] = [
    ':', ',', ';', '?', '!', '：', '，', '"', '\'', '`', '«', '»', '“', '”', '‘', '’', '「', '」',
    '『', '』',
];

/// A bare argument token runs to the punctuation that ends a clause or opens
/// a quote; `after` reads its first piece, otherwise its last.
fn bare_argument(token: &str, after: bool) -> Option<OperationArgument> {
    let mut pieces = token.split(ARGUMENT_ENDS);
    let piece = if after {
        pieces.next()
    } else {
        pieces.next_back()
    }?;
    let text = piece.trim_end_matches(['.', '。']);
    (!text.is_empty()).then(|| OperationArgument {
        text: text.to_owned(),
        quoted: None,
    })
}
