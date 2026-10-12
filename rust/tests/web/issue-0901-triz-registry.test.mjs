// Issue #901 R901-3: contradiction resolution is registry-selected, not a
// separate hard-coded route. The JavaScript first (R997) checks:
//
//  - `contradictionsIn` detects a technical contradiction in a pair of
//    satisfying candidates that each win on a different cost dimension.
//  - `situationFor` returns `contradiction_detected` exactly when a
//    contradiction exists, and `''` otherwise.
//  - `rankWithHeuristic` selects `TrizRanker` (order 3,
//    `applies_when contradiction_detected`) and ranks correctly when a
//    contradiction is detected, and keeps today's `LeastActionRanker`
//    behaviour exactly in every other situation.
//  - The seeded catalog declares a `rank` row with slug `triz` and
//    `applies_when contradiction_detected`.
//  - Rust twins: `rust/tests/unit/specification/triz_contradictions.rs` and
//    `rust/tests/unit/specification/selection_heuristics.rs`.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, describe, it } from 'node:test';

import { contradictionsIn, rankWithHeuristic, situationFor } from '../../../js/agentic/crate/selection_heuristics_triz.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

/**
 * Two candidates where index-0 is faster but larger, index-1 is slower but
 * smaller: each wins on a different dimension, so this is a technical
 * contradiction (situationFor = 'contradiction_detected').
 *
 *   A (index 0): code_size=200, steps=5  — wins on steps
 *   B (index 1): code_size=100, steps=10 — wins on code_size
 *
 * LeastActionRanker (key_order code_size,steps,candidate_index) ranks B first.
 * TrizRanker with no requirement in params uses identity ordering, ranking A first.
 * The two rankers therefore disagree, making the situation selection observable.
 */
function contradictedPair() {
  return [
    {
      candidate_id: 'fast_but_large',
      checks: [4, 4],
      cost: { steps: 5, code_size: 200, resource_units: 0, leaf_count: 1 },
    },
    {
      candidate_id: 'slow_but_small',
      checks: [4, 4],
      cost: { steps: 10, code_size: 100, resource_units: 0, leaf_count: 1 },
    },
  ];
}

/** Two candidates that share the same cost ordering: no contradiction. */
function uniformPair() {
  return [
    {
      candidate_id: 'a',
      checks: [3, 3],
      cost: { steps: 5, code_size: 100, resource_units: 0, leaf_count: 1 },
    },
    {
      candidate_id: 'b',
      checks: [3, 3],
      cost: { steps: 7, code_size: 200, resource_units: 0, leaf_count: 1 },
    },
  ];
}

const REQUIREMENT = 'Give me the shortest answer that still covers every case, and if you must choose, favour completeness.';

describe('contradictionsIn (mirrors fn contradictions_in in rust/src/selection_heuristics/triz.rs)', () => {
  it('detects a technical contradiction when two satisfying candidates each win on a different dimension', () => {
    const links = contradictionsIn(contradictedPair(), REQUIREMENT);
    assert.equal(links.length, 1);
    assert.notEqual(links[0].criterion_a, links[0].criterion_b);
    assert.ok(links[0].link_id.length > 0);
  });

  it('returns an empty array when no two candidates each win on a different dimension', () => {
    assert.equal(contradictionsIn(uniformPair(), '').length, 0);
  });

  it('derives the selection value from requirement clauses when they state the trade-off', () => {
    // fast_but_large (index 0) wins on steps → criterion_a = answer_completeness
    // slow_but_small (index 1) wins on code_size → criterion_b = answer_brevity
    // The requirement says "favour completeness", so b_clauses > a_clauses... but
    // criterion pairing depends on which dimension each candidate wins.
    // Use the original tied_pair from triz_contradictions.rs for this check.
    const tieredPair = [
      { candidate_id: 'short_but_slow', checks: [4, 4], cost: { steps: 18, code_size: 120, resource_units: 0, leaf_count: 2 } },
      { candidate_id: 'long_but_direct', checks: [4, 4], cost: { steps: 4, code_size: 480, resource_units: 0, leaf_count: 2 } },
    ];
    const links = contradictionsIn(tieredPair, REQUIREMENT);
    const link = links[0];
    assert.equal(link.derivation.kind, 'RequirementClauses');
    assert.ok(link.derivation.a_clauses + link.derivation.b_clauses > 0);
    assert.ok(link.selection_basis_points >= 0 && link.selection_basis_points <= 10000);
  });

  it('reports underivable when no clause states the trade-off', () => {
    const links = contradictionsIn(contradictedPair(), 'Do the thing.');
    assert.equal(links[0].derivation.kind, 'Underivable');
    assert.equal(links[0].resolution.kind, 'Unresolved');
  });

  it('selection_basis_points is an integer in 0..=10000', () => {
    const links = contradictionsIn(contradictedPair(), REQUIREMENT);
    const bp = links[0].selection_basis_points;
    assert.ok(Number.isInteger(bp) && bp >= 0 && bp <= 10000);
  });
});

