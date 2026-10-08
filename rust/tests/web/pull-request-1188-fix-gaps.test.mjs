// PR #1188 FIX-GAPS: the open edit gaps of experiments/formal_ai_subagent/gaps.md
// G90, G99, G102, G104, G106 and G107 (ledger rows T371, T377, T471, T473,
// T720, T786, T793 and T794 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md;
// the fixes are logged as rows T840-T869). Each request is replayed through
// the planner the JS server runs (`planChatStep`) over an in-memory workspace
// whose tools answer as the Agent CLI's do. The Rust twin is
// rust/tests/unit/pull_request_1188_fix_gaps.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;
let sequenceSteps;
let wholePayloadEnd;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  ({ sequenceSteps } = await import('../../../js/agentic/request_sequence.mjs'));
  ({ wholePayloadEnd } = await import('../../../js/agentic/quote_nesting.mjs'));
});

const TOOLS = ['bash', 'edit', 'glob', 'grep', 'list', 'read', 'write'];
const pathOf = (args) => args.filePath ?? args.file_path ?? args.path;

/** The Agent CLI's tools over an in-memory workspace; bash knows `sha256sum --` and `cat`. */
function execute(files, tool, args) {
  if (tool === 'read') {
    const path = pathOf(args);
    return files.has(path) ? files.get(path) : `Error: File not found: ${path}`;
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
    const presence = args.command.match(/^test (! )?-e (\S+)$/u);
    if (presence) return files.has(presence[2]) !== Boolean(presence[1]) ? '' : ['Output: ', 'Error: ', 'Exit Code: 1'].join(String.fromCharCode(10));
    if (args.command.startsWith('mkdir -p -- ')) return '';
    const transfer = args.command.match(/^(cp|mv) (\S+) (\S+)$/u);
    if (transfer && files.has(transfer[2])) {
      files.set(transfer[3], files.get(transfer[2]));
      if (transfer[1] === 'mv') files.delete(transfer[2]);
      return '';
    }
    const digest = /^sha256sum -- (\S+)$/u.exec(args.command);
    if (digest) return `${createHash('sha256').update(files.get(digest[1]) ?? '').digest('hex')}  ${digest[1]}\n`;
    const cat = /^cat (\S+)$/u.exec(args.command);
    if (cat) return files.get(cat[1]) ?? `Output: \nError: cat: ${cat[1]}: No such file or directory\nExit Code: 1`;
  }
  return `Error: ${tool} is not simulated`;
}

async function drive(prompt, workspace, maxSteps = 12) {
  const files = new Map(Object.entries(workspace));
  const messages = [{ role: 'user', content: prompt }];
  const calls = [];
  for (let step = 0; step < maxSteps; step += 1) {
    const plan = await planChatStep(messages, TOOLS);
    if (!plan || plan.kind === 'final') return { calls, files, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    const id = `call_${step}`;
    calls.push(call.tool);
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: execute(files, call.tool, args) });
  }
  return { calls, files, answer: null };
}

const MODULE = 'import { A } from "x";\nfoo(C);\n';
const LINES = 'a\nb\nc\nd\n';
const ROW = 'row A. one\nrow C two\n';

