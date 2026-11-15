// Install at rust/tests/web/source-repair-intake.test.mjs. Source-only draft.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { sourceRepairContract } from '../../../js/agentic/source_repair.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
const blocked = () => {
  throw Error('LocalWasmExecutionForbidden');
};
await installNodeHost(new WorkerHost({
  WebAssembly: {
    ...WebAssembly,
    instantiate: blocked,
    compile: blocked,
    Module: blocked,
    Instance: blocked
  }
}));
const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/source-repair-intake-cases.json', import.meta.url), 'utf8'));
for (const entry of fixture.cases) test(entry.name, () => {
  const contract = sourceRepairContract(entry.task);
  if (entry.unowned) {
    assert.equal(contract, null);
    return;
  }
  assert.equal(contract.owner, entry.owner);
  assert.equal(contract.authority, entry.authority);
});
test('actual maintained planner reads owned repair before formalizing embedded code, then honest gap', async context => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'owned-source-repair-'));
  context.after(() => fs.rmSync(directory, {
    recursive: true,
    force: true
  }));
  const entry = fixture.cases[0],
    owner = path.join(directory, entry.owner),
    source = '// Preserve 雪.\nexport const stable = 1;\n';
  fs.mkdirSync(path.dirname(owner), {
    recursive: true
  });
  fs.writeFileSync(owner, source);
  const result = await drive(planChatStep, directory, entry.task, {
    steps: 4,
    tools: ['read', 'edit', 'write'],
    allowedCommands: []
  });
  assert.deepEqual(result.transcript.map(row => row.tool), ['read']);
  assert.equal(JSON.parse(result.transcript[0].arguments).filePath, entry.owner);
  assert.equal(fs.readFileSync(owner, 'utf8'), source);
  const state = JSON.parse(result.answer);
  assert.equal(state.reason, 'DictionaryBindingUnsupported');
  assert.equal(state.task, entry.task);
  assert.equal(state.complete, false);
  assert.equal(state.authored, false);
  assert.equal(state.verified, false);
  for (const requirement of state.requirements) assert.equal(Buffer.from(entry.task).subarray(...requirement.span).toString(), requirement.text);
});
test('owned repair obeys existing no-read policy', async () => {
  const task = 'Do not read lib/bindings.mjs. Fix its legal key handling.';
  const plan = await planChatStep([{
    role: 'user',
    content: task
  }], ['read', 'edit', 'write']);
  assert.equal(JSON.parse(plan.answer).reason, 'ReadPolicyRefused');
});
const goalFixture = JSON.parse(fs.readFileSync(new URL('../fixtures/source-repair-goal-cases.json', import.meta.url), 'utf8'));
for (const entry of goalFixture.cases) test(entry.name, () => {
  assert.equal(sourceRepairContract(entry.task).operation, entry.operation);
});
test('wrong goals read eligible source then remain unsupported without mutation', async context => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'source-repair-goal-'));
  context.after(() => fs.rmSync(directory, {
    recursive: true,
    force: true
  }));
  fs.mkdirSync(path.join(directory, 'lib'));
  const target = path.join(directory, 'lib/records.mjs');
  fs.writeFileSync(target, goalFixture.eligibleSource);
  for (const entry of goalFixture.cases.filter(entry => entry.operation === null)) {
    const result = await drive(planChatStep, directory, entry.task, {
      steps: 4,
      tools: ['read', 'edit', 'write'],
      allowedCommands: []
    });
    assert.deepEqual(result.transcript.map(row => row.tool), ['read']);
    assert.equal(JSON.parse(result.answer).reason, 'SourceRepairOperatorUnavailable');
    assert.equal(fs.readFileSync(target, 'utf8'), goalFixture.eligibleSource);
  }
});
