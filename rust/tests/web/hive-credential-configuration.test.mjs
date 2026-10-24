import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync, mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import YAML from 'yaml';
import {requireTaskCredentialConfiguration, taskCredentialScope,
  requireTaskCredentialAccess} from '../../../scripts/lib/hive-delivery-credentials.mjs';

test('missing or incomplete optional credentials select the default layer', () => {
  for (const environment of [{}, {GH_TOKEN: 'default-repository-token'}, {AUTOMATION_APP_ID: '123'},
    {AUTOMATION_APP_PRIVATE_KEY: 'private-placeholder'}, {AUTOMATION_TOKEN: ' \n'}]) {
    assert.deepEqual(requireTaskCredentialConfiguration(environment),
      {configured: false, layer: 'default', deliveryVerified: false});
  }
});
test('configured token or complete App pair passes presence check without claiming target access or echoing secrets', () => {
  for (const [environment, layer] of [[{AUTOMATION_TOKEN: 'private-placeholder'}, 'token'],
    [{AUTOMATION_APP_ID: '123', AUTOMATION_APP_PRIVATE_KEY: 'private-placeholder'}, 'app']]) {
    assert.deepEqual(requireTaskCredentialConfiguration(environment), {configured: true, layer, deliveryVerified: false});
    assert(!JSON.stringify(requireTaskCredentialConfiguration(environment)).includes('private-placeholder'));
  }
});

test('the default token can authorize only the current repository and staged workflow execution', () => {
  const canonical = {owner: 'external', name: 'task', repository: 'external/task'};
  const target = taskCredentialScope(canonical, {layer: 'default', repository: 'host/project'});
  const repository = {id: 1, full_name: 'host/project', name: 'project', owner: {login: 'host'},
    permissions: {push: true}, private: false};
  const proof = requireTaskCredentialAccess(target, {repository, layer: 'default', currentRepository: 'host/project'});
  assert.equal(proof.workflowWrite, 'staged-act');
  assert.equal(proof.deliveryVerified, false);
  assert.throws(() => requireTaskCredentialAccess(target,
    {repository, layer: 'default', currentRepository: 'other/project'}));
  assert.throws(() => requireTaskCredentialAccess(target,
    {repository: {...repository, permissions: {push: false}}, layer: 'default', currentRepository: 'host/project'}));
  assert.deepEqual(taskCredentialScope(canonical, {layer: 'token', repository: 'host/project'}), canonical);
});

test('the actual resolver selects a minted App token or the usable token/default fallback', () => {
  const action = YAML.parse(readFileSync(new URL('../../../.github/actions/automation-token/action.yml', import.meta.url), 'utf8'));
  assert.equal(action.runs.steps.find(step => step.id === 'app-token')['continue-on-error'], true);
  const script = action.runs.steps.find(step => step.id === 'selected').run;
  for (const [app, fallback, layer, expected] of [['', 'job-placeholder', 'default', 'default'],
    ['', 'token-placeholder', 'token', 'token'], ['app-placeholder', 'job-placeholder', 'default', 'app']]) {
    const directory = mkdtempSync(path.join(tmpdir(), 'hive-credential-selection-'));
    try {
      const output = path.join(directory, 'output'), summary = path.join(directory, 'summary');
      writeFileSync(output, ''); writeFileSync(summary, '');
      execFileSync('bash', ['-c', script], {env: {PATH: process.env.PATH, APP_TOKEN: app,
        FALLBACK_TOKEN: fallback, FALLBACK_LAYER: layer, GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: summary}});
      const selected = readFileSync(output, 'utf8');
      assert(selected.includes('layer=' + expected + '\n'));
      assert(selected.includes('token=' + (app || fallback) + '\n'));
      assert(readFileSync(summary, 'utf8').includes('`' + expected + '`'));
    } finally { rmSync(directory, {recursive: true, force: true}); }
  }
});
