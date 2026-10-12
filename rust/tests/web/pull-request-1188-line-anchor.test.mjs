// PR #1188 dogfooding (T22): Formal AI was asked to add this ledger's own T21
// row with `Insert the line «| T21 | …» after the line containing '| T20 |' in
// ledger.md.` It first planned nothing -- the payload holds the word "before",
// so both position cues were read -- and then spliced the row into the middle
// of the T20 row, replacing the anchor fragment instead of the line it sits
// on. Position cues are now read outside quoted literals only, and a positional
// anchor widens to its whole line. The Rust twin is
// rust/tests/unit/pull_request_1188_line_anchor.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const TABLE = '| id | note |\n| --- | --- |\n| T20 | old row |\n| T30 | last |\n';

/** Run `prompt` over one file `t.md`; the tools act on it as the Agent CLI's do. */
async function drive(prompt, source) {
  let file = source;
  const messages = [{ role: 'user', content: prompt }];
  const calls = [];
  for (let step = 0; step < 6; step += 1) {
    const plan = await planChatStep(messages, ['read', 'edit', 'bash', 'write']);
    if (!plan || plan.kind === 'final') return { calls, file, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    let result = '';
    if (call.tool === 'read') result = file;
    if (call.tool === 'edit') file = file.replace(args.oldString, () => args.newString);
    if (call.tool === 'bash') result = `${createHash('sha256').update(file).digest('hex')}  t.md\n`;
    calls.push(call.tool);
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, file, answer: null };
}

describe('a positional insert places a whole line beside a whole line', () => {
  test('an anchor fragment widens to the line it sits on', async () => {
    const { calls, file } = await drive("Insert the line '| T21 | new row |' after the line containing '| T20 |' in t.md.", TABLE);
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(file, '| id | note |\n| --- | --- |\n| T20 | old row |\n| T21 | new row |\n| T30 | last |\n');
  });

  test('before a fragment, the new line goes above its whole line', async () => {
    const { file } = await drive("Insert the line '| T25 | mid |' before the line containing '| T30 |' in t.md.", TABLE);
    assert.equal(file, '| id | note |\n| --- | --- |\n| T20 | old row |\n| T25 | mid |\n| T30 | last |\n');
  });

  test('a position word inside the payload is text, not the position', async () => {
    const { file } = await drive(
      "Insert the line «| T21 | read before it writes |» after the line containing '| T20 |' in t.md.", TABLE);
    assert.equal(file, '| id | note |\n| --- | --- |\n| T20 | old row |\n| T21 | read before it writes |\n| T30 | last |\n');
  });

  test('a whole-line anchor is unchanged', async () => {
    const { file } = await drive("Insert 'x' after the line '| T20 | old row |' in t.md.", TABLE);
    assert.equal(file, '| id | note |\n| --- | --- |\n| T20 | old row |\nx\n| T30 | last |\n');
  });

  test('lines under a request that ends in a colon are the text inserted (T38)', async () => {
    const { calls, file } = await drive(
      "In t.md, insert these lines after the line '| T20 | old row |':\n\n    | T21 | \"a\" |\n    | T22 | b |\n", TABLE);
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(file, '| id | note |\n| --- | --- |\n| T20 | old row |\n| T21 | "a" |\n| T22 | b |\n| T30 | last |\n');
  });
});