describe('G104: an edit naming files listed with `and` or commas is declined, naming every file', () => {
  test('`In a.mjs and b.mjs, …` names both files and changes neither', async () => {
    const prompt = 'In a.mjs and b.mjs, replace «A» with «B» and replace every «C» with «D».';
    const { calls, files, answer } = await drive(prompt, { 'a.mjs': MODULE, 'b.mjs': MODULE });
    assert.deepEqual(calls, []);
    assert.equal(files.get('a.mjs'), MODULE);
    assert.equal(files.get('b.mjs'), MODULE);
    assert.equal(answer, 'This edit names several files (`a.mjs`, `b.mjs`), and one edit request changes one file, so nothing was changed. '
      + 'Ask for the edit once for each file.');
  });

  test('a comma list of three files names all three', async () => {
    const { calls, answer } = await drive('In a.txt, b.txt and c.txt, replace «x» with «y».', { 'a.txt': 'x\n', 'b.txt': 'x\n', 'c.txt': 'x\n' });
    assert.deepEqual(calls, []);
    assert.equal(answer, 'This edit names several files (`a.txt`, `b.txt`, `c.txt`), and one edit request changes one file, so nothing was changed. '
      + 'Ask for the edit once for each file.');
  });

  test('a path in a later sentence (the ladder leaf asking for evidence) is no edit target', async () => {
    const prompt = 'In the file rust/src/core.rs, replace "statement_negation_cue" with "statement_negation_marker". '
      + 'Leave supporting evidence in .agent-ladder/node-2.1-proof.md.';
    const { files, answer } = await drive(prompt, { 'rust/src/core.rs': 'const statement_negation_cue: u8 = 1;\n' });
    assert.equal(files.get('rust/src/core.rs'), 'const statement_negation_marker: u8 = 1;\n');
    assert.equal(answer, 'Replaced `statement_negation_cue` with `statement_negation_marker` in `rust/src/core.rs` and observed the result.');
  });

  test('one file followed by a comma and the edit is still one edit', async () => {
    const { files, answer } = await drive('In a.mjs, replace «A» with «B».', { 'a.mjs': MODULE });
    assert.equal(files.get('a.mjs'), 'import { B } from "x";\nfoo(C);\n');
    assert.equal(answer, 'Replaced `A` with `B` in `a.mjs` and observed the result.');
  });
});

describe('G99: edits joined by a sequence cue are planned step by step', () => {
  test('`…, then delete …` inserts and then deletes, each over the file the step before left', async () => {
    const prompt = 'Insert the line «b2» after the line «b» in f.txt, then delete the line «c» from f.txt.';
    const { files, answer } = await drive(prompt, { 'f.txt': LINES });
    assert.equal(files.get('f.txt'), 'a\nb\nb2\nd\n');
    assert.equal(answer, 'Inserted `b2` after `b` in `f.txt` and observed the result.\n\nRemoved `c` from `f.txt` and observed the result.');
  });

  test('a step that names no file takes the file the other names', async () => {
    const prompt = 'In f.txt, insert the line «b2» after the line «b» and then delete the line «c».';
    const { files } = await drive(prompt, { 'f.txt': LINES });
    assert.equal(files.get('f.txt'), 'a\nb\nb2\nd\n');
  });

  test('the Russian cue `затем` is a step too, answered in Russian', async () => {
    const prompt = 'Вставь строку «b2» после строки «b» в f.txt, затем удали строку «c» из f.txt.';
    const { files, answer } = await drive(prompt, { 'f.txt': LINES });
    assert.equal(files.get('f.txt'), 'a\nb\nb2\nd\n');
    assert.equal(answer, 'В `f.txt` после `b` вставлено `b2`, результат проверен.\n\nИз `f.txt` удалено `c`, результат проверен.');
  });

  test('a cue that opens no clause, or a step without a quoted text, is no sequence', () => {
    assert.equal(sequenceSteps('Replace «then» with «now» in f.txt.'), null);
    assert.equal(sequenceSteps('Replace «a» with «b» in f.txt, then run the tests.'), null);
    assert.deepEqual(sequenceSteps('In f.txt, replace «a» with «b», then replace «c» with «d».'),
      ['In f.txt, replace «a» with «b»', 'replace «c» with «d» in f.txt']);
  });
});

describe('G106: several Replaces apply all or decline, never drop one', () => {
  test('`and then replace` applies both', async () => {
    const prompt = 'In r.md, replace «A.» with «B.» and then replace «C» with «D».';
    const { files, answer } = await drive(prompt, { 'r.md': ROW });
    assert.equal(files.get('r.md'), 'row B. one\nrow D two\n');
    assert.equal(answer, 'Replaced `A.` with `B.` in `r.md` and observed the result.\n\nReplaced `C` with `D` in `r.md` and observed the result.');
  });

  test('a second clause that is no quoted replacement is named, and nothing changes', async () => {
    const { calls, files, answer } = await drive('In r.md, replace «A.» with «B.» and replace C with D.', { 'r.md': ROW });
    assert.deepEqual(calls, []);
    assert.equal(files.get('r.md'), ROW);
    assert.equal(answer, 'This request asks for several edits, and I cannot read `replace C with D` as a replacement of quoted text, '
      + 'so nothing was changed. Quote its old and new texts, or ask for that edit on its own.');
  });
});

