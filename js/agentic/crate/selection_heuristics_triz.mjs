// TRIZ contradictions as links with a 0-1 selection value: a port of
// rust/src/selection_heuristics/triz.rs (`contradictions_in`, `rank`) and
// rust/src/selection_heuristics.rs (`TrizRanker`, `LeastActionRanker`,
// `HeuristicRole`, `CandidateRanker`), plus the registry-based ranker
// selection (`MethodRegistry::heuristics_for`, `HeuristicMethod::parameter`)
// that R901-3 requires.
//
// A `CandidateScore` is `{candidate_id, checks: [satisfied, declared],
// cost: {steps, code_size, resource_units, leaf_count}}`.
// A `ContradictionLink` is `{link_id, criterion_a, criterion_b,
// selection_basis_points, derivation, resolution, evidence}`.
// Derivation: `{kind: 'RequirementClauses', a_clauses, b_clauses}` |
//             `{kind: 'Underivable', reason}`.
// Resolution: `{kind: 'Range'}` | `{kind: 'Unresolved', reason}`.
//
// Situation slug the registry row declares:
//   `contradiction_detected` — two satisfying candidates each win on a
//   different cost dimension; `TrizRanker` resolves the trade-off.

import { cached, readText } from '../host.mjs';
import { stableId } from './engine_stable_id.mjs';
import { normalizePrompt } from './web_engine_core.mjs';
import { mentionsRoleRaw } from './seed_meanings.mjs';
import { parseRoot, findChildValue } from './seed_parser.mjs';
import { trim } from './rust_str.mjs';

// ---- constants --------------------------------------------------------------

/** Mirrors `DIMENSIONS` in rust/src/selection_heuristics/triz.rs. */
const DIMENSIONS = ['code_size', 'steps', 'resource_units', 'leaf_count'];

/** Mirrors `fn criterion_for` in rust/src/selection_heuristics/triz.rs. */
function criterionFor(dimension) {
  if (dimension === 'code_size') return 'answer_brevity';
  if (dimension === 'steps') return 'answer_completeness';
  if (dimension === 'resource_units') return 'resource_economy';
  return 'decomposition_depth';
}

/** Mirrors `fn cue_role_for` in rust/src/selection_heuristics/triz.rs. */
function cueRoleFor(criterion) {
  if (criterion === 'answer_brevity') return 'selection_criterion_brevity_cue';
  if (criterion === 'answer_completeness') return 'selection_criterion_completeness_cue';
  return null;
}

/** Mirrors `fn dimension_of` in rust/src/selection_heuristics/triz.rs. */
function dimensionOf(criterion) {
  return DIMENSIONS.find((d) => criterionFor(d) === criterion) ?? DIMENSIONS[0];
}

/** Mirrors `fn measure` in rust/src/selection_heuristics/triz.rs. */
function measure(score, dimension) {
  if (dimension === 'steps') return score.cost.steps;
  if (dimension === 'code_size') return score.cost.code_size;
  if (dimension === 'resource_units') return score.cost.resource_units;
  if (dimension === 'leaf_count') return score.cost.leaf_count;
  return 0;
}

/** Mirrors `fn clause_spans` in rust/src/selection_heuristics/triz.rs. */
function clauseSpans(requirement) {
  const spans = [];
  let start = 0;
  for (let offset = 0; offset < requirement.length; ) {
    const cp = requirement.codePointAt(offset);
    const ch = String.fromCodePoint(cp);
    const len = cp > 0xffff ? 2 : 1;
    const isBreak = [',', ';', '.', '!', '?', '\u{FF0C}', '\u{FF1B}', '\u{3002}', '\u{FF01}', '\u{FF1F}', '\u{3001}', '\u{0964}', '\u{0965}'].includes(ch);
    if (isBreak) {
      const end = offset + len;
      if (requirement.slice(start, end).trim().length > 1) spans.push([start, end]);
      start = end;
    }
    offset += len;
  }
  if (requirement.slice(start).trim().length > 1) spans.push([start, requirement.length]);
  return spans;
}

/** Mirrors `fn clauses_demanding` in rust/src/selection_heuristics/triz.rs. */
function clausesDemanding(requirement, criterion) {
  const role = cueRoleFor(criterion);
  if (!role) return [];
  return clauseSpans(requirement)
    .filter(([start, end]) => mentionsRoleRaw(role, normalizePrompt(requirement.slice(start, end))))
    .map(([start, end]) => `clause:${start}-${end}`);
}

