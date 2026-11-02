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
