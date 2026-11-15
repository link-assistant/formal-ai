// Issue #1550: time the worker over the census prompts and list any answer
// without the procedure's metaExecution.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
const { prompts } = JSON.parse(readFileSync('rust/tests/fixtures/procedure-census-prompts.json', 'utf8'));
const host = new WorkerHost();
const started = Date.now();
const missing = [];
for (const prompt of prompts) {
  const answer = await host.solve(prompt);
  if (!answer?.metaExecution) missing.push(prompt);
}
console.log(`${prompts.length} prompts in ${Date.now() - started} ms; without metaExecution: ${missing.length}`);
for (const prompt of missing) console.log(`  ${JSON.stringify(prompt).slice(0, 120)}`);
