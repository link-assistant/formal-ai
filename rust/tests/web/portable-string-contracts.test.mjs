import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { test } from 'node:test';
import { REPO_ROOT } from '../../../js/server/lino.mjs';
import { parseLinks, textOf } from '../../../scripts/self-translation/lino.mjs';
import { stringProgramToMeta, renderStringRust, assertCompleteStringIR } from '../../../scripts/self-translation/portable-string.mjs';
import { generatePortableContracts, SOURCE, META, RUST, TYPESCRIPT } from '../../../scripts/generate-portable-contracts.mjs';
import * as contracts from '../../../js/agentic/portable/string_contracts.mjs';
const file = relative => {
  return fs.readFileSync(new URL('../../../' + relative, import.meta.url), 'utf8');
};
const cases = parseLinks(file('rust/tests/fixtures/portable-string-contracts.lino'));
const source = file(SOURCE);
function runCases(namespace) {
  for (const link of cases) {
    const name = link[1],
      functionName = textOf(link.find(row => {
        return Array.isArray(row) && row[0] === 'function';
      })[1]);
    const args = link.find(row => {
      return Array.isArray(row) && row[0] === 'arguments';
    }).slice(1).map(row => {
      return textOf(row[1]);
    });
    const expected = textOf(link.find(row => {
      return Array.isArray(row) && row[0] === 'expected';
    })[1]);
    assert.equal(namespace[functionName](...args), expected, name);
  }
}
test('shared declarative corpus executes the actual JavaScript contracts', () => {
  return runCases(contracts);
});
test('every TypeScript twin executes the same contracts and emitted files match the exact Links IR', () => {
  const outputs = generatePortableContracts(source);
  for (const target of [META, RUST, TYPESCRIPT]) assert.equal(file(target), outputs[target], target);
  const context = vm.createContext({});
  vm.runInContext(outputs[TYPESCRIPT].replaceAll('export function', 'function') + '\nglobalThis.calls={quoteShellWord,joinFailureInvitation};', context);
  runCases(context.calls);
  assert.equal(renderStringRust(outputs[META]), outputs[RUST].split('\n').slice(2).join('\n'));
});
for (const [name, value] of [['async', '/** @param {string} x */ export async function example(x){return x;}'], ['side effects', '/** @param {string} x */ export function example(x){console.log(x);return x;}'], ['untyped', 'export function example(x){return x;}'], ['unknown call', '/** @param {string} x */ export function example(x){return other(x);}'], ['dynamic replacement', '/** @param {string} x */ export function example(x){return x.replaceAll(x,"a");}'], ['empty needle', '/** @param {string} x */ export function example(x){return x.replaceAll("","a");}'], ['replacement expansion', '/** @param {string} x */ export function example(x){return x.replaceAll("a","$&");}'], ['coercion', '/** @param {string} x */ export function example(x){return x+1;}']]) test('unsupported ' + name + ' is a named refusal rather than a carried success', () => {
  return assert.throws(() => {
    return stringProgramToMeta(value);
  }, /portable-string-v1/);
});
test('unknown and carried IR records cannot be silently dropped', () => {
  const meta = stringProgramToMeta(source);
  assert.throws(() => {
    return assertCompleteStringIR(meta + '(carried function unsupported)\n');
  });
  assert.throws(() => {
    return renderStringRust(meta + '(unknown ignored)\n');
  });
});
test('changing the canonical quote changes both generated roots without editing either implementation', () => {
  const changed = source.replace('"\'" + value.replaceAll', '"prefix" + value.replaceAll');
  assert.notEqual(changed, source);
  assert.notEqual(generatePortableContracts(changed)[RUST], file(RUST));
  assert.notEqual(generatePortableContracts(changed)[TYPESCRIPT], file(TYPESCRIPT));
});
test('all maintained shell adapters and both invitation wrappers delegate to the canonical contract', () => {
  for (const name of ['general_planner', 'git_commit', 'workspace_search', 'local_search', 'report_script', 'algorithm_learning']) {
    const text = file('js/agentic/' + name + '.mjs');
    assert.ok(text.includes('quoteShellWord'), name);
  }
  assert.ok(file('js/agentic/crate/failure_reporting.mjs').includes('joinFailureInvitation'));
  assert.ok(file('rust/src/failure_reporting.rs').includes('join_failure_invitation'));
});

test('canonical meta symbols and parameter references project generically to Rust identifiers', () => {
  const renamed = '/** @param {string} inputValue @param {string} fallbackValue */ export function choosePresentValue(inputValue, fallbackValue){return inputValue === "" ? fallbackValue : inputValue;}';
  const meta = stringProgramToMeta(renamed);
  assert.ok(meta.includes('(function choose-present-value '));
  assert.ok(meta.includes('(parameters (input-value string) (fallback-value string))'));
  assert.ok(meta.includes('(variable input-value)'));
  const rust = renderStringRust(meta);
  assert.ok(rust.includes('pub fn choose_present_value(input_value: &str, fallback_value: &str)'));
  assert.ok(rust.includes('input_value.is_empty()'));
  assert.ok(rust.includes('fallback_value.to_owned()'));
});
test('noncanonical and colliding meta symbol spellings cannot alias emitted Rust names', () => {
  const meta = stringProgramToMeta('/** @param {string} inputValue */ export function preserveValue(inputValue){return inputValue;}');
  for (const changed of [
    meta.replace('preserve-value', 'preserve_value'),
    meta.replace('preserve-value', 'preserve--value'),
    meta.replace('input-value string', 'input_value string'),
    meta.replace('(input-value string)', '(input-value string) (input-value string)'),
    meta.replace('(variable input-value)', '(variable input_value)'),
    meta + meta.split('\n').filter(line => line.startsWith('(function')).join('\n') + '\n',
  ]) assert.throws(() => renderStringRust(changed), /portable-string-v1/);
});
