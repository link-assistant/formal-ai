// Replays main run 44157124408: real clients must receive the observed file text.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import zlib from 'node:zlib';
import test from 'node:test';
import { composeFinalAnswer } from '../../../js/server/request-context.mjs';
import { createChatCompletion } from '../../../js/server/openai.mjs';
import { parseChatRequest } from '../../../js/server/chat-request.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

for (const client of ['opencode', 'qwen']) {
  test(`main run 44157124408: ${client} final file answer preserves observed bytes`, { timeout: 30000 }, async () => {
    const file = new URL(`../../../experiments/issue-1580/evidence/ci/main-826289fc/cli/${client}-interactive-proxy.jsonl.gz`, import.meta.url);
    const records = zlib.gunzipSync(fs.readFileSync(file)).toString().trim().split('\n').map(JSON.parse);
    const requests = records.filter(row => row.method === 'POST' && row.request_tools.length).map(row => JSON.parse(row.request_body));
    assert.equal(requests.length, client === 'qwen' ? 3 : 2, 'retain the complete real exchange, including Qwen suggestion mode');
    const request = parseChatRequest(JSON.stringify(requests[1]));
    const response = await createChatCompletion({ worker: new WorkerHost(), agentMode: true }, request);
    const message = response.choices[0].message;
    assert.ok(!message.content.includes('Cannot satisfy'), message.content);
    assert.equal(message.content, 'Contents of `alpha.txt`:\n\n```text\nALPHA_MARKER_11111\nalpha second line\n```');
    assert.equal(message.tool_calls?.length ?? 0, 0);
    assert.ok(message.thinking_steps.some(step => step.step === 'request_constraint' && step.detail.includes(':applied:')));
  });
}

test('line limits validate exact boundaries and retain unsupported-constraint refusals', () => {
  const compose = (instruction, text) => composeFinalAnswer([{ role: 'system', content: instruction }, { role: 'user', content: 'Show the result.' }], text, []);
  assert.equal(compose('You MUST answer concisely with fewer than 4 lines of text (not including tool use or code generation), unless user asks for detail.', 'a\nb\nc').answer, 'a\nb\nc');
  assert.match(compose('Answer with fewer than 4 lines.', 'a\nb\nc\nd').answer, /Cannot satisfy output constraint/u);
  assert.equal(compose('Reply with at most 2 lines.', 'a\nb').answer, 'a\nb');
  assert.equal(compose('Respond with exactly 2 lines.', 'a\nb').answer, 'a\nb');
  assert.match(compose('Respond with exactly 2 lines.', 'a').answer, /Cannot satisfy output constraint/u);
  assert.match(compose('Answer in XML.', 'a').answer, /Cannot satisfy format constraint/u);
  assert.match(compose('Answer in JSON with a custom schema.', 'a').answer, /Cannot satisfy output constraint/u);
  assert.match(compose('Do not answer in English.', 'a').answer, /Cannot satisfy output constraint/u);
});

test('line limits preserve final newlines, CRLF, and the explicit detail exception', () => {
  const instruction = 'You MUST answer concisely with fewer than 4 lines of text (not including tool use or code generation), unless user asks for detail.';
  const compose = (prompt, text) => composeFinalAnswer([{ role: 'system', content: instruction }, { role: 'user', content: prompt }], text, []);
  for (const text of ['a\nb\nc\n', 'a\r\nb\r\nc\r\n', '']) assert.equal(compose('Show the result.', text).answer, text);
  assert.equal(compose('Please explain in detail.', 'a\nb\nc\nd').answer, 'a\nb\nc\nd');
  assert.match(compose('Show the result.', 'a\nb\nc\nd').answer, /Cannot satisfy output constraint/u);
});

test('code-block line exemptions preserve file bytes and still enforce prose limits', () => {
  const compose = (instruction, text) => composeFinalAnswer([{ role: 'system', content: instruction }, { role: 'user', content: 'Show the result.' }], text, []);
  const text = 'Observed file:\n\n```text\na\nb\nc\nd\n```';
  const exempt = 'Answer with fewer than 4 lines (not including tool use or code generation).';
  assert.equal(compose(exempt, text).answer, text);
  assert.match(compose('Answer with fewer than 4 lines.', text).answer, /Cannot satisfy output constraint/u);
  assert.match(compose(exempt, 'one\ntwo\nthree\nfour\n```text\na\nb\n```').answer, /Cannot satisfy output constraint/u);
});

test('main run 44157125186: brevity requests accept short answers and reject unbounded prose', () => {
  const compose = text => composeFinalAnswer([{ role: 'system', content: 'Answer concisely.' }, { role: 'user', content: 'Hi' }], text, []);
  assert.equal(compose('Hi, how may I help you?').answer, 'Hi, how may I help you?');
  assert.match(compose('word '.repeat(65)).answer, /Cannot satisfy output constraint/u);
  assert.match(compose('a\nb\nc\nd\ne').answer, /Cannot satisfy output constraint/u);
});
