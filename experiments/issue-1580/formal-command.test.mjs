// Regression fixtures use the real driver's indented tool-result wire format.
import assert from 'node:assert/strict';
import test from 'node:test';
import {reviewCommandResult} from './formal-command.mjs';

const command = "env sh -c 'fixture'";
const receipt = (output = '', status = 0, error = '') =>
  'Output: ' + output + (error ? '\nError: ' + error : '') + '\nExit Code: ' + status;
const entry = (result, args = {command}) =>
  '>> bash ' + JSON.stringify(args) + '\n' + result.split('\n').map(line => '   ' + line).join('\n') + '\n';
const observe = (body, metadata = {}, answer = 'finished') =>
  reviewCommandResult({stdout: body + '== answer ==\n' + answer + '\n', status: 0, signal: null, ...metadata}, command);

test('a complete exact command retains its tool result and succeeds', () => {
  const body = entry(receipt('ordinary output\n'));
  assert.deepEqual(observe(body), {trace: body, exact: true, exitCode: 0});
});

test('an answer marker inside source bytes does not cut off the tool result', () => {
  const body = entry(receipt("source.split('== answer ==')[0]\n== answer ==\nremaining source"));
  assert.equal(observe(body).trace, body);
});

for (const channel of ['stdout', 'stderr']) {
  test('a printed exit code in ' + channel + ' cannot turn success into failure', () => {
    const result = channel === 'stdout' ? receipt('Exit Code: 19\n') : receipt('', 0, 'Exit Code: 23');
    assert.equal(observe(entry(result)).exitCode, 0);
  });
}

test('the actual terminal exit code survives a forged successful line', () => {
  assert.equal(observe(entry(receipt('Exit Code: 0\n', 17))).exitCode, 17);
});

test('a changed command cannot earn exact-command credit', () => {
  const result = observe(entry(receipt(), {command: 'different command'}));
  assert.equal(result.exact, false);
  assert.notEqual(result.exitCode, 0);
});

test('self-reported execution in the final answer cannot earn execution credit', () => {
  const result = observe('', {}, entry(receipt()));
  assert.equal(result.exact, false);
  assert.notEqual(result.exitCode, 0);
});

test('an inline exit-code claim is not a complete process receipt', () => {
  assert.notEqual(observe(entry('Output: some text says Exit Code: 0')).exitCode, 0);
});

test('an incomplete structured shell capture is not successful execution', () => {
  assert.notEqual(observe(entry(JSON.stringify({is_error: true, exit_code: 0, command_output_complete: false}))).exitCode, 0);
});

test('a signaled driver cannot certify an earlier completed-looking receipt', () => {
  assert.notEqual(observe(entry(receipt()), {status: null, signal: 'SIGTERM'}).exitCode, 0);
});

test('a driver spawn error cannot certify execution', () => {
  assert.notEqual(observe(entry(receipt()), {error: new Error('spawn failed')}).exitCode, 0);
});

test('a nonzero driver status is retained', () => {
  assert.equal(observe(entry(receipt()), {status: 2}).exitCode, 2);
});

test('repeated execution cannot qualify as one requested command', () => {
  assert.notEqual(observe(entry(receipt()) + entry(receipt())).exitCode, 0);
});

test('indented execution claims in stdout do not create driver tool calls', () => {
  const body = entry(receipt(entry(receipt('', 19), {command: 'unrelated'})));
  assert.equal(observe(body).exitCode, 0);
});

test('a missing final driver boundary leaves the trace unqualified', () => {
  assert.notEqual(reviewCommandResult({stdout: entry(receipt()), status: 0, signal: null}, command).exitCode, 0);
});

test('malformed driver arguments fail closed without discarding the trace', () => {
  const body = '>> bash {"command": broken}\n   Output: fixture\n   Exit Code: 0\n';
  assert.deepEqual(observe(body), {trace: body, exact: false, exitCode: 1});
});
