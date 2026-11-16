// Issue #1562: plan the next action for a recorded chat request of a
// hive-mind session (the request that followed the issue fetch) and print it.
// Usage: node experiments/issue-1562/replay-recorded-request.mjs <request.json>...
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

const worker = new WorkerHost();
await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
for (const file of process.argv.slice(2)) {
  const request = JSON.parse(readFileSync(file, 'utf8'));
  const parsed = parseChatRequest(JSON.stringify({ ...request, stream: false }));
  const prompt = request.messages.findLast(message => message.role === 'user').content;
  const outcome = await agenticOutcome({ worker, agentMode: true }, parsed, () => 'refused');
  console.log(`=== ${file}: ${outcome.kind}`);
  if (outcome.kind !== 'planned') continue;
  const message = chatCompletionFromPlan({}, parsed, prompt, outcome.plan, 0).choices[0].message;
  for (const call of message.tool_calls ?? []) console.log(' ', call.function.name, call.function.arguments.slice(0, 300));
  if (!message.tool_calls?.length) console.log(String(message.content).slice(0, 1200));
}
process.exit(0);
