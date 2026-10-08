// PR #1188 dogfooding (T29, T30, T33, T35 and probe gaps G3, G4): whole-line
// operations a coding agent is asked for every day. Before: "delete the line
// 'A' and the 'B' line directly above it" was routed to a whole-file write
// that replaced a 380-line file with one line (T29); "Delete lines 266-267
// from f.lino." only read the file (T30); "Insert the line '…' after line N"
// planned nothing (T33); a quoted path-shaped needle in a removal was read as
// the file (T35); "Move the line 'x' to the top" duplicated it (G4); "Swap
// the lines 'a' and 'b'" planned nothing (G3). Each is now a computed change
// over the file's lines (js/agentic/workspace_line_operation.mjs), and a
// computed change that would drop most of a file without the request stating
// that extent is refused. The Rust twin is
// rust/tests/unit/pull_request_1188_line_operations.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;
let planRoutedCapabilityStep;
let RoutingStage;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  ({ planRoutedCapabilityStep, RoutingStage } = await import('../../../js/agentic/capability_router.mjs'));
});

const SIX = 'one\ntwo\nthree\nfour\nfive\nsix\n';
const CFG = 'alpha = 1\nbeta = 2\ngamma = 3\n';

/** Run `prompt` over one file `name` holding `source`; the tools act as the Agent CLI's do. */
async function drive(prompt, source, name = 'f.txt') {
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
    if (call.tool === 'write') file = args.content;
    if (call.tool === 'bash') result = `${createHash('sha256').update(file).digest('hex')}  ${name}\n`;
    calls.push(call.tool);
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, file, answer: null };
}

describe('numbered lines are removed (T30)', () => {
  test('a range, a "to" range and one line', async () => {
    const range = await drive('Delete lines 2-3 from f.txt.', SIX);
    assert.deepEqual(range.calls, ['read', 'edit', 'bash']);
    assert.equal(range.file, 'one\nfour\nfive\nsix\n');
    assert.equal(range.answer, 'Deleted line(s) 2-3 of `f.txt` and observed the result.');
    assert.equal((await drive('Delete lines 2 to 3 from f.txt.', SIX)).file, 'one\nfour\nfive\nsix\n');
    assert.equal((await drive('Remove line 6 from f.txt.', 'one\ntwo\nthree\nfour\nfive\nsix')).file, 'one\ntwo\nthree\nfour\nfive');
  });

  test('every registered language names the range', async () => {
    for (const prompt of ['Удали строки с 2 по 3 из f.txt.', 'f.txt से पंक्तियाँ 2 से 3 हटाओ।', '删除 f.txt 的第2到3行。', 'Elimina las líneas 2 a 3 de f.txt.']) {
      assert.equal((await drive(prompt, SIX)).file, 'one\nfour\nfive\nsix\n', prompt);
    }
    assert.equal((await drive('Удали строки с 2 по 3 из f.txt.', SIX)).answer, 'Из `f.txt` удалены строки 2-3, результат проверен.');
  });

  test('a line past the end is an honest failure and the file is untouched', async () => {
    const { calls, file, answer } = await drive('Delete line 9 from f.txt.', SIX);
    assert.deepEqual(calls, ['read']);
    assert.equal(file, SIX);
    assert.equal(answer, 'Verification failed for `f.txt`: the observed bytes differ from the planned workspace effect.');
  });

  test('a numbered line and the line above it (T29 shape, numbered)', async () => {
    assert.equal((await drive('Delete line 4 and the line above it from f.txt.', SIX)).file, 'one\ntwo\nfive\nsix\n');
  });
});

describe('a quoted line and its neighbour are removed together (T29)', () => {
  const LINO = 'header\n      surface\n        text "get it"\n      surface\n        text find\nfooter\n';

  test('the line and the named line directly above it', async () => {
    const { calls, file, answer } = await drive(
      "In m.lino, delete the line '        text \"get it\"' and the '      surface' line directly above it.", LINO, 'm.lino');
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(file, 'header\n      surface\n        text find\nfooter\n');
    assert.equal(answer, 'Removed `      surface`, `        text "get it"` from `m.lino` and observed the result.');
  });

  test('a neighbour that is not the named line is refused, never guessed', async () => {
    const { calls, file } = await drive("In m.lino, delete the line 'footer' and the 'header' line directly above it.", LINO, 'm.lino');
    assert.deepEqual(calls, ['read']);
    assert.equal(file, LINO);
  });

  test('a removal is never routed to a whole-file write', () => {
    const prompt = "In m.lino, delete the line '        text \"get it\"' and the '      surface' line directly above it.";
    const plan = planRoutedCapabilityStep(prompt, [{ role: 'user', content: prompt }], ['read', 'write', 'edit', 'bash'], RoutingStage.NamedOrLocal);
    assert.ok(!(plan?.calls ?? []).some((call) => call.tool === 'write'));
  });
});

