// `crate::summarization::importance` (rust/src/summarization/importance.rs):
// evidence-weighted importance for merged statements. `coverage`, `authority`
// and `agreement` are integer percentages, blended with the static prior at
// 2:1 in favour of observed evidence; the probability side is
// relative_meta_logic.mjs.
//
// Representation: an `ImportanceScore` is `{prior, coverage, authority,
// agreement, evidence, weight}` and a `RankedStatement` is `{statement, score,
// probability, denied_by, evidence}` (`probability` a plain truth-value number,
// `evidence` the `RelativeEvidence` records the probability was computed from).

import {
  Stance, TruthValueConstants, assessAssumedTrue, relativeEvidence, tierWeightPercent,
} from './relative_meta_logic.mjs';
import {
  compareStrings, mergedEvidence, mergedPrior, mergedSources, negatedSignature, sameSignature,
  signatureKey, sourceCount,
} from './summarization_dedup.mjs';
import { renderedResponse } from './summarization_vocabulary.mjs';

const INTENT_EVIDENCE_SUMMARY = 'summarization_evidence_summary';
const INTENT_EVIDENCE_DENIED = 'summarization_evidence_denied';
const INTENT_DISPUTED_STATEMENT = 'summarization_disputed_statement';
const ASSERTED_PLACEHOLDER = '{asserted}';
const TOTAL_PLACEHOLDER = '{total}';
const DENIED_PLACEHOLDER = '{denied}';
const STATEMENT_PLACEHOLDER = '{statement}';
const EVIDENCE_PLACEHOLDER = '{evidence}';

/**
 * Mirrors `ImportanceScore::blend`: `evidence = coverage x authority x
 * agreement / 10_000`, `weight = min(100, (prior + 2 x evidence) / 3)`.
 */
export function blendImportance(prior, coverage, authority, agreement) {
  const product = Math.floor((coverage * authority * agreement) / 10000);
  const evidence = Math.min(product, 100);
  const weight = Math.min(Math.floor((prior + 2 * evidence) / 3), 100);
  return { prior, coverage, authority, agreement, evidence, weight };
}

/** Mirrors `fn percentage`: `part x 100 / whole` clamped to 100, `0` for an empty whole. */
function percentage(part, whole) {
  if (whole === 0) return 0;
  return Math.min(Math.floor((part * 100) / whole), 100);
}

/** Mirrors `fn trust_mass`: the summed trust weights of the distinct asserting sources. */
function trustMass(node) {
  return mergedEvidence(node).reduce((sum, [, tier]) => sum + tierWeightPercent(tier), 0);
}

/**
 * Mirrors `fn score` in rust/src/summarization/importance.rs.
 * @param {object} node a `MergedStatement`
 * @param {object} report the `DedupReport`
 * @param {number} totalSources
 */
export function scoreStatement(node, report, totalSources) {
  const denier = report.statements.find((other) => sameSignature(other.signature, negatedSignature(node.signature))) ?? null;
  const deniedBy = denier === null ? [] : mergedSources(denier);
  const asserting = sourceCount(node);
  const coverage = percentage(asserting, totalSources);
  const supportingMass = trustMass(node);
  const denyingMass = denier === null ? 0 : trustMass(denier);
  const authority = percentage(supportingMass, asserting * 100);
  const agreement = percentage(supportingMass, supportingMass + denyingMass);
  const score = blendImportance(mergedPrior(node), coverage, authority, agreement);

  const evidence = mergedEvidence(node).map(([source, tier]) => relativeEvidence(source, tier, Stance.Supports, TruthValueConstants.TRUE));
  if (denier !== null) {
    for (const [source, tier] of mergedEvidence(denier)) {
      evidence.push(relativeEvidence(source, tier, Stance.Contradicts, TruthValueConstants.TRUE));
    }
  }
  const assessment = assessAssumedTrue(node.representative.text, evidence);
  return { statement: node, score, probability: assessment.posterior, denied_by: deniedBy, evidence };
}

/**
 * Mirrors `fn rank` in rust/src/summarization/importance.rs: every node scored,
 * ordered by weight descending, then source count descending, then signature
 * key ascending (a total order). Denied nodes stay in the list.
 * @param {object} report a `DedupReport`
 */
export function rank(report) {
  const total = report.sources.length;
  const ranked = report.statements.map((node) => scoreStatement(node, report, total));
  ranked.sort((left, right) => (right.score.weight - left.score.weight)
    || (sourceCount(right.statement) - sourceCount(left.statement))
    || compareStrings(signatureKey(left.statement.signature), signatureKey(right.statement.signature)));
  return ranked;
}

/** Mirrors `RankedStatement::is_contested`. */
export function isContested(ranked) {
  return ranked.denied_by.length > 0;
}

/**
 * Mirrors `RankedStatement::evidence_summary_in`: the counts worded by the seed.
 * @param {object} ranked a `RankedStatement`
 * @param {number} totalSources
 * @param {string} [language] a language slug
 */
export function evidenceSummaryIn(ranked, totalSources, language = 'en') {
  const denied = ranked.denied_by.length === 0
    ? ''
    : renderedResponse(INTENT_EVIDENCE_DENIED, language, [[DENIED_PLACEHOLDER, String(ranked.denied_by.length)]]);
  return renderedResponse(INTENT_EVIDENCE_SUMMARY, language, [
    [ASSERTED_PLACEHOLDER, String(sourceCount(ranked.statement))],
    [TOTAL_PLACEHOLDER, String(totalSources)],
    [DENIED_PLACEHOLDER, denied],
  ]);
}

/** Mirrors `RankedStatement::evidence_summary` (English). */
export function evidenceSummary(ranked, totalSources) {
  return evidenceSummaryIn(ranked, totalSources, 'en');
}

/**
 * Mirrors `fn to_statements_in`: the ranked list as plain statements carrying
 * the evidence-weighted weight; a contested one is worded as a disagreement.
 * @param {Array<object>} ranked
 * @param {number} totalSources
 * @param {string} [language]
 */
export function toStatementsIn(ranked, totalSources, language = 'en') {
  return ranked.map((item) => {
    const stmt = { ...item.statement.representative, weight: item.score.weight };
    if (isContested(item)) {
      let text = stmt.text;
      while (text.endsWith('.')) text = text.slice(0, -1);
      stmt.text = renderedResponse(INTENT_DISPUTED_STATEMENT, language, [
        [STATEMENT_PLACEHOLDER, text],
        [EVIDENCE_PLACEHOLDER, evidenceSummaryIn(item, totalSources, language)],
      ]);
    }
    return stmt;
  });
}

/** Mirrors `fn to_statements` (English). */
export function toStatements(ranked, totalSources) {
  return toStatementsIn(ranked, totalSources, 'en');
}
