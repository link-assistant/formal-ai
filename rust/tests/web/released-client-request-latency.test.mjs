import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { createMemory } from '../../../js/server/memory.mjs';
import { ensureNodeHost } from '../../../js/server/node-host-install.mjs';
import { createChatCompletion } from '../../../js/server/openai.mjs';
import { chatPromptAndHistory, parseChatRequest } from '../../../js/server/chat-request.mjs';
import { parseResponsesRequest, toChatCompletionRequest } from '../../../js/server/responses-input.mjs';
import { parseAnthropicRequest, anthropicToChatCompletionRequest } from '../../../js/server/anthropic.mjs';
import { predictionHitEvent } from '../../../js/server/anticipation.mjs';
test('trusted JavaScript-only server policy boots without attempting Rust WASM', {timeout: 10000}, async () => {
  const previousPolicy = process.env.FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR;
  const originalWasm = globalThis.WebAssembly;
  let attempts = 0;
  const denied = () => { attempts++; throw Error('LocalProductWasmForbidden'); };
  globalThis.WebAssembly = new Proxy(originalWasm, {get(target, key) {
    return ['compile', 'compileStreaming', 'instantiate', 'instantiateStreaming', 'Module', 'Instance'].includes(key)
      ? denied : Reflect.get(target, key);
  }});
  process.env.FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR = 'false';
  try {
    const ctx = {worker: new WorkerHost()};
    await ensureNodeHost(ctx);
    assert.equal(attempts, 0);
    await ensureNodeHost(ctx);
    await assert.rejects(ensureNodeHost(ctx, {sourceUsageRustGrammar: true}), /installation_conflict/u);
    const {observedSourceUsageOperation} = await import('../../../js/server/observed-source-usage.mjs');
    assert.throws(() => observedSourceUsageOperation({owner: 'src/lib.rs', language: 'Rust', source: 'pub fn answer() {}'}),
      /SourceUsageRustFrontendUnavailable/u);
    process.env.FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR = 'invalid';
    await assert.rejects(ensureNodeHost({worker: new WorkerHost()}), /environment_must_be_boolean/u);
  } finally {
    globalThis.WebAssembly = originalWasm;
    if (previousPolicy === undefined) delete process.env.FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR;
    else process.env.FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR = previousPolicy;
  }
});

test('ordinary coding history does not invoke a prediction classifier', () => {
  const body = fs.readFileSync(new URL('../fixtures/released-client-requests/agent.json', import.meta.url), 'utf8');
  const request = parseChatRequest(body);
  const {
    prompt
  } = chatPromptAndHistory(request.messages);
  const unexpectedClassifier = () => {
    throw new Error('no prediction exists to classify');
  };
  for (const events of [[], [{
    id: 'ordinary-history',
    kind: 'message',
    content: prompt
  }]]) {
    assert.equal(predictionHitEvent(events, prompt, 'actual-request', unexpectedClassifier), null);
  }
});
test('captured released-client requests produce the grounded first step without tool effects', {
  timeout: 30000
}, async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-released-request-'));
  try {
    const ctx = {
      worker: new WorkerHost(),
      agentMode: true,
      memory: createMemory({
        FORMAL_AI_MEMORY_PATH: path.join(directory, 'memory.lino')
      })
    };
    await ensureNodeHost(ctx, {
      sourceUsageRustGrammar: false
    });
    for (const [client, tool, issue] of [['agent', 'bash', '1411'], ['codex', 'exec_command', '1413'], ['claude', 'Bash', '1415']]) {
      const body = fs.readFileSync(new URL('../fixtures/released-client-requests/' + client + '.json', import.meta.url), 'utf8');
      const request = client === 'codex' ? toChatCompletionRequest(parseResponsesRequest(body)) : client === 'claude' ? anthropicToChatCompletionRequest(parseAnthropicRequest(body)) : parseChatRequest(body);
      const started = performance.now();
      const completion = await createChatCompletion(ctx, request);
      const calls = completion.choices[0].message.tool_calls;
      if (client === 'codex') {
        // Its public export removed the properties object (the schema declares
        // a shell "login" option). Preserve that loss and refuse the call.
        assert.equal(calls.length, 0);
        assert.match(completion.choices[0].message.content, /required tool arguments could not be bound/u);
      } else {
        assert.equal(calls.length, 1, client);
        assert.equal(calls[0].function.name, tool);
        assert(calls[0].function.arguments.includes('https://github.com/link-assistant/formal-ai/issues/' + issue));
      }
      assert(performance.now() - started < 15000, client + ' exceeded its pure-planning latency budget');
    }
  } finally {
    fs.rmSync(directory, {
      recursive: true,
      force: true
    });
  }
});