/** Mirrors `NO_STATED_TRADE_OFF` in rust/src/selection_heuristics/triz.rs. */
const NO_STATED_TRADE_OFF = 'no_clause_demands_either_criterion';

/**
 * Mirrors `fn link_for` in rust/src/selection_heuristics/triz.rs.
 * @param {string} criterionA
 * @param {string} criterionB
 * @param {string} situation
 * @param {string} requirement
 * @returns {object}
 */
function linkFor(criterionA, criterionB, situation, requirement) {
  const aEvidence = clausesDemanding(requirement, criterionA);
  const bEvidence = clausesDemanding(requirement, criterionB);
  const aClauses = aEvidence.length;
  const bClauses = bEvidence.length;
  const linkId = stableId('contradiction', `${criterionA}:${criterionB}:${situation}`);
  if (aClauses + bClauses === 0) {
    return {
      link_id: linkId,
      criterion_a: criterionA,
      criterion_b: criterionB,
      selection_basis_points: 0,
      derivation: { kind: 'Underivable', reason: NO_STATED_TRADE_OFF },
      resolution: { kind: 'Unresolved', reason: NO_STATED_TRADE_OFF },
      evidence: [],
    };
  }
  const total = aClauses + bClauses;
  const basisPoints = Math.min(Math.trunc((bClauses * 10000) / total), 10000);
  return {
    link_id: linkId,
    criterion_a: criterionA,
    criterion_b: criterionB,
    selection_basis_points: basisPoints,
    derivation: { kind: 'RequirementClauses', a_clauses: aClauses, b_clauses: bClauses },
    resolution: { kind: 'Range' },
    evidence: [...aEvidence, ...bEvidence],
  };
}

/**
 * Mirrors `fn contradictions_in` in rust/src/selection_heuristics/triz.rs.
 * @param {Array<object>} scores
 * @param {string} requirement
 * @returns {Array<object>}
 */
export function contradictionsIn(scores, requirement) {
  const satisfying = scores.filter((score) => score.checks[1] > 0 && score.checks[0] === score.checks[1]);
  const situation = stableId('situation', requirement);
  const links = [];
  for (let leftIndex = 0; leftIndex < satisfying.length; leftIndex += 1) {
    for (let rightIndex = leftIndex + 1; rightIndex < satisfying.length; rightIndex += 1) {
      const left = satisfying[leftIndex];
      const right = satisfying[rightIndex];
      const leftWins = DIMENSIONS.find((d) => measure(left, d) < measure(right, d));
      const rightWins = DIMENSIONS.find((d) => measure(right, d) < measure(left, d));
      if (!leftWins || !rightWins) continue;
      const criterionA = criterionFor(leftWins);
      const criterionB = criterionFor(rightWins);
      links.push(linkFor(criterionA, criterionB, situation, requirement));
    }
  }
  return links;
}

// ---- TrizRanker -------------------------------------------------------------

/**
 * Mirrors `TrizRanker::rank` in rust/src/selection_heuristics.rs (via
 * `fn rank` in rust/src/selection_heuristics/triz.rs): resolve detected
 * contradictions by their selection value rather than by the index tie-break.
 * @param {Array<object>} scores
 * @param {Array<Array<string>>} parameters  key-value pairs
 * @returns {Array<number>}
 */
export function trizRank(scores, parameters) {
  const leastActionRanked = leastActionRank(scores, parameters);
  const requirement = (parameters.find(([key]) => key === 'requirement') ?? [])[1] ?? '';
  const links = contradictionsIn(scores, requirement);
  const link = links.find((l) => l.resolution.kind === 'Range');
  if (!link) return leastActionRanked;
  const leaningDimension = link.selection_basis_points > 5000
    ? dimensionOf(link.criterion_b)
    : dimensionOf(link.criterion_a);
  const resolved = [...leastActionRanked];
  resolved.sort((left, right) => measure(scores[left], leaningDimension) - measure(scores[right], leaningDimension));
  return resolved;
}

// ---- LeastActionRanker (local copy for rankWithHeuristic) -------------------

/**
 * Mirrors `fn key_order` in rust/src/selection_heuristics.rs: the declared
 * key order out of a heuristic's parameters, or `[]` when absent.
 * @param {Array<Array<string>>} parameters
 * @returns {Array<string>}
 */
