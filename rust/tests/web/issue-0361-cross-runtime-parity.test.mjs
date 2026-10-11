// Issue #361 (R261): the #349 behaviour (a multi-turn reverse-sort follow-up
// for a Rust list-files program) must be consistent across the Rust core and
// the browser worker. This file pins the browser-worker side of that parity.
//
// The Rust core side is tested by rust/tests/integration/issue_349_reverse_sort.rs.
// Together they satisfy R261 without either side carrying both halves alone.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { createWorkerContext, evaluate, plain } from './support/browser-runtime.mjs';

const FIRST_PROMPT =
  'Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории';
const PATH_ARGUMENT_PROMPT = 'Сделай так, чтобы программа принимала путь как аргумент';
const REVERSE_SORT_PROMPT = 'Сделай сортировку результатов в обратном порядке';

const worker = createWorkerContext();
const ready = evaluate(worker, 'loadSeed()');

async function tryWrite(prompt, history = []) {
  await ready;
  const historyJson = JSON.stringify(history);
  return plain(
    await evaluate(
      worker,
      `tryWriteProgram(${JSON.stringify(prompt)}, ${historyJson}, 'ru', {})`,
    ),
  );
}

test('R261: turn 1 Russian list-files request routes to write_program', async () => {
  const result = await tryWrite(FIRST_PROMPT);
  assert.ok(result, 'turn 1 returned null');
  assert.equal(result.intent, 'write_program', `turn 1 intent: ${result.intent}`);
});

test('R261: turn 3 path-argument follow-up resolves to list_files_arg', async () => {
  const first = await tryWrite(FIRST_PROMPT);
  assert.ok(first, 'turn 1 returned null');
  const history = [
    { role: 'user', content: FIRST_PROMPT },
    { role: 'assistant', content: first.content, intent: first.intent, evidence: first.evidence },
  ];
  const pathArg = await tryWrite(PATH_ARGUMENT_PROMPT, history);
  assert.ok(pathArg, 'turn 3 returned null');
  assert.ok(
    Array.isArray(pathArg.evidence) &&
      pathArg.evidence.includes('program_parameter:task:list_files_arg'),
    `turn 3 did not resolve list_files_arg; evidence: ${JSON.stringify(pathArg.evidence)}`,
  );
});

test('R261: turn 5 reverse-sort follow-up resolves to list_files_arg_reverse_sort (not unknown)', async () => {
  const first = await tryWrite(FIRST_PROMPT);
  assert.ok(first, 'turn 1 returned null');
  const history1 = [
    { role: 'user', content: FIRST_PROMPT },
    { role: 'assistant', content: first.content, intent: first.intent, evidence: first.evidence },
  ];
  const pathArg = await tryWrite(PATH_ARGUMENT_PROMPT, history1);
  assert.ok(pathArg, 'turn 3 returned null');
  const history2 = [
    ...history1,
    { role: 'user', content: PATH_ARGUMENT_PROMPT },
    {
      role: 'assistant',
      content: pathArg.content,
      intent: pathArg.intent,
      evidence: pathArg.evidence,
    },
  ];
  const reverseSort = await tryWrite(REVERSE_SORT_PROMPT, history2);
  assert.ok(reverseSort, 'turn 5 reverse-sort returned null (treated as unknown)');
  assert.equal(
    reverseSort.intent,
    'write_program',
    `turn 5 intent was ${reverseSort.intent}, not write_program`,
  );
  assert.ok(
    Array.isArray(reverseSort.evidence) &&
      reverseSort.evidence.includes('program_parameter:task:list_files_arg_reverse_sort'),
    `turn 5 did not resolve list_files_arg_reverse_sort; evidence: ${JSON.stringify(reverseSort.evidence)}`,
  );
});
