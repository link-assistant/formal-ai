// Compares the worker answer for a task paraphrase with the answer for its canonical prompt (issue #1584).
// Usage: node experiments/issue-1584/compare-canonical.mjs "<prompt>" ...
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {createWorkerContext, evaluate, plain} from '../../rust/tests/web/support/browser-runtime.mjs';
import {policy} from '../../rust/tests/web/support/cnl-matrix.mjs';
import {controlledDialogue} from '../../js/agentic/crate/cnl_output.mjs';
const ROOT = path.resolve(import.meta.dirname, '../..');
const worker = createWorkerContext({fetch: async url => {
  const rel = new URL(String(url), 'http://localhost/').pathname.replace(/^\/+/u, '');
  const file = rel.startsWith('seed/') ? path.join(ROOT, 'data', rel) : path.join(ROOT, 'js', rel);
  try { const text = readFileSync(file, 'utf8'); return {ok: true, status: 200, text: async () => text}; }
  catch { return {ok: false, status: 404, text: async () => ''}; }
}});
await evaluate(worker, 'loadSeed()');
for (const prompt of process.argv.slice(2)) {
  const turn = controlledDialogue(prompt, [], {}, policy);
  const original = plain(await worker.solve(prompt, [], {}, {}, [], {}));
  console.log('PROMPT', JSON.stringify(prompt), '->', JSON.stringify(turn?.prompt));
  console.log('  original', original.intent, JSON.stringify(String(original.content).slice(0, 300)));
  if (turn?.prompt) {
    const canonical = plain(await worker.solve(turn.prompt, [], {}, {}, [], {}));
    console.log('  canonical', canonical.intent, JSON.stringify(String(canonical.content).slice(0, 300)));
  }
}
