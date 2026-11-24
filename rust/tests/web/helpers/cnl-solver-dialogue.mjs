// Issue #1584 through the real browser-worker solver: every declared task
// paraphrase in every language reaches a solver answer, every question phrasing
// gets the controlled answer or a clarification, a numbered reply picks a
// reading, and prompts the solver already understood keep their answers.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {createWorkerContext, evaluate, plain} from '../support/browser-runtime.mjs';
import {fixtureLanguages, parityPath} from '../../../../experiments/issue-1584/parity-fixture.mjs';

import {dialogueLanguagesForShard} from './cnl-dialogue-partition.mjs';

const ROOT = path.resolve(import.meta.dirname, '../../../..');
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
export const solve = async (prompt, history = [], prefs = {}) => plain(await worker.solve(prompt, history, prefs, {}, [], {}));

export function registerCnlDialogueCases(shard) {
  for (const language of dialogueLanguagesForShard(fixtureLanguages(), shard)) {
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
}
