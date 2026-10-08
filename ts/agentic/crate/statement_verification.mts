// `crate::statement_verification` (rust/src/statement_verification.rs), the
// part the multi-source recheck calls: `StatementPlan::new` and
// `grounding_query`. A `StatementPlan` is `{statement, query, assessment}`. The
// document-verification planner, the market-price assessment and the captured
// execution are not ported (outside the #703 synthesis call graph).

import { agenticMessage } from '../messages.mjs';
import { ASSUMED_TRUE_PRIOR, assessStatement, truthValue } from './relative_meta_logic.mjs';

/**
 * Mirrors `fn grounding_query` in rust/src/statement_verification.rs: the
 * statement with whitespace condensed, quoted, and paired with the fact-check
 * intent terms (the wording lives in data/meta/agentic-messages.lino).
 * @param {string} statement
 */
export function groundingQuery(statement) {
  const condensed = statement.split(/\p{White_Space}+/u).filter(Boolean).join(' ');
  return agenticMessage('statement_grounding_query', { statement: condensed });
}

/**
 * Mirrors `StatementPlan::new` in rust/src/statement_verification.rs.
 * @param {string} statement
 * @param {Array<object>} evidence `RelativeEvidence` records already collected
 */
export function statementPlan(statement, evidence) {
  return {
    statement,
    query: groundingQuery(statement),
    assessment: assessStatement(statement, truthValue(ASSUMED_TRUE_PRIOR), evidence),
  };
}
