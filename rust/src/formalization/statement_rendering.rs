//! Rendering formal statements as text (R1188-U18, U19 and U21).
//!
//! The deformalizing half of the shared text formalizer: a meaning is written
//! with the surface that reads back as the same meaning, and a statement is
//! written subject first. It mirrors the rendering functions of
//! `js/agentic/crate/text_formalization.mjs` (`surfacesIn` to
//! `deformalizeStatement`); the reading half is [`super::text_statements`].

use super::segment::{Script, script_of};
use super::text_statements::{
    NEGATION_ROLE, Polarity, Statement, Word, WordKind, fold_inflection, indexed_meaning,
    kind_of_listed, phrase_key, role_words_in,
};
use crate::seed::lexicon;

fn is_han(character: char) -> bool {
    script_of(character) == Script::Han
}

/// Every plain surface `slug` has in `language`, in declaration order.
#[must_use]
pub fn surfaces_in(slug: &str, language: &str) -> Vec<String> {
    lexicon()
        .meaning(slug)
        .and_then(|meaning| {
            meaning
                .lexemes
                .iter()
                .find(|lexeme| lexeme.language == language)
        })
        .map(|lexeme| {
            lexeme
                .words
                .iter()
                .filter(|word| phrase_key(&word.text).is_some())
                .map(|word| word.text.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// The meaning a surface is read as in `language`.
///
/// A function word, anaphor, continuation or negation cue is read as no
/// meaning.
#[must_use]
pub fn resolve_surface(surface: &str, language: &str) -> Option<String> {
    let key = phrase_key(surface)?;
    if kind_of_listed(&key, language).is_some() {
        return None;
    }
    if let Some(slug) = indexed_meaning(&key, language) {
        return Some(slug);
    }
    if key.contains(' ') {
        return None;
    }
    fold_inflection(&key, language)
}

/// The surface a meaning is written with in `language`.
///
/// The first surface that reads back as the same meaning there, else the
/// first surface; `None` when the meaning has no surface in `language`.
#[must_use]
pub fn return_surface(slug: &str, language: &str) -> Option<String> {
    let surfaces = surfaces_in(slug, language);
    let returning = surfaces
        .iter()
        .find(|surface| resolve_surface(surface, language).as_deref() == Some(slug))
        .cloned();
    returning.or_else(|| surfaces.into_iter().next())
}

/// A term rendered in `language` from its meaning alone.
///
/// A name or number keeps its surface, and an unknown word keeps it
/// lowercased, since it is no name.
#[must_use]
pub fn surface_in(term: &Word, language: &str) -> String {
    match term.kind {
        WordKind::Unknown => term.surface.to_lowercase(),
        WordKind::Meaning => {
            return_surface(&term.id, language).unwrap_or_else(|| term.surface.clone())
        }
        _ => term.surface.clone(),
    }
}

/// The first negation cue the lexicon lists for `language`.
#[must_use]
pub fn negation_in(language: &str) -> Option<String> {
    role_words_in(NEGATION_ROLE, language).into_iter().next()
}

/// Pieces joined by a space, except before a Han piece that follows a
/// character outside ASCII.
///
/// Han text and its full-width punctuation are written without spaces.
#[must_use]
pub fn join_surfaces(surfaces: &[String]) -> String {
    let mut out = String::new();
    for surface in surfaces {
        if surface.is_empty() {
            continue;
        }
        let tight = out
            .chars()
            .last()
            .is_some_and(|previous| !previous.is_ascii())
            && surface.chars().next().is_some_and(is_han);
        let separator = if out.is_empty() || tight { "" } else { " " };
        out.push_str(separator);
        out.push_str(surface);
    }
    out
}

/// A statement rendered in `language`.
///
/// The subject comes first, the negation cue after it when the statement is
/// denied, then the other terms in their order.
#[must_use]
pub fn deformalize_statement(statement: &Statement, language: &str) -> String {
    let mut surfaces = Vec::new();
    if let Some(subject) = &statement.subject {
        surfaces.push(surface_in(subject, language));
    }
    if statement.polarity == Polarity::Denied
        && let Some(cue) = negation_in(language)
    {
        surfaces.push(cue);
    }
    for term in &statement.terms {
        surfaces.push(surface_in(term, language));
    }
    join_surfaces(&surfaces)
}
