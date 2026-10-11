// The dreaming learning loop the JavaScript server plans with
// (js/server/dreaming-plan.mjs, js/server/dreaming-learning.mjs,
// js/server/dreaming-retention.mjs), case for case against the Rust tests in
// rust/tests/unit/memory_maintenance.rs and rust/tests/unit/memory_learning.rs
// (issue #540: R409, R414–R417, R424, R426, R538, R539, R542, R543).

import assert from 'node:assert/strict';
import test from 'node:test';

import { memoryEvent } from '../../../js/server/memory-store.mjs';
import { ACTION_KIND, applyDreamingPlan, dreamingConfig, planMemoryDreaming } from '../../../js/server/dreaming-plan.mjs';
import { DURABILITY } from '../../../js/server/dreaming-retention.mjs';
import { memoizedReplay, replayAnswerWithAmendments } from '../../../js/server/dreaming-replay.mjs';

/** rust/tests/unit/memory_learning.rs `requirement_event`. */
function requirementEvent(id, topic, statement) {
  return memoryEvent({ id, kind: 'message', role: 'user', content: statement, conversation_title: topic });
}

/** rust/tests/unit/memory_learning.rs `verified_task_run_event`: the output is the production replay. */
function verifiedTaskRunEvent(id, topic, input, requirement) {
  const outputs = replayAnswerWithAmendments(input, [{ id: `${id}-amendment`, topic, rule: requirement }]);
  return memoryEvent({
    id, kind: 'test_run', role: 'derived', content: input, inputs: input, outputs, conversation_title: topic,
  });
}

/** rust/tests/unit/memory_learning.rs `recomputable_event`. */
function recomputableEvent(id, payload) {
  return memoryEvent({
    id,
    kind: 'source:http',
    role: 'cache',
    tool: 'web_search',
    content: payload,
    outputs: payload,
    evidence: ['rediscover:https://doc.rust-lang.org/std/'],
  });
}

const PRESSURE = dreamingConfig({ storage_capacity_bytes: 1000, free_bytes: 0 });
const plan = (events, config = dreamingConfig()) => planMemoryDreaming(events, config, memoizedReplay());
const observed = (dreamed, id) => dreamed.observations.find((observation) => observation.event_id === id);
const forgets = (dreamed, id) => dreamed.actions
  .some((action) => action.event_id === id && action.kind === ACTION_KIND.ForgetCoveredSpecific);

test('dreaming is default-on and targets 20% free space (R538, R543)', () => {
  assert.deepEqual(dreamingConfig(), {
    daydreaming_enabled: true,
    target_free_ratio_percent: 20,
    storage_capacity_bytes: null,
    free_bytes: null,
    incoming_bytes: 0,
  });
  // capacity 100000 → 20% target = 20000; free 20500 with 1000 incoming leaves 500 to reclaim.
  const dreamed = plan([], dreamingConfig({ storage_capacity_bytes: 100000, free_bytes: 20500, incoming_bytes: 1000 }));
  assert.equal(dreamed.target_free_bytes, 20000);
  assert.equal(dreamed.required_reclaim_bytes, 500);
  assert.equal(dreamed.requires_bigger_storage, true);
});

test('planning is pure: the plan never mutates the events it read (R539)', () => {
  const payload = 'same cache payload'.repeat(20);
  const events = [recomputableEvent('cache-a', payload), recomputableEvent('cache-b', payload)];
  const before = JSON.stringify(events);
  const dreamed = plan(events, PRESSURE);
  assert.ok(dreamed.actions.length > 0);
  assert.equal(JSON.stringify(events), before);
});

test('requirement learning reads the multilingual data cues (R409)', () => {
  const rule = 'Всегда добавляй проверку результата.';
  const dreamed = plan([
    requirementEvent('req-ru', 'доказательства', rule),
    verifiedTaskRunEvent('run-ru', 'доказательства', 'Реши новую задачу', rule),
  ]);
  assert.deepEqual(dreamed.learned_requirements.map((requirement) => [requirement.topic, requirement.statement]),
    [['доказательства', rule]]);
});

test('multilingual topics receive distinct stable amendment ids (R409)', () => {
  const dreamed = plan([
    requirementEvent('req-proof', '证明', '始终添加验证步骤。'),
    verifiedTaskRunEvent('run-proof', '证明', '解释归纳法', '始终添加验证步骤。'),
    requirementEvent('req-code', '编码', '始终添加测试步骤。'),
    verifiedTaskRunEvent('run-code', '编码', '重构解析器', '始终添加测试步骤。'),
  ]);
  assert.equal(dreamed.amendments.length, 2);
  assert.notEqual(dreamed.amendments[0].id, dreamed.amendments[1].id);
});

