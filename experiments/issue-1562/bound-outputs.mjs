// The output operands the obligation clauses of each text bind (issue #1562):
// a code span holding one of them is an operand of an obligation, not a citation.
// Usage: node experiments/issue-1562/bound-outputs.mjs <text-file>...
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
const { boundOutputLiterals } = await import('../../js/agentic/crate/intent_formalization_obligations.mjs');
for (const path of process.argv.slice(2)) {
  console.log(JSON.stringify({ path, outputs: boundOutputLiterals(readFileSync(path, 'utf8')) }));
}
process.exit(0);
