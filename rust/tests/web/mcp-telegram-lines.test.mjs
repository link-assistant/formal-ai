import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {randomUUID} from 'node:crypto';
import {telegramHtmlFromMarkdown} from '../../../js/server/telegram.mjs';
import {handleMcp} from '../../../js/server/mcp.mjs';

test('Telegram keeps its falsy-input empty rendering contract', () => {
  for (const input of [undefined, null, false, 0, -0, NaN, '']) assert.equal(telegramHtmlFromMarkdown(input), '');
});
test('Telegram preserves meaningful invalid-input and split getter failures', () => {
  for (const input of [true, 1, {}, []]) assert.throws(() => telegramHtmlFromMarkdown(input), TypeError);
  let reads = 0;
  const failure = new Error('original split getter');
  assert.throws(() => telegramHtmlFromMarkdown({get split() {
    reads += 1;
    throw failure;
  }}), error => error === failure);
  assert.equal(reads, 1);
});
test('Telegram preserves normal LF/CRLF and Rust bare-CR code bytes', () => {
  assert.equal(telegramHtmlFromMarkdown('oak\r\n> branch\r\n'), 'oak\n<blockquote>branch</blockquote>');
  assert.equal(telegramHtmlFromMarkdown('```text\r\nline\r\n```\r\n'), '<pre><code class="language-text">line\n</code></pre>');
  assert.equal(telegramHtmlFromMarkdown('```text\nline\r'), '<pre><code class="language-text">line\r\n</code></pre>');
});

async function computerPlan(context, content, operation, extra = {}) {
  const plan = 'physical-lines-' + randomUUID();
  const parent = path.join(os.tmpdir(), 'formal-ai-computer-use');
  context.after(() => {
    if (fs.existsSync(parent)) for (const name of fs.readdirSync(parent)) {
      if (name.startsWith(plan + '-' + process.pid + '-')) fs.rmSync(path.join(parent, name), {recursive: true, force: true});
    }
  });
  let sequence = 0;
  async function call(name, fields) {
    const response = await handleMcp({agentMode: true}, {body: JSON.stringify({jsonrpc: '2.0', id: ++sequence,
      method: 'tools/call', params: {name, arguments: {plan_id: plan, step_id: String(sequence),
        precondition: 'input observed', postcondition: 'output verified', confirmed: true, ...fields}}})});
    const envelope = JSON.parse(response.body);
    assert.equal(envelope.error, undefined, response.body);
    const record = envelope.result.structuredContent;
    assert.equal(record.verified, true, response.body);
    assert.equal(envelope.result.isError, false, response.body);
    return record.output;
  }
  await call('fs.write', {path: 'input.txt', content});
  await call('shell.run', {operation, input: 'input.txt', output: 'output.txt', ...extra});
  return (await call('fs.read', {path: 'output.txt'})).content;
}
test('MCP line count retains real file read, result write and observed output', async context => {
  assert.equal(await computerPlan(context, 'oak\r\nbranch\r', 'count_lines'), '2\n');
});
test('MCP CSV rows preserve a bare terminal CR while normal CRLF remains a separator', async context => {
  assert.equal(await computerPlan(context, 'kind,name\nkeep,oak\r', 'filter_csv', {column: 'kind', equals: 'keep'}),
    'kind,name\nkeep,oak\r\n');
  assert.equal(await computerPlan(context, 'kind,name\r\nkeep,oak\r\n', 'filter_csv', {column: 'kind', equals: 'keep'}),
    'kind,name\nkeep,oak\n');
});
test('MCP unique CSV remains ordered and removes repeated ordinary values', async context => {
  assert.equal(await computerPlan(context, 'name\r\noak\r\nbranch\r\noak\r\n', 'unique_csv', {column: 'name'}),
    'branch\noak\n');
});
