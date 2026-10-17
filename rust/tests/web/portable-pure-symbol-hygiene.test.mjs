import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { toMeta, fromMeta } from '../../../scripts/self-translation/envelope.mjs';
import { fromLino } from '../../../scripts/self-translation/ir.mjs';
const sample = '/** @param {number} inputValue @returns {number} */\n'
  + 'export function choosePresentValue(inputValue) { const chosenValue = inputValue + 1; return chosenValue; }';
test('pure frontend and both emitters preserve arbitrary API and binding names through canonical meta symbols', () => {
  const meta = toMeta(sample, 'JavaScript');
  assert(meta.includes('(function choose-present-value'));
  assert(meta.includes('(input-value number)'));
  assert(meta.includes('(let chosen-value'));
  assert(!meta.includes('choose_present_value'));
  const items = fromLino(meta);
  const rust = fromMeta(items, 'Rust');
  assert(rust.includes('fn choose_present_value(input_value: f64)'));
  assert(rust.includes('let chosen_value = input_value + 1.0'));
  assert.equal(toMeta(fromMeta(items, 'JavaScript'), 'JavaScript').split('\n').slice(1).join('\n'),
    meta.split('\n').slice(1).join('\n'));
});
test('calls and explicit constant style use the same canonical symbols and preserve source APIs', () => {
  const source = 'export const MAXIMUM_SCORE = 10;\n' + sample.replace('inputValue + 1', 'inputValue + MAXIMUM_SCORE');
  const meta = toMeta(source, 'JavaScript');
  assert(meta.includes('(constant MAXIMUM-SCORE'));
  assert(meta.includes('(variable MAXIMUM-SCORE)'));
  assert(fromMeta(fromLino(meta), 'Rust').includes('const MAXIMUM_SCORE'));
  assert(fromMeta(fromLino(meta), 'JavaScript').includes('const MAXIMUM_SCORE'));
  const callable = sample + '\n/** @param {number} otherValue @returns {number} */\n'
    + 'export function forwardValue(otherValue) { return choosePresentValue(otherValue); }';
  const callMeta = toMeta(callable, 'JavaScript');
  assert(callMeta.includes('(call choose-present-value'));
  assert(fromMeta(fromLino(callMeta), 'Rust').includes('choose_present_value(other_value)'));
});
test('raw underscore and empty-segment meta aliases are refused before emission', () => {
  const meta = toMeta(sample, 'JavaScript');
  for (const alias of ['choose_present_value', 'choose--present-value']) {
    assert.throws(() => fromMeta(fromLino(meta.replaceAll('choose-present-value', alias)), 'Rust'));
  }
  assert.throws(() => fromMeta(fromLino(meta.replaceAll('input-value', 'input_value')), 'JavaScript'));
});
test('new research kernel meta has canonical symbols while Rust APIs remain stable', () => {
  const meta = fs.readFileSync(new URL('../../../data/meta/coding-research-kernel.lino', import.meta.url), 'utf8');
  assert(meta.includes('(function tap-verification-decision'));
  assert(meta.includes('(call safe-tap-count'));
  assert(fromMeta(fromLino(meta), 'Rust').includes('fn tap_verification_decision'));
});
