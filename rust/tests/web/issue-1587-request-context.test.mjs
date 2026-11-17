// Both runtimes consume the same exact-answer and explicit-refusal fixtures.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import os from 'node:os';
import path from 'node:path';
import { chatCompletionFromPlan, responseFromPlan } from '../../../js/server/agentic.mjs';
import { createMemory } from '../../../js/server/memory.mjs';
import { parseChatRequest } from '../../../js/server/chat-request.mjs';
import { createChatCompletion, handleChatCompletions } from '../../../js/server/openai.mjs';
import { createResponse } from '../../../js/server/responses.mjs';
import { parseResponsesRequest } from '../../../js/server/responses-input.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const rows = JSON.parse(fs.readFileSync(new URL('../../../data/parity/issue-1587-request-context.json', import.meta.url)));
const ctx = { worker: new WorkerHost(), agentMode: false };
test('issue 1587: request-local constraints and user facts', { timeout: 120000 }, async () => {
  for (const row of rows) {
    const result = await createChatCompletion(ctx, parseChatRequest(JSON.stringify({ messages: row.messages })));
    const message = result.choices[0].message;
    if (row.answer) assert.equal(message.content, row.answer, row.id);
    if (row.contains) assert.ok(message.content.includes(row.contains), `${row.id}: ${message.content}`);
    if (row.constraint) assert.ok(message.thinking_steps.some(step => step.step === 'request_constraint' && step.detail.includes(row.constraint)), `${row.id}: missing constraint trace`);
    if (row.no_constraint) assert.ok(!message.thinking_steps.some(step => step.step === 'request_constraint'), `${row.id}: operational instruction mistaken for output`);
    if (row.fact) assert.ok(message.thinking_steps.some(step => step.step === 'request_fact'), `${row.id}: missing fact provenance`);
  }
});
test('issue 1587: published agentic answers honor constraints', () => {
  const request = parseChatRequest(JSON.stringify({ messages: [{ role: 'system', content: 'Answer in JSON.' }, { role: 'user', content: 'Finish the task.' }] }));
  const plan = { kind: 'final', answer: 'done' };
  const completion = chatCompletionFromPlan(ctx, request, 'Finish the task.', plan, 1);
  assert.equal(completion.choices[0].message.content, '{"answer":"done"}');
  assert.ok(completion.choices[0].message.thinking_steps.some(step => step.step === 'request_constraint'));
  const response = responseFromPlan(ctx, {}, request, 'Finish the task.', plan, 1);
  assert.equal(response.output[0].content[0].text, '{"answer":"done"}');
});

test('issue 1587: memory recording does not make constraints or facts cross requests', { timeout: 120000 }, async t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-request-context-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const env = { ...process.env, FORMAL_AI_MEMORY_PATH: path.join(directory, 'memory.lino') };
  const server = { ...ctx, memory: createMemory(env), env };
  const chat = async messages => {
    const response = await handleChatCompletions(server, { body: JSON.stringify({ messages }) });
    assert.equal(response.statusCode, 200, response.body);
    return JSON.parse(response.body).choices[0].message.content;
  };
  assert.match(await chat([{ role: 'system', content: 'Always answer in French.' }, { role: 'user', content: 'What is 2 + 2?' }]), /Cannot satisfy language constraint/u);
  assert.equal(await chat([{ role: 'user', content: 'What is 2 + 2?' }]), '2 + 2 = 4');
  assert.equal(await chat([{ role: 'user', content: 'My pet name is Pip.' }, { role: 'user', content: 'What is my pet name?' }]), 'Pip');
  assert.ok(!(await chat([{ role: 'user', content: 'What is my pet name?' }])).includes('Pip'));
});
test('issue 1587: Responses instructions and concurrent requests remain scoped', { timeout: 120000 }, async () => {
  const requests = [
    [{ role: 'system', content: 'Always answer in French.' }, { role: 'user', content: 'What is 2 + 2?' }],
    [{ role: 'user', content: 'What is 2 + 2?' }],
    [{ role: 'user', content: 'My favourite colour is violet.' }, { role: 'user', content: 'Which colour did I name?' }],
    [{ role: 'user', content: 'Which colour did I name?' }],
  ];
  const results = await Promise.all(requests.map(messages => createChatCompletion(ctx, parseChatRequest(JSON.stringify({ messages })))));
  const texts = results.map(result => result.choices[0].message.content);
  assert.match(texts[0], /Cannot satisfy language constraint/u);
  assert.equal(texts[1], '2 + 2 = 4');
  assert.equal(texts[2], 'violet');
  assert.ok(!texts[3].includes('violet'));
  const response = await createResponse(ctx, parseResponsesRequest(JSON.stringify({ instructions: 'Answer in JSON.', input: 'What is 9 + 4?' })));
  assert.equal(response.output.find(item => item.type === 'message').content[0].text, '{"answer":"9 + 4 = 13"}');
});
