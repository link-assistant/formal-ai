// PR #1188 TEACH-E: edit-composer and line-operation gaps found by driving
// Formal AI as a subagent (experiments/formal_ai_subagent/gaps.md, ledger rows
// T150-T169 in docs/case-studies/pull-request-1188/formal-ai-dogfood.md).
// Each request is replayed through the planner the JS server runs
// (`planChatStep`) over an in-memory workspace whose tools answer as the
// Agent CLI's do. The Rust twin is rust/tests/unit/pull_request_1188_teach_e.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;
let quotedSegments;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  ({ quotedSegments } = await import('../../../js/agentic/crate/normal_markov.mjs'));
});

const TOOLS = ['bash', 'edit', 'glob', 'grep', 'list', 'read', 'write'];
const pathOf = (args) => args.filePath ?? args.file_path ?? args.path;

function agentRead(text) {
  const body = text.split('\n').map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
  return `<file>\n${body}\n\n(End of file - total ${text.split('\n').length} lines)\n</file>`;
}

/** The Agent CLI's tools over an in-memory workspace; bash knows `sha256sum`, `cat`, `test -e`, `mkdir -p` and `mv`. */
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
    if (text.indexOf(args.oldString) !== text.lastIndexOf(args.oldString)) return 'Error: Found multiple matches for oldString.';
    files.set(path, text.replace(args.oldString, () => args.newString));
    return '';
  }
  if (tool === 'bash') {
    const digest = /^sha256sum -- (\S+)$/u.exec(args.command);
    if (digest) return `${createHash('sha256').update(files.get(digest[1]) ?? '').digest('hex')}  ${digest[1]}\n`;
    const cat = /^cat (\S+)$/u.exec(args.command);
    if (cat) return files.get(cat[1]) ?? `cat: ${cat[1]}: No such file or directory`;
    const exists = /^test (!? ?)-e (\S+)$/u.exec(args.command);
    if (exists) return files.has(exists[2]) === (exists[1] === '') ? '' : 'Output: \nError: \nExit Code: 1';
    if (args.command.startsWith('mkdir -p -- ')) return '';
    const move = /^mv (\S+) (\S+)$/u.exec(args.command);
    if (move) {
      files.set(move[2], files.get(move[1]));
      files.delete(move[1]);
      return '';
    }
  }
  return `Error: ${tool} is not simulated`;
}

async function drive(prompt, workspace, maxSteps = 8) {
  const files = new Map(Object.entries(workspace));
  const messages = [{ role: 'user', content: prompt }];
  const commands = [];
  const calls = [];
  for (let step = 0; step < maxSteps; step += 1) {
    const plan = await planChatStep(messages, TOOLS);
    if (!plan || plan.kind === 'final') return { calls, commands, files, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    if (call.tool === 'bash') commands.push(args.command);
    const id = `call_${step}`;
    calls.push(call.tool);
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: execute(files, call.tool, args) });
  }
  return { calls, commands, files, answer: null };
}

describe('G60: a quoted line is the whole line that equals it', () => {
  const NOTES = 'alpha\nx\nbox\nfox\n';

  test('`Delete the line \'x\'` removes the line that is x, not every line holding the letter', async () => {
    const { calls, files, answer } = await drive("Delete the line 'x' from f.md.", { 'f.md': NOTES });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('f.md'), 'alpha\nbox\nfox\n');
    assert.equal(answer, 'Removed `x` from `f.md` and observed the result.');
  });

  test('with no equal line and several containing it, nothing is removed', async () => {
    const { files } = await drive("Delete the line 'x' from f.md.", { 'f.md': 'alpha\nbox\nfox\n' });
    assert.equal(files.get('f.md'), 'alpha\nbox\nfox\n');
  });

  test('the seeded containment cue removes every line containing the text', async () => {
    for (const prompt of ["Delete the lines containing 'ox' from f.md.", "Delete the lines that contain 'ox' from f.md."]) {
      const { files, answer } = await drive(prompt, { 'f.md': NOTES });
      assert.equal(files.get('f.md'), 'alpha\nx\n', prompt);
      assert.equal(answer, 'Deleted 2 line(s) containing `ox` from `f.md` and observed the result.');
    }
  });
});

