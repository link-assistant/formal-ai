//! The object-phrase reader of the software-project request handler (issue
//! #1175).
//!
//! The artifact kind of a software-project request is read only from the
//! *head noun of the authoring verb's object phrase*, so an artifact word
//! that merely modifies another noun — the "extension" in "a regular
//! expression that matches a US ZIP code with an optional 4-digit extension"
//! — does not claim the prompt. Every surface the reader consults (boundary
//! words, lead words, lead bigrams, and the punctuation classes) is seed data
//! in `data/seed/software-project-phrases.lino`; this module holds only the
//! compiled reading algorithm and the lexicon loaders that feed it.

use crate::seed;
use crate::seed::{
    ROLE_SOFTWARE_OBJECT_BOUNDARY_CHARACTER, ROLE_SOFTWARE_OBJECT_BOUNDARY_WORD,
    ROLE_SOFTWARE_OBJECT_LEAD_BIGRAM, ROLE_SOFTWARE_OBJECT_LEAD_WORD,
    ROLE_SOFTWARE_OBJECT_WORD_INTERNAL_CHARACTER, ROLE_SOFTWARE_SENTENCE_END_CHARACTER,
};

/// A matched software-artifact phrase: the surface word it was recognised by
/// (used to locate the target text after it) and the canonical English label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ArtifactMatch {
    pub(super) surface: &'static str,
    pub(super) label: &'static str,
}

/// Words that close the object noun phrase of an authoring verb: the
/// `software_object_boundary_word` surfaces of the seed, in declaration
/// order. Prepositions and conjunctions that open postmodifiers and relative
/// clauses, subject pronouns that start a new clause, and the politeness
/// marker; Russian and Hindi equivalents in their own scripts.
fn object_phrase_boundary_tokens() -> Vec<&'static str> {
    seed::lexicon()
        .meanings_with_role(ROLE_SOFTWARE_OBJECT_BOUNDARY_WORD)
        .flat_map(|meaning| meaning.words())
        .collect()
}

/// Determiners, benefactives and politeness words that may sit between the
/// verb and the head noun without belonging to the object: the
/// `software_object_lead_word` surfaces of the seed ("write **for me**
/// extension for owlbear", "build **me a** web app", "**that** dashboard").
fn object_phrase_lead_words() -> Vec<&'static str> {
    seed::lexicon()
        .meanings_with_role(ROLE_SOFTWARE_OBJECT_LEAD_WORD)
        .flat_map(|meaning| meaning.words())
        .collect()
}

/// Two-word benefactive forms whose first word is otherwise a boundary token
/// ("write **for me** a poem"): the `software_object_lead_bigram` surfaces of
/// the seed.
fn object_phrase_lead_bigrams() -> Vec<&'static str> {
    seed::lexicon()
        .meanings_with_role(ROLE_SOFTWARE_OBJECT_LEAD_BIGRAM)
        .flat_map(|meaning| meaning.words())
        .collect()
}

/// Punctuation that ends the object phrase at the character it appears in:
/// the `software_object_boundary_character` surfaces of the seed. CJK
/// punctuation and the Devanagari danda separate phrases inside unspaced
/// script runs, so they cut a token anywhere; ASCII punctuation only cuts at
/// a token's edges, because inside a token it is part of a name ("a Node.js
/// service") rather than a sentence end.
fn object_phrase_boundary_characters() -> Vec<char> {
    seed::lexicon()
        .meanings_with_role(ROLE_SOFTWARE_OBJECT_BOUNDARY_CHARACTER)
        .flat_map(|meaning| meaning.words())
        .flat_map(str::chars)
        .collect()
}

/// The CJK punctuation and danda subset that cuts a token at any position:
/// the `software_object_word_internal_character` surfaces of the seed.
fn word_internal_boundary_characters() -> Vec<char> {
    seed::lexicon()
        .meanings_with_role(ROLE_SOFTWARE_OBJECT_WORD_INTERNAL_CHARACTER)
        .flat_map(|meaning| meaning.words())
        .flat_map(str::chars)
        .collect()
}

/// Sentence-ending punctuation that bounds the pre-verbal object of a
/// verb-final (subject-object-verb) request from the left: the
/// `software_sentence_end_character` surfaces of the seed, plus the newline —
/// a structural boundary of the input format, not a character of any
/// language, so it is added here rather than seeded.
fn sentence_end_characters() -> Vec<char> {
    let mut characters: Vec<char> = seed::lexicon()
        .meanings_with_role(ROLE_SOFTWARE_SENTENCE_END_CHARACTER)
        .flat_map(|meaning| meaning.words())
        .flat_map(str::chars)
        .collect();
    characters.push('\n');
    characters
}

