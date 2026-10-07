// The formalization records the native solver appends to its event log on an
// intent-cache miss (R1013, server parity): rust/src/translation/selection.rs
// `select_formalization_candidate_with_policy` over the request's candidates
// and rust/src/solver_formalization.rs `record_formalization_selection` /
// `record_formalization`.
//
// The solve the protocol surfaces run (`solve_with_history`) ranks against a
// fresh `ProbabilityStore`, so a candidate's posterior is its structural prior
// (`score / 1000`): no evidence weight, no state-similarity fallback. The
// selection config is `SolverConfig::default()`'s (temperature 0.7, guess
// probability 0.8, questioning rigor 0.4); the protocol surfaces never
// override it. `f32` arithmetic is kept with `Math.fround`.
//
// An event is `{kind, payload}`; the log is a plain array the caller owns.

import { candidateCompactSummary } from './translation_formalization.mjs';

/** Mirrors the selection fields of `SolverConfig::default()` in rust/src/solver.rs. */
export const DEFAULT_SELECTION_CONFIG = Object.freeze({
  temperature: Math.fround(0.7),
  guessProbability: Math.fround(0.8),
  questioningRigor: Math.fround(0.4),
});

const f32 = Math.fround;
const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const MASK = (1n << 64n) - 1n;
const encoder = new TextEncoder();

/**
 * Mirrors `const fn finite_clamped` in rust/src/translation/selection.rs.
 * @param {number} value
 * @param {number} min
 * @param {number} max
 * @returns {number}
 */
function finiteClamped(value, min, max) {
  return Number.isFinite(value) ? Math.min(Math.max(value, min), max) : min;
}

/**
 * Rust `{:.6}` / `{:.4}` of an `f32`.
 * @param {number} value
 * @param {number} digits
 * @returns {string}
 */
function fixed(value, digits) {
  return f32(value).toFixed(digits);
}

/** Mirrors `fn softmax_scores` in rust/src/probability.rs. */
function softmaxScores(scores, temperature) {
  if (!scores.length) return [];
  const t = f32(finiteClamped(temperature, 0, 1));
  if (t <= 1.1920929e-7) {
    let best = 0;
    scores.forEach((score, index) => {
      if (score >= scores[best]) best = index;
    });
    return scores.map((_, index) => (index === best ? 1 : 0));
  }
  const max = scores.reduce((left, right) => Math.max(left, right), -Infinity);
  const weights = scores.map((score) => f32(Math.exp(f32(f32(score - max) / t))));
  const total = weights.reduce((sum, weight) => f32(sum + weight), 0);
  if (!Number.isFinite(total) || total <= 1.1920929e-7) return scores.map(() => f32(1 / Math.max(1, scores.length)));
  return weights.map((weight) => f32(weight / total));
}

/**
 * Mirrors `fn rank_probability_candidates` in rust/src/probability.rs over an
 * empty store: `{target, posterior, probability}` rows, best first.
 */
function rankEmptyStore(targets, priors, temperature) {
  const probabilities = softmaxScores(priors, temperature);
  const ranked = targets.map((target, index) => ({ target, posterior: priors[index], probability: probabilities[index] }));
  ranked.sort((left, right) => (right.probability - left.probability)
    || (right.posterior - left.posterior)
    || (left.target < right.target ? -1 : left.target > right.target ? 1 : 0));
  return ranked;
}

/** Mirrors `ProbabilityRanking::trace_summary` in rust/src/probability.rs. */
function traceSummary(ranked) {
  return ranked.map((row) => `${row.target}:${fixed(row.posterior, 6)}:${fixed(row.probability, 6)}`).join('|');
}

/** Mirrors `fn formalization_probability_target` in rust/src/translation/selection.rs. */
export function formalizationProbabilityTarget(candidate) {
  return `formalization:${candidateCompactSummary(candidate)}`;
}

/**
 * Mirrors `const fn probability_margin_epsilon` in rust/src/translation/selection.rs.
 * @param {number} questioningRigor
 * @returns {number}
 */
function probabilityMarginEpsilon(questioningRigor) {
  return f32(f32(0.23) * f32(finiteClamped(questioningRigor, 0, 1)) + f32(0.02));
}

/** Mirrors `fn should_clarify` in rust/src/translation/selection.rs. */
function shouldClarify(config) {
  return f32(config.questioningRigor * f32(1 - config.guessProbability)) > 0.5;
}

/** Mirrors `fn ranked_indices` in rust/src/translation/selection.rs. */
function rankedIndices(candidates, probabilities) {
  return candidates.map((_, index) => index).sort((left, right) => (probabilities[right] - probabilities[left])
    || (candidates[right].score - candidates[left].score)
    || (left - right));
}

/**
 * Mirrors `fn fnv1a64` in rust/src/translation/selection.rs.
 * @param {string} value
 * @returns {bigint}
 */
function fnv1a64(value) {
  let hash = FNV_OFFSET;
  for (const byte of encoder.encode(value)) {
    hash ^= BigInt(byte);
    hash = (hash * FNV_PRIME) & MASK;
  }
  return hash;
}

/**
 * Mirrors `fn seeded_unit_interval` in rust/src/translation/selection.rs.
 * @param {string} impulse
 * @param {string} seed
 * @returns {number}
 */
function seededUnitInterval(impulse, seed) {
  const bucket = Number(fnv1a64(`${impulse}\n${seed}`) >> 48n);
  return f32(bucket / 65535);
}

/** Mirrors `fn selection_seed` in rust/src/translation/selection.rs. */
function selectionSeed(candidates, config, suffix) {
  const summaries = candidates.map(candidateCompactSummary).join('|');
  return `temperature=${fixed(config.temperature, 4)};guess=${fixed(config.guessProbability, 4)};rigor=${fixed(config.questioningRigor, 4)};${summaries};${suffix}`;
}