const LATEX_RULE = 'Always include a LaTeX verification step in proof solutions.';

test('only a replay-verified specific is covered and forgettable (R414)', () => {
  const dreamed = plan([
    requirementEvent('req-1', 'latex', LATEX_RULE),
    verifiedTaskRunEvent('run-verified', 'latex', 'Explain a latex proof by induction', LATEX_RULE),
    memoryEvent({
      id: 'run-unverified',
      kind: 'test_run',
      role: 'assistant',
      inputs: 'Explain a contradiction proof',
      outputs: 'an unrelated output that replay cannot reproduce',
      content: 'unverified test run',
      conversation_title: 'latex',
    }),
  ], PRESSURE);
  assert.equal(observed(dreamed, 'run-verified').covered_by_amendment, true);
  assert.equal(observed(dreamed, 'run-unverified').covered_by_amendment, false);
  assert.equal(forgets(dreamed, 'run-unverified'), false);
});

test('frequent-topic tasks are simulated with pass/fail evidence and recurring structures are mined (R415, R416)', () => {
  const rule = 'Always include a runnable test with Rust changes.';
  const events = [
    requirementEvent('req-1', 'rust', rule),
    verifiedTaskRunEvent('run-1', 'rust', 'refactor rust parser safely', rule),
    verifiedTaskRunEvent('run-2', 'rust', 'refactor rust renderer safely', rule),
  ];
  const dreamed = plan(events);
  assert.deepEqual(dreamed.candidate_tasks.map((candidate) => [candidate.topic, candidate.source_event_id, candidate.passed]),
    [['rust', 'run-1', true], ['rust', 'run-2', true]]);
  assert.deepEqual(dreamed.patterns.map((pattern) => [pattern.topic, pattern.occurrences, pattern.structure]),
    [['rust', 2, 'refactor rust * safely']]);

  // The mined structure is retained as learning, once.
  const applied = applyDreamingPlan(events, dreamed, memoizedReplay());
  assert.equal(applied.outcome.learned_patterns, 1);
  const again = applyDreamingPlan(applied.events, plan(applied.events), memoizedReplay());
  assert.equal(again.outcome.learned_patterns, 0);
  assert.equal(again.events.filter((event) => event.kind === 'dreaming_pattern').length, 1);
});

test('adding a requirement revokes coverage and preserves the stale specific (R424)', () => {
  const events = [
    requirementEvent('req-1', 'latex', LATEX_RULE),
    verifiedTaskRunEvent('run-1', 'latex', 'Explain a latex proof by induction', LATEX_RULE),
  ];
  const before = plan(events, PRESSURE);
  assert.equal(observed(before, 'run-1').covered_by_amendment, true);
  assert.equal(forgets(before, 'run-1'), true);

  const second = 'Always cite the numbered theorem being applied.';
  const after = plan([...events, requirementEvent('req-2', 'latex', second)], PRESSURE);
  const amendment = after.amendments.find((entry) => entry.topic === 'latex');
  assert.ok(amendment.rule.includes(LATEX_RULE), amendment.rule);
  assert.ok(amendment.rule.includes(second), amendment.rule);
  assert.equal(observed(after, 'run-1').covered_by_amendment, false);
  assert.equal(forgets(after, 'run-1'), false);
});

test('a failed replay never makes an original observation disposable (R424)', () => {
  const dreamed = plan([
    requirementEvent('req-1', 'latex', LATEX_RULE),
    verifiedTaskRunEvent('run-covered', 'latex', 'Explain a latex proof by induction', LATEX_RULE),
    memoryEvent({
      id: 'run-unverified',
      kind: 'test_run',
      role: 'assistant',
      inputs: 'Explain a latex contradiction proof',
      outputs: 'an unrelated output that replay cannot reproduce',
      content: 'unverified test run',
      conversation_title: 'latex',
    }),
    recomputableEvent('public-cache', 'refetchable public source payload'),
  ], dreamingConfig({ storage_capacity_bytes: 1000000, free_bytes: 0 }));
  const position = (id) => dreamed.actions.findIndex((action) => action.event_id === id);
  assert.equal(position('run-unverified'), -1);
  assert.equal(position('req-1'), -1);
  assert.equal(dreamed.actions[position('run-covered')].kind, ACTION_KIND.ForgetCoveredSpecific);
  assert.ok(position('run-covered') < position('public-cache'));
});

