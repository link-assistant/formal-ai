import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { captureProgram, shellQuote } from '../../../js/agentic/algorithm_learning.mjs';
import { executeBoundedSourceCommand } from '../../../scripts/check-native-source-operations.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

async function capture(source, options) {
  const command = shellQuote(process.execPath) + ' -e ' + shellQuote(source);
  const program = captureProgram(command, options);
  // The independent outer guard remains longer than the producer's own deadline.
  try {
    const result = await executeBoundedSourceCommand(process.execPath,
      ['-e', program], {}, 2500);
    return { status: 0, receipt: JSON.parse(result.output) };
  } catch (error) {
    let result;
    try { result = JSON.parse(error.message); } catch { throw error; }
    return { status: result.code, receipt: JSON.parse(result.output) };
  }
}

test('capture deadline includes descendant-held pipes after parent success', async () => {
  const observed = await capture(
    "require('node:child_process').spawn(process.execPath," +
    "['-e','setInterval(()=>{},1000);setTimeout(()=>process.exit(99),5000)'],{stdio:['ignore',1,2]});process.exit(0);",
    { timeout: 500 });
  assert.equal(observed.status, 1);
  assert.equal(observed.receipt.complete, false);
  assert.equal(observed.receipt.timed_out, true);
});

test('capture forcefully stops a child that ignores SIGTERM', async () => {
  const observed = await capture(
    "process.on('SIGTERM',()=>{});process.stdout.write('started');setInterval(()=>{},1000);setTimeout(()=>process.exit(99),5000);",
    { timeout: 500 });
  assert.equal(observed.status, 1);
  assert.equal(observed.receipt.stdout, 'started');
  assert.equal(observed.receipt.complete, false);
  assert.equal(observed.receipt.timed_out, true);
});

test('capture overflow is bounded and cannot become successful parent status', async () => {
  const observed = await capture('process.stdout.write(Buffer.alloc(1024*1024,120));',
    { timeout: 1500, maxBuffer: 128 });
  assert.equal(observed.status, 1);
  assert.equal(observed.receipt.stdout.length, 128);
  assert.equal(observed.receipt.complete, false);
  assert.equal(observed.receipt.truncated, true);
});

test('capture preserves a completely observed real nonzero exit', async () => {
  const observed = await capture("process.stdout.write('retained');process.exit(7);",
    { timeout: 1500 });
  assert.equal(observed.status, 7);
  assert.equal(observed.receipt.exit_code, 7);
  assert.equal(observed.receipt.stdout, 'retained');
  assert.equal(observed.receipt.complete, true);
});
