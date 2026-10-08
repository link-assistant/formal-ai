//! Requirement extraction (R1188-U20): the exact requirements an issue states.
//!
//! The Rust twin of `js/agentic/crate/requirement_extraction.mjs`. An issue
//! body is split into units: list items, and the sentences of every other
//! prose line. Fenced code, block quotes, tables and headings are not
//! requirements. A unit is a requirement when it carries a seeded obligation
//! cue (`must`, `нужно`, `必须`, …), opens with a seeded directive verb (`Add`,
//! `Исправь`, `添加`, …), is a task-list checkbox, or sits under a heading that
//! names requirements (`Acceptance criteria`, `Требования`, …). A unit whose
//! content words another kept unit already holds restates it and is dropped.
//! The vocabulary lives in `data/seed/meanings-requirement-extraction.lino`;
//! nothing here names a language.

use std::collections::BTreeSet;

use unicode_general_category::{GeneralCategory, get_general_category};

use crate::coding::contains_cjk;
use crate::seed::{lexicon, surface_present};

/// The role whose words mark a unit as stating an obligation.
const OBLIGATION_ROLE: &str = "requirement_obligation_cue";

/// The role whose words, opening a unit, mark it as an instruction.
const DIRECTIVE_ROLE: &str = "requirement_directive_verb";

/// The role whose words, in a heading, mark the section below as requirements.
const SECTION_ROLE: &str = "requirement_section_heading";

/// The role of words that carry no content of their own.
const FUNCTION_WORD_ROLE: &str = "statement_function_word";

/// A content word has at least this many letters.
const CONTENT_WORD_LENGTH: usize = 3;

/// Words are compared by this many leading letters, so inflections meet.
const STEM_LENGTH: usize = 6;

/// Two units whose content words overlap this much say the same thing.
const RESTATEMENT_OVERLAP: f64 = 0.8;

/// The Markdown fences that open and close a code block.
const FENCES: [&str; 2] = ["```", "~~~"];

/// A candidate unit of an issue body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementUnit {
    /// The sentence or list item, trimmed.
    pub text: String,
    /// Whether its place (a checkbox, a requirement section) makes it required.
    pub structural: bool,
}

/// A letter, a digit, or a mark that joins the letter before it.
fn is_word_character(character: char) -> bool {
    character.is_alphanumeric()
        || matches!(
            get_general_category(character),
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
        )
}

/// The lowercase words of `text`, punctuation removed, joined by single spaces.
#[must_use]
pub fn normalize_unit(text: &str) -> String {
    let spaced: String = text
        .to_lowercase()
        .chars()
        .map(|character| {
            if is_word_character(character) || character == '\'' || character == '-' {
                character
            } else {
                ' '
            }
        })
        .collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The list item of `line`: its text and whether it is a task-list checkbox.
#[must_use]
pub fn list_item_of(line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim_start();
    let rest = if let Some(rest) = trimmed.strip_prefix(['-', '*', '+', '•']) {
        rest
    } else {
        let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
        if digits == 0 {
            return None;
        }
        trimmed[digits..].strip_prefix(['.', ')'])?
    };
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = rest.trim_start();
    for checkbox in ["[ ]", "[x]", "[X]"] {
        if let Some(after) = rest.strip_prefix(checkbox) {
            if after.starts_with(char::is_whitespace) {
                return Some((after.trim().to_string(), true));
            }
        }
    }
    Some((rest.trim().to_string(), false))
}

/// The sentences of a prose line: Latin stops end one before a space,
/// full-width and danda stops always.
#[must_use]
pub fn sentences_of(line: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        current.push(character);
        let latin_stop = matches!(character, '.' | '!' | '?')
            && characters.peek().is_some_and(|next| next.is_whitespace());
        let wide_stop = matches!(character, '。' | '！' | '？' | '।');
        if latin_stop || wide_stop {
            while characters.peek().is_some_and(|next| next.is_whitespace()) {
                characters.next();
            }
            sentences.push(std::mem::take(&mut current));
        }
    }
    sentences.push(current);
    sentences
        .into_iter()
        .map(|sentence| sentence.trim().to_string())
        .filter(|sentence| !sentence.is_empty())
        .collect()
}

/// The heading level of `line` (`#` = 1), or 0 when it is no heading.
fn heading_level(line: &str) -> usize {
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let followed_by_space = line[hashes..].starts_with(char::is_whitespace);
    if (1..=6).contains(&hashes) && followed_by_space {
        hashes
    } else {
        0
    }
}

