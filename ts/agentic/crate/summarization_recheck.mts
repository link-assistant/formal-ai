// `crate::summarization::recheck` (rust/src/summarization/recheck.rs): the
// lightweight, capture-independent preflight for merged statements. The verdict
// is read off the relative-meta-logic assessment; withholding is not deletion.
//
// Representation: a `Verdict` is its slug string; a `RecheckedStatement` is
// `{ranked, plan, verdict}` (`plan` a `StatementPlan`); a `RecheckReport` is
// `{checked}`.

import { agenticMessage } from '../messages.mjs';
import { assessmentIsProbable, assessmentTracePayload } from './relative_meta_logic.mjs';
import { statementPlan } from './statement_verification.mjs';
import { sourceCount } from './summarization_dedup.mjs';

/** Mirrors `enum Verdict` in rust/src/summarization/recheck.rs (the slugs). */
export const Verdict = Object.freeze({
  Confirmed: 'confirmed',
  Contested: 'contested',
  Refuted: 'refuted',
  Unsupported: 'unsupported',
});

/** Mirrors `Verdict::is_presentable`. @param {string} verdict */
export function verdictIsPresentable(verdict) {
  return verdict === Verdict.Confirmed || verdict === Verdict.Contested;
}

/** Mirrors `Verdict::slug`. @param {string} verdict */
export function verdictSlug(verdict) {
  return verdict;
}

/** Mirrors `Verdict::reason` (the wording lives in data/meta/agentic-messages.lino). @param {string} verdict */
export function verdictReason(verdict) {
  return agenticMessage(`recheck_reason_${verdict}`);
}

/** Mirrors `fn verdict_for` in rust/src/summarization/recheck.rs. */
function verdictFor(plan) {
  const assessment = plan.assessment;
  if (assessment.support <= 0) return Verdict.Unsupported;
  if (assessment.contradiction <= 0) return Verdict.Confirmed;
  return assessmentIsProbable(assessment) ? Verdict.Contested : Verdict.Refuted;
}

/**
 * Mirrors `fn recheck` in rust/src/summarization/recheck.rs.
 * @param {Array<object>} ranked `RankedStatement` values
 * @returns {{checked: Array<{ranked: object, plan: object, verdict: string}>}}
 */
export function recheck(ranked) {
  return {
    checked: ranked.map((item) => {
      const plan = statementPlan(item.statement.representative.text, item.evidence);
      return { ranked: item, verdict: verdictFor(plan), plan };
    }),
  };
}

/** Mirrors `RecheckedStatement::text`. */
export function checkedText(item) {
  return item.plan.statement;
}

/** Mirrors `RecheckedStatement::query`. */
export function checkedQuery(item) {
  return item.plan.query;
}

/** Mirrors `RecheckedStatement::trace_payload`. */
export function checkedTracePayload(item) {
  return [
    `verdict=${verdictSlug(item.verdict)}`,
    assessmentTracePayload(item.plan.assessment),
    `sources=${sourceCount(item.ranked.statement)}`,
    `denied=${item.ranked.denied_by.length}`,
  ].join(' ');
}

/** Mirrors `RecheckReport::survivors`: the statements cleared for presentation. */
export function survivors(report) {
  return report.checked.filter((item) => verdictIsPresentable(item.verdict));
}

/** Mirrors `RecheckReport::withheld`. */
export function withheld(report) {
  return report.checked.filter((item) => !verdictIsPresentable(item.verdict));
}

/** Mirrors `RecheckReport::queries`. */
export function reportQueries(report) {
  return report.checked.map(checkedQuery);
}

/** Mirrors `RecheckReport::trace`. */
export function reportTrace(report) {
  return report.checked.map(checkedTracePayload).join('\n');
}