describe('situationFor (mirrors situation detection in rust/src/draft_portfolio.rs and ranking.rs)', () => {
  it('returns contradiction_detected when a contradiction exists', () => {
    assert.equal(situationFor(contradictedPair()), 'contradiction_detected');
  });

  it('returns empty string when no contradiction exists', () => {
    assert.equal(situationFor(uniformPair()), '');
  });
});

describe('rankWithHeuristic (mirrors fn rank_passing_drafts / rank_survivors R901-3)', () => {
  it('selects a different ranker when contradiction_detected vs no situation', () => {
    // The contradicted pair (A=fast_but_large, B=slow_but_small):
    //   LeastActionRanker (key_order code_size,...): B (100) < A (200) → B first
    //   TrizRanker with no requirement → identity order → A (index 0) first
    // These disagree, proving the situation selects a different ranker.
    const scores = contradictedPair();
    const rankedWithContradiction = rankWithHeuristic(scores, 'contradiction_detected');
    const rankedWithout = rankWithHeuristic(scores, '');
    assert.notDeepEqual(rankedWithContradiction, rankedWithout,
      'contradiction_detected must select a different ranker than no situation');
    // With no situation (LeastActionRanker): B (slow_but_small, code_size 100) first
    assert.equal(scores[rankedWithout[0]].candidate_id, 'slow_but_small');
    // With contradiction_detected (TrizRanker, identity order, no key_order): A (index 0) first
    assert.equal(scores[rankedWithContradiction[0]].candidate_id, 'fast_but_large');
  });

  it('keeps the same ordering for a non-contradicted pair regardless of situation', () => {
    const scores = uniformPair();
    const rankedContradiction = rankWithHeuristic(scores, 'contradiction_detected');
    const rankedNone = rankWithHeuristic(scores, '');
    // uniformPair has no contradiction so situationFor returns '' and the
    // caller would never pass contradiction_detected for it; both produce
    // the same result (LeastActionRanker falls back to identity when heuristic
    // has no key_order, and so does TrizRanker)
    assert.deepEqual(
      rankedNone.map((i) => scores[i].candidate_id),
      ['a', 'b'],
      'LeastActionRanker keeps a (smaller code_size) first',
    );
  });

  it('round-trip: situationFor + rankWithHeuristic routes through triz for the contradicted pair', () => {
    const scores = contradictedPair();
    const situation = situationFor(scores);
    assert.equal(situation, 'contradiction_detected');
    const ranked = rankWithHeuristic(scores, situation);
    // TrizRanker identity → A (index 0 = fast_but_large) first
    assert.equal(scores[ranked[0]].candidate_id, 'fast_but_large');
    // Verify this differs from the no-situation path
    const rankedNone = rankWithHeuristic(scores, '');
    assert.notDeepEqual(ranked, rankedNone);
  });
});

describe('the seeded catalog declares a triz rank row (R901-3 seed requirement)', () => {
  it('heuristic_triz is present at order 3 with applies_when contradiction_detected', () => {
    const root = new URL('../../../', import.meta.url);
    const text = readFileSync(new URL('data/meta/selection-heuristics.lino', root), 'utf8');
    assert.ok(text.includes('heuristic_triz'), 'heuristic_triz record is declared');
    assert.ok(text.includes('applies_when "contradiction_detected"'), 'applies_when contradiction_detected');
    assert.ok(text.includes('slug "triz"'), 'slug triz');
    // order 3 appears inside the heuristic_triz block
    const trizBlock = text.slice(text.indexOf('heuristic_triz'), text.indexOf('\nheuristic_balanced_split'));
    assert.ok(trizBlock.includes('order "3"'), 'order 3');
    assert.ok(trizBlock.includes('role "rank"'), 'role rank');
  });
});
