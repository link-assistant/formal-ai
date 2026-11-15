// Issue #1559 §4: the same statement made by two users is generalized into one
// unscoped amendment only when replay verifies it.
// Usage: node experiments/issue-1559-generalize.mjs
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { WorkerHost } from '../js/server/worker-host.mjs';
import { createMemory } from '../js/server/memory.mjs';
import { handleChatCompletions } from '../js/server/openai.mjs';
import { runCoreDreamingOnce } from '../js/server/dreaming-runtime.mjs';
import { memoizedReplay } from '../js/server/dreaming-replay.mjs';
import { SyncStore } from '../js/server/memory-store.mjs';

const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-issue-1559-gen-'));
const file = path.join(directory, 'memory.lino');
const env = { ...process.env, FORMAL_AI_MEMORY_PATH: file };
const ctx = { worker: new WorkerHost(), agentMode: false, memory: createMemory(env), env };
async function chat(user, content) {
  const body = { model: 'formal-ai', messages: [{ role: 'user', content }], ...(user ? { user } : {}) };
  return JSON.parse((await handleChatCompletions(ctx, { body: JSON.stringify(body) })).body).choices[0].message.content;
}
await chat('alice', 'Always answer in Russian.');
await chat('bob', 'Always answer in Russian.');
console.log('dream:', runCoreDreamingOnce(file, { env, replay: memoizedReplay() }));
console.log(SyncStore.open(env).events.filter((e) => e.kind === 'meta_algorithm_amendment').map((e) => [e.id, e.inputs, e.outputs, e.evidence, e.covered_event_ids]));
console.log('carol:', await chat('carol', 'What is 2 + 2?'));
fs.rmSync(directory, { recursive: true, force: true });
