// Probe: full planner routing for the reported prompt with Claude Code's tools (issue #1632 R6).
import { WorkerHost } from '../js/server/worker-host.mjs';
import { installNodeHost } from '../js/agentic/node-host.mjs';
import { planChatStep } from '../js/agentic/planner.mjs';
await installNodeHost(new WorkerHost());
const tools = ['Task', 'Bash', 'Glob', 'Grep', 'Read', 'Edit', 'Write', 'WebFetch', 'WebSearch', 'AskUserQuestion'];
const plan = await planChatStep([{ role: 'user', content: process.argv[2] ?? 'Отправь отчёт об ошибке' }], tools);
console.log(JSON.stringify(plan).slice(0, 600));
