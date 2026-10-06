// `crate::calculation::calculation_expression_candidates`
// (rust/src/calculation.rs). The planner only asks whether the list is empty;
// the booted worker realm's calculator extractor
// (js/worker/formal_ai_worker_02.js `extractArithmeticExpression`) reads the
// same calculation cue roles and is its twin for that question.

import { realm } from '../host.mjs';

/**
 * Mirrors `fn calculation_expression_candidates` in rust/src/calculation.rs,
 * reduced to the candidate expressions (empty when the prompt is not a
 * calculation request).
 * @param {string} prompt
 * @returns {Array<{expression: string}>}
 */
export function calculationExpressionCandidates(prompt) {
  const extracted = realm().extractArithmeticExpression(prompt);
  return extracted && extracted.expression ? [{ expression: extracted.expression }] : [];
}
