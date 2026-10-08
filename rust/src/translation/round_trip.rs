//! Round-trip translation of sentences and texts (R1188-U19, extends R526-1).
//!
//! The best translation is the one that survives the round trip source ->
//! meta -> target -> meta -> source. It mirrors
//! `js/agentic/crate/round_trip_translation.mjs`, the JavaScript root.
//!
//! The meta language is the formal statement of
//! [`crate::formalization::text_statements`]: a sentence is formalized in its
//! language, every meaning term is rendered with a surface of the target
//! language, and the rendering is formalized again in the target language.
//! Names and numbers pass through unchanged; unknown words stay in their
//! source form. Where a meaning has several target surfaces, the one whose
//! meaning comes back the same wins (meaning identity first), then the one
//! whose return restores the source word (surface second), then the one
//! declared first.

use crate::formalization::statement_rendering::{
    join_surfaces, negation_in, resolve_surface, return_surface, surfaces_in,
};
use crate::formalization::text_statements::{
    Polarity, Statement, Word, WordKind, clause_statement, formalize_text, known_ids,
    statement_terms,
};

/// One term of a rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTerm {
    /// The source surface.
    pub source: String,
    /// The target surface.
    pub target: String,
    /// Whether the term crossed into the target language.
    pub translated: bool,
}

/// A statement rendered in a target language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendering {
    /// The rendered text.
    pub text: String,
    /// The rendered terms, subject first.
    pub terms: Vec<RenderedTerm>,
}

/// One statement through the round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementTrip {
    /// The statement in the target language.
    pub forward: String,
    /// The target rendering brought back to the source language.
    pub backward: String,
    /// Known terms whose meaning came back the same.
    pub surviving: usize,
    /// Known terms of the source statement.
    pub known: usize,
    /// Every word known and every meaning the same in the target and back.
    pub survives: bool,
}

/// A text through the round trip, statement by statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundTrip {
    /// The source text.
    pub source: String,
    /// The text in the target language.
    pub forward: String,
    /// The text back in the source language.
    pub backward: String,
    /// Every statement's trip.
    pub statements: Vec<StatementTrip>,
    /// Known terms that survived.
    pub surviving_terms: usize,
    /// Known terms in the source.
    pub known_terms: usize,
    /// Every statement survived.
    pub survives: bool,
}

/// How well `candidate`, a surface in `target`, carries `meaning`, read from
/// `surface` in `source`, through the round trip.
///
/// Two points when the candidate is read back as the same meaning (meaning
/// identity), one more when that meaning returns to `source` as `surface`
/// again (surface identity).
#[must_use]
pub fn round_trip_score(
    candidate: &str,
    meaning: &str,
    surface: &str,
    source: &str,
    target: &str,
) -> usize {
    let back = resolve_surface(candidate, target);
    let returned = back
        .as_deref()
        .and_then(|slug| return_surface(slug, source));
    let same_meaning = back.as_deref() == Some(meaning);
    let same_surface =
        returned.is_some_and(|returned| returned.to_lowercase() == surface.to_lowercase());
    2 * usize::from(same_meaning) + usize::from(same_surface)
}

/// The index and score of the candidate that survives the round trip best,
/// the first one on a tie.
fn best_candidate<S: AsRef<str>>(
    candidates: &[S],
    meaning: &str,
    surface: &str,
    source: &str,
    target: &str,
) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    for (index, candidate) in candidates.iter().enumerate() {
        let score = round_trip_score(candidate.as_ref(), meaning, surface, source, target);
        if best.is_none_or(|(_, held)| score > held) {
            best = Some((index, score));
        }
    }
    best
}

/// The target surface of a meaning term whose round trip survives best.
///
/// `None` when the meaning has no surface in `target`.
#[must_use]
pub fn best_surface(term: &Word, source: &str, target: &str) -> Option<String> {
    let mut candidates = surfaces_in(&term.id, target);
    let (index, _) = best_candidate(&candidates, &term.id, &term.surface, source, target)?;
    Some(candidates.swap_remove(index))
}

