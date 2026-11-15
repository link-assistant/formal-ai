// Issue #1571 A5: which payload helper accepts each recorded work-item read.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';
await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
const { normalizedPayload, observedPayload } = await import('../../js/agentic/tool_result.mjs');
for (const issue of [1484, 1485, 1487, 1488, 1507]) {
  const raw = readFileSync(`data/benchmarks/fixtures/issue-1558/issue-${issue}.txt`, 'utf8');
  console.log(issue, 'normalizedPayload:', normalizedPayload(raw) === null ? 'null' : 'text', 'observedPayload:', observedPayload(raw) === null ? 'null' : 'text');
}
process.exit(0);
