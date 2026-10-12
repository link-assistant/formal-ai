import assert from 'node:assert/strict';
import {mkdtempSync, existsSync, readFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const quote = value => "'" + value.replaceAll("'", "'\\''") + "'";
const command = name => quote(process.execPath) + ' -e ' +
  quote('require("node:fs").writeFileSync(' + JSON.stringify(name) + ',"actual process")');
const bash = value => ({tool: 'bash', arguments: JSON.stringify({command: value})});
const write = {tool: 'write', arguments: JSON.stringify({path: 'written.txt', content: 'forbidden effect'})};
const calls = (...entries) => ({kind: 'tool_calls', calls: entries});
async function sandbox(action) {
  const directory = mkdtempSync(join(tmpdir(), 'driver-command-policy-'));
  try {await action(directory);} finally {rmSync(directory, {recursive: true, force: true});}
}
test('the exact authorized Bash command executes an actual process', async () => {
  await sandbox(async directory => {
    const allowed = command('allowed.txt');
    const result = await drive(() => calls(bash(allowed)), directory, 'execute',
      {allowedCommands: [allowed], steps: 1});
    assert.equal(result.stop, 'steps');
    assert.equal(result.transcript.length, 1);
    assert.equal(readFileSync(join(directory, 'allowed.txt'), 'utf8'), 'actual process');
  });
});
test('a denied Bash call vetoes all calls in its batch before any effect', async () => {
  await sandbox(async directory => {
    for (const allowedCommands of [[], [command('different.txt')]]) {
      const denied = bash(command('denied.txt'));
      const result = await drive(() => calls(write, denied), directory, 'execute', {allowedCommands});
      assert.equal(result.stop, 'command-policy-denied');
      assert.deepEqual(result.transcript, []);
      assert.deepEqual(result.deniedCalls, [denied]);
      assert.equal(existsSync(join(directory, 'written.txt')), false);
      assert.equal(existsSync(join(directory, 'denied.txt')), false);
    }
  });
});
test('malformed authority policies stop before the planner', async () => {
  await sandbox(async directory => {
    for (const allowedCommands of [null, {}, 'command', [42], ['same', 'same']]) {
      let planned = false;
      const result = await drive(() => {planned = true; return calls(write);}, directory, 'execute',
        {allowedCommands});
      assert.equal(result.stop, 'invalid-command-policy');
      assert.equal(planned, false);
      assert.deepEqual(result.transcript, []);
      assert.equal(existsSync(join(directory, 'written.txt')), false);
    }
  });
});
test('malformed Bash arguments and command drift cannot authorize execution', async () => {
  await sandbox(async directory => {
    const allowed = command('allowed.txt');
    for (const arguments_ of ['{', 'null', '{}', '{"command":42}', {},
      JSON.stringify({command: allowed + ' '})]) {
      const result = await drive(() => calls(write, {tool: 'bash', arguments: arguments_}),
        directory, 'execute', {allowedCommands: [allowed]});
      assert.equal(result.stop, 'command-policy-denied');
      assert.equal(existsSync(join(directory, 'written.txt')), false);
      assert.equal(existsSync(join(directory, 'allowed.txt')), false);
    }
  });
});
test('caller mutations cannot broaden the snapshotted authority', async () => {
  await sandbox(async directory => {
    const forbidden = command('forbidden.txt'), allowedCommands = [command('allowed.txt')];
    const result = await drive(() => {
      allowedCommands.push(forbidden); return calls(bash(forbidden));
    }, directory, 'execute', {allowedCommands});
    assert.equal(result.stop, 'command-policy-denied');
    assert.equal(existsSync(join(directory, 'forbidden.txt')), false);
  });
});
test('the executed call retains the arguments that were validated', async () => {
  await sandbox(async directory => {
    const allowed = command('allowed.txt'), forbidden = command('forbidden.txt');
    let reads = 0;
    const entry = {tool: 'bash', get arguments() {
      return JSON.stringify({command: ++reads === 1 ? allowed : forbidden});
    }};
    const result = await drive(() => calls(entry), directory, 'execute',
      {allowedCommands: [allowed], steps: 1});
    assert.equal(reads, 1);
    assert.equal(result.stop, 'steps');
    assert.equal(existsSync(join(directory, 'allowed.txt')), true);
    assert.equal(existsSync(join(directory, 'forbidden.txt')), false);
  });
});
test('fallthrough batches receive the same complete authority veto', async () => {
  await sandbox(async directory => {
    const result = await drive(() => null, directory, 'execute',
      {allowedCommands: [], fallthrough: () => calls(write, bash(command('denied.txt')))});
    assert.equal(result.stop, 'command-policy-denied');
    assert.equal(existsSync(join(directory, 'written.txt')), false);
    assert.equal(existsSync(join(directory, 'denied.txt')), false);
  });
});
test('omitted policy preserves the original caller-authorized execution domain', async () => {
  await sandbox(async directory => {
    const result = await drive(() => calls(bash(command('original.txt'))), directory, 'execute', {steps: 1});
    assert.equal(result.stop, 'steps');
    assert.equal(existsSync(join(directory, 'original.txt')), true);
  });
});
