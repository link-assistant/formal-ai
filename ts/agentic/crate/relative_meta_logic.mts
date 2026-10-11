// `crate::relative_meta_logic` (rust/src/relative_meta_logic.rs): relative
// statement probability, modelled on link-foundation/relative-meta-logic.
// Pure arithmetic over caller-supplied evidence: no clocks, no randomness.
//
// Representation (no classes, so the translator's portable subset applies): a
// `TruthValue` is a plain number already clamped to `[0, 1]` and snapped to
// the six-decimal grid (build one with `truthValue`; the `get()` of Rust is the
// identity); a `SourceTier` and a `Stance` are their slug strings; a
// `RelativeEvidence` is `{source_label, tier, stance, strength}` and a
// `StatementAssessment` is `{statement, prior, support, contradiction,
// posterior, ignored_sources}`.

/** Mirrors `TRUTH_VALUE_DECIMALS` in rust/src/relative_meta_logic.rs. */
const TRUTH_VALUE_DECIMALS = 6;

/** Mirrors `ASSUMED_TRUE_PRIOR` in rust/src/relative_meta_logic.rs. */
export const ASSUMED_TRUE_PRIOR = 0.6;

/** Mirrors `TruthValue::{FALSE, TRUE, UNKNOWN}` in rust/src/relative_meta_logic.rs. */
export const TruthValueConstants = Object.freeze({ FALSE: 0, TRUE: 1, UNKNOWN: 0.5 });

/** Mirrors `fn round_decimal` in rust/src/relative_meta_logic.rs. */
function roundDecimal(value) {
  const scale = 10 ** TRUTH_VALUE_DECIMALS;
  return Math.round(value * scale) / scale;
}

/**
 * Mirrors `TruthValue::new`: clamp to `[0, 1]`, snap to the decimal grid, and
 * send a non-finite input to the `UNKNOWN` midpoint.
 * @param {number} value
 */
export function truthValue(value) {
  if (!Number.isFinite(value)) return TruthValueConstants.UNKNOWN;
  return roundDecimal(Math.min(1, Math.max(0, value)));
}

/** Mirrors `TruthValue::negate`. @param {number} value */
export function negateTruthValue(value) {
  return truthValue(1 - value);
}

/** Mirrors `TruthValue::to_decimal_string` (and its `Display`). @param {number} value */
export function truthValueToString(value) {
  return value.toFixed(TRUTH_VALUE_DECIMALS);
}

/** Mirrors `enum Aggregator` in rust/src/relative_meta_logic.rs. */
export const Aggregator = Object.freeze({
  Min: 'min',
  Max: 'max',
  Average: 'average',
  Product: 'product',
  ProbabilisticSum: 'probabilistic_sum',
});

/**
 * Mirrors `fn probabilistic_sum`: `1 - prod(1 - v)` over `[0, 1]` magnitudes.
 * @param {Iterable<number>} values
 */
function probabilisticSum(values) {
  let complement = 1;
  for (const value of values) complement *= 1 - Math.min(1, Math.max(0, value));
  return 1 - complement;
}

/**
 * Mirrors `Aggregator::combine` in rust/src/relative_meta_logic.rs.
 * @param {string} aggregator an `Aggregator`
 * @param {Array<number>} values truth values
 */
export function combine(aggregator, values) {
  switch (aggregator) {
    case Aggregator.Min: return truthValue(values.reduce((acc, value) => Math.min(acc, value), 1));
    case Aggregator.Max: return truthValue(values.reduce((acc, value) => Math.max(acc, value), 0));
    case Aggregator.Average:
      if (values.length === 0) return TruthValueConstants.UNKNOWN;
      return truthValue(values.reduce((sum, value) => sum + value, 0) / values.length);
    case Aggregator.Product: return truthValue(values.reduce((acc, value) => acc * value, 1));
    default: return truthValue(probabilisticSum(values));
  }
}

