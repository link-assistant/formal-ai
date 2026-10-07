// PR #1188: tasks the coordinator delegated to the JavaScript Formal AI as a
// sub-agent, replayed through the planner the JS server runs (`planChatStep`)
// with the Agent CLI's tools and tool-result shapes. Each prompt is one that
// failed before its fix: a path, a cue word or a full stop inside the quoted
// payload was read as part of the request instead of as payload.

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

const AGENT_CLI_TOOLS = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read', 'task',
  'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];

const pathOf = (args) => args.filePath ?? args.file_path ?? args.path;

function agentRead(text) {
  const body = text.split('\n').map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
  return `<file>\n${body}\n\n(End of file - total ${text.split('\n').length} lines)\n</file>`;
}

/** The Agent CLI's read/edit/write tools and the digest and cat observations, in memory. */
function execute(files, tool, args) {
  const path = pathOf(args);
  if (tool === 'read') return files.has(path) ? agentRead(files.get(path)) : `Error: File not found: ${path}`;
  if (tool === 'write') {
    files.set(path, args.content);
    return '';
  }
  if (tool === 'edit') {
    const text = files.get(path) ?? '';
    if (!text.includes(args.oldString)) return 'Error: oldString not found in content';
    if (text.indexOf(args.oldString) !== text.lastIndexOf(args.oldString)) return 'Error: Found multiple matches for oldString.';
    files.set(path, text.replace(args.oldString, () => args.newString));
    return '';
  }
  if (tool === 'bash') {
    const digest = /^sha256sum -- (\S+)$/.exec(args.command);
    if (digest) return `${createHash('sha256').update(files.get(digest[1]) ?? '').digest('hex')}  ${digest[1]}\n`;
    const cat = /^cat (\S+)$/.exec(args.command);
    if (cat) return files.get(cat[1]) ?? `cat: ${cat[1]}: No such file or directory`;
  }
  return `Error: ${tool} is not simulated`;
}

async function drive(prompt, workspace, maxSteps = 8) {
  const files = new Map(Object.entries(workspace));
  const messages = [{ role: 'user', content: prompt }];
  const reads = [];
  for (let step = 0; step < maxSteps; step += 1) {
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    if (!plan || plan.kind === 'final') return { reads, files, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    if (call.tool === 'read') reads.push(pathOf(args));
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: execute(files, call.tool, args) });
  }
  return { reads, files, answer: null };
}

const ROW = '| R361 | Links. | Implemented; covered by the same release-script unit test. |\n';

describe('a quoted replacement may carry a path, a cue word and a full stop', () => {
  test('the named file is edited, not the path inside the new text', async () => {
    const { reads, files, answer } = await drive(
      "Replace 'covered by the same release-script unit test.' with 'covered by the same release-script unit test, compiled into the unit suite through `rust/tests/unit/ci-cd/mod.rs`.' in req.md.",
      { 'req.md': ROW },
    );
    assert.deepEqual(reads, ['req.md']);
    assert.equal(
      files.get('req.md'),
      '| R361 | Links. | Implemented; covered by the same release-script unit test, compiled into the unit suite through `rust/tests/unit/ci-cd/mod.rs`. |\n',
    );
    assert.equal(files.has('rust/tests/unit/ci-cd/mod.rs'), false);
    assert.equal(
      answer,
      'Replaced `covered by the same release-script unit test.` with `covered by the same release-script unit test, compiled into the unit suite through `rust/tests/unit/ci-cd/mod.rs`.` in `req.md` and observed the result.',
    );
  });
});

describe('an inserted line may itself name a path', () => {
  test('the import line is inserted into the named module, not read as the target', async () => {
    const { reads, files } = await drive(
      "Insert the line \"import { b } from './crate/b.mjs';\" after the line \"import { a } from './crate/a.mjs';\" in mod.mjs.",
      { 'mod.mjs': "import { a } from './crate/a.mjs';\nexport const x = 1;\n" },
    );
    assert.deepEqual(reads, ['mod.mjs']);
    assert.equal(
      files.get('mod.mjs'),
      "import { a } from './crate/a.mjs';\nimport { b } from './crate/b.mjs';\nexport const x = 1;\n",
    );
  });
});

describe('an append keeps the file it appends to', () => {
  const LEDGER = '# Ledger\n\nError: oldString not found in content\n- the run failed, then passed\n';

  test('a file whose text quotes error lines is read as the file, not as a missing one', async () => {
    const { files, answer } = await drive("Append the line '- one more entry' to ledger.md.", { 'ledger.md': LEDGER });
    assert.equal(files.get('ledger.md'), `${LEDGER}- one more entry\n`);
    assert.equal(answer, 'Appended `- one more entry` to the end of `ledger.md` and observed the result.');
  });

  test('"append an empty line" adds one empty line and nothing else', async () => {
    const { files } = await drive('Append an empty line to ledger.md.', { 'ledger.md': LEDGER });
    assert.equal(files.get('ledger.md'), `${LEDGER}\n`);
  });

  // An unquoted line is grounded between the seeded line lead and the last
  // destination cue before the path (PR #1188, gap 1); one with no lead is
  // still declined, never a whole-file rewrite.
  test('an unquoted line after a line lead is appended', async () => {
    const { files } = await drive('Append the line third to notes.txt.', { 'notes.txt': 'first\nsecond\n' });
    assert.equal(files.get('notes.txt'), 'first\nsecond\nthird\n');
  });

  test('an append the arm cannot ground is declined, never a whole-file rewrite', async () => {
    const { files } = await drive('Append third to notes.txt.', { 'notes.txt': 'first\nsecond\n' });
    assert.equal(files.get('notes.txt'), 'first\nsecond\n');
  });
});