describe('G61: a payload that quotes a single-quoted literal of its own', () => {
  const PROMPT = "Append the line '- G31 \"Delete the line 'x' from f.md.\" removed lines' to g2.md.";

  test('the outer quote pair is the payload', () => {
    assert.deepEqual(quotedSegments(PROMPT), ["- G31 \"Delete the line 'x' from f.md.\" removed lines"]);
    assert.deepEqual(quotedSegments("Replace 'a' with 'b' in f."), ['a', 'b']);
    assert.deepEqual(quotedSegments("Insert 'don't' after 'x' in f"), ["don't", 'x']);
  });

  test('the line is appended; no file is moved', async () => {
    const { commands, files } = await drive(PROMPT, { 'g2.md': '# gaps\n', 'f.md': 'keep\n' });
    assert.ok(!commands.some((command) => command.startsWith('mv ')), commands.join(' | '));
    assert.equal(files.get('f.md'), 'keep\n');
    assert.equal(files.get('g2.md'), "# gaps\n- G31 \"Delete the line 'x' from f.md.\" removed lines\n");
  });
});

describe('G53, G54: several quoted line literals are lines, each as written', () => {
  test('an insert keeps a backslash-n inside one of several literals', async () => {
    const { files } = await drive("Insert the lines '  text \"a\\nb\"' and '  row z' after the line '  row a' in f.lino.",
      { 'f.lino': 'table t\n  row a\nend\n' });
    assert.equal(files.get('f.lino'), 'table t\n  row a\n  text "a\\nb"\n  row z\nend\n');
  });

  test('an append of a list of literals appends each as its own line, never a whole-file write', async () => {
    const { calls, files } = await drive("Append the lines '  row b', '  row c \"q\" d' to f.lino.", { 'f.lino': 'table t\n  row a\n' });
    assert.ok(!calls.includes('write'));
    assert.equal(files.get('f.lino'), 'table t\n  row a\n  row b\n  row c "q" d\n');
    const joined = await drive("Append the lines 'b1' and 'b2' to f.txt.", { 'f.txt': 'a\n' });
    assert.equal(joined.files.get('f.txt'), 'a\nb1\nb2\n');
  });
});

describe('G65: a replace whose needle quotes an escape the file holds as written', () => {
  test('the needle is the text as written when the file has no unescaped match', async () => {
    const source = "const x = parts.join('\\n');\n";
    const { files } = await drive("Replace «parts.join('\\n')» with «parts.join('\\n\\n')» in m.mjs.", { 'm.mjs': source });
    assert.equal(files.get('m.mjs'), "const x = parts.join('\\n\\n');\n");
  });

  test('an escape that names a real line break still does', async () => {
    const { files } = await drive("Replace 'a\\nb' with 'c' in f.txt.", { 'f.txt': 'a\nb\n' });
    assert.equal(files.get('f.txt'), 'c\n');
  });
});

describe('G50, G51: another file\'s contents are the text placed', () => {
  test('`Append the contents of a.txt to the end of b.txt.` reads a.txt and appends its lines; no cat', async () => {
    const { calls, commands, files, answer } = await drive('Append the contents of a.txt to the end of b.txt.',
      { 'a.txt': 'a1\n  a2\n', 'b.txt': 'b1\n' });
    assert.deepEqual(calls, ['read', 'read', 'edit', 'bash']);
    assert.ok(!commands.some((command) => command.startsWith('cat ')));
    assert.equal(files.get('b.txt'), 'b1\na1\n  a2\n');
    assert.equal(files.get('a.txt'), 'a1\n  a2\n');
    assert.equal(answer, 'Appended `a1\n  a2` to the end of `b.txt` and observed the result.');
  });

  test('an insert places the source after the anchor its context names', async () => {
    const { files } = await drive("Insert the contents of rows.txt after the line 'Y' that follows the line 'Z' in f.lino.",
      { 'rows.txt': 'r1\nr2\n', 'f.lino': 'X\nY\nZ\nY\n' });
    assert.equal(files.get('f.lino'), 'X\nY\nZ\nY\nr1\nr2\n');
  });

  test('a block under the request may also name the line its anchor follows', async () => {
    const { files } = await drive("Insert these lines after the line '  Y' that follows the line 'Z' in f.lino:\n```\n  r1\n```",
      { 'f.lino': 'X\n  Y\nZ\n  Y\n' });
    assert.equal(files.get('f.lino'), 'X\n  Y\nZ\n  Y\n  r1\n');
  });
});

