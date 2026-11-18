// Issue #1550: one recursive procedure produces every answer. At every level
// it splits the request into tasks, derives a test per task, drafts more than
// one candidate per test, selects the smallest sufficient passing candidate,
// composes the selected drafts against the parent test, and recurses within
// configurable depth, breadth and budget bounds. The shared fixture
// rust/tests/fixtures/meta-executor-parity.json is replayed here and by the
// native rust/tests/unit/meta_executor_parity.rs, so both runtimes emit the
// same event sequence for the same request (acceptance criterion e).
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let CASES;
let quotationCases;
let replayCase;
let executor;
const fixture = JSON.parse(
  readFileSync(new URL('../fixtures/meta-executor-parity.json', import.meta.url), 'utf8'),
);
const statusFixture = JSON.parse(
  readFileSync(new URL('../fixtures/meta-executor-status-parity.json', import.meta.url), 'utf8'),
);
assert.equal(statusFixture.schema, fixture.schema);
fixture.cases.push(...statusFixture.cases);
const named = (name) => fixture.cases.find((entry) => entry.name === name);
const kinds = (events) => events.map((event) => event.split(' ')[0]);

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ CASES, QUOTATION_CASES: quotationCases, replayCase } = await import('./helpers/issue-1550-meta-executor-cases.mjs'));
  executor = await import('../../../js/agentic/crate/meta_executor.mjs');
});

test('the shared fixture is current: every case replays to its recorded result', async () => {
  assert.equal(fixture.schema, 'meta-executor-parity/v1');
  assert.deepEqual(fixture.cases.map((entry) => entry.name), CASES.map((entry) => entry.name));
  for (const entry of fixture.cases) {
    const { expected, ...input } = entry;
    assert.deepEqual(await replayCase(input), expected, entry.name);
  }
});

test('a two-level split shows tasks, tests, drafts, selection and composition at both levels', () => {
  const { events, shape, status } = named('two-level-en').expected;
  assert.equal(shape, 'sequence(leaf,aggregation(leaf,leaf))');
  assert.equal(status, 'resolved');
  for (const task of ['t0', 't0.2']) {
    for (const kind of ['task', 'test', 'compose', 'select']) {
      assert.ok(events.some((event) => event.startsWith(`meta_exec:${kind} `)
        && (event.includes(`task=${task} `) || event.includes(`id=${task} `))), `${kind} at ${task}`);
    }
    const drafts = events.filter((event) => event.startsWith(`meta_exec:draft task=${task} `));
    assert.ok(drafts.length > 1, `more than one draft at ${task}`);
  }
  assert.ok(events.includes(
    'meta_exec:select task=t0.2 method=composition reason=least_action steps=5 code_units=7 passing=1/2'));
  assert.equal(events.at(-1), 'meta_exec:result task=t0 status=resolved method=composition budget_left=3');
});

test('removing a sub-task test yields an explicit unresolved result', () => {
  const full = named('two-level-en').expected;
  const withheld = named('withheld-sub-task-test').expected;
  assert.notEqual(withheld.status, full.status);
  assert.equal(withheld.content, null);
  assert.ok(withheld.events.includes('meta_exec:unresolved task=t0.2.1 reason=no_test'));
  assert.ok(withheld.events.includes('meta_exec:unresolved task=t0 reason=children'));
  assert.equal(named('withheld-sub-task-test-zh').expected.status, 'unresolved');
});

test('held-out ru, hi, zh and es requests decompose like their English form', () => {
  const english = named('two-level-en').expected;
  for (const language of ['ru', 'hi', 'zh', 'es']) {
    const other = named(`two-level-${language}`).expected;
    assert.equal(other.shape, english.shape, language);
    assert.equal(other.leaves.length, english.leaves.length, language);
    assert.equal(other.status, english.status, language);
    const strip = (events) => events.map((event) => event.replace(/ text=.*$/u, '')
      .replace(/ code_units=[0-9]+/u, ''));
    assert.deepEqual(strip(other.events), strip(english.events), language);
  }
});

