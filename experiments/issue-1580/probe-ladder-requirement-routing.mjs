import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

const [, , tracePath] = process.argv;
if (!tracePath) throw new Error('Usage: node probe-ladder-requirement-routing.mjs <decoded-formal-ai-trace>');
const trace = readFileSync(tracePath, 'utf8');
const match = /^\s*\d+ \[trace\] agentic_received=([\s\S]+?)(?=^\s*\d+ \[trace\])/mu.exec(trace);
if (!match) throw new Error('The decoded trace has no original agentic_received event');

await installNodeHost(new WorkerHost());
const { planChatStepResolved } = await import('../../js/agentic/planner.mjs');
const { composeEditRequest } = await import('../../js/agentic/write_request.mjs');
const request = match[1].trim();
const tools = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read',
  'task', 'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];
const plan = await planChatStepResolved([{ role: 'user', content: request }], tools);
const edit = composeEditRequest(request);
process.stdout.write(`${JSON.stringify({ requestLength: request.length, edit, plan }, null, 2)}\n`);