/** Mirrors `fn sample_index` in rust/src/translation/selection.rs. */
function sampleIndex(probabilities, impulse, seed) {
  const draw = seededUnitInterval(impulse, seed);
  let cumulative = 0;
  for (const [index, probability] of probabilities.entries()) {
    cumulative = f32(cumulative + probability);
    if (draw <= cumulative) return index;
  }
  return Math.max(0, probabilities.length - 1);
}

/**
 * Mirrors `fn select_formalization_candidate_with_policy` (and
 * `select_from_probabilities`) in rust/src/translation/selection.rs over the
 * empty probability store a protocol solve ranks against. A clarification is
 * returned as `{kind: 'clarify', top, runnerUp, margin, epsilon}`; with the
 * default config `should_clarify` never holds, so its question is not built.
 * @param {Array<object>} candidates `formalizePromptCandidates` output
 * @param {string} impulse the prompt
 * @param {object} [config]
 */
export function selectFormalizationCandidate(candidates, impulse, config = DEFAULT_SELECTION_CONFIG) {
  const targets = candidates.map(formalizationProbabilityTarget);
  const ranked = rankEmptyStore(targets, candidates.map((candidate) => f32(candidate.score / 1000)), config.temperature);
  const probabilities = targets.map((target) => ranked.find((row) => row.target === target)?.probability ?? 0);
  const epsilon = probabilityMarginEpsilon(config.questioningRigor);
  const selection = (decision) => ({ candidates, probabilities, decision });
  if (!candidates.length) return selection({ kind: 'no_candidate' });
  if (candidates.length === 1) {
    return selection({ kind: 'selected', index: 0, probability: 1, margin: 1, epsilon, reason: 'only_candidate' });
  }
  const order = rankedIndices(candidates, probabilities);
  const [top, runnerUp] = order;
  const margin = f32(probabilities[top] - probabilities[runnerUp]);
  if (margin > epsilon) {
    return selection({ kind: 'selected', index: top, probability: probabilities[top], margin, epsilon, reason: 'clearly_best' });
  }
  if (shouldClarify(config)) return selection({ kind: 'clarify', top, runnerUp, margin, epsilon });
  const index = sampleIndex(probabilities, impulse, selectionSeed(candidates, config, traceSummary(ranked)));
  return selection({ kind: 'selected', index, probability: probabilities[index], margin, epsilon, reason: 'guessed_under_ambiguity' });
}

/** Mirrors `FormalizationSelection::selected_candidate`. */
export function selectedCandidate(selection) {
  return selection.decision.kind === 'selected' ? selection.candidates[selection.decision.index] ?? null : null;
}

/** Mirrors `fn record_formalization_selection` in rust/src/solver_formalization.rs. */
export function recordFormalizationSelection(log, selection) {
  selection.candidates.forEach((candidate, index) => {
    const probability = selection.probabilities[index] ?? 0;
    log.push({ kind: 'candidate', payload: `formalization:${index} score=${candidate.score} probability=${fixed(probability, 6)} ${candidateCompactSummary(candidate)}` });
  });
  selection.candidates.forEach((candidate, index) => {
    const weight = selection.probabilities[index] ?? 0;
    log.push({ kind: 'statement_weight', payload: `formalization:${index} weight=${fixed(weight, 6)} ${candidateCompactSummary(candidate)}` });
  });
  const decision = selection.decision;
  if (decision.kind === 'no_candidate') {
    log.push({ kind: 'policy:temperature_selection', payload: 'no_candidate' });
  } else if (decision.kind === 'selected') {
    log.push({
      kind: 'policy:temperature_selection',
      payload: `selected=formalization:${decision.index} probability=${fixed(decision.probability, 6)} margin=${fixed(decision.margin, 6)} epsilon=${fixed(decision.epsilon, 6)} reason=${decision.reason}`,
    });
    if (decision.reason === 'guessed_under_ambiguity') {
      log.push({ kind: 'policy:guessed_under_ambiguity', payload: `selected=formalization:${decision.index}` });
    }
  } else {
    log.push({
      kind: 'policy:clarify_under_ambiguity',
      payload: `top=formalization:${decision.top} runner_up=formalization:${decision.runnerUp} margin=${fixed(decision.margin, 6)} epsilon=${fixed(decision.epsilon, 6)}`,
    });
  }
}

/**
 * Mirrors the `(role, anchor kind)` match of `fn record_formalization`.
 * @param {string} role
 * @param {string} anchorKind
 * @returns {string}
 */
export function formalizationSlotKind(role, anchorKind) {
  if (anchorKind === 'wikidata_item') {
    if (role === 'subject') return 'formalization:subject_q';
    if (role === 'object') return 'formalization:object_q';
    return 'formalization:item_q';
  }
  if (anchorKind === 'wikidata_property') return role === 'predicate' ? 'formalization:predicate_p' : 'formalization:property_p';
  if (anchorKind === 'wikipedia_article' || anchorKind === 'wiktionary_entry') return 'formalization:fallback';
  return 'formalization:raw';
}

/** Mirrors `fn record_formalization` in rust/src/solver_formalization.rs. */
export function recordFormalization(log, candidate) {
  if (!candidate.slots.length) return;
  log.push({ kind: 'formalization', payload: candidateCompactSummary(candidate) });
  for (const slot of candidate.slots) log.push({ kind: formalizationSlotKind(slot.role, slot.anchor.kind), payload: slot.anchor.id });
  for (const term of candidate.unresolved_terms) log.push({ kind: 'formalization_unresolved', payload: term });
}
