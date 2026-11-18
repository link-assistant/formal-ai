// Print the browser-worker answers the issue #1584 solver tests pin.
// Usage: node experiments/issue-1584/worker-answers.mjs
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {createWorkerContext, evaluate, plain} from '../../rust/tests/web/support/browser-runtime.mjs';

const ROOT = path.resolve(import.meta.dirname, '../..');
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
const solve = async (prompt, history = []) => plain(await worker.solve(prompt, history, {}, {}, [], {}));
const question = 'Is a cat an animal?';
const first = await solve(question);
const history = [{role: 'user', content: question}, {role: 'assistant', content: first.content}];
for (const switchPrompt of ['max-precision', 'Use max-precision.', 'Rewrite in max-precision mode']) {
  const switched = await solve(switchPrompt, history);
  console.log(JSON.stringify(switchPrompt), switched.intent, JSON.stringify(switched.content));
}
for (const prompt of ['add 10 and 3', '10 + 3']) {
  const answer = await solve(prompt);
  console.log(JSON.stringify(prompt), answer.intent, JSON.stringify(answer.content), JSON.stringify(answer.evidence));
}
