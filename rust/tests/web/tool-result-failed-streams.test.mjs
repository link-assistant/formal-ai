// Regression from a real failed rg inspection: stdout survived custody but vanished from its final failure.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { reportedExitCode, stepOutcome, normalizedPayload, renderFailure } from '../../../js/agentic/tool_result.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
before(async () => {
  await installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false});
});
test('failed shell result retains stdout and stderr once with its nonzero exit', () => {
  const raw = 'Output: retained-match\nError: missing-path\nExit Code: 2';
  assert.equal(reportedExitCode(raw), 2);
  assert.equal(stepOutcome(raw), 'failed');
  const answer = renderFailure('bash', raw, 'Run the inspection.');
  for (const stream of ['retained-match', 'missing-path']) {
    assert.equal(answer.split(stream).length - 1, 1);
  }
  assert.match(answer, /2/);
});
test('single streams, zero exit and signal failures keep their established outcomes', () => {
  assert.equal(renderFailure('bash', 'Output: retained-match\nError: \nExit Code: 2', 'Run it.').includes('retained-match'), true);
  assert.equal(renderFailure('bash', 'Output: \nError: missing-path\nExit Code: 2', 'Run it.').includes('missing-path'), true);
  const success = 'Output: retained-match\nError: diagnostic\nExit Code: 0';
  assert.equal(reportedExitCode(success), 0);
  assert.equal(stepOutcome(success), 'succeeded');
  assert.equal(normalizedPayload(success), 'retained-match');
  const signalRaw = 'Output: retained-match\nError: missing-path\nSignal: SIGTERM';
  const signalled = renderFailure('bash', signalRaw, 'Run it.');
  assert.equal(reportedExitCode(signalRaw), null);
  assert.match(signalled, /retained-match/);
  assert.match(signalled, /missing-path/);
  assert.match(signalled, /SIGTERM/);
});