describe('G33: a line slice answers only those lines', () => {
  const N = 'one\ntwo\nthree\nfour\n';
  for (const [prompt, expected] of [
    ['Show the last 2 lines of n.txt.', 'Lines 3-4 of `n.txt`:\n\n```text\nthree\nfour\n```'],
    ['Show the first 2 lines of n.txt.', 'Lines 1-2 of `n.txt`:\n\n```text\none\ntwo\n```'],
    ['Show lines 2-3 of n.txt.', 'Lines 2-3 of `n.txt`:\n\n```text\ntwo\nthree\n```'],
    ['Show the last line of n.txt.', 'Lines 4-4 of `n.txt`:\n\n```text\nfour\n```'],
  ]) {
    test(prompt, async () => {
      const { calls, answer } = await drive(prompt, { 'n.txt': N });
      assert.deepEqual(calls, ['read']);
      assert.equal(answer, expected);
    });
  }

  test('a read that names no lines still shows the file', async () => {
    const { answer } = await drive('Show n.txt.', { 'n.txt': N });
    assert.equal(answer, 'Contents of `n.txt`:\n\n```text\none\ntwo\nthree\nfour\n```');
  });
});

describe('G35: an insert at one end of a Markdown section', () => {
  const README = '# Demo\n\nIntro.\n\n## Usage\n\nRun it.\n\n### Flags\n\n- a\n\n## License\n\nMIT\n';

  test('the end of a section is its last line before the next heading of the same or a higher level', async () => {
    const { calls, files, answer } = await drive("Insert the line 'See also docs.' at the end of the section '## Usage' in README.md.",
      { 'README.md': README });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('README.md'), README.replace('- a\n', '- a\nSee also docs.\n'));
    assert.equal(answer, 'Inserted `See also docs.` after `- a` in `README.md` and observed the result.');
  });

  test('the start of a section is under its heading', async () => {
    const { files } = await drive("Add the line 'First.' at the start of the section '## Usage' in README.md.", { 'README.md': README });
    assert.equal(files.get('README.md'), README.replace('## Usage\n\n', '## Usage\n\nFirst.\n'));
  });

  test('a heading inside fenced code does not end the section', async () => {
    const source = '## A\n\n```sh\n# comment\n```\n\n## B\n';
    const { files } = await drive("Append the line 'x' to the end of the section '## A' in r.md.", { 'r.md': source });
    assert.equal(files.get('r.md'), '## A\n\n```sh\n# comment\n```\nx\n\n## B\n');
  });
});

describe('G37: a rename with both paths quoted moves the file', () => {
  for (const prompt of ["Rename the file 'a.txt' to 'b.txt'.", 'Rename the file a.txt to b.txt.', 'Rename the file "a.txt" to "b.txt".']) {
    test(prompt, async () => {
      const { commands, files } = await drive(prompt, { 'a.txt': 'x\n' });
      assert.ok(commands.includes('mv a.txt b.txt'), commands.join(' | '));
      assert.deepEqual([...files.entries()], [['b.txt', 'x\n']]);
    });
  }

  test('a rename inside a file still edits it', async () => {
    const { files } = await drive("Rename the import './a.mjs' to './b.mjs' in m.mjs.", { 'm.mjs': "import x from './a.mjs';\n" });
    assert.equal(files.get('m.mjs'), "import x from './b.mjs';\n");
  });
});

describe('G62: a backticked compound command runs as written', () => {
  test('`Run \\`a && b\\`` runs the one compound command, never reads its operands', async () => {
    const files = { 'a.lino': 'a\n', 'b.lino': 'b\n', 'c.lino': 'c\n', 'd.lino': 'd\n' };
    const { calls, commands } = await drive('Run `cat a.lino >> b.lino && cat c.lino >> d.lino`', files, 2);
    assert.deepEqual(calls, ['bash']);
    assert.deepEqual(commands, ['cat a.lino >> b.lino && cat c.lino >> d.lino']);
  });
});

describe('G52 (not reproduced; pinned): a replace inside a 7000-character line', () => {
  test('the needle is found, edited once and the digest matches', async () => {
    const long = `    note "${'word '.repeat(700)}Lowered from 37 to 36 on 2026-10-08 by the thirteenth ${'tail '.repeat(700)}"`;
    const source = `ratchet\n${long}\nend\n`;
    for (let attempt = 0; attempt < 3; attempt += 1) {
      const { calls, files, answer } = await drive("Replace 'Lowered from 37 to 36' with 'Lowered from 37 to 35' in d.lino.", { 'd.lino': source });
      assert.deepEqual(calls, ['read', 'edit', 'bash']);
      assert.equal(files.get('d.lino'), source.replace('37 to 36', '37 to 35'));
      assert.equal(answer, 'Replaced `Lowered from 37 to 36` with `Lowered from 37 to 35` in `d.lino` and observed the result.');
    }
  });
});
