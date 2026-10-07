// PR #1188 dogfooding ledger (docs/case-studies/pull-request-1188/formal-ai-dogfood.md):
// every task the JavaScript Formal AI failed while driving the real
// link-assistant Agent CLI, replayed through the planner the JS server runs
// (`planChatStep`) with the Agent CLI's advertised tools and its tool-result
// shapes (`read` → numbered `<file>` block, `write`/`edit` → empty string,
// `bash` → stdout). The workspace is an in-memory map, so a regression shows
// up as the exact wrong file content the CLI run produced.

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

/** The Agent CLI's tools over an in-memory workspace; the bash tool knows `cat` and `sha256sum`. */
function execute(files, tool, args) {
  if (tool === 'read') {
    const path = pathOf(args);
    return files.has(path) ? agentRead(files.get(path)) : `Error: File not found: ${path}`;
  }
  if (tool === 'write') {
    files.set(pathOf(args), args.content);
    return '';
  }
  if (tool === 'edit') {
    const path = pathOf(args);
    const text = files.get(path) ?? '';
    if (!text.includes(args.oldString)) return 'Error: oldString not found in content';
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
  const calls = [];
  for (let step = 0; step < maxSteps; step += 1) {
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    if (!plan || plan.kind === 'final') return { calls, files, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const id = `call_${step}`;
    const result = execute(files, call.tool, JSON.parse(call.arguments));
    calls.push(call.tool);
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, files, answer: null };
}

describe('PR #1188 dogfood: an append keeps the file it appends to', () => {
  test('`Append the line \'third line\' to notes.txt.` adds one line instead of overwriting the file', async () => {
    const { calls, files, answer } = await drive("Append the line 'third line' to notes.txt.", {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first line\nsecond line\nthird line\n');
    assert.equal(answer, 'Appended `third line` to the end of `notes.txt` and observed the result.');
    assert.equal(files.has('.formal-ai/general-change-plan.lino'), false);
  });

  test('a prepend puts the quoted line first', async () => {
    const { files, answer } = await drive('Prepend "# Notes" to notes.txt', { 'notes.txt': 'first line\n' });
    assert.equal(files.get('notes.txt'), '# Notes\nfirst line\n');
    assert.equal(answer, 'Added `# Notes` to the start of `notes.txt` and observed the result.');
  });

  test('a file without a final newline gains the line without one', async () => {
    const { files } = await drive("Append 'second' to n.txt", { 'n.txt': 'no newline' });
    assert.equal(files.get('n.txt'), 'no newline\nsecond');
  });

  test('appending to a missing file creates it with the line', async () => {
    const { calls, files, answer } = await drive("Append 'hello' to new.txt", {});
    assert.deepEqual(calls, ['read', 'write', 'bash']);
    assert.equal(files.get('new.txt'), 'hello\n');
    assert.equal(answer, 'Appended `hello` to the end of `new.txt` and observed the result.');
  });

  test('the position words are seeded per language, not English-only', async () => {
    const { files, answer } = await drive('Добавь строку «третья строка» в конец notes.txt', {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.equal(files.get('notes.txt'), 'first line\nsecond line\nтретья строка\n');
    assert.equal(answer, 'В конец `notes.txt` добавлено `третья строка`, результат проверен.');
  });
});
