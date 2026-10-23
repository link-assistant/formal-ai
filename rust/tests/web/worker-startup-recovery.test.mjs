import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import vm from 'node:vm';
import { renderWorkerPrerequisites } from '../../../scripts/generate-worker-prerequisites.mjs';

const entry = readFileSync(new URL('../../../js/worker/formal_ai_worker.js', import.meta.url), 'utf8');
const captured = JSON.parse(readFileSync(new URL('../fixtures/web/pages-worker-startup-network-error.json', import.meta.url)));
const moduleAsset = captured.startupAssets[1];

function boot({ failures = {}, moduleSource = 'postMessage({kind:"ready",mode:"fixture-ready"})' } = {}) {
  const attempts = new Map(), messages = [], warnings = [], errors = [];
  const context = vm.createContext({
    URL, Map,
    location: { href: 'https://example.test/formal-ai/worker/formal_ai_worker.js?v=fixture', search: '?v=fixture' },
    fetch() { throw Error('UnexpectedFixtureFetch'); },
    console: { warn: (...args) => warnings.push(args), error: (...args) => errors.push(args) },
    postMessage: message => messages.push(message),
  });
  context.self = context;
  context.importScripts = url => {
    const asset = new URL(url).pathname.replace(/^\/formal-ai\//, '');
    const count = (attempts.get(asset) || 0) + 1;
    attempts.set(asset, count);
    if (count <= (failures[asset] || 0)) {
      const error = new Error(captured.message.replace('{url}', url));
      error.name = captured.errorName;
      throw error;
    }
    if (asset === 'worker-script-prerequisites.js') {
      vm.runInContext(renderWorkerPrerequisites([]).browser, context);
    } else if (asset === 'worker-modules.js') {
      context.FORMAL_AI_WORKER_MODULES = [moduleAsset];
    } else if (asset === moduleAsset) {
      vm.runInContext(moduleSource, context, { filename: asset });
    }
  };
  let error;
  try { vm.runInContext(entry, context, { filename: 'worker-entry.js', timeout: 2000 }); }
  catch (caught) { error = caught; }
  return { context, attempts, messages, warnings, errors, error };
}

test('captured transient startup transport failures recover without replaying successful scripts', () => {
  const result = boot({ failures: Object.fromEntries(captured.startupAssets.map(asset => [asset, 1])) });
  assert.equal(result.error, undefined);
  assert.deepEqual(result.messages.map(message => message.mode), ['fixture-ready']);
  for (const asset of captured.startupAssets) assert.equal(result.attempts.get(asset), 2);
  assert.equal(result.attempts.get('worker-bootstrap-responses.js'), 1);
  assert.equal(result.warnings.length, 2);
});

for (const asset of captured.startupAssets) {
  test(`permanent network failure stops after two attempts: ${asset}`, () => {
    const result = boot({ failures: { [asset]: Infinity } });
    assert.equal(result.error?.name, 'NetworkError');
    assert.equal(result.attempts.get(asset), 2);
    assert.equal(result.messages.length, 0);
    assert.throws(() => vm.runInContext(`loadWorkerAsset(${JSON.stringify(asset)})`, result.context), error => error === result.error);
    assert.equal(result.attempts.get(asset), 2, 'A later loader call cannot reset an exhausted failure budget');
  });
}

test('an executing module network error does not replay its side effects', () => {
  const result = boot({ moduleSource: 'globalThis.partialRuns=(globalThis.partialRuns||0)+1; const error=new Error("Nested service failed"); error.name="NetworkError"; throw error' });
  assert.equal(result.error?.name, 'NetworkError');
  assert.equal(result.attempts.get(moduleAsset), 1);
  assert.equal(result.context.partialRuns, 1);
  assert.equal(result.warnings.length, 0);
});

for (const kind of ['SyntaxError', 'Error']) {
  test(`${kind} after script side effects is never re-executed`, () => {
    const result = boot({ moduleSource: `globalThis.partialRuns=(globalThis.partialRuns||0)+1; throw new ${kind}("broken module")` });
    assert.equal(result.error?.name, kind);
    assert.equal(result.attempts.get(moduleAsset), 1);
    assert.equal(result.context.partialRuns, 1);
    assert.equal(result.warnings.length, 0);
  });
}
