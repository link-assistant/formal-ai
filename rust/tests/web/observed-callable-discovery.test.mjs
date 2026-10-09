import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { moduleFunctionRequest, observedCallableRequest, planModuleFunctionStep } from '../../../js/agentic/module_function.mjs';
import { FinalDisposition, canDeliverFinal } from '../../../js/agentic/final_result.mjs';
await installNodeHost(new WorkerHost());
const prompt = 'Add exported caption(pick) to nested/output.mjs. Read first.mjs and second.mjs before authoring. '
  + 'Run the supplied accept.test.mjs acceptance command, node --test accept.test.mjs, and report the real result.';
const sha = (content) => createHash('sha256').update(content).digest('hex');
const fixture = (body) => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'callable-discovery-'));
  const entries = { 'nested/output.mjs': '// existing destination\n',
    'first.mjs': 'export function take(input) { return input.value; }\n',
    'second.mjs': 'export function print(value) { return String(value); }\n',
    'accept.test.mjs': 'throw new Error("acceptance must not run before a candidate exists");\n' };
  for (const [name, content] of Object.entries(entries)) {
    fs.mkdirSync(path.dirname(path.join(dir, name)), { recursive: true });
    fs.writeFileSync(path.join(dir, name), content);
  }
  return Promise.resolve(body(dir, entries)).finally(() => fs.rmSync(dir, { recursive: true, force: true }));
};
const receipt = (name, content) => [{ role: 'assistant', content: '', tool_calls: [
  { id: name, type: 'function', function: { name: 'read', arguments: JSON.stringify({ path: name }) } } ] },
{ role: 'tool', content, tool_call_id: name }];

test('destination, immutable inputs and acceptance are declaration-scoped', () => {
  assert.deepEqual(observedCallableRequest(prompt), { name: 'caption', parameters: ['pick'], destination: 'nested/output.mjs',
    inputs: ['first.mjs', 'second.mjs'], acceptance: ['accept.test.mjs'], command: 'node --test accept.test.mjs' });
});

test('adjacent declared module and alias signature bind the same reusable discovery', () => {
  const request = observedCallableRequest('Implement a JavaScript module reports/result.mjs. Export describeChosen(selection). '
    + 'Read left.mjs and right.mjs. Run node --test gate.mjs.');
  assert.equal(request.destination, 'reports/result.mjs');
  assert.equal(request.name, 'describeChosen');
  assert.deepEqual(request.inputs, ['left.mjs', 'right.mjs']);
});

test('ambiguous destinations and quoted call shapes are not guessed', () => {
  assert.equal(observedCallableRequest('Add caption(pick) to a.mjs and b.mjs. Read source.mjs.'), null);
  assert.equal(observedCallableRequest('Implement a module result.mjs. Read source.mjs with the label «caption(pick)». '), null);
  assert.equal(observedCallableRequest('Read existing.mjs then execute arbitrary(input). Read source.mjs.'), null);
});

test('existing arithmetic module/test request remains on its original route', () => {
  const arithmetic = 'Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, and run node --test.';
  assert.equal(observedCallableRequest(arithmetic), null);
  assert.equal(moduleFunctionRequest(arithmetic).module, 'math.mjs');
  assert.equal(moduleFunctionRequest(arithmetic).command, 'node --test');
});

test('actual planner reads all operands, reports exact missing contracts, and changes no bytes', async () => {
  await fixture(async (dir, entries) => {
    const result = await drive(planChatStep, dir, prompt + ' Incidental issue numbers are 188 and 558.', { steps: 12 });
    assert.deepEqual(result.transcript.map((call) => [call.tool, JSON.parse(call.arguments).path]),
      Object.keys(entries).map((name) => ['read', name]));
    const evidence = JSON.parse(result.answer.slice(result.answer.indexOf('\n') + 1));
    assert.equal(evidence.reason, 'MissingContract');
    assert.equal(evidence.authored, false);
    assert.equal(evidence.verified, false);
    assert.deepEqual(evidence.missingContracts.map((item) => item.path), ['first.mjs', 'second.mjs']);
    for (const observation of evidence.observations) {
      assert.equal(observation.contentId, sha(entries[observation.path]));
      assert.equal(fs.readFileSync(path.join(dir, observation.path), 'utf8'), entries[observation.path]);
    }
  });
});

test('an absent destination is observed without creating it', async () => {
  await fixture(async (dir) => {
    fs.unlinkSync(path.join(dir, 'nested/output.mjs'));
    const result = await drive(planChatStep, dir, prompt, { steps: 12 });
    assert.ok(result.answer.includes('MissingContract'));
    assert.equal(fs.existsSync(path.join(dir, 'nested/output.mjs')), false);
    assert.equal(result.transcript.some((call) => call.tool !== 'read'), false);
  });
});

