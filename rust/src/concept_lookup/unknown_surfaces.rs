use std::collections::BTreeSet;
use std::sync::OnceLock;

/// The surfaces in `normalized` that no seeded meaning accounts for — the words
/// the system must ask about.
///
/// Language-neutral by construction: it asks the lexicon which surfaces are
/// already seeded and returns the rest, so no literal word list decides what is
/// unknown. Quoted spans are excluded: a quoted example is the *subject* of the
/// question, not a concept the system is missing — asking a dictionary what
/// `quick brown fox` means would be asking about the user's own illustration.
#[must_use]
pub fn unknown_surfaces(normalized: &str, language: &str) -> Vec<String> {
    let _ = language;
    let mut out: Vec<String> = Vec::new();
    for token in outside_quotes(normalized)
        .split(crate::source_walk::is_word_boundary)
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        let folded = token.to_lowercase();
        if !is_unknown_surface(token) || out.contains(&folded) {
            continue;
        }
        out.push(folded);
    }
    out
}

/// Whether one token is a surface no seeded meaning accounts for.
///
/// The rule lives here rather than in each caller so the coding path, which
/// asks about a normalized sentence, and the deep formalizer, which asks about
/// an exact span of the user's own bytes, agree on what "unknown" means. A
/// token shorter than three characters or made only of digits is a fragment
/// rather than a concept; everything the seed lexicon already declares is
/// known by definition.
#[must_use]
pub fn is_unknown_surface(token: &str) -> bool {
    let folded = token.to_lowercase();
    token.chars().count() >= 3
        && !token.chars().all(char::is_numeric)
        && !seeded_surfaces().contains(&folded)
}

/// Every unresolved surface in `text`, with the exact byte span it occupies.
///
/// Spans are relative to `text`, so a caller that segmented a document adds the
/// segment's own offset and reaches a span that selects exactly the surface in
/// the original bytes. Quoted spans are skipped for the same reason
/// [`unknown_surfaces`] skips them: a quoted example is the subject of the
/// question, not a concept the system is missing.
#[must_use]
pub fn unknown_surface_spans(text: &str) -> Vec<(String, usize, usize)> {
    const PAIRS: [(char, char); 5] = [
        ('"', '"'),
        ('«', '»'),
        ('\u{201c}', '\u{201d}'),
        ('\u{2018}', '\u{2019}'),
        ('\u{300c}', '\u{300d}'),
    ];
    let mut out: Vec<(String, usize, usize)> = Vec::new();
    let mut closing: Option<char> = None;
    let mut start: Option<usize> = None;
    let push = |start: usize, end: usize, out: &mut Vec<(String, usize, usize)>| {
        let token = &text[start..end];
        if is_unknown_surface(token) {
            out.push((token.to_owned(), start, end));
        }
    };
    for (offset, character) in text.char_indices() {
        match closing {
            Some(close) if character == close => {
                closing = None;
                start = None;
            }
            Some(_) => {}
            None => {
                if let Some((_, close)) = PAIRS.iter().find(|(open, _)| *open == character) {
                    if let Some(begin) = start.take() {
                        push(begin, offset, &mut out);
                    }
                    closing = Some(*close);
                } else if crate::source_walk::is_word_boundary(character) {
                    if let Some(begin) = start.take() {
                        push(begin, offset, &mut out);
                    }
                } else if start.is_none() {
                    start = Some(offset);
                }
            }
        }
    }
    if let Some(begin) = start {
        push(begin, text.len(), &mut out);
    }
    out
}
/// The text of `value` with every quoted span removed, in any of the quotation
/// marks the five seeded languages use.
fn outside_quotes(value: &str) -> String {
    const PAIRS: [(char, char); 5] = [
        ('"', '"'),
        ('«', '»'),
        ('\u{201c}', '\u{201d}'),
        ('\u{2018}', '\u{2019}'),
        ('\u{300c}', '\u{300d}'),
    ];
    let mut out = String::with_capacity(value.len());
    let mut closing: Option<char> = None;
    for character in value.chars() {
        match closing {
            Some(close) if character == close => closing = None,
            Some(_) => {}
            None => {
                if let Some((_, close)) = PAIRS.iter().find(|(open, _)| *open == character) {
                    closing = Some(*close);
                } else {
                    out.push(character);
                }
            }
        }
    }
    out
}
/// Whether `value` carries at least one quoted span.
///
/// [`unknown_surfaces`] skips quoted spans because a quoted example is the
/// subject of the question, never a concept the system is missing; callers
/// that need the same subject test on the raw prompt use this instead of
/// re-scanning the pairs.
#[must_use]
pub fn has_quoted_span(value: &str) -> bool {
    outside_quotes(value) != value
}
/// Every surface the seed lexicon declares, folded once.
fn seeded_surfaces() -> &'static BTreeSet<String> {
    static CACHE: OnceLock<BTreeSet<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        crate::seed::lexicon()
            .meanings
            .iter()
            .flat_map(crate::seed::Meaning::words)
            .flat_map(|surface| {
                surface
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .collect::<Vec<_>>()
            })
            .collect()
    })
}
