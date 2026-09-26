//! Seed-driven quantity and entity extraction, five languages (#1138 B8).
//!
//! Spelled-out numerals resolve through the arithmetic normalization tables the
//! seed lexicon already carries, not through an ASCII-digit scan, so `two`,
//! `два`, `दो`, `两` and `dos` are one code path. Multiplicity attaches to the
//! entity it modifies, which is what today's object-count table cannot express.
//!
//! Wave T lands the shapes only; wave I8 leaf 08-L4 fills the bodies in.

use crate::seed::{ROLE_CARDINAL_NUMBER_WORD, ROLE_MEASUREMENT_UNIT};

/// Seed role of the indefinite determiners ("a", "an", …), each of which
/// states a multiplicity of one for the entity it introduces.
const ROLE_INDEFINITE_DETERMINER: &str = "verifiable_quantity_determiner";

/// A quantity the prompt states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantity {
    /// The value, as written or as normalized from a spelled-out numeral.
    pub value: String,
    /// The unit the prompt attaches, when it attaches one.
    pub unit: Option<String>,
    /// The label the prompt attaches, when it attaches one.
    pub label: Option<String>,
    /// Byte offset in the prompt, so the trace can point at the evidence.
    pub offset: usize,
}

/// An entity the prompt lists, for `Count` tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    /// The surface exactly as written.
    pub surface: String,
    /// Multiplicity the prompt states for this entity, default `1`.
    pub multiplicity: String,
    /// Byte offset in the prompt.
    pub offset: usize,
}

/// Every quantity the prompt states, in order of appearance.
#[must_use]
pub fn extract_quantities(prompt: &str, language: &str) -> Vec<Quantity> {
    quantity_matches(prompt, language)
        .into_iter()
        .map(|found| {
            let (unit, label) = following_meaning(prompt, found.end, language);
            Quantity {
                value: found.value,
                unit,
                label,
                offset: found.start,
            }
        })
        .collect()
}

/// Every entity the prompt lists, with the multiplicity it states.
#[must_use]
pub fn extract_entities(prompt: &str, language: &str) -> Vec<Entity> {
    let quantities = quantity_matches_with_indefinites(prompt, language);
    let Some(first) = quantities.first() else {
        return Vec::new();
    };
    let end = prompt[first.start..]
        .char_indices()
        .find(|(_, character)| is_sentence_end(*character))
        .map_or(prompt.len(), |(offset, _)| first.start + offset);
    let start = prompt[..first.start]
        .char_indices()
        .rev()
        .find(|(_, character)| is_sentence_end(*character))
        .map_or(0, |(offset, character)| offset + character.len_utf8());
    let list = &prompt[start..end];
    let mut cuts = list_separators(list, language);
    cuts.push((list.len(), list.len()));

    let mut out = Vec::new();
    let mut cursor = 0;
    for (cut_start, cut_end) in cuts {
        if cut_start < cursor {
            continue;
        }
        let raw = &list[cursor..cut_start];
        let leading = raw.len() - raw.trim_start().len();
        let trailing = raw.trim_end().len();
        if trailing > leading {
            let mut surface_start = start + cursor + leading;
            let surface_end = start + cursor + trailing;
            let item = &prompt[surface_start..surface_end];
            let item_quantities = quantity_matches_with_indefinites(item, language);
            let multiplicity = item_quantities
                .first()
                .map_or_else(|| String::from("1"), |found| found.value.clone());
            if let Some(found) = item_quantities.first() {
                // The first list item also carries the list introduction (for
                // example, "I have two …"). Everything through its stated
                // multiplicity is syntax around the entity, not its surface.
                surface_start += found.end;
                while prompt[surface_start..surface_end]
                    .chars()
                    .next()
                    .is_some_and(|character| {
                        character.is_whitespace() || is_entity_prefix(character)
                    })
                {
                    surface_start += prompt[surface_start..]
                        .chars()
                        .next()
                        .map_or(0, char::len_utf8);
                }
            } else if let Some(prefix_len) = leading_role_surface_len(
                &prompt[surface_start..surface_end],
                language,
                ROLE_INDEFINITE_DETERMINER,
            ) {
                surface_start += prefix_len;
            }
            let surface = prompt[surface_start..surface_end]
                .trim_matches(is_entity_edge)
                .to_owned();
            if !surface.is_empty() {
                let offset = prompt[..surface_end]
                    .rfind(&surface)
                    .unwrap_or(surface_start);
                out.push(Entity {
                    surface,
                    multiplicity,
                    offset,
                });
            }
        }
        cursor = cut_end;
    }
    out
}

