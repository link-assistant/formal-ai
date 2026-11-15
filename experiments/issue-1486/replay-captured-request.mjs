// Replays a captured OpenAI-compatible request body through the current JS planner.
// Usage: node experiments/issue-1486/replay-captured-request.mjs <request.json>
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';
await installNodeHost(new WorkerHost({ WebAssembly: { compile() { throw Error('LocalProductWasmForbidden'); }, instantiate() { throw Error('LocalProductWasmForbidden'); } } }));
const request = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const tools = (request.tools ?? []).map(tool => tool.function?.name ?? tool.name);
const plan = await planChatStep(request.messages, tools);
console.log(JSON.stringify({ kind: plan.kind, calls: plan.calls, answer: plan.answer?.slice?.(0, 1500) }, null, 1));
