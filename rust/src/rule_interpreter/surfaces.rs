use super::ConditionSurface;
use crate::seed::Slot;

/// Whether a seeded role surface is present, including the open slot carried by
/// a semantic form such as `how many … remain`.
///
/// The rule interpreter used to compare the literal ellipsis character for
/// [`Slot`] forms. That made slot-backed meanings usable by their dedicated
/// recognizers but invisible to data-authored handler promotions. Interpreting
/// the same slot metadata here keeps the lexicon as the single recognition
/// authority: a newly seeded paraphrase can affect dispatch without a Rust
/// phrase branch.
///
/// A prefix surface's lead half ("prove …") is a phrase the request is expected
/// to *start some rendering of*, so it must be present as complete words. A raw
/// substring would let an unrelated word of another language that merely embeds
/// the phrase — Spanish "proveedores" embeds "prove" — claim the surface and
/// fire the promotion that reads it (issue #1138, plan 03 wave F).
pub(super) fn spelled_surface_present(text: &str, surface: &ConditionSurface) -> bool {
    let expected = surface.text.as_str();
    match surface.slot {
        Slot::Bare => bounded_surface_present(text, expected),
        Slot::Prefix => expected
            .split_once('…')
            .is_some_and(|(before, _)| phrase_present_as_words(text, before.trim())),
        Slot::Suffix => expected
            .split_once('…')
            .is_some_and(|(_, after)| text.contains(after.trim())),
        Slot::Circumfix => expected.split_once('…').is_some_and(|(before, after)| {
            let before = before.trim();
            let after = after.trim();
            if before.is_empty() || after.is_empty() {
                return false;
            }
            text.find(before).is_some_and(|start| {
                let tail = &text[start + before.len()..];
                tail.find(after)
                    .is_some_and(|offset| !tail[..offset].trim().is_empty())
            })
        }),
    }
}

fn bounded_surface_present(text: &str, expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    if crate::coding::contains_cjk(expected) {
        return text.contains(expected);
    }
    text == expected
        || text.starts_with(&format!("{expected} "))
        || text.ends_with(&format!(" {expected}"))
        || text.contains(&format!(" {expected} "))
}

/// Whether `phrase` occurs in `text` as complete words: every character border
/// of the match is a word boundary, where a word boundary is the text edge or a
/// non-alphanumeric character. Punctuation counts as a boundary, so "¿Cuántos"
/// still begins the phrase "cuántos …", while the "prove" inside
/// "proveedores" does not — an embedded word is not the phrase.
fn phrase_present_as_words(text: &str, phrase: &str) -> bool {
    if phrase.is_empty() {
        return false;
    }
    if crate::coding::contains_cjk(phrase) {
        return text.contains(phrase);
    }
    let mut search = 0;
    while let Some(start) = text[search..].find(phrase) {
        let start = search + start;
        let end = start + phrase.len();
        let opens_word = text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric());
        let closes_word = text[end..]
            .chars()
            .next()
            .is_none_or(|after| !after.is_alphanumeric());
        if opens_word && closes_word {
            return true;
        }
        search = end;
    }
    false
}

/// The furthest a phrasal verb's object may separate its verb and particle.
///
/// This mirrors the semantic lexicon's bound, but operates on
/// [`super::ConditionSource`] surfaces so every condition backend evaluates the same
/// data-owned condition.
const SEPARATED_ROLE_OBJECT_LIMIT: usize = 6;

pub(super) fn separated_surface_present(text: &str, surface: &str) -> bool {
    let mut parts = surface.split_whitespace();
    let (Some(verb), Some(particle), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    words.iter().enumerate().any(|(index, word)| {
        *word == verb
            && words
                .iter()
                .skip(index + 2)
                .take(SEPARATED_ROLE_OBJECT_LIMIT)
                .any(|later| *later == particle)
    })
}

pub(super) fn before_slot(text: &str) -> &str {
    text.split_once('…').map_or(text, |(before, _)| before)
}
