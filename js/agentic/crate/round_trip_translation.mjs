// Round-trip translation (R1188-U19, extends R526-1): the best translation is
// the one that survives the round trip source -> meta -> target -> meta ->
// source. The Rust twin is rust/src/translation/round_trip.rs.
//
// The meta language is the formal statement of text_formalization.mjs: a
// sentence is formalized in its language, every meaning term is rendered with
// a surface of the target language, and the rendering is formalized again in
// the target language. Names and numbers pass through unchanged; unknown
// words stay in their source form and are reported, never silently lost.
//
// Where a meaning has several surfaces in the target language, each one is
// tried on the way back: the surface whose meaning comes back the same wins
// (meaning identity first), then the one whose return to the source language
// restores the source word (surface second), then the one declared first.
//
// Representation: a rendering is `{text, terms}` with each term `{source,
// target, translated}`; a round trip is `{source, forward, backward,
// statements, survivingTerms, knownTerms, survives}`.

import {
  Polarity, WordKind, clauseStatement, formalizeText, joinSurfaces, knownIds, negationIn, resolveSurface,
  returnSurface, statementTerms, surfacesIn,
} from './text_formalization.mjs';

/**
 * Mirrors `fn best_surface` in rust/src/translation/round_trip.rs: the target surface of a meaning term whose round
 * trip survives best, or null when the meaning has no target surface.
 * @param {{kind: string, id: string, surface: string}} term
 * @param {string} source
 * @param {string} target
 */
export function bestSurface(term, source, target) {
  let best = null;
  for (const candidate of surfacesIn(term.id, target)) {
    const back = resolveSurface(candidate, target);
    const returned = back === null ? null : returnSurface(back, source);
    const score = (back === term.id ? 2 : 0)
      + (returned !== null && returned.toLowerCase() === term.surface.toLowerCase() ? 1 : 0);
    if (best === null || score > best.score) best = { surface: candidate, score };
  }
  return best === null ? null : best.surface;
}

/** Mirrors `fn render_term` in rust/src/translation/round_trip.rs: one term in `target`, `{source, target, translated}`. */
export function renderTerm(term, source, target) {
  if (term.kind !== WordKind.Meaning) return { source: term.surface, target: term.surface, translated: term.kind !== WordKind.Unknown };
  const surface = bestSurface(term, source, target);
  return surface === null
    ? { source: term.surface, target: term.surface, translated: false }
    : { source: term.surface, target: surface, translated: true };
}

/**
 * Mirrors `fn render_statement` in rust/src/translation/round_trip.rs: a statement in `target`, subject first, the
 * target's negation cue after it when the statement is denied, then the other
 * terms in their order.
 */
export function renderStatement(statement, source, target) {
  const terms = [];
  const surfaces = [];
  if (statement.subject !== null) {
    const rendered = renderTerm(statement.subject, source, target);
    terms.push(rendered);
    surfaces.push(rendered.target);
  }
  if (statement.polarity === Polarity.Denied) {
    const cue = negationIn(target);
    if (cue !== null) surfaces.push(cue);
  }
  for (const term of statement.terms) {
    const rendered = renderTerm(term, source, target);
    terms.push(rendered);
    surfaces.push(rendered.target);
  }
  return { text: joinSurfaces(surfaces), terms };
}

/** Mirrors `fn translate_text` in rust/src/translation/round_trip.rs: every statement of `text` rendered in `target`, joined. */
export function translateText(text, source, target) {
  const pieces = formalizeText(text, source).map((statement) => renderStatement(statement, source, target).text);
  return joinSurfaces(pieces);
}

/** Two sorted id lists are equal (the Rust twin compares the vectors). */
const sameIds = (left, right) => left.length === right.length && left.every((id, index) => id === right[index]);

/**
 * Mirrors `fn round_trip_statement` in rust/src/translation/round_trip.rs: one statement through source -> meta ->
 * target -> meta -> source. `{forward, backward, surviving, known, survives}`.
 */
export function roundTripStatement(statement, source, target) {
  const forward = renderStatement(statement, source, target);
  const meta = clauseStatement(forward.text, target, null, false);
  const backward = renderStatement(meta, target, source);
  const returned = clauseStatement(backward.text, source, null, false);
  const original = knownIds(statement);
  const middle = new Set(knownIds(meta));
  const end = new Set(knownIds(returned));
  const surviving = original.filter((id) => middle.has(id) && end.has(id)).length;
  const clean = statementTerms(statement).every((term) => term.kind !== WordKind.Unknown);
  return {
    forward: forward.text,
    backward: backward.text,
    surviving,
    known: original.length,
    survives: clean && original.length > 0 && sameIds(original, knownIds(meta)) && sameIds(original, knownIds(returned)),
  };
}

/**
 * Mirrors `fn round_trip` in rust/src/translation/round_trip.rs: a text through the round trip, statement by
 * statement. It survives when every statement survives: all its words known,
 * the same meanings in the target and back in the source.
 * @param {string} text
 * @param {string} source
 * @param {string} target
 */
export function roundTrip(text, source, target) {
  const statements = formalizeText(text, source).map((statement) => roundTripStatement(statement, source, target));
  return {
    source: text,
    forward: joinSurfaces(statements.map((statement) => statement.forward)),
    backward: joinSurfaces(statements.map((statement) => statement.backward)),
    statements,
    survivingTerms: statements.reduce((sum, statement) => sum + statement.surviving, 0),
    knownTerms: statements.reduce((sum, statement) => sum + statement.known, 0),
    survives: statements.length > 0 && statements.every((statement) => statement.survives),
  };
}
