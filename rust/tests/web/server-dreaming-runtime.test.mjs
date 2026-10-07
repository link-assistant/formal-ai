// The default-on background dreaming runtime of the JavaScript server
// (js/server/dreaming-runtime.mjs, js/server/dreaming-runtime-worker.mjs),
// case for case against rust/tests/unit/dreaming_runtime.rs and
// rust/tests/unit/memory_maintenance.rs
// `core_background_dreaming_learns_but_does_not_free_without_consent`
// (issue #540: R419, R427, R538).

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { exportLinksNotation, memoryEvent, parseLinksNotation } from '../../../js/server/memory-store.mjs';
import {
  FOREGROUND, beginForegroundActivity, composeRecipeWithAmendments, coreIsIdle, dreamingDisabled, learningCycleRecordPath,
  recipePath, runCoreDreamingOnce, runOnDreamingThread, startCoreDreaming,
} from '../../../js/server/dreaming-runtime.mjs';
import { googleTrendsLearningCycle, learningCycleLinksNotation } from '../../../js/server/learning-cycle.mjs';
import { persistAutoFreeSpaceChoice } from '../../../js/server/storage-policy.mjs';
import { memoizedReplay, replayAnswerWithAmendments } from '../../../js/server/dreaming-replay.mjs';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const RULE = 'Always include a runnable test with Rust changes.';

function tempMemory(name) {
  return path.join(fs.mkdtempSync(path.join(os.tmpdir(), `formal-ai-dreaming-runtime-${name}-`)), 'memory.lino');
}

const requirementOnly = () => [memoryEvent({
  id: 'req-1',
  kind: 'requirement',
  role: 'user',
  content: 'latex proofs: Always include a LaTeX verification step.',
  conversation_title: 'latex',
  write_count: 1,
})];

test('foreground activity blocks idleness until released', () => {
  const guard = beginForegroundActivity();
  assert.equal(coreIsIdle(0), false);
  guard.end();
  guard.end();
  assert.equal(Atomics.load(FOREGROUND, 0), 0, 'ending twice counts the request out once');
  assert.equal(coreIsIdle(3600), false, 'release stamps the idle clock');
  assert.equal(coreIsIdle(0), true);
});

test('the FORMAL_AI_DREAMING opt-out honours only explicit off values', () => {
  for (const [value, disabled] of [
    [undefined, false], ['1', false], ['on', false], ['true', false], ['maybe', false],
    ['0', true], ['off', true], ['OFF', true], ['false', true], ['False', true], ['no', true],
  ]) {
    assert.equal(dreamingDisabled(value === undefined ? {} : { FORMAL_AI_DREAMING: value }), disabled, String(value));
  }
});

test('a foreground request cancels a run mid-flight without touching the log', () => {
  const file = tempMemory('yield');
  fs.writeFileSync(file, exportLinksNotation(requirementOnly(), 2));
  const before = fs.readFileSync(file, 'utf8');
  const guard = beginForegroundActivity();
  try {
    const outcome = runCoreDreamingOnce(file);
    assert.equal(outcome.learned_amendments, 0);
    assert.equal(outcome.removed_events, 0);
  } finally {
    guard.end();
  }
  assert.equal(fs.readFileSync(file, 'utf8'), before);
  assert.equal(fs.existsSync(recipePath(file)), false);
});

test('an idle run learns but does not free without consent, and writes the composed recipe', () => {
  const file = tempMemory('learn');
  const input = 'refactor rust parser safely';
  const payload = 'same public cache';
  const cache = (id) => memoryEvent({
    id, kind: 'source:http', role: 'cache', tool: 'web_search', content: payload, outputs: payload,
    evidence: ['rediscover:https://doc.rust-lang.org/std/'], write_count: 1,
  });
  fs.writeFileSync(file, exportLinksNotation([
    memoryEvent({ id: 'req-core', kind: 'message', role: 'user', content: RULE, conversation_title: 'rust', write_count: 1 }),
    memoryEvent({
      id: 'run-core', kind: 'test_run', role: 'derived', content: input, inputs: input, conversation_title: 'rust', write_count: 1,
      outputs: replayAnswerWithAmendments(input, [{ id: 'run-core-amendment', topic: 'rust', rule: RULE }]),
    }),
    cache('duplicate-a'),
    cache('duplicate-b'),
  ], 1));
  persistAutoFreeSpaceChoice(file, false);

  const outcome = runCoreDreamingOnce(file, { replay: memoizedReplay() });
  assert.equal(outcome.removed_events, 0);
  assert.equal(outcome.learned_amendments, 1);
  const text = fs.readFileSync(file, 'utf8');
  assert.ok(!text.includes('schema_version'), 'a background write keeps the released schema');
  const events = parseLinksNotation(text);
  assert.ok(events.some((event) => event.id === 'duplicate-a'));
  assert.ok(events.some((event) => event.id === 'duplicate-b'));
  const amendment = events.find((event) => event.kind === 'meta_algorithm_amendment');
  assert.equal(amendment.inputs, 'topic=rust');
  assert.equal(fs.readFileSync(recipePath(file), 'utf8'), composeRecipeWithAmendments(events));
  assert.ok(fs.readFileSync(recipePath(file), 'utf8').endsWith(
    `\n  meta_amendment ${amendment.id}\n    topic: rust\n    rule: ${RULE}\n    applied_by: fn_solve_with_standing_requirements\n\n`,
  ));
});

