// PR #1188 TEACH-D: the edit-composer gaps Formal AI showed as a subagent
// (experiments/formal_ai_subagent/gaps.md G16, G19, G20, G21, G22, G24, G28,
// G31). Before: lines under `Insert … after the line 'b' in f.lino:` lost
// their indentation (G16); `Remove the first line of f.txt.` only read the
// file (G19); a Spanish line operation answered in English (G20); a quoted
// path-shaped anchor was taken for the file (G21); `with these three lines in
// f:` replaced the line by the words (G22); `Rename the file m.py to x.py`
// rewrote the word `the` (G24); an empty line beside an anchor only read the
// file (G31). The Rust twin is
// rust/tests/unit/agentic-coding/pull_request_1188_edit_composer_gaps.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;
let composeEditRequest;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  ({ composeEditRequest } = await import('../../../js/agentic/write_request.mjs'));
});

const THREE = 'one\ntwo\nthree\n';
const LINO = 'meanings\n  table a\n    row x\n  b\nc\n';

/** Run `prompt` over one file `name` holding `source`; the tools act as the Agent CLI's do. */
async function drive(prompt, source, name) {
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
    calls.push(call.tool === 'bash' ? args.command : call.tool);
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, file, answer: null };
}

describe('lines under a request keep their structure (G16)', () => {
  test('an unfenced block is rebased on the anchor line, as its siblings', async () => {
    const { file } = await drive("Insert the following lines after the line 'b' in f.lino:\ntable q\n  row 1", LINO, 'f.lino');
    assert.equal(file, 'meanings\n  table a\n    row x\n  b\n  table q\n    row 1\nc\n');
  });

  test('a fenced block is inserted as written', async () => {
    const { file } = await drive("Insert these lines after the line 'b' in f.lino:\n```\ntable q\n```", LINO, 'f.lino');
    assert.equal(file, 'meanings\n  table a\n    row x\n  b\ntable q\nc\n');
  });

  test('appended lines keep an indentation the file already uses', async () => {
    const { file } = await drive('Append these lines to f.lino:\n  table z\n    row 9', LINO, 'f.lino');
    assert.equal(file, `${LINO}  table z\n    row 9\n`);
  });

  test('inserted lines keep an indentation the file already uses, whatever the anchor\'s (G93)', async () => {
    const prompt = "Insert these lines after the line containing 'row x' in f.lino:\n  table q\n    row 1";
    const { file } = await drive(prompt, LINO, 'f.lino');
    assert.equal(file, 'meanings\n  table a\n    row x\n  table q\n    row 1\n  b\nc\n');
  });

  test('lines under a request are its payload: a delivery or a path inside them is not acted on (G94, G95)', async () => {
    const rows = '| T527 | z, append gap G93 to `gaps.md` | **Pass** | w. |\n| T528 | x in js/a.mjs and rust/b.rs | **Pass** | y. |';
    const prompt = `In d.md, insert these lines after the line containing '| T447 |':\n${rows}`;
    const { file, calls } = await drive(prompt, '| T446 | a |\n| T447 | b |\n| T448 | c |\n', 'd.md');
    assert.equal(file, `| T446 | a |\n| T447 | b |\n${rows}\n| T448 | c |\n`);
    assert.ok(!calls.includes('write'), calls.join(', '));
  });
});

describe('a line named by its ordinal (G19)', () => {
  test('first, last and a counted line, in every registered language', async () => {
    const cases = [
      ['Remove the first line of f.txt.', 'two\nthree\n'],
      ['Delete the last line of f.txt.', 'one\ntwo\n'],
      ['Удали вторую строку из f.txt.', 'one\nthree\n'],
      ['f.txt की तीसरी पंक्ति हटाओ।', 'one\ntwo\n'],
      ['删除 f.txt 的最后一行。', 'one\ntwo\n'],
      ['Elimina la primera línea de f.txt.', 'two\nthree\n'],
    ];
    for (const [prompt, expected] of cases) assert.equal((await drive(prompt, THREE, 'f.txt')).file, expected, prompt);
    const last = await drive('Delete the last line of f.txt.', THREE, 'f.txt');
    assert.equal(last.answer, 'Deleted line(s) 3 of `f.txt` and observed the result.');
  });

  test('an ordinal is an insertion anchor', async () => {
    const { file, answer } = await drive("Insert the line 'zero' before the first line of f.txt.", THREE, 'f.txt');
    assert.equal(file, 'zero\none\ntwo\nthree\n');
    assert.equal(answer, 'Inserted `zero` before line 1 of `f.txt` and observed the result.');
  });
});

describe('the answer is in the language the request is written in (G20)', () => {
  test('a Spanish request without a Spanish marker is answered in Spanish', async () => {
    const removed = await drive('Elimina las líneas 2 a 3 de f.txt.', THREE, 'f.txt');
    assert.equal(removed.answer, 'Se eliminaron las líneas 2-3 de `f.txt` y se verificó el resultado.');
    const swapped = await drive("Intercambia las líneas 'one' y 'two' en f.txt.", THREE, 'f.txt');
    assert.equal(swapped.file, 'two\none\nthree\n');
    assert.equal(swapped.answer, 'Se intercambiaron `one` y `two` en `f.txt` y se verificó el resultado.');
  });

  test('an English request stays English', async () => {
    const { answer } = await drive('Delete lines 2-3 from f.txt.', THREE, 'f.txt');
    assert.equal(answer, 'Deleted line(s) 2-3 of `f.txt` and observed the result.');
  });
});

