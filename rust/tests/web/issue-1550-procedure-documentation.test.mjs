// Issue #1550 (acceptance criterion f): the README table and
// docs/meta-algorithm.md describe what the code does, and this check fails when
// they diverge: the tables differ, an event is not the procedure's, a named
// function is gone or does not emit its event, an event is never executed, or
// the entry chain from a runtime's solver no longer reaches the procedure.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { DOCUMENTS, procedureDocDivergences } from './helpers/issue-1550-procedure-documentation.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const read = (file) => readFileSync(new URL(file, new URL('../../../', import.meta.url)), 'utf8');
const both = (edit) => Object.fromEntries(DOCUMENTS.map((file) => [file, edit(read(file))]));

test('README and docs/meta-algorithm.md describe the procedure the code runs', () => {
  assert.deepEqual(procedureDocDivergences(ROOT), []);
});

test('the README no longer describes the recorded loop as the answer path', () => {
  const readme = read('README.md');
  for (const stale of ['record_candidates', 'record_validation', 'smallest sufficient candidate`', 'are not yet the procedure']) {
    assert.ok(!readme.includes(stale), `README still says ${stale}`);
  }
});

test('the check fails when one document diverges from the other', () => {
  const texts = { [DOCUMENTS[0]]: read(DOCUMENTS[0]).replace('`compose` |', '`compose`, `split_request` |') };
  assert.ok(procedureDocDivergences(ROOT, texts).some((message) => message.includes('describe the procedure differently')));
});

test('the check fails when a documented function is gone or does not emit its event', () => {
  const renamed = procedureDocDivergences(ROOT, both((text) => text.replace('`derive_task_test`', '`derive_task_tests`')));
  assert.ok(renamed.some((message) => message.includes('`derive_task_tests`') && message.includes('not defined')), renamed.join('\n'));
  const wrong = procedureDocDivergences(ROOT, both((text) => text.replace(
    '| `meta_exec:compose` | `draft_candidates`, `compose` |',
    '| `meta_exec:compose` | `compose` |',
  )));
  assert.ok(wrong.some((message) => message.includes('none of compose emits meta_exec:compose')), wrong.join('\n'));
});

test('the check fails when the documented events are not the ones the procedure emits', () => {
  const swapped = procedureDocDivergences(ROOT, both((text) => text.replace('`meta_exec:select`', '`meta_exec:choose`')));
  assert.ok(swapped.some((message) => message.includes('Rust emits')), swapped.join('\n'));
  assert.ok(swapped.some((message) => message.includes('JavaScript emits')), swapped.join('\n'));
});

test('the check fails when the entry chain no longer reaches the procedure', () => {
  const broken = procedureDocDivergences(ROOT, both((text) => text.replace(
    '`run_meta_procedure` → `solve_answer`',
    '`run_meta_procedure` → `solve_turn`',
  )));
  assert.ok(broken.some((message) => message.includes('`run_meta_procedure` does not call `solve_turn`')), broken.join('\n'));
  const executor = procedureDocDivergences(ROOT, both((text) => text.replace('| `solveRecursively` |', '| `solveRecursivelyWithin` |')));
  assert.ok(executor.some((message) => message.includes('no function of the entry chain runs `solveRecursivelyWithin`')), executor.join('\n'));
});
