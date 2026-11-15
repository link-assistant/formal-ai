// Issue #1558: replay a recorded Agent CLI request body through the server's
// agentic entry and print the next planned action for each transcript prefix.
// Usage: node experiments/issue-1558/replay-session.mjs <request-body.json> [workspace-root]
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

const [bodyFile, workspace] = process.argv.slice(2);
const worker = new WorkerHost();
await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
const body = JSON.parse(readFileSync(bodyFile, 'utf8'));
let messages = body.messages;
if (workspace) messages = JSON.parse(JSON.stringify(messages).replaceAll(/\/tmp\/gh-issue-solver-\d+/g, workspace));
const cut = messages.findIndex(m => m.role === 'tool') + 1;
for (let end = cut; end <= messages.length; end += 2) {
  const request = parseChatRequest(JSON.stringify({ ...body, messages: messages.slice(0, end), stream: false }));
  const prompt = messages.findLast(m => m.role === 'user').content;
  const outcome = await agenticOutcome({ worker, agentMode: true }, request, () => 'refused');
  if (outcome.kind !== 'planned') { console.log(end, outcome.kind); continue; }
  const chat = chatCompletionFromPlan({}, request, prompt, outcome.plan, 0);
  const message = chat.choices[0].message;
  console.log(end, JSON.stringify(message.tool_calls?.map(c => [c.function.name, c.function.arguments]) ?? []), String(message.content).slice(0, 600));
}
