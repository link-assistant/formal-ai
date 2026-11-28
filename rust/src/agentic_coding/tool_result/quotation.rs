/// How many cited lines it takes before a result counts as quoting.
const CITED_LINES_THAT_MAKE_A_QUOTATION: usize = 2;

/// The part of a result the result is saying itself, before it starts quoting.
pub(super) fn own_words(text: &str) -> &str {
    let mut consumed = 0usize;
    let mut first_citation = None;
    let mut cited = 0usize;
    for line in text.split_inclusive('\n') {
        if let Some(offset) = citation_offset(line) {
            cited += 1;
            first_citation.get_or_insert(consumed + offset);
        }
        consumed += line.len();
    }
    match first_citation {
        Some(offset) if cited >= CITED_LINES_THAT_MAKE_A_QUOTATION => &text[..offset],
        _ => text,
    }
}

/// Where in `line` the result stops speaking and starts naming what it quotes.
fn citation_offset(line: &str) -> Option<usize> {
    line.match_indices(':').find_map(|(colon, _)| {
        let before = &line[..colon];
        let stem = before.trim_end_matches(|character: char| character.is_ascii_digit());
        let cites = stem.len() < before.len()
            && stem.chars().next_back().is_none_or(|character| {
                matches!(character, ':' | '/' | '.') || character.is_whitespace()
            });
        cites.then(|| word_start(before))
    })
}

/// Where the last whitespace-separated word of `text` begins.
fn word_start(text: &str) -> usize {
    text.char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map_or(0, |(index, character)| index + character.len_utf8())
}