test('missing input is a failed read, never an empty source or a writing fallback', async () => {
  await fixture(async (dir, entries) => {
    fs.unlinkSync(path.join(dir, 'first.mjs'));
    const result = await drive(planChatStep, dir, prompt, { steps: 12 });
    assert.ok(result.answer.includes('ReadFailed'));
    assert.deepEqual(result.transcript.map((call) => JSON.parse(call.arguments).path), ['nested/output.mjs', 'first.mjs']);
    assert.equal(fs.readFileSync(path.join(dir, 'nested/output.mjs'), 'utf8'), entries['nested/output.mjs']);
  });
});

test('permission errors on the destination retain typed failure rather than absence', async () => {
  const messages = [{ role: 'user', content: prompt }, ...receipt('nested/output.mjs', JSON.stringify({ is_error: true, error: 'EACCES denied' }))];
  const plan = await planModuleFunctionStep(prompt, messages, ['read', 'write']);
  assert.equal(plan.result.disposition, FinalDisposition.Failure);
  assert.equal(plan.result.discovery.reason, 'ReadFailed');
  assert.equal(canDeliverFinal(plan), false);
});

test('missing read capability is an exact typed gap, not a declined route', async () => {
  const plan = await planModuleFunctionStep(prompt, [{ role: 'user', content: prompt }], ['write']);
  assert.equal(plan.result.disposition, FinalDisposition.Gap);
  assert.equal(plan.result.discovery.reason, 'MissingReadTool');
  assert.equal(canDeliverFinal(plan), false);
});

test('old request evidence is not reused for a new discovery request', async () => {
  const messages = [{ role: 'user', content: 'old request' }, ...receipt('nested/output.mjs', 'old bytes'), { role: 'user', content: prompt }];
  const plan = await planModuleFunctionStep(prompt, messages, ['read']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(JSON.parse(plan.calls[0].arguments).path, 'nested/output.mjs');
});


test('explicit client error flags stay bound to the observed read', async () => {
  const records = receipt('nested/output.mjs', 'not permitted');
  records[1].is_error = true;
  const plan = await planModuleFunctionStep(prompt, [{ role: 'user', content: prompt }, ...records], ['read', 'write']);
  assert.equal(plan.result.disposition, FinalDisposition.Failure);
  assert.equal(plan.result.discovery.reason, 'ReadFailed');
});

test('numbered source payload failure words remain data', async () => {
  await fixture(async (dir) => {
    fs.writeFileSync(path.join(dir, 'first.mjs'), 'export const label = "failed error ENOENT";\n');
    const result = await drive(planChatStep, dir, prompt, { steps: 12 });
    assert.ok(result.answer.includes('MissingContract'));
    assert.equal(result.transcript.length, 4);
  });
});


test('signature identifiers and path components cannot supply instruction verbs', () => {
  for (const request of ['Read existing f(write) from first.mjs. Read second.mjs.',
    'Read existing f(x) from create.mjs. Read second.mjs.',
    'Read existing implement(x) from first.mjs. Read second.mjs.']) {
    assert.equal(observedCallableRequest(request), null, request);
  }
  assert.equal(observedCallableRequest(prompt).destination, 'nested/output.mjs');
});

test('public request-local outcome preserves the actual typed gap and complete witness', async () => {
  const { planObservedCallableOutcome } = await import('../../../js/agentic/module_function/discovery.mjs');
  const request = observedCallableRequest(prompt);
  const messages = [{ role: 'user', content: prompt }, ...receipt('nested/output.mjs', '// destination'),
    ...receipt('first.mjs', 'export function take(x) { return x; }'),
    ...receipt('second.mjs', 'export function render(x) { return x === null; }'),
    ...receipt('accept.test.mjs', '// immutable acceptance')];
  const outcome = planObservedCallableOutcome(request, messages, ['read', 'write']);
  assert.equal(outcome.disposition, FinalDisposition.Gap);
  assert.equal(outcome.witness, outcome.plan.result.discovery);
  assert.equal(outcome.witness.reason, 'MissingContract');
  assert.equal(outcome.witness.authored, false);
  assert.equal(outcome.witness.observations[1].contentId, sha('export function take(x) { return x; }'));
  assert.equal(canDeliverFinal(outcome.plan), false);
});

test('public request-local outcome retains failure metadata without prose classification', async () => {
  const { planObservedCallableOutcome } = await import('../../../js/agentic/module_function/discovery.mjs');
  const records = receipt('nested/output.mjs', 'apparently successful prose'); records[1].is_error = true;
  const outcome = planObservedCallableOutcome(observedCallableRequest(prompt), [{ role: 'user', content: prompt }, ...records], ['read']);
  assert.equal(outcome.disposition, FinalDisposition.Failure);
  assert.equal(outcome.witness.reason, 'ReadFailed');
  assert.equal(canDeliverFinal(outcome.plan), false);
});

test('a planned read has no final disposition or stale discovery witness', async () => {
  const { planObservedCallableOutcome } = await import('../../../js/agentic/module_function/discovery.mjs');
  const outcome = planObservedCallableOutcome(observedCallableRequest(prompt), [{ role: 'user', content: prompt }], ['read']);
  assert.equal(outcome.plan.kind, 'tool_calls');
  assert.equal(outcome.disposition, null);
  assert.equal(outcome.witness, null);
});