test('a numeric pattern on a top topic synthesizes a new retained trial (R426)', () => {
  const task = (id, input) => memoryEvent({
    id, kind: 'test_run', role: 'assistant', inputs: input, outputs: 'done', content: input, conversation_title: 'math',
  });
  const events = [task('run-1', 'add 1 2'), task('run-2', 'add 3 4')];
  const dreamed = plan(events);
  assert.ok(dreamed.patterns.some((pattern) => pattern.topic === 'math' && pattern.structure === 'add # #'));
  const trial = dreamed.synthesized_tasks.find((entry) => entry.topic === 'math');
  assert.equal(trial.input, 'add 2 3');
  assert.notEqual(trial.answer, '');
  const applied = applyDreamingPlan(events, dreamed, memoizedReplay());
  assert.equal(applied.outcome.recorded_trials, 1);
  assert.ok(applied.events.some((event) => event.kind === 'dreaming_trial' && event.inputs === 'add 2 3'));
});

for (const [language, rule, input, kind, topic] of [
  ['Russian', 'Всегда добавляй проверку результата.', 'Реши задачу по индукции', 'проверка', 'доказательства'],
  ['Hindi', 'हमेशा परिणाम की जाँच जोड़ें।', 'आगमन द्वारा प्रमाण हल करें', 'परीक्षण', 'प्रमाण'],
]) {
  test(`a ${language}-kind run is replayed as a candidate through the data lexicon (R426)`, () => {
    const outputs = replayAnswerWithAmendments(input, [{ id: 'amendment', topic, rule }]);
    const dreamed = plan([
      requirementEvent('req', topic, rule),
      memoryEvent({ id: 'run', kind, role: 'assistant', inputs: input, outputs, conversation_title: topic }),
    ]);
    const candidate = dreamed.candidate_tasks.find((entry) => entry.source_event_id === 'run');
    assert.equal(candidate.passed, true, candidate.simulated_output);
  });
}

test('usage recalculation covers cached and seed links (R417)', () => {
  const seed = (id, content) => memoryEvent({
    id, kind: 'seed_cache', role: 'cache', content, evidence: ['rediscover:https://doc.rust-lang.org/std/'],
  });
  const dreamed = plan([
    seed('seed-unused', 'reconstructable seeded catalog entry'.repeat(20)),
    seed('seed-used', 'another seeded catalog entry'.repeat(20)),
    memoryEvent({ id: 'task-reference', kind: 'task', content: 'solve using seed-used', evidence: ['seed-used'] }),
  ], PRESSURE);
  assert.equal(observed(dreamed, 'seed-unused').usage_count, 1);
  assert.ok(observed(dreamed, 'seed-used').usage_count > 1);
  const unused = dreamed.actions.findIndex((action) => action.event_id === 'seed-unused');
  const used = dreamed.actions.findIndex((action) => action.event_id === 'seed-used');
  assert.ok(unused >= 0);
  assert.ok(used === -1 || unused < used);
});

test('deleted-thread data, refetchable caches and replay-proved intermediates are the reclaimable tiers (R542)', () => {
  const dreamed = plan([
    memoryEvent({ id: 'raw', kind: 'message', role: 'user', content: 'irreplaceable user experience '.repeat(20) }),
    memoryEvent({ id: 'gone', kind: 'message', role: 'user', content: 'forget me', conversation_id: 'conv-gone' }),
    memoryEvent({ id: 'gone-marker', kind: 'conversation_deleted', role: 'system', conversation_id: 'conv-gone' }),
    memoryEvent({ id: 'unproved', kind: 'intermediate_conclusion', content: 'an intermediate conclusion '.repeat(20) }),
    requirementEvent('req-1', 'latex', LATEX_RULE),
    verifiedTaskRunEvent('run-1', 'latex', 'Explain a latex proof by induction', LATEX_RULE),
    recomputableEvent('external-cache', 'cached public source '.repeat(20)),
  ], dreamingConfig({ storage_capacity_bytes: 1000000, free_bytes: 0 }));
  assert.equal(observed(dreamed, 'gone').durability, DURABILITY.DeletedConversation);
  assert.equal(observed(dreamed, 'external-cache').durability, DURABILITY.RecomputableCache);
  // A derived record is an intermediate only once replay re-derives it; an
  // intermediate without that proof is retained like raw experience.
  assert.equal(observed(dreamed, 'run-1').durability, DURABILITY.RecomputableIntermediate);
  assert.equal(observed(dreamed, 'unproved').durability, DURABILITY.IrreplaceableRaw);
  assert.equal(observed(dreamed, 'raw').durability, DURABILITY.IrreplaceableRaw);
  assert.deepEqual(dreamed.actions.map((action) => [action.kind, action.event_id]), [
    [ACTION_KIND.PurgeDeletedConversation, 'gone'],
    [ACTION_KIND.PurgeDeletedConversation, 'gone-marker'],
    [ACTION_KIND.ForgetCoveredSpecific, 'run-1'],
    [ACTION_KIND.EvictLowUseRecomputable, 'external-cache'],
  ]);
});
