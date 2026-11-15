import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { factoryOf, importClosure } from '../../../scripts/generate-worker-crate-modules.mjs';
import { quotedSegmentSpans, quotedSegments } from '../../../js/agentic/crate/normal_markov.mjs';
import { quotedSegmentSpans as leafSpans } from '../../../js/agentic/crate/quoted_segments.mjs';
import { isAsciiAlphanumeric } from '../../../js/agentic/write_str.mjs';

const source = name => readFileSync(new URL('../../../js/agentic/' + name, import.meta.url), 'utf8');
function workerModules() {
  const realm = vm.createContext({ self: {}, TextEncoder, TextDecoder, URL });
  const entries = ['content.mjs', 'crate/normal_markov.mjs'];
  for (const name of importClosure(entries, source)) {
    vm.runInContext(factoryOf(name, source(name)), realm);
  }
  const runtime = readFileSync(new URL('../../../js/worker/formal_ai_worker_crate_modules.js', import.meta.url), 'utf8');
  const start = runtime.indexOf('function crateInstance(name) {');
  const end = runtime.indexOf('\n}\n', start) + 2;
  assert.ok(start >= 0 && end > start);
  vm.runInContext('const crateInstances = new Map();\n' + runtime.slice(start, end), realm);
  return name => realm.crateInstance(name);
}

test('worker lazy factories initialize quote readers from either entry without a dependency cycle', () => {
  for (const first of ['content.mjs', 'crate/normal_markov.mjs']) {
    const requireModule = workerModules();
    requireModule(first);
    assert.equal(requireModule('content.mjs').plainText('request'), 'request');
    const spans = requireModule('crate/normal_markov.mjs').quotedSegmentSpans('write «outer «inner» end»');
    assert.equal(JSON.stringify(spans), JSON.stringify([{ text: 'outer «inner» end', start: 6, end: 25 }]));
    assert.equal(requireModule('write_str.mjs').isAsciiAlphanumeric('A'), true);
  }
});

test('quote parser extraction preserves public bindings and exact nested span behavior', () => {
  assert.equal(quotedSegmentSpans, leafSpans);
  assert.equal(isAsciiAlphanumeric('é'), false);
  for (const text of ["don't open a quote", 'write «outer «inner» end»', 'say “nested ‘text’ here”', 'say "quoted"', 'unpaired «tail']) {
    assert.deepEqual(quotedSegments(text), leafSpans(text).map(segment => segment.text));
  }
  assert.deepEqual(leafSpans('write «outer «inner» end»'), [{ text: 'outer «inner» end', start: 6, end: 25 }]);
  assert.deepEqual(leafSpans("don't open a quote"), []);
});

import { previousChar } from '../../../js/agentic/crate/quoted_segments.mjs';
const previousCharReference = (text, index) => {
  const before = Array.from(text.slice(0, index));
  return before[before.length - 1];
};

test('predecessor semantics preserve UTF16 prefixes, coercions and lone surrogates', () => {
  const text = 'a😀Ω\ud800z\udc00';
  for (let index = -text.length - 1; index <= text.length + 1; index++)
    assert.equal(previousChar(text, index), previousCharReference(text, index));
  for (const index of [undefined, null, NaN, Infinity, -Infinity, 1.9, -1.9, '2'])
    assert.equal(previousChar(text, index), previousCharReference(text, index));
  for (const index of [Symbol('index'), 1n]) {
    assert.throws(() => previousChar(text, index), TypeError);
    assert.throws(() => previousCharReference(text, index), TypeError);
  }
  assert.equal(previousChar('a😀b', 3), '😀');
  assert.equal(previousChar('a😀b', 2), '\ud83d');
});

test('predecessor character iteration is bounded independently of prefix length', () => {
  const original = Array.from, visited = [];
  Array.from = function(value, ...rest) {
    if (typeof value === 'string') visited.push(value.length);
    return Reflect.apply(original, this, [value, ...rest]);
  };
  try {
    assert.equal(previousChar('Ω'.repeat(2048) + '😀', 2050), '😀');
  } finally {
    Array.from = original;
  }
  assert.ok(visited.length > 0);
  assert.ok(visited.every(units => units <= 2));
  assert.ok(visited.reduce((sum, n) => sum + n, 0) <= 2);
});
