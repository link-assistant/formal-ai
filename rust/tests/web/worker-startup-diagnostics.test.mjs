import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { createRequire } from 'node:module';
import test from 'node:test';

const { withWorkerDiagnostics } = createRequire(import.meta.url)('../e2e/tests/support/worker-diagnostics.js');

test('startup failure records network cause and visible state, then preserves the failed assertion', async () => {
  const page = new EventEmitter();
  page.evaluate = async () => ({ status: 'loading worker', composerDisabled: true });
  const attachments = [], logs = [], original = new Error('composer remained disabled');
  const info = { attach: async (name, data) => attachments.push({ name, ...data }) };
  const previous = console.error;
  console.error = (...args) => logs.push(args);
  try {
    await assert.rejects(withWorkerDiagnostics(page, info, async () => {
      page.emit('response', { status: () => 503, url: () => 'https://example.test/worker/module.js?v=source' });
      page.emit('pageerror', new Error('NetworkError: importScripts failed'));
      for (let index = 0; index < 100; index++) page.emit('console', { type: () => 'warning', text: () => 'transport unavailable' });
      throw original;
    }), error => error === original);
  } finally { console.error = previous; }
  assert.equal(attachments.length, 1);
  const diagnostic = JSON.parse(attachments[0].body);
  assert.equal(diagnostic.events.length, 40);
  assert.deepEqual(diagnostic.events[0], { kind: 'http', url: 'https://example.test/worker/module.js', status: 503 });
  assert.match(diagnostic.events[1].message, /importScripts/);
  assert.equal(diagnostic.state.composerDisabled, true);
  assert(logs.some(args => String(args[0]).includes('WorkerStartupDiagnostics')));
  assert.equal(page.eventNames().length, 0, 'Listeners are removed after failure');
});

test('healthy smoke assertions incur no diagnostic evaluation or attachments', async () => {
  const page = new EventEmitter();
  page.evaluate = () => { throw Error('UnexpectedDiagnosticEvaluation'); };
  const info = { attach: () => { throw Error('UnexpectedDiagnosticAttachment'); } };
  assert.equal(await withWorkerDiagnostics(page, info, async () => 'real assertions passed'), 'real assertions passed');
  assert.equal(page.eventNames().length, 0);
});
