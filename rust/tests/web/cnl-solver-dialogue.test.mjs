// Issue #1584 through the real browser-worker solver: every declared task
// paraphrase in every language reaches a solver answer, every question phrasing
// gets the controlled answer or a clarification, a numbered reply picks a
// reading, and prompts the solver already understood keep their answers.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {createWorkerContext, evaluate, plain} from './support/browser-runtime.mjs';
import {fixtureLanguages, parityPath} from '../../../experiments/issue-1584/parity-fixture.mjs';

const ROOT = path.resolve(import.meta.dirname, '../../..');
const worker = createWorkerContext({fetch: async url => {
  const rel = new URL(String(url), 'http://localhost/').pathname.replace(/^\/+/u, '');
  const file = rel.startsWith('seed/') ? path.join(ROOT, 'data', rel) : path.join(ROOT, 'js', rel);
  try {
    const text = readFileSync(file, 'utf8');
    return {ok: true, status: 200, text: async () => text};
  } catch {
    return {ok: false, status: 404, text: async () => ''};
  }
}});
await evaluate(worker, 'loadSeed()');
const solve = async (prompt, history = [], prefs = {}) => plain(await worker.solve(prompt, history, prefs, {}, [], {}));

for (const language of fixtureLanguages()) {
  const fixture = JSON.parse(readFileSync(path.join(ROOT, parityPath(language)), 'utf8'));
  test(`${language}: the worker answers every controlled dialogue case like the engine`, {timeout: 300_000}, async () => {
    const failures = [];
    const tasks = new Set();
    for (const item of fixture.dialogue) {
      const expected = item.expected;
      if (Object.keys(item.configuration).length || expected === null) continue;
      // One paraphrase per task shape here (program synthesis is slow); the
      // engine fixture pins every paraphrase's canonical prompt.
      const shape = expected.intent === 'cnl_task' ? expected.prompt.replace(/[0-9]+/gu, 'N') : null;
      if (shape !== null && tasks.has(shape)) continue;
      if (shape !== null) tasks.add(shape);
      const answer = await solve(item.prompt, item.history);
      const label = `${JSON.stringify(item.prompt)} -> ${answer.intent} ${JSON.stringify(String(answer.content).slice(0, 80))}`;
      if (expected.intent === 'cnl_task') {
        // The user's wording may keep its own answer, but it must reach the
        // same intent as the controlled reading of the whole prompt.
        const canonical = await solve(expected.prompt);
        if (answer.intent === 'unknown' || answer.intent !== canonical.intent) {
          failures.push(`${label}: canonical ${JSON.stringify(expected.prompt)} -> ${canonical.intent}`);
        }
      } else if (answer.intent !== expected.intent || answer.content !== expected.content) {
        failures.push(`${label}: expected ${expected.intent} ${JSON.stringify(expected.content)}`);
      } else if (!answer.evidence.includes(`cnl:${language}`)) failures.push(`${label}: missing cnl evidence`);
    }
    assert.deepEqual(failures.slice(0, 10), []);
  });
}

test('prompts the solver already understood keep their answers', {timeout: 60_000}, async () => {
  // Issue #962 word operators in every language (rust/tests/unit/issue_962_word_operator_parity.rs).
  for (const [prompt, expected] of [['What is 2 plus 2?', '2 plus 2 = 4'], ['Сколько будет 2 плюс 2?', '2 плюс 2 = 4'],
    ['2 जोड़ 2 कितना होता है?', '2 जोड़ 2 = 4'], ['2 जमा 2 कितना होता है?', '2 जमा 2 = 4'],
    ['2 加 2 等于多少?', '2 加 2 = 4'], ['¿Cuánto es 2 más 2?', '2 más 2 = 4']]) {
    const sum = await solve(prompt);
    assert.ok(String(sum.content).includes(expected), `${prompt} -> ${sum.content}`);
    assert.ok(!(sum.evidence || []).some(item => String(item).startsWith('cnl:task')), `no CNL fallback for ${prompt}`);
  }
  // The task paraphrase reaches the canonical calculation.
  const paraphrase = await solve('add 10 and 3');
  assert.equal(paraphrase.intent, 'calculation');
  assert.equal(paraphrase.content, '10 + 3 = 13');
  assert.ok(paraphrase.evidence.includes('cnl:task:en'), JSON.stringify(paraphrase.evidence));
});

const MAX_PRECISION_CAT = 'Maximum precision uses specific words. Look up unfamiliar words in a dictionary to check '
  + 'their exact meaning.\n\nYes. Cats are animals.';

test('an in-dialog density switch re-renders the same controlled answer deterministically', {timeout: 60_000}, async () => {
  const question = 'Is a cat an animal?';
  const first = await solve(question);
  assert.equal(first.intent, 'cnl_answer');
  const history = [{role: 'user', content: question}, {role: 'assistant', content: first.content}];
  // A declared directive (a bare `max-precision` is not one) re-renders the previous answer.
  const switched = await solve('Use max-precision.', history);
  const again = await solve('Use max-precision.', history);
  assert.equal(first.content, 'Yes. Every cat is an animal.');
  assert.equal(switched.intent, 'cnl_paraphrase');
  assert.equal(switched.content, MAX_PRECISION_CAT);
  assert.equal(again.content, switched.content);
  const named = await solve('Rewrite in max-precision mode', history);
  assert.equal(named.content, MAX_PRECISION_CAT);
});
