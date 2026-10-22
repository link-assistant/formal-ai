import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { compactSourceRequest, observedCallableRequest } from '../../../js/agentic/module_function.mjs';
import { deriveCompleteSourceRequest } from '../../../js/agentic/module_function/complete-source-preflight.mjs';
import { readText } from '../../../js/agentic/host.mjs';
await installNodeHost(new WorkerHost());
// Independent syntax fixture changes all declaration operands; no expected implementation is supplied.
const task = 'Implement exported renderChosen(pick) to nested/caption.mjs: return the compact summary of the selected candidate or null. '
  + 'Discover and compose observed left.mjs and right.mjs exports through imports using aliases chooseRecord as chosen, describeRecord as describe; '
  + 'preserve inputs and reuse semantics. Run node --test gate.test.mjs; keep source modules and tests unchanged.';
const seed = () => readText('data/seed/source-authoring-grammar.lino');
test('compact request binds technical operands and covers all real source clauses', () => {
  const result = observedCallableRequest(task);
  assert.equal(result.name, 'renderChosen');
  assert.deepEqual(result.parameters, ['pick']);
  assert.equal(result.destination, 'nested/caption.mjs');
  assert.deepEqual(result.inputs, ['left.mjs', 'right.mjs']);
  assert.deepEqual(result.acceptance, ['gate.test.mjs']);
  assert.equal(result.command, 'node --test gate.test.mjs');
  assert.deepEqual(result.aliases, [{exported:'chooseRecord',local:'chosen'},{exported:'describeRecord',local:'describe'}]);
  for (const clause of result.sourceClauses) assert.equal(task.slice(clause.start, clause.end), clause.sentence);
});
test('complete frame preserves exact goal evidence and labels mechanical prerequisites as inferred', () => {
  const frame = deriveCompleteSourceRequest(task, seed());
  assert.ok(frame);
  const goal = frame.clauses.find(clause => clause.kind === 'goal');
  assert.equal(goal.sentence, 'return the compact summary of the selected candidate or null');
  assert.equal(goal.evidence.source, goal.sentence);
  assert.equal(task.slice(goal.evidence.span.start, goal.evidence.span.end), goal.sentence);
  assert.deepEqual(frame.inferredOperationPrerequisites.map(value => value.kind), ['read-prerequisites','repair-plan','gap-before-write']);
  assert.equal(frame.semanticAuthority, false);
  assert.equal(frame.writeAuthority, false);
});
test('original nine-clause source task retains all explicitly stated requirements', () => {
  const original = JSON.parse(readFileSync(new URL('../../../experiments/formal_ai_subagent/evidence/formal-ai-only-1188/g132/T1804-request.json', import.meta.url))).prompt;
  assert.equal(Buffer.byteLength(original), 955);
  const frame = deriveCompleteSourceRequest(original, seed());
  assert.ok(frame);
  assert.equal(frame.clauses.length, 9);
  assert.equal(frame.inferredOperationPrerequisites, undefined);
  assert.equal(frame.clauses.find(clause => clause.kind === 'goal').sentence,
    'Return the compact canonical summary of the candidate chosen by the selection, or null when no candidate is selected');
});
for (const [name, mutate] of [
  ['unknown trailing effect', task => task + ' Delete another.mjs.'],
  ['unknown inner effect', task => task.replace('preserve inputs and reuse semantics', 'preserve inputs and reuse semantics and deploy the release')],
  ['missing immutable-source obligation', task => task.replace('; keep source modules and tests unchanged', '')],
  ['duplicate accepted operation', task => task + ' Run node --test other.mjs.'],
  ['duplicate source', task => task.replace('left.mjs and right.mjs', 'left.mjs and left.mjs')],
  ['source/destination conflict', task => task.replace('left.mjs and right.mjs', 'nested/caption.mjs and right.mjs')],
  ['source/test conflict', task => task.replace('left.mjs and right.mjs', 'gate.test.mjs and right.mjs')],
  ['parent traversal', task => task.replace('left.mjs and right.mjs', '../left.mjs and right.mjs')],
  ['unsupported destination language', task => task.replace('nested/caption.mjs', 'nested/caption.rs')],
  ['duplicate export alias', task => task.replace('describeRecord as describe', 'chooseRecord as describe')],
  ['duplicate local alias', task => task.replace('describeRecord as describe', 'describeRecord as chosen')],
  ['alias expression', task => task.replace('chooseRecord as chosen', 'chooseRecord as chosen()')],
  ['unconsumed quoted instructions', task => task.replace('preserve inputs and reuse semantics', '"preserve inputs and reuse semantics"')],
]) test('refuse incomplete ownership: ' + name, () => {
  const changed = mutate(task);
  assert.equal(compactSourceRequest(changed), null);
  assert.equal(deriveCompleteSourceRequest(changed, seed()), null);
});
test('missing compact seed cannot authorize a request from its operand names', () => {
  const changed = seed().replace('source-request-compact', 'unowned-request-compact');
  assert.equal(compactSourceRequest(task, changed), null);
  assert.equal(deriveCompleteSourceRequest(task, changed), null);
});
