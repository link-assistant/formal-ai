import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import { compileDeployedStagedAuthority, maintainedStagedInventory } from '../../../scripts/maintained-staged-authority.mjs';
import { buildStagedReleaseProjection } from '../../../scripts/generate-staged-release.mjs';

const root = new URL('../../../', import.meta.url);
const YAML = createRequire(import.meta.url)('yaml');
const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const read = file => fs.readFileSync(new URL(file, root), 'utf8');

test('current Pages smoke command is conserved by the strict deployed staged projection', () => {
  const inventory = maintainedStagedInventory(compileDeployedStagedAuthority());
  assert.equal(inventory.operations, 52);
  assert.equal(inventory.bindings, 104);
  assert.equal(inventory.productionAuthority, false);
  const source = read(evidence + 'original-release-workflow.yml');
  const packet = JSON.parse(read(evidence + 'stage-source-coverage.json'));
  const projection = buildStagedReleaseProjection(source, packet);
  assert.equal(read('.github/workflows/release.yml'), projection.outputs.get(evidence + 'candidate-release-caller.yml'));
  const command = doc => doc.jobs['test-e2e-pages'].steps.find(step => step.name === 'Run GitHub Pages e2e tests').run;
  const deployed = YAML.parse(read('.github/workflows/release.yml'));
  assert.equal(command(deployed), command(YAML.parse(source)));
  assert(command(deployed).includes('--config=playwright.pages.smoke.config.js'));
});
