import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';
import {indexByBasename, renameRustModules}
  from '../../../experiments/formal_ai_subagent/rename-by-rule.mjs';

const file = 'rust/tests/unit/coding_discovery/python_docs.rs';
const original = readFileSync(new URL('../../tests/unit/coding_discovery/python_docs.rs', import.meta.url), 'utf8');
const sources = new Map([
  ['rust/Cargo.toml', readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8')],
  ['rust/src/lib.rs', readFileSync(new URL('../../src/lib.rs', import.meta.url), 'utf8')],
  ['rust/src/coding/mod.rs', readFileSync(new URL('../../src/coding/mod.rs', import.meta.url), 'utf8')],
]);
const move = {
  from: 'rust/src/coding/function_catalog/python_docs.rs',
  to: 'rust/src/coding/function_catalog/python_documentation.rs',
};
const inventory = [move.from, file];

test('original source-owned exported module alias retains the actual unit import', () => {
  const result = renameRustModules(original, file, [move], indexByBasename(inventory), sources);
  const expected = original.replace(
    'formal_ai::coding_function_catalog::python_docs::fetch_index',
    'formal_ai::coding_function_catalog::python_documentation::fetch_index');
  assert.notEqual(expected, original);
  assert.equal(result.text, expected);
  assert.equal(result.count, 1);
});