fn leading_role_surface_len(text: &str, language: &str, role: &str) -> Option<usize> {
    let lowered = text.to_lowercase();
    crate::seed::lexicon()
        .meanings_with_role(role)
        .flat_map(|meaning| &meaning.lexemes)
        .filter(|lexeme| lexeme.language == language)
        .flat_map(|lexeme| &lexeme.words)
        .filter_map(|word| {
            let surface = word.text.to_lowercase();
            lowered.starts_with(&surface).then_some(surface.len())
        })
        .filter(|length| {
            text[*length..]
                .chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric())
        })
        .max()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuantityMatch {
    value: String,
    start: usize,
    end: usize,
}

fn quantity_matches(prompt: &str, language: &str) -> Vec<QuantityMatch> {
    let mut found = digit_matches(prompt);
    found.extend(cardinal_word_matches(prompt, language));
    ordered_unique(found)
}

/// Quantities plus the indefinite determiners, each stating multiplicity one.
///
/// A list can name its items without any numeral — "a clarinet, a violin" —
/// because the indefinite determiner itself asserts one of each. Entity
/// extraction needs those anchors; [`normalized_shape`] keeps the bare
/// quantity view so prompt identity does not change.
fn quantity_matches_with_indefinites(prompt: &str, language: &str) -> Vec<QuantityMatch> {
    let mut found = digit_matches(prompt);
    found.extend(cardinal_word_matches(prompt, language));
    found.extend(indefinite_determiner_matches(prompt, language));
    ordered_unique(found)
}

fn cardinal_word_matches(prompt: &str, language: &str) -> Vec<QuantityMatch> {
    let lowered = prompt.to_lowercase();
    let mut found = Vec::new();
    for meaning in crate::seed::lexicon().meanings_with_role(ROLE_CARDINAL_NUMBER_WORD) {
        let Some(value) = meaning
            .words()
            .find(|surface| surface.chars().all(|character| character.is_ascii_digit()))
        else {
            continue;
        };
        for surface in meaning
            .lexemes
            .iter()
            .filter(|lexeme| lexeme.language == language)
            .flat_map(|lexeme| &lexeme.words)
            .map(|word| word.text.as_str())
            .filter(|surface| surface.chars().any(char::is_alphabetic))
        {
            let needle = surface.to_lowercase();
            for start in matching_offsets(&lowered, &needle) {
                found.push(QuantityMatch {
                    value: value.to_owned(),
                    start,
                    end: start + needle.len(),
                });
            }
        }
    }
    found
}

fn indefinite_determiner_matches(prompt: &str, language: &str) -> Vec<QuantityMatch> {
    let lowered = prompt.to_lowercase();
    let mut found = Vec::new();
    for meaning in crate::seed::lexicon().meanings_with_role(ROLE_INDEFINITE_DETERMINER) {
        for surface in meaning
            .lexemes
            .iter()
            .filter(|lexeme| lexeme.language == language)
            .flat_map(|lexeme| &lexeme.words)
            .map(|word| word.text.as_str())
            .filter(|surface| surface.chars().any(char::is_alphabetic))
        {
            let needle = surface.to_lowercase();
            for start in matching_offsets(&lowered, &needle) {
                found.push(QuantityMatch {
                    value: String::from("1"),
                    start,
                    end: start + needle.len(),
                });
            }
        }
    }
    found
}

fn ordered_unique(mut found: Vec<QuantityMatch>) -> Vec<QuantityMatch> {
    found.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
    });
    let mut deduplicated: Vec<QuantityMatch> = Vec::new();
    for candidate in found {
        if deduplicated
            .last()
            .is_some_and(|previous| candidate.start < previous.end)
        {
            continue;
        }
        deduplicated.push(candidate);
    }
    deduplicated
}

