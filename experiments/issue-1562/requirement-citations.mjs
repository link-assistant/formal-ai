// Issue #1562: list each requirement row of the fixtures as an obligation (O)
// or a citation (C), as the evidence record now marks it.
// Usage: node experiments/issue-1562/requirement-citations.mjs [fixture.txt...]
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
const { obligationText, isCitation } = await import('../../js/agentic/work_item_citations.mjs');
const { workItemRequirements } = await import('../../js/agentic/work_item_requirements.mjs');
const files = process.argv.slice(2).length ? process.argv.slice(2) : ['4031', '4034', '4035'].map(n => `data/benchmarks/fixtures/issue-1562/hive-mind-${n}.txt`);
for (const file of files) {
  const text = readFileSync(file, 'utf8');
  const obligations = obligationText(text);
  const rows = workItemRequirements(text);
  console.log(`=== ${file}: ${rows.length} rows, ${rows.filter(([, span]) => isCitation(obligations, span)).length} citations, bytes kept: ${Buffer.byteLength(obligations) === Buffer.byteLength(text)}`);
  for (const [row, span] of rows) console.log(' ', isCitation(obligations, span) ? 'C' : 'O', row.slice(0, 100).replace(/\n/gu, ' '));
}
process.exit(0);