/// Trim whitespace and object-phrase boundary characters from both ends of
/// `span`.
fn trim_object_phrase(span: &str) -> &str {
    let boundaries = object_phrase_boundary_characters();
    span.trim_matches(|character: char| {
        character.is_whitespace() || boundaries.contains(&character)
    })
}

/// Drop the determiner/benefactive/politeness words that lead `span`, so the
/// object phrase starts at its first content word.
fn skip_object_phrase_lead(mut span: &str) -> &str {
    let lead_words = object_phrase_lead_words();
    let lead_bigrams = object_phrase_lead_bigrams();
    loop {
        let trimmed = span.trim_start();
        let first = trimmed.split_whitespace().next().unwrap_or_default();
        if !first.is_empty() && lead_words.contains(&first) {
            span = &trimmed[first.len()..];
            continue;
        }
        let bigram = lead_bigrams.iter().find(|bigram| {
            trimmed.starts_with(*bigram) && trimmed[bigram.len()..].starts_with(char::is_whitespace)
        });
        if let Some(bigram) = bigram {
            span = &trimmed[bigram.len()..];
            continue;
        }
        return trimmed;
    }
}

/// Where `token` ends the object phrase, as a byte offset into the token:
/// word-internal separators (CJK punctuation, danda) cut anywhere, ASCII
/// punctuation cuts only at the token's first or last character, and any
/// other token cuts nowhere.
fn boundary_cut(token: &str) -> Option<usize> {
    let word_internal = word_internal_boundary_characters();
    let boundaries = object_phrase_boundary_characters();
    for (index, character) in token.char_indices() {
        if word_internal.contains(&character) {
            return Some(index);
        }
    }
    if let Some(first) = token.chars().next()
        && boundaries.contains(&first)
    {
        return Some(0);
    }
    if let Some((index, last)) = token.char_indices().next_back()
        && boundaries.contains(&last)
    {
        return Some(index);
    }
    None
}

