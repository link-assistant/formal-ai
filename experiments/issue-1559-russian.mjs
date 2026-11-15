// Issue #1559: "Always answer in Russian" from one user is learned with that
// user's scope and applied to that user's later conversations only.
// Usage: node experiments/issue-1559-russian.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { WorkerHost } from '../js/server/worker-host.mjs';
import { createMemory } from '../js/server/memory.mjs';
import { handleChatCompletions } from '../js/server/openai.mjs';
import { runCoreDreamingOnce } from '../js/server/dreaming-runtime.mjs';
import { memoizedReplay } from '../js/server/dreaming-replay.mjs';
import { SyncStore } from '../js/server/memory-store.mjs';

const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-issue-1559-ru-'));
const file = path.join(directory, 'memory.lino');
const env = { ...process.env, FORMAL_AI_MEMORY_PATH: file };
const ctx = { worker: new WorkerHost(), agentMode: false, memory: createMemory(env), env };

async function chat(user, content) {
  const body = { model: 'formal-ai', messages: [{ role: 'user', content }], ...(user ? { user } : {}) };
  const response = await handleChatCompletions(ctx, { body: JSON.stringify(body) });
  return JSON.parse(response.body).choices[0].message.content;
}

console.log('teach:', await chat('alice', 'Always answer in Russian.'));
console.log('dream:', runCoreDreamingOnce(file, { env, replay: memoizedReplay() }));
const events = SyncStore.open(env).events;
console.log('amendments:', events.filter((event) => event.kind === 'meta_algorithm_amendment').map((e) => [e.id, e.inputs, e.outputs, e.evidence]));
console.log('alice later:', await chat('alice', 'What is 2 + 2?'));
console.log('bob later:', await chat('bob', 'What is 2 + 2?'));
console.log('anonymous later:', await chat(null, 'What is 2 + 2?'));
fs.rmSync(directory, { recursive: true, force: true });
