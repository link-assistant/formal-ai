// `crate::concepts::{extract_concept_query, lookup_concept_query}`
// (rust/src/concepts.rs), answered by the booted worker realm's twins
// (js/worker/formal_ai_worker_concept_queries_and_arithmetic.js `extractConceptQuery`,
// js/worker/formal_ai_worker_seed_responses_and_language.js `lookupConceptQuery`), which read the same
// concept seed. A `ConceptQuery` is `{term, context}`.

import { realm } from '../host.mjs';

/**
 * Mirrors `fn extract_concept_query` in rust/src/concepts.rs.
 * @param {string} prompt
 * @returns {{term: string, context: string|null}|null}
 */
export function extractConceptQuery(prompt) {
  const query = realm().extractConceptQuery(prompt);
  if (!query || typeof query.term !== 'string') return null;
  return { term: query.term, context: query.context ?? null };
}

/**
 * Mirrors `fn lookup_concept_query` in rust/src/concepts.rs: the lookup hit or null.
 * @param {{term: string, context: string|null}} query
 */
export function lookupConceptQuery(query) {
  return realm().lookupConceptQuery(query) ?? null;
}