test('selection is by least action and records the real reason', () => {
  const direct = named('direct-derivation-wins-when-it-passes').expected;
  assert.ok(direct.events.includes(
    'meta_exec:select task=t0 method=derivation reason=least_action steps=2 code_units=16 passing=2/2'));
  const cached = named('cache-row-is-the-least-action').expected;
  assert.ok(cached.events.some((event) => event.startsWith('meta_exec:select task=t0.2.1 method=cache ')));
  const alternative = named('alternative-takes-the-first-passing-sub-task').expected;
  assert.equal(alternative.shape, 'alternative(leaf,leaf)');
  assert.equal(alternative.content, 'Bonjour');
});

test('depth, breadth and budget bounds are configurable and visible in the trace', () => {
  const depth = named('depth-bound').expected;
  assert.equal(depth.events[0], 'meta_exec:bounds depth=1 breadth=6 budget=8');
  assert.equal(depth.shape, 'sequence(leaf,leaf)');
  assert.ok(depth.events.some((event) => event.includes('id=t0.2 ') && event.includes('atomic=depth_bound')));
  const breadth = named('breadth-bound').expected;
  assert.ok(breadth.events.some((event) => event.includes('id=t0 ') && event.includes('atomic=breadth_bound')));
  const budget = named('budget-bound').expected;
  assert.ok(budget.events.includes('meta_exec:draft task=t0.2.2 method=derivation skipped=budget'));
  assert.equal(budget.status, 'unresolved');
  assert.deepEqual(executor.decompositionBounds({ depth: 3 }), { depth: 3, breadth: 6, budget: 8 });
});

test('quoted connectives stay inside one task and an atomic request is one leaf', () => {
  assert.equal(named('quoted-connective-is-not-split').expected.shape, 'leaf');
  const atomic = named('atomic-request').expected;
  assert.deepEqual(kinds(atomic.events), [
    'meta_exec:bounds', 'meta_exec:task', 'meta_exec:test', 'meta_exec:draft',
    'meta_exec:select', 'meta_exec:result',
  ]);
});

test('the split reads its connectives from the grammar, not from code', () => {
  const source = readFileSync(new URL('../../../js/agentic/crate/meta_executor.mjs', import.meta.url), 'utf8');
  for (const word of ['and', 'then', 'or', 'и', 'और', '然后']) {
    assert.doesNotMatch(source, new RegExp(`['"\`]${word}['"\`]`, 'u'), word);
  }
  assert.deepEqual(executor.decompositionRelations(), ['sequence', 'alternative', 'aggregation']);
  assert.deepEqual(executor.splitRequest('A and then B', 'en'), { relation: 'sequence', parts: ['A', 'B'] });
});

test('ASCII quoted connectives stay in their task while apostrophes and escaped quotes keep their boundaries', () => {
  for (const [name, text, relation, parts, language = 'en'] of quotationCases) {
    assert.deepEqual(executor.splitRequest(text, language), { relation, parts }, name);
    const expected = named(name).expected;
    assert.equal(expected.shape, `${relation}(leaf,leaf)`, name);
    assert.deepEqual(expected.leaves, parts, name);
    assert.equal(expected.status, 'resolved', name);
  }
});

test('unverified drafts cannot satisfy a leaf test or complete an all-required parent', async () => {
  for (const name of ['unverified-program-leaf', 'unverified-program-child', 'unverified-program-cache']) {
    assert.equal(named(name).expected.status, 'unresolved', name);
    assert.equal(named(name).expected.content, null, name);
  }
  assert.equal(named('verified-program-child').expected.status, 'resolved');
  for (const intent of [
    'meta_reasoned_program_unverified',
    'coding_workspace_written_unverified',
    'general_plan_unverified',
  ]) {
    assert.equal(executor.answerValidation(intent, 'A nonempty draft'), `answered(${intent})=false`, intent);
  }
  for (const intent of ['meta_reasoned_program_unverified', 'meta_reasoned_program']) {
    const run = await executor.executeProcedure({ text: 'First task and second task', language: 'en' }, {
      cached: async () => null,
      draft: async (text) => text === 'First task'
        ? { intent, content: 'A nonempty program', confidence: 1 }
        : text === 'second task'
          ? { intent: 'answered', content: 'Second result', confidence: 1 }
          : null,
    });
    assert.equal(run.outcome.status, intent.endsWith('_unverified') ? 'unresolved' : 'resolved', intent);
  }
});
