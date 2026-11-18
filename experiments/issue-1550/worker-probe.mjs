// Issue #1550 probe: real prompts through the browser worker carry the
// recursive procedure's trace (`metaExecution`) and adopt a composition only
// when the root derivation fails its test.
//   node experiments/issue-1550/worker-probe.mjs ["prompt" ...]
import { WorkerHost } from '../../js/server/worker-host.mjs';

const prompts = process.argv.slice(2).length ? process.argv.slice(2) : [
  'Hi',
  'What is 2 + 2?',
  'What is 2 + 2 and what is 3 + 5?',
  'Calculate 12 * 3, then translate hello to French',
  'Сколько будет 2 + 2 и сколько будет 3 + 5?',
  'xyzzy frobnicate the quux',
];
const host = new WorkerHost();
await host.boot();
host.context.fetch = () => { throw new Error('offline probe'); };
for (const prompt of prompts) {
  const result = await host.solve(prompt);
  const meta = result.metaExecution;
  console.log(`\n# ${prompt}\n  intent=${result.intent} content=${JSON.stringify(String(result.content).slice(0, 120))}`);
  console.log(`  meta: ${meta ? `${meta.status} ${meta.method} ${meta.shape}` : 'MISSING'}`);
  for (const event of meta?.events ?? []) console.log(`    ${event}`);
}
process.exit(0);
