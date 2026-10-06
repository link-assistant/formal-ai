// The part of `crate::intent_formalization::formalize_intent`
// (rust/src/intent_formalization.rs) the write-side planner reads.
//
// The planner reads only `impulse_id` and `source_text` of the
// `IntentFormalization` (general_planner.rs); both are pure functions of the
// prompt, so this returns exactly those two fields. The remaining fields
// (route, relevants, kind, parameters) need the method registry and are not
// ported - no planner route reads them.

import { stableId } from './engine_stable_id.mjs';
import { normalizePrompt } from './engine.mjs';

/** Mirrors `fn impulse_id_for`. @param {string} prompt */
export function impulseIdFor(prompt) {
  return stableId('impulse', normalizePrompt(prompt));
}

/**
 * Mirrors `fn formalize_intent` for the fields the planner reads.
 * @param {string} prompt
 * @param {string} language
 * @returns {{impulse_id: string, source_text: string, language: string}}
 */
export function formalizeIntent(prompt, language) {
  return { impulse_id: impulseIdFor(prompt), source_text: prompt, language };
}
