import assert from 'node:assert/strict';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {checkListedFiles} from '../../../scripts/check-file-size.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url)).replace(/\/$/u, '');
const files = [
  'scripts/check-dependencies-latest.rs',
  'scripts/check-dependencies-snapshot.rs',
  "js/worker/formal_ai_worker_coding_idioms_and_text_manipulation.js",
  "js/worker/formal_ai_worker_arithmetic_and_numeric_lists.js",
  "js/worker/formal_ai_worker_solve.js",
  "js/worker/formal_ai_worker_solver_events.js",
  "ts/worker/formal_ai_worker_coding_idioms_and_text_manipulation.ts",
  "ts/worker/formal_ai_worker_arithmetic_and_numeric_lists.ts",
  "ts/worker/formal_ai_worker_solve.ts",
  "ts/worker/formal_ai_worker_solver_events.ts",
  "experiments/issue-1580/evidence/checks/java-historical-forward-035/manifest.json",
  "experiments/issue-1580/evidence/checks/template-delta-20261122/manifest.json",

  'scripts/detect-code-changes.rs',
  'scripts/detect-code-changes-tests.rs',
  'rust/src/coding/list_program.rs',
  'rust/src/coding/list_program/literals.rs',
  'rust/tests/unit/specification/chat_surface.rs',
  'rust/tests/unit/specification/chat_surface/rule_details.rs',
  'rust/tests/unit/specification/debug_session.rs',
  'rust/tests/unit/specification/debug_session/calculator_origins.rs',
];
test('reported source warnings have headroom under the maintained policy', {timeout: 10000}, () => {
  const observed = checkListedFiles(root, new Set(files));
  assert.deepEqual(observed.errors, []);
  assert.deepEqual(observed.violations, []);
  assert.deepEqual(observed.warnings, []);
});
