// Evaluates the read policy over a recorded user turn and over the fetched work item.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { readPolicyBlocksPlan } from '../../js/agentic/file_read/ownership.mjs';
await installNodeHost(new WorkerHost({}));
const step = { kind: 'tool_calls', calls: [{ tool: 'read', arguments: JSON.stringify({ path: 'README.md' }) }] };
for (const file of process.argv.slice(2)) {
  const text = file.endsWith('.json') ? (() => { const v = JSON.parse(readFileSync(file, 'utf8')); return v.messages ? v.messages.findLast(m => m.role === 'user').content : (v.title + '\n\n' + v.body); })() : readFileSync(file, 'utf8');
  console.log(file, readPolicyBlocksPlan(text, step));
}