function keyOrder(parameters) {
  const pair = parameters.find(([key]) => key === 'key_order');
  if (!pair) return [];
  return pair[1].split(',').map((part) => trim(part));
}

/**
 * Mirrors `LeastActionRanker::rank` in rust/src/selection_heuristics.rs.
 * @param {Array<object>} scores
 * @param {Array<Array<string>>} parameters
 * @returns {Array<number>}
 */
function leastActionRank(scores, parameters) {
  const ranked = scores
    .map((score, index) => index)
    .filter((index) => scores[index].checks[1] > 0 && scores[index].checks[0] === scores[index].checks[1]);
  const order = keyOrder(parameters);
  if (order.length === 0) return ranked;
  ranked.sort((left, right) => {
    for (const dim of order) {
      const diff = measure(scores[left], dim) - measure(scores[right], dim);
      if (diff !== 0) return diff;
      if (dim === 'candidate_index') return left - right;
    }
    return left - right;
  });
  return ranked;
}

// ---- registry-based ranker selection ----------------------------------------

const HEURISTICS_PATH = 'data/meta/selection-heuristics.lino';
const HEURISTIC_RECORD_TYPE = 'selection_heuristic';
const HEURISTIC_STRUCTURAL_FIELDS = new Set(['record_type', 'role', 'order', 'applies_when']);

/** Mirrors `fn catalog_from` / `fn shipped_catalog` in rust/src/selection_heuristics.rs. */
function shippedHeuristics() {
  return cached('selection-heuristics-triz-catalog', () =>
    (parseRoot(readText(HEURISTICS_PATH)).children || [])
      .filter((record) => findChildValue(record, 'record_type') === HEURISTIC_RECORD_TYPE)
      .map((record) => ({
        name: record.name,
        role: findChildValue(record, 'role'),
        order: Number(findChildValue(record, 'order')),
        applies_when: (record.children || []).filter((f) => f.name === 'applies_when').map((f) => f.id),
        parameters: (record.children || [])
          .filter((f) => !HEURISTIC_STRUCTURAL_FIELDS.has(f.name))
          .map((f) => [f.name, f.id]),
      })));
}

/** Mirrors `MethodRegistry::heuristics_for` in rust/src/method_registry.rs. */
function heuristicsFor(role, situation) {
  return shippedHeuristics()
    .filter((h) => h.role === role && (!h.applies_when.length || h.applies_when.includes(situation)))
    .sort((left, right) => left.order - right.order);
}

/**
 * Mirrors the registry-based ranker selection in `fn rank_passing_drafts`
 * (rust/src/draft_portfolio.rs) and `fn rank_survivors`
 * (rust/src/algorithm_discovery/ranking.rs): resolve `scores` with the
 * most-specific `rank` heuristic that applies in `situation` (R901-3).
 * Mirrors `fn rank_with_heuristic` in rust/src/selection_heuristics.rs, which both call.
 *
 * When `situation` is `contradiction_detected`, `TrizRanker` (order 3) wins
 * over `LeastActionRanker` (order 1). Every other situation keeps today's
 * behaviour exactly.
 * @param {Array<object>} scores
 * @param {string} situation
 * @returns {Array<number>}
 */
export function rankWithHeuristic(scores, situation) {
  const heuristics = heuristicsFor('rank', situation);
  const heuristic = heuristics[heuristics.length - 1];
  const slug = heuristic?.parameters.find(([key]) => key === 'slug')?.[1] ?? '';
  const params = heuristic?.parameters ?? [];
  if (slug === 'triz') return trizRank(scores, params);
  return leastActionRank(scores, params);
}

/**
 * The situation slug for a scored candidate set: `contradiction_detected` when
 * any two satisfying candidates each win on a different cost dimension, else `''`.
 * Mirrors `fn situation_for` in rust/src/selection_heuristics.rs.
 * Mirrors the situation detection in `fn rank_passing_drafts`
 * (rust/src/draft_portfolio.rs) and `fn rank_survivors`
 * (rust/src/algorithm_discovery/ranking.rs).
 * @param {Array<object>} scores
 * @returns {string}
 */
export function situationFor(scores) {
  return contradictionsIn(scores, '').length > 0 ? 'contradiction_detected' : '';
}