describe('G90: a quoted text holding its own quote mark is declined, naming the quote', () => {
  const RUST = "    (text.matches('`').count() % 2 == 0).then_some(text)\n";

  test('single-quoted texts holding single quotes (T471)', async () => {
    const prompt = "Replace '(text.matches('`').count() % 2 == 0).then_some(text)' with "
      + "'text.matches('`').count().is_multiple_of(2).then_some(text)' in p.rs.";
    const { calls, files, answer } = await drive(prompt, { 'p.rs': RUST });
    assert.deepEqual(calls, []);
    assert.equal(files.get('p.rs'), RUST);
    assert.equal(answer, "A quoted text in this request holds its own quote mark (``'(text.matches('`').count() % 2``), so I cannot tell where it ends, "
      + 'and nothing was done. Quote such a text with «», or with a mark it does not hold.');
  });

  test('double-quoted texts ending in a double quote of their own (T377)', async () => {
    const { calls, files, answer } = await drive('Replace "a synthesized."" with "a holds."" in f.lino.', { 'f.lino': 'x = "a synthesized."\n' });
    assert.deepEqual(calls, []);
    assert.equal(files.get('f.lino'), 'x = "a synthesized."\n');
    assert.equal(answer, 'A quoted text in this request holds its own quote mark (`" with "a holds."" in f.lino.`), so I cannot tell where it ends, '
      + 'and nothing was done. Quote such a text with «», or with a mark it does not hold.');
  });

  test('the same texts in «» are edited', async () => {
    const { files } = await drive('Replace «"a synthesized."» with «"a holds."» in f.lino.', { 'f.lino': 'x = "a synthesized."\n' });
    assert.equal(files.get('f.lino'), 'x = "a holds."\n');
  });
});

describe('G107: a backticked payload whose inner spans pair runs to its own close', () => {
  test('a sentence end inside the payload keeps the whole replacement', async () => {
    const { files, answer } = await drive('In r.md, replace `x` with `a. `b` x`.', { 'r.md': 'one x two\n' });
    assert.equal(files.get('r.md'), 'one a. `b` x two\n');
    assert.equal(answer, 'Replaced `x` with ``a. `b` x`` in `r.md` and observed the result.');
  });

  test('the T794 shape: paths, a semicolon and a sentence end inside the payload', async () => {
    const prompt = 'In r.md, replace `Remaining: x` with `In chat, "f <u>" (the `p_f` meaning of `d/m.lino`, in five) fetch '
      + '(`r/p.rs`, `r/w.rs`; worker `j/w.js`); offline nothing. `t/u.rs` and `t/w.mjs` pin it. Remaining: x`';
    const { files } = await drive(prompt, { 'r.md': '| U18 | Remaining: x, since y. |\n' });
    assert.equal(files.get('r.md'), '| U18 | In chat, "f <u>" (the `p_f` meaning of `d/m.lino`, in five) fetch (`r/p.rs`, `r/w.rs`; '
      + 'worker `j/w.js`); offline nothing. `t/u.rs` and `t/w.mjs` pin it. Remaining: x, since y. |\n');
  });

  test('the payload ends where its own mark closes, before a later sentence', () => {
    const request = 'In r.md, replace `x` with `a. `b` x`. Then add `z`.';
    const from = request.indexOf('with ') + 'with '.length;
    assert.equal(request.slice(from, wholePayloadEnd(request, from, request.indexOf('. `b'))), '`a. `b` x`');
  });
});

