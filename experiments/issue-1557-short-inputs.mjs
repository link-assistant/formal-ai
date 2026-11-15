// Prints how detectProse classifies short mixed inputs (issue #1557 thresholds).
// Usage: node experiments/issue-1557-short-inputs.mjs ["text" ...]
import { WorkerHost } from '../js/server/worker-host.mjs';
import { installNodeHost } from '../js/agentic/node-host.mjs';
import { detect, detectProse, proseOf } from '../js/agentic/crate/language.mjs';

await installNodeHost(new WorkerHost());
const inputs = process.argv.slice(2);
for (const text of inputs.length ? inputs : [
  'please fix the bug, gracias, por favor',
  'fix the café bug',
  'Gracias, please fix the build',
  'see formal_ai::language::detect and parseInput in src/parser.rs now',
]) {
  console.log(`${JSON.stringify(text)} => prose ${detectProse(text)}, whole ${detect(text)} | ${JSON.stringify(proseOf(text))}`);
}
