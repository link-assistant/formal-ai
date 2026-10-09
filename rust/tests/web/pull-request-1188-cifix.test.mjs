// PR #1188 CIFIX: defects found while making the CI run 37738783201 failures
// green (dogfood ledger rows T180-T189 in
// docs/case-studies/pull-request-1188/formal-ai-dogfood.md). Each request is
// replayed through the planner the JS server runs (`planChatStep`) over an
// in-memory workspace. The Rust twin is
// rust/tests/unit/agentic-coding/pull_request_1188_cifix.rs.

import { runPlanEvent } from './helpers/plan-event-shell.mjs';
import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const TOOLS = ['read', 'write', 'edit', 'bash'];

/** Plan until the first write and return the written content (null when none). */
async function firstWrite(prompt, path, source) {
  const messages = [{ role: 'user', content: prompt }];
  for (let turn = 0; turn < 4; turn += 1) {
    const plan = await planChatStep(messages, TOOLS);
    if (!plan || plan.kind !== 'tool_calls') return null;
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    if (call.tool === 'write') return args.content;
    const id = `call_${turn}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    const result = call.tool === 'read' && (args.filePath ?? args.file_path ?? args.path) === path ? source : '';
    messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: result });
  }
  return null;
}

describe('a member the request quotes twice is inserted once (T180)', () => {
  test('the ladder node prompt repeats the member in its result clause', async () => {
    const path = 'src/intents.rs';
    const source = 'const OTHER_INTENTS: &[&str] = &[\n    "code_debugging",\n    "regex_synthesis",\n];\n';
    const prompt = `Edit the tracked file \`${path}\`: add "code_formatting" to the OTHER_INTENTS list. `
      + 'Change only that file and keep it valid Rust. Then write a result line that states the change '
      + 'and contains the exact text "code_formatting".';
    const written = await firstWrite(prompt, path, source);
    assert.ok(written, 'the planner writes the edited list');
    assert.equal(written.split('"code_formatting"').length - 1, 1, written);
  });
});

/** Plan up to six steps and return what is written to `path` (null when nothing is). */
async function writeTo(prompt, path, source) {
  const files = new Map([[path, source]]);
  const messages = [{ role: 'user', content: prompt }];
  for (let turn = 0; turn < 6; turn += 1) {
    const plan = await planChatStep(messages, TOOLS);
    if (!plan || plan.kind !== 'tool_calls') return null;
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    const named = args.filePath ?? args.file_path ?? args.path;
    if (call.tool === 'write' && named === path) return args.content;
    const id = `call_${turn}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    const event = call.tool === 'bash' ? runPlanEvent(files, args.command) : null;
    const result = event ?? (call.tool === 'read' && named === path ? source : '');
    messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: result });
  }
  return null;
}

describe('setting a file\'s contents writes the file (G68, issue #745)', () => {
  for (const prompt of ['set the contents of note.txt to hello', 'pon el contenido de note.txt en hello']) {
    test(prompt, async () => {
      assert.equal(await writeTo(prompt, 'note.txt', 'old\n'), 'hello');
    });
  }
});

describe('seeded Spanish assignment retains same-statement write authority', () => {
  for (const [prompt, source] of [['pon el contenido de heldout.txt en «λ🙂 42»', 'prior bytes\n'], ['coloca el contenido de heldout.txt en «λ🙂 42»', '']]) {
    test(prompt, async () => {
      assert.equal(await writeTo(prompt, 'heldout.txt', source), 'λ🙂 42');
    });
  }
  for (const prompt of ['lee el contenido de heldout.txt', 'muestra el contenido de heldout.txt',
    'el contenido de heldout.txt en hello', 'pon el contenido. de heldout.txt en hello',
    'coloca el contenido de heldout.txt en «λ🙂 42»']) {
    test('refuses a write for ' + prompt, async () => {
      assert.equal(await writeTo(prompt, 'heldout.txt', 'prior bytes\n'), null);
    });
  }
});
