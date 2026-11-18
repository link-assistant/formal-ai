// Externally authored bootstrap adapter. Supplied commands/bytes earn no
// autonomous-authorship credit. The maintained Formal AI planner chooses tools.
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installDefaultNodeSourceHost } from '../../js/server/default-node-source-bootstrap.mjs';
import { drive } from '../js_dogfood/drive.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';

const [requestFile, evidenceFile, mode] = process.argv.slice(2);
const source = fs.readFileSync(requestFile, 'utf8');
const request = mode === 'command' ? 'Run ' + source.trim() : source;
const git = (...args) => execFileSync('git', args, {encoding: 'utf8'});
const diff = git('diff', '--no-ext-diff');
const runtime = {node: process.version, source_commit: git('rev-parse', 'HEAD').trim(),
  source_status: git('status', '--porcelain'), source_diff_sha256: createHash('sha256').update(diff).digest('hex')};
await installDefaultNodeSourceHost(new WorkerHost());
const result = await drive(planChatStep, process.cwd(), request, {steps: 8});
fs.mkdirSync(path.dirname(evidenceFile), {recursive: true});
fs.writeFileSync(evidenceFile, JSON.stringify({runtime, request, ...result}, null, 2) + '\n');
fs.writeFileSync(evidenceFile + '.diff', diff);
let failed = result.stop !== 'final' || (mode === 'command' && result.transcript.length === 0);
for (const entry of result.transcript) {
  console.log('TOOL', entry.tool, entry.arguments);
  console.log(entry.result ?? '');
  if (/Exit Code: [1-9][0-9]*|"is_error":true/u.test(String(entry.result))) failed = true;
}
console.log('STOP', result.stop);
if (mode !== 'command') console.log('ANSWER', result.answer);
process.exit(failed ? 1 : 0);