/// Split `span` into its boundary-delimited object-phrase segments, in order,
/// dropping empty ones. "a regular expression that matches a zip code" yields
/// "a regular expression" first: the head noun of the object phrase is the
/// end of the *first* segment, and later segments are subordinate material
/// where an artifact word is incidental (issue #1175).
fn object_phrase_segments(span: &str) -> Vec<&str> {
    let boundary_tokens = object_phrase_boundary_tokens();
    let mut segments = Vec::new();
    let mut segment_start = 0;
    let mut offset = 0;
    for token in span.split_whitespace() {
        let Some(found) = span[offset..].find(token) else {
            break;
        };
        let token_start = offset + found;
        let token_end = token_start + token.len();
        if boundary_tokens.contains(&token) {
            segments.push(&span[segment_start..token_start]);
            segment_start = token_end;
        } else if let Some(cut) = boundary_cut(token) {
            segments.push(&span[segment_start..token_start + cut]);
            segment_start = token_end;
        }
        offset = token_end;
    }
    segments.push(&span[segment_start..]);
    segments
        .into_iter()
        .map(trim_object_phrase)
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// The artifact kind of `segment` when its head noun — the last word or
/// phrase of the segment — is a software-artifact surface. The longest
/// matching surface wins ("browser extension" over "extension"), and the
/// surface must start on a word boundary, so "an owlbear extension" matches
/// `extension` while "maxextension" matches nothing.
fn match_artifact_head(
    segment: &str,
    table: &[(&'static str, &'static str)],
) -> Option<ArtifactMatch> {
    let trimmed = trim_object_phrase(segment);
    let mut best: Option<ArtifactMatch> = None;
    for &(surface, label) in table {
        if !trimmed.ends_with(surface) {
            continue;
        }
        let start = trimmed.len() - surface.len();
        if !is_start_boundary(trimmed, start) {
            continue;
        }
        let longer = best
            .as_ref()
            .is_none_or(|current| surface.chars().count() > current.surface.chars().count());
        if longer {
            best = Some(ArtifactMatch { surface, label });
        }
    }
    best
}

/// Match a software-authoring verb at the start of `input`, returning the
/// consumed byte length and the matched meaning's slug.
fn match_action(input: &str, table: &[(&'static str, &'static str)]) -> Option<(usize, &'static str)> {
    for &(surface, slug) in table {
        if input.starts_with(surface) {
            return Some((surface.len(), slug));
        }
    }
    None
}

/// The next authoring-verb match at or after `from`, reporting the verb's
/// byte offsets as well as its slug: the object phrase is the text after the
/// verb, so the caller needs where the verb ends, not just what it was. The
/// same [`is_start_boundary`]/[`is_end_boundary`] word-boundary discipline
/// the prefix scans use.
fn scan_action_from(
    subject: &str,
    table: &[(&'static str, &'static str)],
    from: usize,
) -> Option<(usize, usize, &'static str)> {
    for (offset, _) in subject[from..].char_indices() {
        let index = from + offset;
        if !is_start_boundary(subject, index) {
            continue;
        }
        if let Some((consumed, slug)) = match_action(&subject[index..], table)
            && consumed > 0
            && is_end_boundary(subject, index + consumed)
        {
            return Some((index, index + consumed, slug));
        }
    }
    None
}

/// The authoring verb `subject` carries and the artifact kind of the object
/// phrase that verb governs (issue #1175).
///
/// The artifact is read only from the *head noun of the verb's object
/// phrase*: the first boundary-delimited segment after the verb, with leading
/// determiners and benefactives skipped, must end with an artifact surface.
/// "Write a browser extension that blocks ads" qualifies ("browser extension"
/// is the head of what is written); "Write a regular expression that matches
/// a US ZIP code with an optional 4-digit extension" does not (the head is
/// "regular expression"; the later "extension" modifies "ZIP code").
///
/// A verb whose object is not an artifact does not end the search — the next
/// authoring verb is tried, so "design and build a dashboard" resolves
/// through "build", and "solve this puzzle" simply does not claim.
///
/// Subject-object-verb languages (Hindi writes the object before the verb)
/// are covered by a fallback that reads the head of the phrase immediately
/// before a *sentence-final* verb. The sentence-final gate keeps a fronted
/// English prepositional phrase ("For a US ZIP code with an optional 4-digit
/// extension, write a regular expression") from smuggling its incidental
/// noun in through the pre-verbal door.
pub(super) fn object_phrase_artifact(
    subject: &str,
    actions: &[(&'static str, &'static str)],
    artifacts: &[(&'static str, &'static str)],
) -> Option<(&'static str, ArtifactMatch)> {
    let boundary_tokens = object_phrase_boundary_tokens();
    let lead_words = object_phrase_lead_words();
    let sentence_ends = sentence_end_characters();
    let mut search_from = 0;
    while let Some((start, end, slug)) = scan_action_from(subject, actions, search_from) {
        search_from = end;
        let remainder = skip_object_phrase_lead(&subject[end..]);
        if let Some(segment) = object_phrase_segments(remainder).first().copied()
            && let Some(artifact) = match_artifact_head(segment, artifacts)
        {
            return Some((slug, artifact));
        }
        // Verb-final (subject-object-verb) request: the object phrase ends
        // just before the verb. Only when nothing but boundaries follows the
        // verb — an English verb with an object after it is not verb-final.
        if trim_object_phrase(&subject[end..]).is_empty() {
            let mut prefix = &subject[..start];
            if let Some(sentence_end) = prefix
                .rfind(|character: char| sentence_ends.contains(&character))
            {
                let after = match prefix[sentence_end..].chars().next() {
                    Some(character) => sentence_end + character.len_utf8(),
                    None => sentence_end,
                };
                prefix = &prefix[after..];
            }
            // Politeness or a determiner may trail the object ("a browser
            // extension, please"): the head noun ends before it.
            let mut phrase = trim_object_phrase(prefix);
            while let Some(last) = phrase.split_whitespace().next_back()
                && (lead_words.contains(&last) || boundary_tokens.contains(&last))
            {
                phrase = trim_object_phrase(&phrase[..phrase.len() - last.len()]);
            }
            if let Some(segment) = object_phrase_segments(phrase).last().copied()
                && let Some(artifact) = match_artifact_head(segment, artifacts)
            {
                return Some((slug, artifact));
            }
        }
    }
    None
}

fn is_start_boundary(value: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    value[..index]
        .chars()
        .next_back()
        .is_some_and(|character| !is_word_character(character))
}

fn is_end_boundary(value: &str, index: usize) -> bool {
    if index >= value.len() {
        return true;
    }
    value[index..]
        .chars()
        .next()
        .is_some_and(|character| !is_word_character(character))
}

/// A "word character" for the recognition scan: an alphanumeric that is *not*
/// CJK.
///
/// CJK scripts write without inter-word spaces, so a CJK surface must match as a
/// substring (every CJK codepoint is its own boundary). Latin, Cyrillic, and
/// Devanagari keep strict whole-token boundaries so a short surface like `апи`
/// never matches inside `напиши`. This mirrors the substring-vs-token contract
/// in [`crate::coding::contains_cjk`].
fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() && !is_cjk_character(character)
}

/// Whether `character` belongs to a CJK script (per the codepoint ranges in
/// [`crate::coding::contains_cjk`]), and so matches as a substring not a token.
fn is_cjk_character(character: char) -> bool {
    let codepoint = character as u32;
    (0x3400..=0x4DBF).contains(&codepoint)
        || (0x4E00..=0x9FFF).contains(&codepoint)
        || (0xF900..=0xFAFF).contains(&codepoint)
        || (0x3040..=0x30FF).contains(&codepoint)
        || (0x3100..=0x312F).contains(&codepoint)
}