/** Mirrors `enum SourceTier` in rust/src/relative_meta_logic.rs (the slugs). */
export const SourceTier = Object.freeze({
  OriginalFirstParty: 'original_first_party',
  OriginalJournalism: 'original_journalism',
  IndependentCorroboration: 'independent_corroboration',
  Unoriginal: 'unoriginal',
});

/** Mirrors `SourceTier::weight_percent`. @param {string} tier */
export function tierWeightPercent(tier) {
  switch (tier) {
    case SourceTier.OriginalFirstParty: return 100;
    case SourceTier.OriginalJournalism: return 85;
    case SourceTier.IndependentCorroboration: return 50;
    default: return 0;
  }
}

/** Mirrors `SourceTier::weight`. @param {string} tier */
export function tierWeight(tier) {
  return tierWeightPercent(tier) / 100;
}

/** Mirrors `SourceTier::is_original`. @param {string} tier */
export function tierIsOriginal(tier) {
  return tier !== SourceTier.Unoriginal;
}

/** Mirrors `enum Stance` in rust/src/relative_meta_logic.rs (the slugs). */
export const Stance = Object.freeze({
  Supports: 'supports',
  Contradicts: 'contradicts',
  Neutral: 'neutral',
});

/**
 * Mirrors `RelativeEvidence::new`.
 * @param {string} sourceLabel
 * @param {string} tier a `SourceTier`
 * @param {string} stance a `Stance`
 * @param {number} strength
 */
export function relativeEvidence(sourceLabel, tier, stance, strength) {
  return { source_label: sourceLabel, tier, stance, strength: truthValue(strength) };
}

/** Mirrors `RelativeEvidence::effective_mass`. */
export function effectiveMass(evidence) {
  if (evidence.stance === Stance.Neutral || !tierIsOriginal(evidence.tier)) return 0;
  return tierWeight(evidence.tier) * evidence.strength;
}

/** Mirrors `RelativeEvidence::is_ignored`. */
export function evidenceIsIgnored(evidence) {
  return effectiveMass(evidence) <= 0;
}

/** Mirrors `RelativeEvidence::trace_payload`. */
export function evidenceTracePayload(evidence) {
  return `source=${evidence.source_label} tier=${evidence.tier} stance=${evidence.stance} strength=${truthValueToString(evidence.strength)} mass=${effectiveMass(evidence).toFixed(6)} ignored=${evidenceIsIgnored(evidence)}`;
}

/**
 * Mirrors `StatementAssessment::assess`.
 * @param {string} statement
 * @param {number} prior a truth value
 * @param {Array<object>} evidence `RelativeEvidence` records
 */
export function assessStatement(statement, prior, evidence) {
  const support = probabilisticSum(evidence.filter((item) => item.stance === Stance.Supports).map(effectiveMass));
  const contradiction = probabilisticSum(evidence.filter((item) => item.stance === Stance.Contradicts).map(effectiveMass));
  const raised = probabilisticSum([prior, support]);
  const posterior = raised * (1 - contradiction);
  return {
    statement,
    prior,
    support: truthValue(support),
    contradiction: truthValue(contradiction),
    posterior: truthValue(posterior),
    ignored_sources: evidence.filter(evidenceIsIgnored).map((item) => item.source_label),
  };
}

/** Mirrors `StatementAssessment::assess_assumed_true`. */
export function assessAssumedTrue(statement, evidence) {
  return assessStatement(statement, truthValue(ASSUMED_TRUE_PRIOR), evidence);
}

/** Mirrors `StatementAssessment::is_probable`. */
export function assessmentIsProbable(assessment) {
  return assessment.posterior > 0.5;
}

/** Mirrors `StatementAssessment::trace_payload`. */
export function assessmentTracePayload(assessment) {
  return `prior=${truthValueToString(assessment.prior)} support=${truthValueToString(assessment.support)} contradiction=${truthValueToString(assessment.contradiction)} posterior=${truthValueToString(assessment.posterior)} ignored=${assessment.ignored_sources.length}`;
}