describe('a quoted path-shaped anchor is not the file (G21)', () => {
  test('the line goes after the path line in the named file', async () => {
    const { calls, file } = await drive("Insert the line 'foo' after the line 'js/ocr.bundle.js' in paths.txt.", 'x\njs/ocr.bundle.js\ny\n', 'paths.txt');
    assert.deepEqual(calls, ['read', 'edit', 'sha256sum -- paths.txt']);
    assert.equal(file, 'x\njs/ocr.bundle.js\nfoo\ny\n');
  });
});

describe('a new clause that describes lines takes the lines under the request (G22)', () => {
  test('the block replaces the line, at its indentation', async () => {
    const { file } = await drive(
      "Replace the line 'const X = 2;' with these three lines in f.mjs:\nconst X = 2;\nconst Y = 3;\nconst Z = 4;",
      'function f() {\n  const X = 2;\n  return X;\n}\n', 'f.mjs',
    );
    assert.equal(file, 'function f() {\n  const X = 2;\n  const Y = 3;\n  const Z = 4;\n  return X;\n}\n');
  });
});

describe('renames (G24, G28)', () => {
  test('a file renamed to a path is no edit of its bytes; the verified move runs', async () => {
    assert.equal(composeEditRequest('Rename the file m.py to math_utils.py.'), null);
    const plan = await planChatStep([{ role: 'user', content: 'Rename the file m.py to math_utils.py.' }], ['read', 'edit', 'bash', 'write']);
    assert.equal(JSON.parse(plan.calls[0].arguments).command, 'test -e m.py');
  });

  test('a function renamed to a name containing the old one', async () => {
    const source = 'export function add(a, b) {\n  return a + b;\n}\nexport function mul(a, b) {\n  return a * b;\n}\n';
    const { file, answer } = await drive('Rename the function mul to multiply in src/m.mjs.', source, 'src/m.mjs');
    assert.equal(file, source.replace('function mul(', 'function multiply('));
    assert.equal(answer, 'Renamed `mul` to `multiply` in `src/m.mjs` and observed the result.');
  });
});

describe('an empty line beside an anchor line (G31)', () => {
  test('before and after the named line', async () => {
    const readme = '# T\ntext\n## Probe\nx\n';
    const before = await drive("Insert an empty line before the line '## Probe' in README.md.", readme, 'README.md');
    assert.equal(before.file, '# T\ntext\n\n## Probe\nx\n');
    assert.equal(before.answer, 'Added an empty line to `README.md` and observed the result.');
    const after = await drive("Add a blank line after the line 'text' in README.md.", readme, 'README.md');
    assert.equal(after.file, '# T\ntext\n\n## Probe\nx\n');
  });
});

describe('a quoted anchor line holding an escape (G96)', () => {
  test('the escape is its two characters when the file holds them so inside the line', async () => {
    const source = " * `\\r`, as `split('\\n')` does.)\n * @param {string} text\n";
    const prompt = "Insert the line « * Rust built-in `str::lines`.» after the line « * `\\r`, as `split('\\n')` does.)» in m.mjs.";
    const { file } = await drive(prompt, source, 'm.mjs');
    assert.equal(file, " * `\\r`, as `split('\\n')` does.)\n * Rust built-in `str::lines`.\n * @param {string} text\n");
  });

  test('the escape stays a line break when the file holds the two lines', async () => {
    const { file } = await drive("Insert the line 'x' after the line 'one\\ntwo' in f.txt.", THREE, 'f.txt');
    assert.equal(file, 'one\ntwo\nx\nthree\n');
  });
});

describe('a path names a file, not the answer language (G97)', () => {
  test('an English request naming a file whose name starts with a Spanish marker is answered in English', async () => {
    const { file, answer } = await drive("Insert the line 'x' after the line 'one' in formalization_segment.mjs.", THREE, 'formalization_segment.mjs');
    assert.equal(file, 'one\nx\ntwo\nthree\n');
    assert.equal(answer, 'Inserted `x` after `one` in `formalization_segment.mjs` and observed the result.');
  });
});

describe('an article between the target cues is no part of the new text (G98)', () => {
  test('the file clause runs back over the seeded function words to its first cue', () => {
    assert.deepEqual(composeEditRequest("Change 'mundo' to 'amigo' in the file f.txt."), ['f.txt', 'mundo', 'amigo']);
    assert.deepEqual(composeEditRequest("Cambia 'mundo' a 'amigo' en el archivo f.txt."), ['f.txt', 'mundo', 'amigo']);
    assert.deepEqual(composeEditRequest('Replace foo with bar in the file f.txt.'), ['f.txt', 'foo', 'bar']);
    assert.deepEqual(composeEditRequest("Change 'a' to 'b' in the f.txt file."), ['f.txt', 'a', 'b']);
  });

  test('the edit writes the new text alone', async () => {
    const { file } = await drive("Change 'two' to 'deux' in the file f.txt.", THREE, 'f.txt');
    assert.equal(file, 'one\ndeux\nthree\n');
  });
});
