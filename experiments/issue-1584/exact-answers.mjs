// Usage: node experiments/issue-1584/exact-answers.mjs — prints the exact worker answers pinned by rust/tests/unit/cnl_output.rs.
import test from 'node:test';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {createWorkerContext, evaluate, plain} from '../../rust/tests/web/support/browser-runtime.mjs';

const ROOT = path.resolve(import.meta.dirname, '../..');
const worker = createWorkerContext({fetch: async url => {
  const relative = new URL(String(url), 'http://localhost/').pathname.replace(/^\/+/u, '');
  const file = relative.startsWith('seed/') ? path.join(ROOT, "data", relative) : path.join(ROOT, "js", relative);
  try { const text = readFileSync(file, 'utf8'); return {ok: true, status: 200, text: async () => text}; }
  catch { return {ok: false, status: 404, text: async () => ''}; }
}});
await evaluate(worker, 'loadSeed()');
const solve = async (prompt, history = []) => plain(await worker.solve(prompt, history, {}, {}, [], {}));
test('print', {timeout: 120_000}, async () => {
  for (const prompt of ['add 10 and 3', 'What is 2 plus 2?', 'Is a cat an animal?']) {
    console.log(JSON.stringify(prompt), "=>", JSON.stringify((await solve(prompt)).content));
  }
  const first = await solve('Is a cat an animal?');
  const history = [{role: 'user', content: 'Is a cat an animal?'}, {role: 'assistant', content: first.content}];
  console.log('max-precision =>', JSON.stringify((await solve('max-precision', history)).content));
});
test('switch phrasings', {timeout: 120_000}, async () => {
  const first = await solve('Is a cat an animal?');
  const history = [{role: 'user', content: 'Is a cat an animal?'}, {role: 'assistant', content: first.content}];
  for (const prompt of ['Rewrite in max-precision mode', 'Use max-precision', 'maximum precision', 'Answer in ACE', 'primes only', 'Use primes']) {
    const reply = await solve(prompt, history);
    console.log(JSON.stringify(prompt), "=>", reply.intent, JSON.stringify(String(reply.content).slice(0, 200)));
  }
});
