// Times each controlled-dialogue fixture case through the worker solver (issue #1584).
// Usage: node experiments/issue-1584/time-solver.mjs [language]
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {createWorkerContext, evaluate, plain} from '../../rust/tests/web/support/browser-runtime.mjs';
const ROOT = path.resolve(import.meta.dirname, '../..');
const fixture = JSON.parse(readFileSync(path.join(ROOT, 'data/parity/issue-1584-cnl-parity.json'), 'utf8'));
const worker = createWorkerContext({fetch: async url => {
  const rel = new URL(String(url), 'http://localhost/').pathname.replace(/^\/+/u, '');
  const file = rel.startsWith('seed/') ? path.join(ROOT, 'data', rel) : path.join(ROOT, 'js', rel);
  try { const text = readFileSync(file, 'utf8'); return {ok: true, status: 200, text: async () => text}; }
  catch { return {ok: false, status: 404, text: async () => ''}; }
}});
await evaluate(worker, 'loadSeed()');
for (const item of fixture.dialogue[process.argv[2] || 'en'].filter(entry => !Object.keys(entry.configuration).length)) {
  const start = performance.now();
  const answer = plain(await worker.solve(item.prompt, item.history, {}, {}, [], {}));
  console.log(Math.round(performance.now() - start), item.expected?.intent, answer.intent, JSON.stringify(item.prompt));
}
