// Scoped standing requirements (issue #1559).
//
// One memory store serves two kinds of clients, as on a Hive Mind host:
// - an agent (a request that carries tools) delivering its solver prompt as
//   the first user message, once per repository it works on;
// - a chat user who states a standing requirement in plain words.
//
// The agent's prompt is recorded as an instruction and never becomes a learned
// requirement, so the second repository's session shows none of the first
// one's text. The chat user's "Always answer in Russian." is learned with that
// user's scope and applied to that user's later conversations only.
//
// Usage: node rust/examples/scoped-standing-requirements.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { WorkerHost } from '../../js/server/worker-host.mjs';
import { createMemory } from '../../js/server/memory.mjs';
import { handleChatCompletions } from '../../js/server/openai.mjs';
import { runCoreDreamingOnce } from '../../js/server/dreaming-runtime.mjs';
import { memoizedReplay } from '../../js/server/dreaming-replay.mjs';
import { SyncStore } from '../../js/server/memory-store.mjs';

const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-scoped-requirements-'));
const file = path.join(directory, 'memory.lino');
const env = { ...process.env, FORMAL_AI_MEMORY_PATH: file };
const memory = createMemory(env);
const worker = new WorkerHost();
const agent = { worker, agentMode: true, memory, env };
const chatClient = { worker, agentMode: false, memory, env };
const tools = [{
  type: 'function',
  function: { name: 'bash', description: 'Run a shell command', parameters: { type: 'object', properties: { command: { type: 'string' } } } },
}];

const solverPrompt = (repository) => [
  'You are an AI issue solver.',
  `Your prepared issue: https://github.com/example/${repository}/issues/1`,
  'Always push only to the prepared branch.',
].join('\n');

async function send(ctx, body) {
  const response = await handleChatCompletions(ctx, { body: JSON.stringify({ model: 'formal-ai', ...body }) });
  return JSON.parse(response.body).choices[0].message;
}

await send(agent, { messages: [{ role: 'user', content: solverPrompt('first-repository') }], tools });
await send(chatClient, { user: 'alice', messages: [{ role: 'user', content: 'Always answer in Russian.' }] });
runCoreDreamingOnce(file, { env, replay: memoizedReplay() });

const events = SyncStore.open(env).events;
console.log('recorded roles:', events.filter((event) => event.kind === 'message').map((event) => event.role));
console.log('learned:', events
  .filter((event) => event.kind === 'meta_algorithm_amendment')
  .map((event) => ({ topic: event.inputs, rule: event.outputs, scope: event.evidence.filter((entry) => entry.startsWith('scope:')) })));

const second = await send(agent, { messages: [{ role: 'user', content: solverPrompt('second-repository') }], tools });
console.log('second agent session mentions the first repository:', JSON.stringify(second).includes('first-repository'));

for (const user of ['alice', 'bob']) {
  const answer = await send(chatClient, { user, messages: [{ role: 'user', content: 'What is 2 + 2?' }] });
  console.log(`${user}:`, JSON.stringify(answer.content));
}
fs.rmSync(directory, { recursive: true, force: true });
