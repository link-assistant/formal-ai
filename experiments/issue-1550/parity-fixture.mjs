// Issue #1550: write (or check) the shared meta-executor parity fixture
// rust/tests/fixtures/meta-executor-parity.json from the JavaScript module.
// The JavaScript test rust/tests/web/issue-1550-meta-executor.test.mjs and the
// native test rust/tests/unit/meta_executor_parity.rs replay every case over
// the same table host, so a drift on either side fails one of the suites.
// Usage: node experiments/issue-1550/parity-fixture.mjs --write|--check
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const FIXTURE = path.join(REPO, 'rust/tests/fixtures/meta-executor-parity.json');
await installNodeHost(new WorkerHost());
const { replayCase } = await import('../../rust/tests/web/helpers/issue-1550-meta-executor-cases.mjs');
const { CASES } = await import('../../rust/tests/web/helpers/issue-1550-meta-executor-cases.mjs');

const cases = [];
for (const entry of CASES) cases.push({ ...entry, expected: await replayCase(entry) });
const text = JSON.stringify({ schema: 'meta-executor-parity/v1', cases }, null, 2) + '\n';
if (process.argv.includes('--write')) {
  fs.writeFileSync(FIXTURE, text);
  console.log(`wrote ${path.relative(REPO, FIXTURE)} with ${cases.length} cases`);
} else if (fs.readFileSync(FIXTURE, 'utf8') !== text) {
  console.error('meta-executor parity fixture is stale; run with --write');
  process.exitCode = 1;
} else {
  console.log(`meta-executor parity fixture: ${cases.length} cases current`);
}
