import { host } from '../../../js/agentic/host.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { ownedSourceMaintenanceRequest, planSourceMaintenanceStep } from '../../../js/agentic/source_maintenance.mjs';
import { deriveSourceMaintenance } from '../../../js/agentic/source_maintenance_transform.mjs';
import { sourceMaintenanceStageKernel, sourceMaintenanceEffectKernel } from '../../../js/agentic/portable/source_maintenance_state.mjs';
await installNodeHost(new WorkerHost({
  WebAssembly: {
    instantiate() {
      throw Error('LocalWasmExecutionForbidden');
    }
  }
}));
const fixtures = new URL('../fixtures/source-maintenance/', import.meta.url);
function projected(plan) {
  if (plan === null) return null;
  if (plan.kind === 'tool_calls') return {
    kind: plan.kind,
    calls: plan.calls.map(call => ({
      tool: call.tool,
      arguments: JSON.parse(call.arguments)
    }))
  };
  return {
    kind: plan.kind,
    answer: plan.answer,
    disposition: plan.result.disposition,
    origin: plan.result.origin
  };
}
test('shared source observations retain exact bytes or refuse opaque unsupported syntax', () => {
  const data = JSON.parse(fs.readFileSync(new URL('observations.json', fixtures)));
  for (const row of data.cases) {
    const result = deriveSourceMaintenance(row.property, row.language, row.source);
    assert.deepEqual(result.gap ? {
      kind: 'gap',
      reason: result.gap
    } : {
      kind: 'normalized',
      source: result.source
    }, row.expected, row.id);
  }
});
test('shared declarative production route receipts preserve real requested effects and terminal evidence', async () => {
  const data = JSON.parse(fs.readFileSync(new URL('routes.json', fixtures)));
  for (const row of data.cases) {
    assert.deepEqual(projected(await planSourceMaintenanceStep(row.prompt, row.messages, row.tools)), row.expected, row.id);
  }
});
test('shared ownership rejects extra tasks and preserves unknown-source cold requests', () => {
  for (const prompt of ['Fix trailing whitespace in fresh.mjs. Also delete logs.', 'Explain how to fix trailing whitespace in fresh.mjs.', 'Do not fix trailing whitespace in fresh.mjs.', 'Fix trailing whitespace in ../outside.mjs.']) assert.equal(ownedSourceMaintenanceRequest(prompt), null, prompt);
  for (const [name, language] of [['new-orchid.mjs', 'javascript'], ['new-cedar.ts', 'typescript'], ['new-lumen.MJS', 'javascript']]) assert.equal(ownedSourceMaintenanceRequest('Trim trailing spaces in ' + name + '.').language, language);
});
test('typed canonical refusal states never silently repeat a rejected edit', () => {
  assert.deepEqual(sourceMaintenanceStageKernel(true, true, true, true, true, true, true), {
    $: 'refused',
    reason: 'SourceMaintenanceEditFailed',
    failure: true
  });
  assert.deepEqual(sourceMaintenanceStageKernel(true, true, false, false, false, false, false), {
    $: 'refused',
    reason: 'SourceMaintenanceCompleteReadRequired',
    failure: false
  });
  assert.deepEqual(sourceMaintenanceEffectKernel('', false, true, true), {
    $: 'refused',
    reason: 'SourceMaintenancePropertyNotSatisfied',
    failure: true
  });
  assert.deepEqual(sourceMaintenanceEffectKernel('', true, true, true), {
    $: 'finding'
  });
});
test('unavailable optional maintenance grammar preserves unrelated plans and grants no maintenance authority', () => {
  const installed = host();
  const original = installed.readText;
  installed.readText = relative => relative === 'data/meta/source-maintenance-grammar.lino' ? undefined : original(relative);
  try {
    assert.equal(ownedSourceMaintenanceRequest('What is 17 plus 2?'), null);
    assert.equal(ownedSourceMaintenanceRequest('Explain module fresh.mjs.'), null);
    assert.equal(ownedSourceMaintenanceRequest('Fix trailing whitespace in fresh.mjs.'), null);
  } finally {
    installed.readText = original;
  }
});
