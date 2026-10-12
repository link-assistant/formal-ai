import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import YAML from 'yaml';
import { automaticRecoveryInputs } from '../../../scripts/release-recovery-decision.mjs';
import { buildStagedReleaseProjection } from '../../../scripts/generate-staged-release.mjs';
const environment = {
  GITHUB_REF: 'refs/heads/main', GITHUB_EVENT_NAME: 'push',
  GITHUB_SHA: 'a'.repeat(40), GITHUB_REPOSITORY: 'owner/repository',
  GITHUB_WORKFLOW_REF: 'owner/repository/.github/workflows/release.yml@refs/heads/main',
  RELEASE_RECOVERY_PROTOCOL_ROOT: '_protocol', RELEASE_RECOVERY_PROTOCOL_COMMIT: 'a'.repeat(40)
};
for (const event of ['push', 'workflow_dispatch']) test('ordinary context accepts supported production event ' + event, () => {
  const selected = automaticRecoveryInputs({...environment, GITHUB_EVENT_NAME: event}, '1.2.3');
  assert.equal(selected.protocolCommit, environment.GITHUB_SHA);
  assert.equal(selected.protocolRoot, '_protocol');
  assert.equal(Object.isFrozen(selected), true);
  assert.equal('publicationComplete' in selected, false);
});
for (const [key, value] of [
  ['GITHUB_REF', 'refs/tags/v1.2.3'], ['GITHUB_EVENT_NAME', 'pull_request'],
  ['GITHUB_EVENT_NAME', 'workflow_run'], ['GITHUB_WORKFLOW_REF', 'owner/foreign/.github/workflows/release.yml@refs/heads/main'],
  ['RELEASE_RECOVERY_PROTOCOL_ROOT', '../untrusted'], ['RELEASE_RECOVERY_PROTOCOL_COMMIT', 'b'.repeat(40)],
  ['GITHUB_SHA', 'truncated'], ['GITHUB_REPOSITORY', 'owner/repository/foreign']
]) test('ordinary context refuses changed ' + key + ' ' + value, () => {
  assert.throws(() => automaticRecoveryInputs({...environment, [key]: value}, '1.2.3'));
});
test('ordinary context rejects noncanonical release version', () => {
  assert.throws(() => automaticRecoveryInputs(environment, '01.2.3'));
  assert.throws(() => automaticRecoveryInputs(environment, '1.2'));
});
test('generated ordinary source checks preserve commands and use immutable context', () => {
  const prefix = '../../../experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  const original = fs.readFileSync(new URL(prefix + 'original-release-workflow.yml', import.meta.url), 'utf8');
  const packet = JSON.parse(fs.readFileSync(new URL(prefix + 'stage-source-coverage.json', import.meta.url)));
  const generated = buildStagedReleaseProjection(original, packet);
  assert.equal(generated.operations, 52);
  assert.equal(generated.binding.length, 104);
  const workflow = YAML.parse(generated.outputs.get('.github/workflows/release-staged.yml'));
  let selected = 0;
  for (const job of Object.values(workflow.jobs)) for (const step of job.steps) {
    if (step.run === 'rust-script scripts/check-release-needed.rs') {
      selected++;
      assert.equal(step.env.RELEASE_RECOVERY_PROTOCOL_ROOT, '_protocol');
      assert.equal(step.env.RELEASE_RECOVERY_PROTOCOL_COMMIT, '${{ github.sha }}');
      assert.ok(['check', 'prepared'].includes(step.id));
    } else assert.equal(step.env?.RELEASE_RECOVERY_PROTOCOL_ROOT, undefined);
  }
  assert.equal(selected, 2);
  for (const [name, job] of Object.entries(workflow.jobs)) {
    if (name.endsWith('_prepare-source')) {
      const checkout = job.steps.find(step => step.with?.path === '_protocol');
      assert.equal(checkout.with['fetch-depth'], 0);
      assert.equal(checkout.with['persist-credentials'], false);
      const check = job.steps.find(step => step.id === 'check' && step.run === 'rust-script scripts/check-release-needed.rs');
      if (check) {
      assert.equal(job.outputs.check__recovery_missing_assets, '${{ steps.check.outputs.recovery_missing_assets }}');
      } else {
        assert.equal(job.steps.some(step => step.id === 'check'), false);
        assert.equal(job.outputs.check__recovery_missing_assets, undefined);
        assert.equal(job.outputs.check__recovery_state, undefined);
      }
    }
  }
});
