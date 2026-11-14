// Print how a work item's text is split into requirements (issue #1560).
// Usage: node experiments/issue-1560/requirements.mjs <issue.json> [splitter]
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost({}));
const issue = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const text = issue.title + '\n\n' + issue.body;
const splitter = process.argv[3] === 'clauses'
  ? (await import('../../js/agentic/crate/obligation_ledger.mjs')).clausesWithSpans
  : (await import('../../js/agentic/work_item_requirements.mjs')).workItemRequirements;
const rows = splitter(text);
console.log(rows.length, 'requirements');
for (const row of rows) console.log(JSON.stringify(row).slice(0, 160));
const bytes = new TextEncoder().encode(text);
const bad = rows.filter(([clause, [start, end]]) => new TextDecoder().decode(bytes.subarray(start, end)) !== clause);
console.log('span mismatches:', bad.length);
