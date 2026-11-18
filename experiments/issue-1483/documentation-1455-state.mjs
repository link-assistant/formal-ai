// Drives the #1455 pre-fix fixture through the documentation controller and
// prints the terminal record (facts, checks, requirements) and the answer.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { drive } from '../js_dogfood/drive.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { workItemRecordOf } from '../../js/agentic/work_item_report.mjs';
await installNodeHost(new WorkerHost({}));
const fixture = JSON.parse(fs.readFileSync(new URL('../../rust/tests/fixtures/repository-documentation-original.json', import.meta.url), 'utf8'));
const files = fixture.files.map(file => process.argv[2] === 'contradicted' && file.path.endsWith('.md') ? file : file);
let task = fixture.issue.title + '\n\n' + fixture.issue.body;
if (process.argv[2] === 'contradicted') task = task.replace('command-stream = "1.5"', 'command-stream = "1.9"');
const root = fs.mkdtempSync(path.join(os.tmpdir(), 'docs-1455-'));
for (const file of files) { fs.mkdirSync(path.dirname(path.join(root, file.path)), { recursive: true }); fs.writeFileSync(path.join(root, file.path), file.source); }
for (const round of [1, 2]) {
  const result = await drive(planChatStep, root, task, { steps: 24, tools: ['read', 'edit'], allowedCommands: [] });
  const record = result.record ?? workItemRecordOf(result);
  console.log('== round', round, result.transcript.map(row => row.tool).join(','));
  console.log(JSON.stringify({ reason: record.reason, authored: record.authored, verified: record.verified, complete: record.complete,
    checks: record.checks, unresolved: record.unresolved, remainingWork: record.remainingWork,
    requirements: record.requirements.map(row => [row.status, row.text.slice(0, 60)]),
    facts: (record.facts ?? []).map(row => JSON.stringify(row).slice(0, 300)) }, null, 1));
  console.log(result.answer);
}
fs.rmSync(root, { recursive: true, force: true });
