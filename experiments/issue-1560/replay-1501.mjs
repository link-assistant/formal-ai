// Replays a recorded OpenAI-compatible request body through the JS planner.
// Usage: node experiments/issue-1560/replay-1501.mjs <request.json>
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';
await installNodeHost(new WorkerHost({}));
const body = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const tools = (body.tools ?? []).map(tool => tool.function?.name ?? tool.name);
const step = await planChatStep(body.messages, tools);
console.log(JSON.stringify(step, null, 1).slice(0, 3000));