describe('G102: a file described rather than given is never written from a failed or refused step', () => {
  test('a create request whose names and values are «» fragments writes nothing', async () => {
    const prompt = 'Create the file abbreviations.test.mjs. It imports node:assert/strict as assert and node:test as test, and imports '
      + 'wordsOfIdentifier from «./scripts/measure-abbreviations.mjs». Write one test:\n'
      + '1. test «a declared name splits at underscores»: assert.deepEqual(wordsOfIdentifier(«parseArgs»), [«parse», «args»]).';
    const { calls, files, answer } = await drive(prompt, { 'scripts/measure-abbreviations.mjs': 'export function wordsOfIdentifier() {}\n' });
    // The planner plans no step: no `cat` of the fragments and no write of a
    // failed or refused step. The server's solver then answers that it cannot
    // write the program (experiments/js_dogfood/drive.mjs falls through to it).
    assert.deepEqual(calls, []);
    assert.equal(answer, null);
    assert.deepEqual([...files.keys()], ['scripts/measure-abbreviations.mjs']);
  });
});

test('G109: copy then comma-joined replacements preserves the source', async () => {
  const { files, answer } = await drive('Copy a.lino to b.lino, then in b.lino replace every «x» with «y» and replace «m» with «n».', { 'a.lino': 'x m\n' }, 24);
  assert.equal(files.get('a.lino'), 'x m\n');
  assert.equal(files.get('b.lino'), 'y n\n');
  assert.ok(answer && !answer.includes('failed'));
});

test('G109: move then a quoted edit runs in order', async () => {
  const { files, answer } = await drive('Move a.lino to b.lino, then in b.lino replace «x» with «y».', { 'a.lino': 'x\n' }, 24);
  assert.equal(files.has('a.lino'), false);
  assert.equal(files.get('b.lino'), 'y\n');
  assert.ok(answer && !answer.includes('failed'));
});

test('G108: scalar payload code spans keep comma-separated backticks whole', async () => {
  const { renderSeededChange } = await import('../../../js/agentic/code_task.mjs');
  const payload = 'values `left`, `right` remain scalar';
  const answer = renderSeededChange('coding_text_replaced', 'Replace text in f.txt', 'f.txt', [['{old}', 'x'], ['{new}', payload]]);
  assert.equal(answer, 'Replaced `x` with ``values `left`, `right` remain scalar`` in `f.txt` and observed the result.');
});

test('G110: payload file paths and cues cannot become the creation target', async () => {
  const content = 'record destination to rust/src/solver_handlers/policy_gates.rs';
  const { files } = await drive('Create budget.lino with the content «' + content + '».', {});
  assert.equal(files.get('budget.lino'), content);
  assert.equal(files.has('rust/src/solver_handlers/policy_gates.rs'), false);
});

test('G113: exact quoted content retains indentation and terminal newline', async () => {
  const content = '  seed method-execution\n    bundle true\n';
  const { files } = await drive('Create rows.lino with the content «' + content + '».', {});
  assert.equal(files.get('rows.lino'), content);
  const single = await drive('Create r.md with exactly this content «x\n».', {});
  assert.equal(single.files.get('r.md'), 'x\n');
});

test('G114: an objective label in quoted source is data', async () => {
  const { objectiveText } = await import('../../../js/agentic/general_planner.mjs');
  const payload = ['pub fn emit(', '    task: &str,', ') {}'].join(String.fromCharCode(10));
  const prompt = 'In f.rs replace «old» with «' + payload + '»';
  assert.equal(objectiveText(prompt), prompt);
  assert.equal(objectiveText('Task: Read f.txt'), 'Read f.txt');
});

test('G115: replacement preserves a new-only source newline escape', async () => {
  const old = 'const lines = text;';
  const next = 'const lines = text.split(' + String.fromCharCode(39, 92) + 'n' + String.fromCharCode(39) + ');';
  const { files } = await drive('In f.mjs replace «' + old + '» with «' + next + '»', { 'f.mjs': old });
  assert.equal(files.get('f.mjs'), next);
});