/// Which of several `candidates`, surfaces in `target` offered for `surface`
/// in `source`, survives the round trip best: the index of the highest
/// [`round_trip_score`], the first one on a tie.
///
/// `None` when `surface` is read as no meaning in `source`, or when no
/// candidate comes back as its meaning or as `surface`; the caller then keeps
/// its own choice. The chat translation route uses it wherever it picks among
/// several surfaces (R1188-U19).
#[must_use]
pub fn round_trip_choice<S: AsRef<str>>(
    surface: &str,
    source: &str,
    target: &str,
    candidates: &[S],
) -> Option<usize> {
    let meaning = resolve_surface(surface, source)?;
    best_candidate(candidates, &meaning, surface, source, target)
        .filter(|(_, score)| *score > 0)
        .map(|(index, _)| index)
}

/// One term in `target`.
#[must_use]
pub fn render_term(term: &Word, source: &str, target: &str) -> RenderedTerm {
    if term.kind != WordKind::Meaning {
        return RenderedTerm {
            source: term.surface.clone(),
            target: term.surface.clone(),
            translated: term.kind != WordKind::Unknown,
        };
    }
    let surface = best_surface(term, source, target);
    RenderedTerm {
        source: term.surface.clone(),
        translated: surface.is_some(),
        target: surface.unwrap_or_else(|| term.surface.clone()),
    }
}

/// A statement in `target`.
///
/// The subject comes first, the target's negation cue after it when the
/// statement is denied, then the other terms in their order.
#[must_use]
pub fn render_statement(statement: &Statement, source: &str, target: &str) -> Rendering {
    let mut terms = Vec::new();
    let mut surfaces = Vec::new();
    if let Some(subject) = &statement.subject {
        let rendered = render_term(subject, source, target);
        surfaces.push(rendered.target.clone());
        terms.push(rendered);
    }
    if statement.polarity == Polarity::Denied
        && let Some(cue) = negation_in(target)
    {
        surfaces.push(cue);
    }
    for term in &statement.terms {
        let rendered = render_term(term, source, target);
        surfaces.push(rendered.target.clone());
        terms.push(rendered);
    }
    Rendering {
        text: join_surfaces(&surfaces),
        terms,
    }
}

/// Every statement of `text` rendered in `target`, joined.
#[must_use]
pub fn translate_text(text: &str, source: &str, target: &str) -> String {
    let pieces: Vec<String> = formalize_text(text, source)
        .iter()
        .map(|statement| render_statement(statement, source, target).text)
        .collect();
    join_surfaces(&pieces)
}

/// One statement through source -> meta -> target -> meta -> source.
#[must_use]
pub fn round_trip_statement(statement: &Statement, source: &str, target: &str) -> StatementTrip {
    let forward = render_statement(statement, source, target);
    let meta = clause_statement(&forward.text, target, None, false);
    let backward = render_statement(&meta, target, source);
    let returned = clause_statement(&backward.text, source, None, false);
    let original = known_ids(statement);
    let middle = known_ids(&meta);
    let end = known_ids(&returned);
    let surviving = original
        .iter()
        .filter(|id| middle.contains(id) && end.contains(id))
        .count();
    let clean = statement_terms(statement)
        .iter()
        .all(|term| term.kind != WordKind::Unknown);
    StatementTrip {
        survives: clean && !original.is_empty() && original == middle && original == end,
        forward: forward.text,
        backward: backward.text,
        surviving,
        known: original.len(),
    }
}

/// A text through the round trip, statement by statement.
///
/// It survives when every statement survives: all its words known, the same
/// meanings in the target and back in the source.
#[must_use]
pub fn round_trip(text: &str, source: &str, target: &str) -> RoundTrip {
    let statements: Vec<StatementTrip> = formalize_text(text, source)
        .iter()
        .map(|statement| round_trip_statement(statement, source, target))
        .collect();
    let forward: Vec<String> = statements.iter().map(|trip| trip.forward.clone()).collect();
    let backward: Vec<String> = statements
        .iter()
        .map(|trip| trip.backward.clone())
        .collect();
    RoundTrip {
        source: text.to_owned(),
        forward: join_surfaces(&forward),
        backward: join_surfaces(&backward),
        surviving_terms: statements.iter().map(|trip| trip.surviving).sum(),
        known_terms: statements.iter().map(|trip| trip.known).sum(),
        survives: !statements.is_empty() && statements.iter().all(|trip| trip.survives),
        statements,
    }
}
