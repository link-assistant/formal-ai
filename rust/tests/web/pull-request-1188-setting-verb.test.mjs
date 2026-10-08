// PR #1188 dogfooding (T24): `Set beta to 5 in cfg.toml.` overwrote the key --
// `beta = 2` became `5 = 2` -- because "set" was only an edit action, so the
// unquoted key read as the text to replace. "Set" now also names a setting
// (the seeded `config_value_lead` meaning, in every registered language): the
// line assigning that key gets the value, and a key assigned nowhere is an
// honest verification failure that leaves the file untouched. The Rust twin is
// rust/tests/unit/pull_request_1188_setting_verb.rs.

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

const CONFIG = 'alpha = 1\nbeta = 2\ngamma = 3\n';

/** Run `prompt` over one file `cfg.toml`; the tools act on it as the Agent CLI's do. */
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
    if (call.tool === 'bash') result = `${createHash('sha256').update(file).digest('hex')}  cfg.toml\n`;
    calls.push(call.tool);
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, file, answer: null };
}

describe('"set" assigns a setting instead of replacing its key', () => {
  test('the line assigning the key gets the value', async () => {
    const { calls, file, answer } = await drive('Set beta to 5 in cfg.toml.', CONFIG);
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(file, 'alpha = 1\nbeta = 5\ngamma = 3\n');
    assert.equal(answer, 'Set `beta` to `5` in `cfg.toml` and observed the result.');
  });

  test('every registered language says "set" in its own words', async () => {
    const { file } = await drive('Установи beta на 5 в cfg.toml.', CONFIG);
    assert.equal(file, 'alpha = 1\nbeta = 5\ngamma = 3\n');
  });

  test('a key assigned nowhere leaves the file as it was', async () => {
    const { file } = await drive('Set delta to 5 in cfg.toml.', CONFIG);
    assert.equal(file, CONFIG);
  });
});
