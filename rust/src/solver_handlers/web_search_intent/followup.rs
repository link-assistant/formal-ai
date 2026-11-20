use crate::coding::contains_cjk;

/// Drop a trailing follow-up instruction clause ("… and summarize who won",
/// "… . Compare their patents") from a query.
///
/// This is a universal boundary algorithm, not a list of memorised fragments: a
/// follow-up clause is one of the lexicon's [`ROLE_FOLLOWUP_INSTRUCTION_VERB`]
/// surfaces sitting immediately after a *boundary* — either sentence
/// punctuation ([`is_sentence_boundary`]) or a run of
/// [`ROLE_CLAUSE_CONTINUATION_MARKER`] words (and / then / and then, walked back
/// so the compound needs no stored surface). The query is cut at the start of
/// the earliest such boundary. A bare verb with no boundary before it is part of
/// the topic and left untouched.
pub(super) fn truncate_instruction_tail<'a>(
    value: &'a str,
    followup_verbs: &[&str],
    continuation_markers: &[&str],
    is_sentence_boundary: fn(char) -> bool,
) -> &'a str {
    // ASCII-lowercase keeps byte offsets identical to `value` (it only folds
    // A–Z), so indices computed here slice `value` safely; the non-ASCII verbs
    // are already lowercase in the lexicon and unaffected by the fold.
    let lower = value.to_ascii_lowercase();
    let mut cut = value.len();
    for &verb in followup_verbs {
        let cjk = contains_cjk(verb);
        let mut from = 0;
        while let Some(relative) = lower[from..].find(verb) {
            let start = from + relative;
            let end = start + verb.len();
            from = end;
            // Space-delimited scripts require a whole-token match; CJK verbs have
            // no word boundaries and match as bare substrings.
            if !cjk && (!is_token_start(&lower, start) || !is_token_end(&lower, end)) {
                continue;
            }
            if let Some(boundary) =
                boundary_before(&lower, start, continuation_markers, is_sentence_boundary)
            {
                cut = cut.min(boundary);
            }
        }
    }
    value[..cut].trim()
}

/// Whether `index` begins a whitespace/punctuation-delimited token in `text`
/// (the preceding char is non-alphanumeric, or there is none).
fn is_token_start(text: &str, index: usize) -> bool {
    !text[..index]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
}

/// Whether `index` ends a whitespace/punctuation-delimited token in `text` (the
/// following char is non-alphanumeric, or there is none).
fn is_token_end(text: &str, index: usize) -> bool {
    !text[index..]
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric)
}

/// If the text immediately before `verb_start` is a follow-up boundary, return
/// the byte offset at which to cut (the start of the boundary run); otherwise
/// `None`.
fn boundary_before(
    text: &str,
    verb_start: usize,
    continuation_markers: &[&str],
    is_sentence_boundary: fn(char) -> bool,
) -> Option<usize> {
    let head = text[..verb_start].trim_end();
    if head.is_empty() {
        // The verb opens the value — there is no preceding clause to split off.
        return None;
    }
    if head.ends_with(is_sentence_boundary) {
        return Some(head.len());
    }
    // Walk back over a run of clause-continuation markers ("and", "then",
    // "and then"); the cut falls at the start of the run.
    let mut cursor = head;
    let mut matched = false;
    loop {
        let trimmed = cursor.trim_end();
        let shortened = continuation_markers
            .iter()
            .find(|&&marker| ends_with_token(trimmed, marker))
            .map(|&marker| &trimmed[..trimmed.len() - marker.len()]);
        match shortened {
            Some(rest) => {
                cursor = rest;
                matched = true;
            }
            None => break,
        }
    }
    matched.then(|| cursor.trim_end().len())
}

/// Whether `haystack` ends with `marker` as a whole token. CJK markers match as
/// bare substrings; space-delimited markers require a preceding whitespace (or
/// for the whole string to be exactly the marker).
fn ends_with_token(haystack: &str, marker: &str) -> bool {
    if contains_cjk(marker) {
        haystack.ends_with(marker)
    } else {
        haystack == marker
            || haystack
                .strip_suffix(marker)
                .is_some_and(|head| head.ends_with(char::is_whitespace))
    }
}
