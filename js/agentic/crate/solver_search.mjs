// `crate::solver_search::recognizes_reachability_problem`
// (rust/src/solver_search.rs, rust/src/solver_search/problem.rs): the
// recognizer only; the budget search itself stays native.

import { mentionsRoleRaw, wordsForRole } from './seed_meanings.mjs';
import { parseI64 } from './rust_str.mjs';

const MAX_OPERANDS = 6;

/** Mirrors `fn extract_integers_with_positions` (values only; positions are unused by the gate). */
function extractIntegers(span) {
  const numbers = [];
  for (const run of span.match(/[0-9]+/g) || []) {
    const value = parseI64(run);
    if (value !== null) numbers.push(value);
  }
  return numbers;
}

/** Whether any `reachability_target_marker` word occurs (`target_marker_positions` non-empty). */
const hasTargetMarker = (lower) => wordsForRole('reachability_target_marker').some((marker) => lower.includes(marker));

/**
 * Mirrors `fn recognizes_reachability_problem` in rust/src/solver_search.rs
 * (`parse_search_problem(prompt).is_some()`).
 */
export function recognizesReachabilityProblem(prompt) {
  const lower = prompt.toLowerCase();
  if (!mentionsRoleRaw('reachability_operand_framing', lower) || !mentionsRoleRaw('reachability_search_cue', lower)) return false;
  const integers = extractIntegers(lower);
  if (integers.length < 3) return false;
  if (!hasTargetMarker(lower)) return false;
  const operands = integers.length - 1;
  return operands >= 2 && operands <= MAX_OPERANDS;
}
