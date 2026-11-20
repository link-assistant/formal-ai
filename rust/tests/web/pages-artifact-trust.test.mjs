// PR #1188 T937-T938: Pages inputs verify the artifact, never select executable code.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {createRequire} from 'node:module';
const YAML=createRequire(import.meta.url)('yaml');
const workflow = readFileSync(new URL('../../../.github/workflows/pages-artifact.yml', import.meta.url), 'utf8');
function shellFor(name) {
  const step = workflow.split('      - name: ' + name + '\n')[1].split('\n      - ')[0];
  return step.split('        run: |\n')[1].split('\n').map((line) => line.slice(10)).join('\n');
}
test('checkout uses the trusted triggering ref before verification or execution', () => {
  assert.match(workflow, /ref: \$\{\{ github\.ref \}\}/u);
  assert.doesNotMatch(workflow, /ref: \$\{\{ (?:inputs\.|steps\.pages_ref)/u);
  assert.ok(workflow.indexOf('Verify trusted release branch') < workflow.indexOf('Install web bundle dependencies'));
});
test('ref validation rejects missing values and shell payloads; accepts a full SHA', () => {
  for (const [sha, valid] of [['', false], ['main', false], ['a'.repeat(40), true], ['$(echo injected)', false], ['a'.repeat(40) + '\nother', false]]) {
    const result = spawnSync('bash', ['-c', shellFor('Select Pages deployment ref')], {
      encoding: 'utf8', env: { ...process.env, PAGES_DEPLOY_SHA: sha, GITHUB_OUTPUT: '/dev/null' },
    });
    assert.equal(result.status === 0, valid, sha);
  }
});
test('a changed branch or arbitrary input fails before build; matching release commit passes', () => {
  const script = 'git() { printf "%s\\n" "$CHECKOUT_SHA"; }\n' + shellFor('Verify trusted release branch matches the requested artifact commit');
  for (const [requested, expected] of [['a'.repeat(40), 0], ['b'.repeat(40), 1]]) {
    const result = spawnSync('bash', ['-c', script], {
      encoding: 'utf8', env: { ...process.env, CHECKOUT_SHA: 'a'.repeat(40), PAGES_DEPLOY_SHA: requested },
    });
    assert.equal(result.status, expected, result.stdout + result.stderr);
  }
});


test('Pages upload exposes its exact artifact ID through both reusable output boundaries', () => {
  const doc=YAML.parse(workflow);
  const upload=doc.jobs.build.steps.find(step => step.name==='Upload GitHub Pages artifact');
  assert.equal(upload.id,'pages_artifact');
  assert.equal(upload.uses,'actions/upload-pages-artifact@v5');
  // upload-pages-artifact uses artifact_id; upload-artifact uses artifact-id.
  assert.equal(doc.jobs.build.outputs.artifact_id,'${{ steps.pages_artifact.outputs.artifact_id }}');
  assert.equal(doc.on.workflow_call.outputs.artifact_id.value,'${{ jobs.build.outputs.artifact_id }}');
});

test('Pages deploy receives the exact artifact and stamped source through a workflow-owned action', () => {
  const doc=YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8'));
  const job=doc.jobs['deploy-pages'];
  assert.deepEqual(job.needs,['build-pages']);
  assert.equal(job['timeout-minutes'],15);
  assert.deepEqual(job.permissions,{contents:'read',pages:'write','id-token':'write'});
  assert.equal(job.concurrency.group,'formal-ai-repository-writes');
  const checkout=job.steps.find(step=>step.name==='Checkout Pages deployment protocol');
  assert.equal(checkout.uses,'actions/checkout@v7');
  assert.equal(checkout.with.ref,'${{ github.workflow_sha }}');
  assert.equal(checkout.with['persist-credentials'],false);
  assert.equal(checkout.with['fetch-depth'],1);
  assert.equal(checkout.with.filter,'blob:none');
  assert.equal(checkout.with['sparse-checkout'].trim(),'/.github/actions/deploy-pages/');
  const deployment=job.steps.find(step=>step.id==='deployment');
  assert.equal(deployment.uses,`./${checkout.with.path}/.github/actions/deploy-pages`);
  assert.deepEqual(deployment.with,{artifact_id:'${{ needs.build-pages.outputs.artifact_id }}',source_sha:'${{ needs.build-pages.outputs.pages_sha }}',timeout:600000});
  assert.equal(deployment.env.NODE_OPTIONS,'--throw-deprecation');
  assert.equal(job.outputs.pages_sha,'${{ needs.build-pages.outputs.pages_sha }}');
  assert.equal(job.environment.url,'${{ steps.deployment.outputs.page_url }}');
  assert(job.steps.indexOf(checkout)<job.steps.indexOf(deployment));
});
