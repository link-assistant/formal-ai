// Issue #1559 reproduction against a checkout of the code before the fix.
// Usage: node experiments/issue-1559-reproduce.mjs <checkout-root>
// Records the Scala solver prompt as one agent session, dreams, then renders
// the agentic final answer ("summary") of a second session in another
// repository and reports whether it carries text of the first prompt.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const root = path.resolve(process.argv[2] ?? '.');
const load = (file) => import(pathToFileURL(path.join(root, file)).href);
const { WorkerHost } = await load('js/server/worker-host.mjs');
const { createMemory } = await load('js/server/memory.mjs');
const { handleChatCompletions } = await load('js/server/openai.mjs');
const { chatCompletionFromPlan } = await load('js/server/agentic.mjs');
const { runCoreDreamingOnce } = await load('js/server/dreaming-runtime.mjs');
const { memoizedReplay } = await load('js/server/dreaming-replay.mjs');
const { SyncStore } = await load('js/server/memory-store.mjs');

const scala = fs.readFileSync(new URL('../rust/tests/fixtures/issue-1559/scala-solver-prompt.txt', import.meta.url), 'utf8');
const rust = scala.replaceAll('019fb330-00e1-73b9-955e-f357a1600d5b', '019fb331-c107-78c7-8ff6-9f127a3c593c')
  .replaceAll('issue-1-1f3e3886bcb8', 'issue-1-09b0c76bd0e4');
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-issue-1559-repro-'));
const file = path.join(directory, 'memory.lino');
const env = { ...process.env, FORMAL_AI_MEMORY_PATH: file };
const ctx = { worker: new WorkerHost(), agentMode: true, memory: createMemory(env), env };
const tools = [{ type: 'function', function: { name: 'bash', description: 'Run a shell command', parameters: { type: 'object', properties: { command: { type: 'string' } }, required: ['command'] } } }];
const system = { role: 'system', content: 'You are Agent, the best coding agent on the planet.' };

await handleChatCompletions(ctx, { body: JSON.stringify({ model: 'formal-ai', messages: [system, { role: 'user', content: scala }], tools }) });
runCoreDreamingOnce(file, { env, replay: memoizedReplay() });
const amendments = SyncStore.open(env).events.filter((event) => event.kind === 'meta_algorithm_amendment');
console.log('stored amendments:', amendments.map((event) => ({ topic: event.inputs, rule_chars: String(event.outputs).length })));
const messages = [system, { role: 'user', content: rust }];
const summary = chatCompletionFromPlan(ctx, { model: 'formal-ai', messages, tools }, rust, { kind: 'final', answer: 'Summary: done.' }, 1);
const content = String(summary.choices[0].message.content);
console.log('summary starts:', JSON.stringify(content.slice(0, 160)));
console.log('summary carries the Scala repository:', content.includes('019fb330-00e1-73b9-955e-f357a1600d5b'));
console.log('summary carries the Scala branch:', content.includes('issue-1-1f3e3886bcb8'));
fs.rmSync(directory, { recursive: true, force: true });
