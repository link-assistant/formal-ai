import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { argumentValuesEqual, qualifiedTranscriptFrames } from '../../../js/agentic/qualified_tool_observation.mjs';
import { Progress, qualifiedToolAttempt } from '../../../js/agentic/progress.mjs';
import { Capability } from '../../../js/agentic/capability.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { workspaceDiscoveryCommand, workspaceDiscoveryStep } from '../../../js/agentic/workspace_discovery.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

for (const [label, left, right, expected] of [
  ['one numeric representations', '1', '1.0', true],
  ['signed zero', '-0.0', '0', true],
  ['equal finite fractions', '0.125', '0.125', true],
  ['different finite fractions', '0.125', '0.25', false],
  ['positive safe boundary', '9007199254740991', '9007199254740991.0', true],
  ['negative safe boundary', '-9007199254740991', '-9007199254740991.0', true],
  ['unsafe reflexive integer', '9007199254740992', '9007199254740992', false],
  ['unsafe distinct integer alias', '9007199254740992', '9007199254740993', false],
  ['negative unsafe integer', '-9007199254740992', '-9007199254740992', false],
  ['unsigned native boundary', '18446744073709551615', '18446744073709551615', false],
  ['nested common domain', '{"a":[1,null]}', '{"a":[1.0,null]}', true],
  ['nested unsafe alias', '{"a":[9007199254740992]}', '{"a":[9007199254740993]}', false],
  ['unordered own keys', '{"a":1,"b":true}', '{"b":true,"a":1.0}', true],
  ['distinct string control', '{"command":"one"}', '{"command":"two"}', false],
]) test(label, () => {
  assert.equal(argumentValuesEqual(JSON.parse(left), JSON.parse(right)), expected);
});

for (const [label, value] of [
  ['undefined', undefined], ['infinity', Infinity], ['negative infinity', -Infinity],
  ['NaN', NaN], ['undefined child', { value: undefined }], ['function child', { value() {} }],
  ['symbol child', { value: Symbol('unknown') }], ['Date', new Date(0)],
  ['custom prototype', Object.create({ inherited: true })], ['sparse array', Array(1)],
  ['lone surrogate value', '\ud800'], ['lone surrogate key', { '\udfff': true }],
]) test('closed JSON domain refuses ' + label, () => {
  assert.equal(argumentValuesEqual(value, value), false);
});

test('null prototype own-key map preserves JSON object semantics', () => {
  const value = Object.assign(Object.create(null), { command: 'one', nested: [1] });
  assert.equal(argumentValuesEqual(value, { nested: [1.0], command: 'one' }), true);
});

test('accessors are refused without executing the getter', () => {
  let observed = 0;
  const value = Object.defineProperty({}, 'command', { enumerable: true, get() { observed++; return 'one'; } });
  assert.equal(argumentValuesEqual(value, value), false);
  assert.equal(observed, 0);
});

test('array extra properties and own symbols cannot hide behind identity', () => {
  const extra = [1]; extra.unobserved = true;
  const symbol = { command: 'one', [Symbol('unobserved')]: true };
  assert.equal(argumentValuesEqual(extra, extra), false);
  assert.equal(argumentValuesEqual(symbol, symbol), false);
});

test('cyclic arguments cannot acquire a reflexive receipt match', () => {
  const cyclic = {}; cyclic.self = cyclic;
  assert.equal(argumentValuesEqual(cyclic, cyclic), false);
});

function messages(argumentsText) {
  return [{ role: 'user', content: 'Inspect the task model and record the concrete result.' },
    { role: 'assistant', content: '', tool_calls: [{ id: 'source-call', type: 'function',
      function: { name: 'bash', arguments: argumentsText } }] },
    { role: 'tool', name: 'bash', tool_call_id: 'source-call', content: 'Output: complete\nExit Code: 0' }];
}

test('unsafe declared argument rejects a named successful receipt before frame acceptance', () => {
  const text = '{"command":"true","nested":[9007199254740993]}';
  const transcript = messages(text);
  assert.deepEqual(qualifiedTranscriptFrames(transcript, 0), []);
  assert.equal(qualifiedToolAttempt(Progress.scan(transcript), Capability.Run, JSON.parse(text)), null);
});

test('unsafe query cannot match a safely declared source receipt', () => {
  const transcript = messages('{"command":"true","nested":[1]}');
  const progress = Progress.scan(transcript);
  assert(qualifiedToolAttempt(progress, Capability.Run, { command: 'true', nested: [1.0] }));
  assert.equal(qualifiedToolAttempt(progress, Capability.Run, { command: 'true', nested: [9007199254740992] }), null);
});

test('unpaired source JSON surrogate refuses receipt qualification', () => {
  assert.deepEqual(qualifiedTranscriptFrames(messages('{"command":"\\ud800"}'), 0), []);
});

test('unsafe receipt cannot authorize discovery output writes', () => {
  const need = 'Inspect the task model and record the concrete result.';
  const command = workspaceDiscoveryCommand(need);
  assert.equal(typeof command, 'string');
  const transcript = messages(JSON.stringify({ command, nested: [9007199254740992] }));
  transcript[2].content = 'Output: workspace-discovery-v1\ntask_model.lino\nworkspace-discovery-end\nExit Code: 0';
  const progress = Progress.scan(transcript);
  assert.equal(qualifiedToolAttempt(progress, Capability.Run, { command, nested: [9007199254740992] }), null);
  const step = workspaceDiscoveryStep(need, progress, ['bash', 'read', 'write', 'list'], need);
  assert.equal(step.kind, 'tool_calls');
  assert.equal(step.calls.length, 1);
  assert.equal(step.calls[0].tool, 'list');
  assert.deepEqual(JSON.parse(step.calls[0].arguments), { path: '.' });
});

for (const order of ['unsafe-first', 'unsafe-last']) test('reused declaration ID stays contradicted across ' + order, () => {
  const safe = messages('{"command":"true","nested":[1]}');
  const unsafe = messages('{"command":"true","nested":[9007199254740993]}');
  const transcript = [safe[0], ...(order === 'unsafe-first' ? [unsafe[1], unsafe[2], safe[1], safe[2]] : [safe[1], safe[2], unsafe[1], unsafe[2]])];
  const frames = qualifiedTranscriptFrames(transcript, 0);
  assert.equal(frames.length, 1);
  assert.equal(frames[0].binding, 'contradicted');
  assert.equal(frames[0].succeeded, false);
  assert.equal(qualifiedToolAttempt(Progress.scan(transcript), Capability.Run, { command: 'true', nested: [1] }).binding, 'contradicted');
});
