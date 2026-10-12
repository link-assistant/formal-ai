import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, dirname} from 'node:path';
import {pathToFileURL, fileURLToPath} from 'node:url';
import test from 'node:test';
import {drive, executeResult, observedOperation} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const driverUrl = new URL('../../../experiments/js_dogfood/drive.mjs', import.meta.url);
const quote = value => "'" + value.replaceAll("'", "'\\''") + "'";
const command = quote(process.execPath) + ' -e ' + quote('process.stdout.write("observed")');
const need = 'Run ' + command;
const call = arguments_ => ({tool: 'bash', arguments: arguments_});
const planner = arguments_ => () => ({kind: 'tool_calls', calls: [call(arguments_)]});
const options = {tools: ['bash'], allowedCommands: [command], steps: 1};
async function sandbox(action) {
  const directory = mkdtempSync(join(tmpdir(), 'driver-operation-observation-'));
  try {await action(directory);} finally {rmSync(directory, {recursive: true, force: true});}
}
async function issue(directory, driver = {drive, observedOperation}) {
  const result = await driver.drive(planner(JSON.stringify({command})), directory, need, options);
  const token = result.transcript[0].operationObservation;
  return {result, token, observation: driver.observedOperation(token, need, directory)};
}
test('actual policy-validated command receipt has a frozen observation with unknown authority and effects', async () => {
  await sandbox(async directory => {
    const {token, observation} = await issue(directory);
    assert(observation); assert.match(observation.receipt.content, /Output: observed/u);
    assert.equal(observation.command, command); assert.equal(observation.need.original, need);
    for (const field of ['approval', 'effects', 'sourceClosure']) assert.equal(observation[field], 'Unknown');
    assert(Object.isFrozen(observation.need)); assert(Object.isFrozen(observation.receipt));
    assert.throws(() => {observation.need.original = 'changed';}, TypeError);
    assert.equal(observedOperation(token, need + ' changed', directory), null);
    assert.equal(observedOperation(token, need, join(directory, 'different')), null);
  });
});
test('caller records, JSON identities, direct execution and callback answers cannot forge observations', async () => {
  await sandbox(async directory => {
    const {token, observation} = await issue(directory);
    for (const forged of [JSON.parse(JSON.stringify(token)), {...observation}, {state: 'accepted'},
      {receipt: observation.receipt}, executeResult(directory, call(JSON.stringify({command})))]) {
      assert.equal(observedOperation(forged, need, directory), null);
    }
    const answer = await drive(() => ({kind: 'final', answer: 'success'}), directory, need, options);
    assert.equal(answer.transcript.length, 0); assert.equal(observedOperation(answer, need, directory), null);
  });
});
test('omitted policy and extra caller metadata retain execution behavior without issuing an observation', async () => {
  await sandbox(async directory => {
    const omitted = await drive(planner(JSON.stringify({command})), directory, need, {tools: ['bash'], steps: 1});
    assert.equal(omitted.transcript[0].operationObservation, undefined);
    const extra = await drive(planner(JSON.stringify({command, accepted: true})), directory, need, options);
    assert.equal(extra.transcript[0].operationObservation, undefined);
  });
});
test('refused allowlists and mutable arguments never reach observation issuance', async () => {
  await sandbox(async directory => {
    for (const allowedCommands of [[], [command + ' drift']]) {
      const result = await drive(planner(JSON.stringify({command})), directory, need, {...options, allowedCommands});
      assert.equal(result.stop, 'command-policy-denied'); assert.deepEqual(result.transcript, []);
    }
    const mutable = await drive(planner({command}), directory, need, options);
    assert.equal(mutable.stop, 'command-policy-denied'); assert.deepEqual(mutable.transcript, []);
  });
});
test('a failed actual command is an observation, never a successful Need or approval', async () => {
  await sandbox(async directory => {
    const failed = quote(process.execPath) + ' -e ' + quote('process.exit(7)');
    const original = 'Run ' + failed;
    const result = await drive(planner(JSON.stringify({command: failed})), directory, original,
      {...options, allowedCommands: [failed]});
    const observation = observedOperation(result.transcript[0].operationObservation, original, directory);
    assert(observation); assert.match(observation.receipt.content, /Exit Code: 7/u);
    assert.equal(observation.approval, 'Unknown');
  });
});
test('stable provider source drift invalidates only a private fixture identity', async () => {
  await sandbox(async directory => {
    // Fixture imports retain the actual owned dependency locations; only its provider is copied.
    let source = readFileSync(driverUrl, 'utf8');
    source = source.replace(/from (['"])([^'"]+)\1/gu, (whole, quoteStyle, path) => {
      if (!path.startsWith('.') || path === './shell-capture.mjs') return whole;
      return 'from ' + JSON.stringify(new URL(path, driverUrl).href);
    });
    writeFileSync(join(directory, 'drive.mjs'), source);
    const provider = join(directory, 'shell-capture.mjs');
    writeFileSync(provider, readFileSync(new URL('./shell-capture.mjs', driverUrl)));
    const fixture = await import(pathToFileURL(join(directory, 'drive.mjs')).href);
    const {token, observation} = await issue(directory, fixture); assert(observation);
    writeFileSync(provider, readFileSync(provider, 'utf8') + '\n// private fixture source drift\n');
    assert.equal(fixture.observedOperation(token, need, directory), null);
    assert.equal(observedOperation(token, need, directory), null);
  });
});

test('default command and read/write transcripts retain their exact original enumerable shape', async () => {
  await sandbox(async directory => {
    const arguments_ = JSON.stringify({command});
    const result = await drive(planner(arguments_), directory, need, {tools: ['bash'], steps: 1});
    assert.deepEqual(result.transcript, [{tool: 'bash', arguments: arguments_, result: 'Output: observed\nExit Code: 0'}]);
    const writeArguments = JSON.stringify({path: 'preserved.txt', content: 'kept'});
    const writeResult = await drive(() => ({kind: 'tool_calls', calls: [{tool: 'write', arguments: writeArguments}]}),
      directory, 'write', {tools: ['write'], steps: 1});
    assert.deepEqual(writeResult.transcript, [{tool: 'write', arguments: writeArguments, result: ''}]);
    const readArguments = JSON.stringify({path: 'preserved.txt'});
    const readResult = await drive(() => ({kind: 'tool_calls', calls: [{tool: 'read', arguments: readArguments}]}),
      directory, 'read', {tools: ['read'], steps: 1});
    assert.deepEqual(readResult.transcript, [{tool: 'read', arguments: readArguments, result: 'kept',
      source_read: {path: 'preserved.txt', success: true, complete: true, format: 'raw'}}]);
  });
});
