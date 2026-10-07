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
    if (text.indexOf(args.oldString) !== text.lastIndexOf(args.oldString)) {
      return 'Error: Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.';
    }
    files.set(path, text.replace(args.oldString, () => args.newString));
    return '';
  }
  if (tool === 'bash') {
    const digest = /^sha256sum -- (\S+)$/.exec(args.command);
    if (digest) return `${createHash('sha256').update(files.get(digest[1]) ?? '').digest('hex')}  ${digest[1]}\n`;
    const cat = /^cat (\S+)$/.exec(args.command);
    if (cat) return files.get(cat[1]) ?? `cat: ${cat[1]}: No such file or directory`;
    const exists = (path) => files.has(path) || files.has(`${path}/`);
    const test = /^test (!? ?)-([ed]) (\S+)$/.exec(args.command);
    if (test) {
      const holds = test[2] === 'd' ? files.has(`${test[3]}/`) : exists(test[3]);
      return holds === (test[1] === '') ? '' : `Error: exit status 1`;
    }
    const mkdir = /^mkdir (\S+)$/.exec(args.command);
    if (mkdir) {
      files.set(`${mkdir[1]}/`, '');
      return '';
    }
    if (args.command.startsWith('git add -A && git commit')) return '5498cccac709f99fc533d81030c088d0b292e594\n';
    if (args.command === 'python3 greet.py') return 'Hello, World\n';
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

describe('PR #1188 dogfood: a replacement edits only what it names', () => {
  const README = '# Project\n\nA small tool.\nThis is a smal project.\n';

  test('`smal` -> `small` fixes the word, not the `smal` inside `small`', async () => {
    const { calls, files, answer } = await drive("Replace 'smal' with 'small' in README.md.", { 'README.md': README });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('README.md'), '# Project\n\nA small tool.\nThis is a small project.\n');
    assert.equal(answer, 'Replaced `smal` with `small` in `README.md` and observed the result.');
  });

  test('a single-quoted positional insert places the line after its anchor', async () => {
    const { calls, files, answer } = await drive("Insert 'middle' after the line 'first line' in notes.txt.", {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first line\nmiddle\nsecond line\n');
    assert.equal(answer, 'Inserted `middle` after `first line` in `notes.txt` and observed the result.');
  });

  test('an anchor that occurs twice does not say where the line goes', async () => {
    const { files, answer } = await drive("Insert 'x' before the line 'same' in t.txt.", { 't.txt': 'same\nsame\n' });
    assert.equal(files.get('t.txt'), 'same\nsame\n');
    assert.equal(answer, 'Verification failed for `t.txt`: the observed bytes differ from the planned workspace effect.');
  });
});

describe('PR #1188 dogfood: a removal takes out what it quotes', () => {
  test('`Delete the line \'second line\' from notes.txt.` removes that line', async () => {
    const { calls, files, answer } = await drive("Delete the line 'second line' from notes.txt.", {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first line\n');
    assert.equal(answer, 'Removed `second line` from `notes.txt` and observed the result.');
  });

  test('quoted text inside a line is removed from that line only', async () => {
    const { files } = await drive("Remove 'smal ' from README.md.", { 'README.md': '# P\n\nThis is a smal project.\n' });
    assert.equal(files.get('README.md'), '# P\n\nThis is a project.\n');
  });

  test('deleting a file is still a file deletion, not a text removal', async () => {
    const { calls } = await drive('Delete the file notes.txt.', { 'notes.txt': 'x\n' });
    assert.deepEqual(calls, ['bash']);
  });

  test('text found nowhere is reported, not written', async () => {
    const { files, answer } = await drive("Delete the line 'absent' from notes.txt.", { 'notes.txt': 'first line\n' });
    assert.equal(files.get('notes.txt'), 'first line\n');
    assert.equal(answer, 'Verification failed for `notes.txt`: the observed bytes differ from the planned workspace effect.');
  });
});

describe('PR #1188 dogfood: shell requests run what they name', () => {
  test('`Run python3 greet.py.` runs the command without the sentence\'s full stop', async () => {
    const { calls, answer } = await drive('Run python3 greet.py.', { 'greet.py': 'print("Hello, World")\n' });
    assert.deepEqual(calls, ['bash']);
    assert.equal(answer, 'The `python3 greet.py` command completed. Output:\n\n```text\nHello, World\n```');
  });

  test('`Create a directory named src.` makes the directory as a verified recipe, not a listing', async () => {
    const { calls, files, answer } = await drive('Create a directory named src.', {});
    assert.deepEqual(calls, ['bash', 'bash', 'bash']);
    assert.equal(files.has('src/'), true);
    assert.equal(answer, 'Completed the action `mkdir src` and verified it with `test -d src`.');
  });

  test('a commit carries the message the request quotes and pushes only where a remote exists', async () => {
    const files = new Map();
    const messages = [{ role: 'user', content: "Commit all changes with the message 'initial notes'." }];
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(JSON.parse(plan.calls[0].arguments).command,
      "git add -A && git commit -q -m 'initial notes' && if test -n \"$(git remote)\"; then git push -q origin HEAD; fi && git rev-parse HEAD");
    assert.equal(files.size, 0);
  });
});