pub(crate) fn normalized_shape(prompt: &str, language: &str) -> String {
    let lowered = prompt.to_lowercase();
    let matches = quantity_matches(&lowered, language);
    let mut out = String::new();
    let mut cursor = 0;
    for found in matches {
        if found.start < cursor {
            continue;
        }
        out.push_str(&lowered[cursor..found.start]);
        out.push_str("{n}");
        cursor = found.end;
    }
    out.push_str(&lowered[cursor..]);
    crate::engine::normalize_prompt(&out)
}

fn digit_matches(prompt: &str) -> Vec<QuantityMatch> {
    let mut out = Vec::new();
    let mut start = None;
    for (offset, character) in prompt
        .char_indices()
        .chain(core::iter::once((prompt.len(), '\0')))
    {
        if character.is_ascii_digit() {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            out.push(QuantityMatch {
                value: prompt[begin..offset].to_owned(),
                start: begin,
                end: offset,
            });
        }
    }
    out
}

fn matching_offsets(haystack: &str, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    let token_bounded = needle
        .chars()
        .all(|character| character.is_alphabetic() || character.is_whitespace());
    haystack
        .match_indices(needle)
        .filter(|(start, matched)| {
            if !token_bounded || matched.chars().any(is_unspaced_script) {
                return true;
            }
            let before = haystack[..*start].chars().next_back();
            let after = haystack[*start + matched.len()..].chars().next();
            before.is_none_or(|character| !character.is_alphanumeric())
                && after.is_none_or(|character| !character.is_alphanumeric())
        })
        .map(|(start, _)| start)
        .collect()
}

fn following_meaning(
    prompt: &str,
    offset: usize,
    language: &str,
) -> (Option<String>, Option<String>) {
    let tail = prompt[offset..].trim_start();
    let mut units = crate::seed::lexicon()
        .meanings_with_role(ROLE_MEASUREMENT_UNIT)
        .flat_map(|meaning| {
            meaning
                .lexemes
                .iter()
                .filter(move |lexeme| lexeme.language == language)
                .flat_map(|lexeme| lexeme.words.iter().map(|word| word.text.as_str()))
        })
        .collect::<Vec<_>>();
    units.sort_by_key(|unit| core::cmp::Reverse(unit.len()));
    let unit = units
        .into_iter()
        .find(|unit| starts_with_surface(tail, unit))
        .map(str::to_owned);
    let label = tail
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    ',' | '.' | '?' | '!' | ';' | ':' | '，' | '。' | '？'
                )
        })
        .find(|part| !part.is_empty())
        .map(str::to_owned);
    (unit, label)
}

fn starts_with_surface(text: &str, surface: &str) -> bool {
    text.to_lowercase().starts_with(&surface.to_lowercase())
        && text[surface.len()..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() || is_unspaced_script(character))
}

fn list_separators(list: &str, language: &str) -> Vec<(usize, usize)> {
    let lowered = list.to_lowercase();
    let mut out = list
        .char_indices()
        .filter(|(_, character)| matches!(character, ',' | ';' | '，' | '、'))
        .map(|(offset, character)| (offset, offset + character.len_utf8()))
        .collect::<Vec<_>>();
    for surface in crate::seed::lexicon()
        .meanings_with_role("verifiable_entity_separator")
        .flat_map(|meaning| &meaning.lexemes)
        .filter(|lexeme| lexeme.language == language)
        .flat_map(|lexeme| &lexeme.words)
        .map(|word| word.text.to_lowercase())
    {
        out.extend(
            matching_offsets(&lowered, &surface)
                .into_iter()
                .map(|offset| (offset, offset + surface.len())),
        );
    }
    out.sort_unstable();
    out.dedup();
    out
}

const fn is_sentence_end(character: char) -> bool {
    matches!(character, '.' | '?' | '!' | '。' | '？' | '！' | '।')
}

const fn is_unspaced_script(character: char) -> bool {
    matches!(character as u32, 0x3400..=0x9fff)
}

const fn is_entity_prefix(character: char) -> bool {
    matches!(character, ':' | '：')
}

const fn is_entity_edge(character: char) -> bool {
    character.is_whitespace()
        || matches!(character, ',' | ';' | ':' | '，' | '、' | '。' | '？' | '।')
}