/// The candidate units of an issue body, in order.
#[must_use]
pub fn requirement_units(text: &str) -> Vec<RequirementUnit> {
    let mut units = Vec::new();
    let mut in_fence = false;
    let mut section_level = 0;
    for raw in text.lines() {
        let line = raw.trim();
        if FENCES.iter().any(|fence| line.starts_with(fence)) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || line.is_empty() || line.starts_with('>') || line.starts_with('|') {
            continue;
        }
        let level = heading_level(line);
        if level > 0 {
            if section_level > 0 && level <= section_level {
                section_level = 0;
            }
            if section_level == 0 && names_requirement_section(line) {
                section_level = level;
            }
            continue;
        }
        if let Some((item, checkbox)) = list_item_of(raw) {
            if !item.is_empty() {
                units.push(RequirementUnit {
                    text: item,
                    structural: checkbox || section_level > 0,
                });
            }
            continue;
        }
        for sentence in sentences_of(line) {
            units.push(RequirementUnit {
                text: sentence,
                structural: section_level > 0,
            });
        }
    }
    units
}

/// The normalized, distinct words of `role`.
fn words_of_role(role: &str) -> Vec<String> {
    let mut words = Vec::new();
    for form in lexicon().role_word_forms(role) {
        let word = normalize_unit(&form.text);
        if !word.is_empty() && !words.contains(&word) {
            words.push(word);
        }
    }
    words
}

/// Whether `unit` carries an obligation cue anywhere in it.
#[must_use]
pub fn states_obligation(unit: &str) -> bool {
    let normalized = normalize_unit(unit);
    words_of_role(OBLIGATION_ROLE)
        .iter()
        .any(|cue| surface_present(&normalized, cue))
}

/// Whether `unit` opens with a directive verb.
#[must_use]
pub fn opens_with_directive(unit: &str) -> bool {
    let normalized = normalize_unit(unit);
    words_of_role(DIRECTIVE_ROLE).iter().any(|verb| {
        if contains_cjk(verb) {
            normalized.starts_with(verb.as_str())
        } else {
            normalized == *verb || normalized.starts_with(&format!("{verb} "))
        }
    })
}

/// Whether the heading `line` names a section of requirements.
#[must_use]
pub fn names_requirement_section(line: &str) -> bool {
    let normalized = normalize_unit(line);
    words_of_role(SECTION_ROLE)
        .iter()
        .any(|word| surface_present(&normalized, word))
}

/// Whether the sentence or list item `unit` states a requirement by its words.
#[must_use]
pub fn is_requirement(unit: &str) -> bool {
    states_obligation(unit) || opens_with_directive(unit)
}

/// The content words of `unit` (CJK text counts each character).
#[must_use]
pub fn content_words(unit: &str) -> BTreeSet<String> {
    let normalized = normalize_unit(unit);
    if contains_cjk(&normalized) {
        return normalized
            .chars()
            .filter(|character| !character.is_whitespace())
            .map(String::from)
            .collect();
    }
    let function_words = words_of_role(FUNCTION_WORD_ROLE);
    normalized
        .split(' ')
        .filter(|word| {
            word.chars().count() >= CONTENT_WORD_LENGTH
                && !function_words.iter().any(|function| function == word)
        })
        .map(|word| word.chars().take(STEM_LENGTH).collect())
        .collect()
}

/// The share of the smaller word set that the other one also holds.
#[must_use]
pub fn overlap(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    let smaller = left.len().min(right.len());
    if smaller == 0 {
        return 0.0;
    }
    let shared = left.intersection(right).count();
    #[allow(clippy::cast_precision_loss)]
    let share = shared as f64 / smaller as f64;
    share
}

/// The requirements `text` states, each once, in order.
#[must_use]
pub fn extract_requirements(text: &str) -> Vec<String> {
    let mut kept: Vec<(String, BTreeSet<String>)> = Vec::new();
    for unit in requirement_units(text) {
        if !unit.structural && !is_requirement(&unit.text) {
            continue;
        }
        let words = content_words(&unit.text);
        let restates = kept
            .iter()
            .any(|(_, earlier)| overlap(earlier, &words) >= RESTATEMENT_OVERLAP);
        if words.is_empty() || restates {
            continue;
        }
        kept.push((unit.text, words));
    }
    kept.into_iter().map(|(text, _)| text).collect()
}
