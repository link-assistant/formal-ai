// R1188-U1: generalize, don't specialize. scripts/check-prompt-specialization.mjs
// notices a prompt that a test sends to Formal AI held verbatim in code, which
// is a branch, cue or canned answer keyed to that one prompt, and holds their
// count to data/meta/prompt-specialization-ratchet.lino.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

import { codeLines, promptsOf, specializations } from '../../../scripts/check-prompt-specialization.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

test('prompts are the natural-language literals a test sends', () => {
  const source = [
    'const run = await drive("Append the line x to notes.txt", files);',
    "assert.equal(solve('What is your name?'), 'Formal AI');",
    'let answer = chat("ok");',
    'const prompt = "read ${path} and fix it now";',
    "const label = 'a label that is no prompt at all';",
  ].join('\n');
  assert.deepEqual(promptsOf(source), ['Append the line x to notes.txt', 'What is your name?']);
});

test('only code lines count, never comments', () => {
  const code = codeLines('// What is your name? is answered from seed\nif (text === "What is your name?") {}');
  assert.equal(code, 'if (text === "What is your name?") {}');
  const found = specializations(['What is your name?', 'Never in code at all'], [{ path: 'js/a.js', code }]);
  assert.deepEqual(found, [{ path: 'js/a.js', prompt: 'What is your name?' }]);
});

test('the repository holds the specialization ratchet', () => {
  const output = execFileSync('node', ['scripts/check-prompt-specialization.mjs'], { cwd: REPO_ROOT, encoding: 'utf8' });
  assert.match(output, /test prompts held verbatim in code: (\d+) \(ceiling \1\)/u);
});

test('the scan reads the app JSX too, and rust/src and the worker hold no pair', () => {
  const output = execFileSync('node', ['scripts/check-prompt-specialization.mjs', '--list'], { cwd: REPO_ROOT, encoding: 'utf8' });
  const listed = output.split('\n').filter((line) => line.startsWith('  '));
  assert.ok(listed.every((line) => /^ {2}js\/app\/[\w-]+\.jsx: /u.test(line)), output);
});
