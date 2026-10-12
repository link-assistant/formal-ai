// Actual capture bytes retain UTF-8 validity and current-call receipt ownership.
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {captureCommand, shellQuote, commandPayload} from '../../../js/agentic/algorithm_learning.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

function executeCapture(program) {
  const command = shellQuote(process.execPath) + ' -e ' + shellQuote(program);
  const wrapper = captureCommand(command);
  const direct = spawnSync(process.execPath, ['-e', program]);
  const processResult = spawnSync('/bin/sh', ['-c', wrapper], {encoding: 'utf8'});
  const receipt = JSON.parse(processResult.stdout);
  const messages = [{role: 'user', content: 'observe bounded process output'},
    {role: 'assistant', tool_calls: [{id: 'capture', type: 'function',
      function: {name: 'Bash', arguments: JSON.stringify({command: wrapper})}}]},
    {role: 'tool', name: 'Bash', tool_call_id: 'capture', content: processResult.stdout}];
  return {direct, processResult, receipt, messages, wrapper};
}

test('valid multilingual stdout retains its leading BOM and exact bytes', () => {
  const text = '\uFEFFβ天气नमस्ते';
  const observed = executeCapture('process.stdout.write(' + JSON.stringify(text) + ')');
  assert.equal(observed.direct.status, 0);
  assert.equal(observed.processResult.status, 0);
  assert.equal(observed.receipt.complete, true);
  assert.equal(observed.receipt.stdout, text);
  assert.deepEqual(Buffer.from(observed.receipt.stdout), observed.direct.stdout);
  assert.equal(commandPayload(observed.messages, observed.wrapper, null), text);
});

for (const [stream, bytes] of [['stdout', [255]], ['stdout', [195]], ['stderr', [237, 160, 128]]]) {
  test('malformed ' + stream + ' bytes refuse a completed text receipt: ' + bytes.join(','), () => {
    const observed = executeCapture('process.' + stream + '.write(Buffer.from(' + JSON.stringify(bytes) + '))');
    assert.equal(observed.direct.status, 0);
    assert.deepEqual([...observed.direct[stream]], bytes);
    assert.equal(observed.processResult.status, 1);
    assert.equal(observed.receipt.exit_code, 0);
    assert.equal(observed.receipt.complete, false);
    assert.equal(observed.receipt.error, stream === 'stdout' ? 'InvalidUTF8Stdout' : 'InvalidUTF8Stderr');
    assert.equal(observed.receipt.stdout, '');
    assert.equal(observed.receipt.stderr, '');
    assert.equal(commandPayload(observed.messages, observed.wrapper, null), null);
  });
}
