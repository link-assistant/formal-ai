// The `SymbolicAnswer` readers of rust/src/engine_answer.rs the planner
// consults. An answer is the host solver's `{intent, answer, ...}` shape.

/** Mirrors `SymbolicAnswer::asks_for_clarification`. */
export function asksForClarification(answer) {
  return answer.intent.startsWith('clarify');
}

/** Mirrors `SymbolicAnswer::is_inconclusive`. */
export function isInconclusive(answer) {
  return ['unknown', 'ill_formed', 'punctuation_only_prompt', 'concept_lookup_unresolved'].includes(answer.intent)
    || asksForClarification(answer);
}

/** Mirrors `SymbolicAnswer::defers_to_the_open_web`. */
export function defersToTheOpenWeb(answer) {
  return answer.intent === 'web_search';
}

/** Mirrors `SymbolicAnswer::announces_a_list_it_does_not_make`. */
export function announcesAListItDoesNotMake(answer) {
  const text = answer.answer.replace(/\p{White_Space}+$/u, '');
  return text.endsWith(':') || text.endsWith('：');
}