test('a missing log is never created, yet the run leaves its ledger and learning-cycle record; an incompatible one is refused unmodified', () => {
  // rust/tests/unit/issue_701_learning_adoption.rs: every idle run leaves the
  // proposal-only record, the same artifact `learn cycle --dry-run` prints.
  const missing = tempMemory('missing');
  assert.equal(runCoreDreamingOnce(missing).learned_amendments, 0);
  assert.deepEqual(fs.readdirSync(path.dirname(missing)).sort(), [
    'memory.anticipation.lino', 'memory.anticipation.lino.lock', 'memory.learning-cycle.lino', 'memory.learning-cycle.lino.lock',
  ]);
  assert.equal(fs.readFileSync(learningCycleRecordPath(missing), 'utf8'), `${learningCycleLinksNotation(googleTrendsLearningCycle())}\n`);

  const future = tempMemory('future');
  const text = 'demo_memory\n  schema_version "99"\n  event "future"\n    content "untouched"\n';
  fs.writeFileSync(future, text);
  assert.throws(() => runCoreDreamingOnce(future));
  assert.equal(fs.readFileSync(future, 'utf8'), text);
});

test('the recipe sidecar replaces the memory log extension like Path::with_extension', () => {
  assert.equal(recipePath('/data/memory.lino'), '/data/memory.recipe.lino');
  assert.equal(recipePath('/data/memory'), '/data/memory.recipe.lino');
});

test('the dreaming thread runs a pass and shares the foreground counter', async () => {
  const file = tempMemory('thread');
  fs.writeFileSync(file, exportLinksNotation(requirementOnly(), 2));
  const guard = beginForegroundActivity();
  try {
    assert.equal((await runOnDreamingThread(file)).learned_amendments, 0, 'a live request cancels the threaded run');
  } finally {
    guard.end();
  }
  const outcome = await runOnDreamingThread(file);
  assert.equal(outcome.learned_amendments, 1);
  assert.ok(parseLinksNotation(fs.readFileSync(file, 'utf8')).some((event) => event.kind === 'meta_algorithm_amendment'));
});

test('startCoreDreaming waits for idleness, runs once, and is off on an explicit opt-out', async () => {
  assert.equal(startCoreDreaming({ env: { FORMAL_AI_DREAMING: 'off' }, memoryPath: '/unused' }), null);
  const ran = [];
  let handle;
  // The loop's own timers are unrefed; this one keeps the test process alive.
  const keepAlive = setTimeout(() => {}, 10000);
  await new Promise((resolve) => {
    handle = startCoreDreaming({
      env: {},
      memoryPath: '/data/memory.lino',
      idleSeconds: 0,
      intervalSeconds: 3600,
      run: async (memoryPath) => {
        ran.push(memoryPath);
        resolve();
      },
    });
  });
  handle.stop();
  clearTimeout(keepAlive);
  assert.deepEqual(ran, ['/data/memory.lino']);
});

test('serve starts the dreaming worker and every request guards the idle clock', () => {
  const main = fs.readFileSync(path.join(REPO, 'js/server/main.mjs'), 'utf8');
  const http = fs.readFileSync(path.join(REPO, 'js/server/http.mjs'), 'utf8');
  const started = main.indexOf('startCoreDreaming({ env: process.env, memoryPath: memoryPath(process.env) });');
  assert.ok(started > 0 && started < main.indexOf('await startServer(options)'));
  assert.ok(http.includes('const foreground = beginForegroundActivity();'));
  assert.ok(http.includes('foreground.end();'));
});
