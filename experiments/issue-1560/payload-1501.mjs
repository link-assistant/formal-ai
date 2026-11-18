// Shows observedPayload for the recorded gh issue view output.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { observedPayload } from '../../js/agentic/tool_result.mjs';
await installNodeHost(new WorkerHost({}));
const body = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const raw = body.messages.find(m => m.role === 'tool').content;
const payload = observedPayload(raw);
console.log(payload === null ? null : JSON.stringify(payload.slice(0, 300)), payload?.includes('\n\n'));