describe('an insert anchored at a numbered line (T33)', () => {
  test('after and before line N', async () => {
    const after = await drive("Insert the line 'X' after line 2 in f.txt.", SIX);
    assert.deepEqual(after.calls, ['read', 'edit', 'bash']);
    assert.equal(after.file, 'one\ntwo\nX\nthree\nfour\nfive\nsix\n');
    assert.equal(after.answer, 'Inserted `X` after line 2 of `f.txt` and observed the result.');
    assert.equal((await drive("Insert the line 'X' before line 1 in f.txt.", SIX)).file, 'X\none\ntwo\nthree\nfour\nfive\nsix\n');
    assert.equal((await drive("Insert the line 'X' after line 6 in f.txt.", SIX)).file, `${SIX}X\n`);
  });

  test('in Russian', async () => {
    assert.equal((await drive("Вставь строку 'X' после строки 2 в f.txt.", SIX)).file, 'one\ntwo\nX\nthree\nfour\nfive\nsix\n');
  });
});

describe('a quoted path is the payload when the file is named unquoted (T35)', () => {
  test('every line containing the path goes from the named file', async () => {
    const source = 'keep a\nrust/src/x.rs\nkeep b\n  - rust/src/x.rs (old)\nkeep c\n';
    const { calls, file } = await drive("Delete the lines containing 'rust/src/x.rs' from allow.txt.", source, 'allow.txt');
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(file, 'keep a\nkeep b\nkeep c\n');
  });
});

describe('a line is moved, never duplicated (G4)', () => {
  test('to the top, to the end, after another line', async () => {
    const top = await drive("Move the line 'gamma = 3' to the top of cfg.toml.", CFG, 'cfg.toml');
    assert.equal(top.file, 'gamma = 3\nalpha = 1\nbeta = 2\n');
    assert.equal(top.answer, 'Moved `gamma = 3` to the start of `cfg.toml` and observed the result.');
    assert.equal((await drive("Move the line 'alpha = 1' to the end of cfg.toml.", CFG, 'cfg.toml')).file, 'beta = 2\ngamma = 3\nalpha = 1\n');
    assert.equal((await drive("Move the line 'alpha = 1' after the line 'beta = 2' in cfg.toml.", CFG, 'cfg.toml')).file,
      'beta = 2\nalpha = 1\ngamma = 3\n');
    assert.equal((await drive("Перемести строку 'gamma = 3' в начало cfg.toml.", CFG, 'cfg.toml')).file, 'gamma = 3\nalpha = 1\nbeta = 2\n');
  });
});

describe('two lines are swapped (G3)', () => {
  test('the two whole lines exchange places', async () => {
    const { calls, file, answer } = await drive("Swap the lines 'alpha = 1' and 'beta = 2' in cfg.toml.", CFG, 'cfg.toml');
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(file, 'beta = 2\nalpha = 1\ngamma = 3\n');
    assert.equal(answer, 'Swapped `alpha = 1` and `beta = 2` in `cfg.toml` and observed the result.');
    assert.equal((await drive("交换 cfg.toml 中的 'alpha = 1' 和 'gamma = 3'。", CFG, 'cfg.toml')).file, 'gamma = 3\nbeta = 2\nalpha = 1\n');
  });
});

describe('a computed change never leaves a fragment of the file (T29)', () => {
  const MOSTLY = `${Array.from({ length: 9 }, (_, index) => `x ${index}`).join('\n')}\nkeep\n`;

  test('a removal by content that would drop most of the file is refused', async () => {
    const { calls, file, answer } = await drive("Delete the lines containing 'x' from f.txt.", MOSTLY);
    assert.deepEqual(calls, ['read']);
    assert.equal(file, MOSTLY);
    assert.equal(answer, 'Refused to change `f.txt`: the computed result would drop most of the file, and the request does not ask for that.');
  });

  test('a numbered range states its extent and may', async () => {
    assert.equal((await drive('Delete lines 1-9 from f.txt.', MOSTLY)).file, 'keep\n');
  });
});
