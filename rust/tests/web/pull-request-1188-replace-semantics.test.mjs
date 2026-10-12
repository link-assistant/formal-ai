// PR #1188 dogfooding (T50-T55): replace, setting and multi-insert semantics
// Formal AI still got wrong.
//
// - G2: `Change MAXIMUM_RATIO from 8 to 16 in p.rs.` sent the old clause
//   `MAXIMUM_RATIO from 8` to the edit tool. A seeded old-value lead
//   (`file_edit_old_lead_cue`) now states the value the key holds, and a
//   declaration line (`const K: T = v;`) is an assignment.
// - T32: `Replace the line 'text pay' with …` also changed `text pays`. A
//   request that names a line replaces whole lines.
// - T31: several quoted lines inserted at one anchor, and two insert clauses
//   in one request (`…, and insert …`), made no edit.
// - T34: `… after the line 'y' that follows 'z'` -- the anchor is the `y`
//   after `z` (`file_edit_anchor_context_cue`).
// - `Append these lines to f:` with lines under it wrote an answer over the
//   whole file; the lines are now the text appended (fenced lines verbatim).
//
// The Rust twin is rust/tests/unit/pull_request_1188_replace_semantics.rs.

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

const digest = (text) => createHash('sha256').update(text).digest('hex');

/** Run `prompt` over the workspace `files`; the tools act on it as the Agent CLI's do. */
async function drive(prompt, files) {
  const workspace = { ...files };
  const messages = [{ role: 'user', content: prompt }];
  const calls = [];
  for (let step = 0; step < 10; step += 1) {
    const plan = await planChatStep(messages, ['read', 'edit', 'bash', 'write']);
    if (!plan || plan.kind === 'final') return { calls, workspace, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    const path = args.filePath ?? args.path;
    let result = '';
    if (call.tool === 'read') result = workspace[path] ?? 'Error: file not found';
    if (call.tool === 'edit') workspace[path] = workspace[path].replace(args.oldString, () => args.newString);
    if (call.tool === 'write') workspace[path] = args.content;
    if (call.tool === 'bash') {
      const target = args.command.split(' ').pop();
      result = `${digest(workspace[target] ?? '')}  ${target}\n`;
    }
    calls.push(call.tool);
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, workspace, answer: null };
}

const RUST = 'fn f() {\n    const MAXIMUM_RATIO: u32 = 8;\n    let x = 8;\n}\n';

describe('a stated old value names the assignment it changes (G2)', () => {
  test('a declaration line gets the new value; the other 8 stays', async () => {
    const { calls, workspace, answer } = await drive('Change MAXIMUM_RATIO from 8 to 16 in p.rs.', { 'p.rs': RUST });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(workspace['p.rs'], 'fn f() {\n    const MAXIMUM_RATIO: u32 = 16;\n    let x = 8;\n}\n');
    assert.equal(answer, 'Set `MAXIMUM_RATIO` to `16` in `p.rs` and observed the result.');
  });

  test('the same request in Russian', async () => {
    const { workspace } = await drive('Измени MAXIMUM_RATIO с 8 на 16 в p.rs.', { 'p.rs': RUST });
    assert.equal(workspace['p.rs'], 'fn f() {\n    const MAXIMUM_RATIO: u32 = 16;\n    let x = 8;\n}\n');
  });

  test('only the named key changes, and a wrong old value changes nothing', async () => {
    const config = 'ratio = 8\nlimit = 8\n';
    assert.equal((await drive('Change limit from 8 to 9 in c.toml.', { 'c.toml': config })).workspace['c.toml'],
      'ratio = 8\nlimit = 9\n');
    const { calls, workspace } = await drive('Change limit from 7 to 9 in c.toml.', { 'c.toml': config });
    assert.deepEqual(calls, ['read']);
    assert.equal(workspace['c.toml'], config);
  });

  test('"bump" is a setting verb, with the file named before the new value (G16)', async () => {
    const pkg = '{\n  "name": "x",\n  "version": "1.0.0",\n  "private": true\n}\n';
    const { calls, workspace } = await drive('Bump the version in package.json to 1.1.0.', { 'package.json': pkg });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(workspace['package.json'], pkg.replace('1.0.0', '1.1.0'));
  });

  test('a key assigned nowhere changes the stated value where it occurs once as a word', async () => {
    const { workspace } = await drive('Change the ratio from 8 to 12 in n.md.', { 'n.md': 'The ratio is 8 here.\n' });
    assert.equal(workspace['n.md'], 'The ratio is 12 here.\n');
  });
});

describe('a request that names a line replaces whole lines (T32)', () => {
  const LINO = '      surface\n        text pay\n      surface\n        text pays\n';

  test('the line equal to the quoted text, never text inside a longer line', async () => {
    const { workspace } = await drive("Replace the line '        text pay' with '        text \"pay\"' in m.lino.", { 'm.lino': LINO });
    assert.equal(workspace['m.lino'], '      surface\n        text "pay"\n      surface\n        text pays\n');
  });

  test('a line quoted without its indentation keeps the indentation it has', async () => {
    const { workspace } = await drive("Replace the line 'text pay' with 'text \"pay\"' in m.lino.", { 'm.lino': LINO });
    assert.equal(workspace['m.lino'], '      surface\n        text "pay"\n      surface\n        text pays\n');
  });
});

describe('a line named by the line it follows is replaced alone (T62)', () => {
  test('only the repeated line after the context changes, as one edit', async () => {
    const ledger = '  handler a\n    status pending\n  handler b\n    status pending\n';
    const { calls, workspace } = await drive(
      "In l.lino, replace the line '    status pending' that follows the line '  handler b' with '    status migrated'.",
      { 'l.lino': ledger });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(workspace['l.lino'], '  handler a\n    status pending\n  handler b\n    status migrated\n');
  });
});

describe('several lines and several inserts in one request (T31)', () => {
  test('two quoted lines go in together, in order', async () => {
    const { calls, workspace } = await drive(
      "In m.lino, insert the two lines '      surface' and '        text find' after the line '        text solve'.",
      { 'm.lino': 'a\n        text solve\nb\n' });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(workspace['m.lino'], 'a\n        text solve\n      surface\n        text find\nb\n');
  });

  test('two insert clauses in one file are two edits, verified once', async () => {
    const { calls, workspace, answer } = await drive(
      "Insert the line 'x1' after the line 'a' in m.lino, and insert the line 'y1' before the line 'b' in m.lino.",
      { 'm.lino': 'a\nmid\nb\n' });
    assert.deepEqual(calls, ['read', 'edit', 'edit', 'bash']);
    assert.equal(workspace['m.lino'], 'a\nx1\nmid\ny1\nb\n');
    assert.equal(answer.split('\n').length, 2);
  });

  test('two insert clauses in two files read, edit and verify each', async () => {
    const { calls, workspace } = await drive(
      "Insert 'x' after 'a' in one.txt, and insert 'y' after 'b' in two.txt.",
      { 'one.txt': 'a\n', 'two.txt': 'b\n' });
    assert.deepEqual(calls, ['read', 'edit', 'read', 'edit', 'bash', 'bash']);
    assert.deepEqual(workspace, { 'one.txt': 'a\nx\n', 'two.txt': 'b\ny\n' });
  });

  test('literals not joined as a list are no insert', async () => {
    const { workspace } = await drive("Insert 'x' after 'a', 'b' in one.txt.", { 'one.txt': 'a\nb\n' });
    assert.equal(workspace['one.txt'], 'a\nb\n');
  });
});

describe('an anchor named by the line it follows (T34)', () => {
  test('the repeated anchor after the context is the one used', async () => {
    const registry = '  seed alpha\n    bundle true\n  seed coding-guidance\n    bundle true\n  seed omega\n';
    const { calls, workspace } = await drive(
      "Insert '    web true' after the line '    bundle true' that follows '  seed coding-guidance' in r.lino.",
      { 'r.lino': registry });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(workspace['r.lino'],
      '  seed alpha\n    bundle true\n  seed coding-guidance\n    bundle true\n    web true\n  seed omega\n');
  });

  test('after the context, a line that is the anchor outranks one that contains it', async () => {
    const { workspace } = await drive("Insert 'b' after the line 'text set' that follows 'role' in s.lino.",
      { 's.lino': 'text set\nrole\ntext setting\ntext set\n' });
    assert.equal(workspace['s.lino'], 'text set\nrole\ntext setting\ntext set\nb\n');
  });
});

describe('lines under an append request are the text appended', () => {
  test('indented lines lose their shared indentation', async () => {
    const { calls, workspace } = await drive('Append these lines to n.txt:\n  x\n  y', { 'n.txt': 'a\nb\n' });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(workspace['n.txt'], 'a\nb\nx\ny\n');
  });

  test('a `when … then` inside the block is text, not a skill being taught (T57)', async () => {
    const { workspace } = await drive('Append these lines to l.md:\n```\n| T | when `a` holds, then `b` |\n```', { 'l.md': '# L\n' });
    assert.equal(workspace['l.md'], '# L\n| T | when `a` holds, then `b` |\n');
  });

  test('a fenced block keeps its own indentation', async () => {
    const { workspace } = await drive('Append these lines to s.lino:\n```\n  k\n    v\n```', { 's.lino': 'meanings\n' });
    assert.equal(workspace['s.lino'], 'meanings\n  k\n    v\n');
  });
});
