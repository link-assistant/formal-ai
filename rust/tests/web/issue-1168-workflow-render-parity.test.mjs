// Issue #1168 R1168-8 parity: the workflow the agentic planner attaches to a
// program-contract recipe carries every generated-version pin. Its version
// source is injected (`renderWith` / Rust's `render_ci_workflow`), so both
// runtimes render it from the offline shipped baseline and must produce the
// fixture text rust/tests/unit/issue_1168_workflow_render_parity.rs asserts.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { baselineVersionSet, forGeneration } from '../../../js/agentic/crate/version_resolution.mjs';
import { render, renderWith } from '../../../js/agentic/ci_workflow.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

function fixture() {
  return JSON.parse(readText('rust/tests/fixtures/issue-1168/workflow-render-parity.json'));
}

test('the planner workflow renders the fixture pins from the injected baseline', () => {
  const { cases } = fixture();
  assert.equal(cases.length, 4);
  const versions = baselineVersionSet();
  for (const entry of cases) {
    const recipe = programContractAnswer(entry.prompt)?.execution_recipe;
    assert.ok(recipe, entry.id);
    assert.equal(recipe.path, entry.path, entry.id);
    assert.equal(renderWith(recipe, versions), entry.workflow, entry.id);
  }
});

test('render injects the generation-time set, which is the baseline offline', () => {
  const [entry] = fixture().cases;
  const recipe = programContractAnswer(entry.prompt).execution_recipe;
  assert.deepEqual(forGeneration(), baselineVersionSet());
  assert.equal(render(recipe), entry.workflow);
});
