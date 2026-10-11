import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import YAML from 'yaml';
import { validateQueueSnapshot, validateFixtureOutputs } from '../../../scripts/release-runtime-fixture.mjs';
const root = new URL('../../../', import.meta.url);
const caller = () => YAML.parse(readFileSync(new URL('.github/workflows/release-runtime-fixture.yml', root), 'utf8'));
const callee = () => YAML.parse(readFileSync(new URL('.github/workflows/release-runtime-fixture-callee.yml', root), 'utf8'));
const helper = () => readFileSync(new URL('scripts/release-runtime-fixture.mjs', root), 'utf8');
function downloadInputs(step) {
  assert.equal(step.run, 'node scripts/release-runtime-fixture.mjs download');
  assert.equal(step.uses, undefined);
  assert.deepEqual(Object.keys(step.env).sort(), ['ARTIFACT_DESTINATION', 'ARTIFACT_ID', 'ARTIFACT_NAME', 'DIGEST_MISMATCH', 'GH_TOKEN']);
  assert.equal(step.env.GH_TOKEN, '${{ github.token }}');
  assert.equal(step.env.DIGEST_MISMATCH, 'error');
  return { 'artifact-ids': step.env.ARTIFACT_ID, path: step.env.ARTIFACT_DESTINATION,
    'digest-mismatch': step.env.DIGEST_MISMATCH };
}
function validateGraph(parent, child) {
  assert.deepEqual(parent.permissions, {
    contents: 'read',
    actions: 'read'
  });
  assert.deepEqual(child.permissions, parent.permissions);
  assert.equal(child.concurrency, undefined);
  assert.equal(parent.on.pull_request.types.includes('synchronize'), true);
  const privateGroup = 'formal-ai-release-fixture-${{ github.run_id }}-${{ github.run_attempt }}';
  for (const id of ['active-fixture', 'inactive-fixture']) {
    const job = parent.jobs[id];
    assert.equal(job.uses, './.github/workflows/release-runtime-fixture-callee.yml');
    assert.equal(job.needs, 'auto_compile-release');
    assert.equal(job.secrets, undefined);
    assert.equal(job.environment, undefined);
    for (const key of ['outputs', 'steps', 'runs-on', 'timeout-minutes', 'env']) assert.equal(job[key], undefined);
    for (const [input, output] of Object.entries({
      source: 'source',
      tree: 'tree',
      'compiler-marker-sha': 'compiler_marker_sha'
    })) assert.equal(job.with[input], '${{ needs.auto_compile-release.outputs.' + output + ' }}');
    assert.equal(job.concurrency.queue, 'max');
    assert.equal(job.concurrency['cancel-in-progress'], undefined);
  }
  assert.equal(parent.jobs['active-fixture'].concurrency.group, privateGroup);
  assert.equal(parent.jobs['inactive-fixture'].concurrency.group, 'formal-ai-release-fixture-inactive-${{ github.run_id }}-${{ github.run_attempt }}');
  assert.equal(parent.jobs['active-fixture'].with.active, true);
  assert.equal(parent.jobs['inactive-fixture'].with.active, false);
  for (const [input, output] of Object.entries({
    'artifact-id': 'artifact_id',
    'bundle-sha': 'bundle_sha',
    'descriptor-sha': 'descriptor_sha'
  })) {
    assert.equal(parent.jobs['active-fixture'].with[input], '${{ needs.auto_compile-release.outputs.' + output + ' }}');
    assert.equal(parent.jobs['inactive-fixture'].with[input], '');
  }
  assert.deepEqual(parent.jobs['fixture-contender'].needs, ['auto_compile-release', 'observer'], 'contender must be independent from reusable caller completion');
  assert.equal(parent.jobs['fixture-contender'].concurrency.group, privateGroup);
  assert.equal(parent.jobs['fixture-contender'].concurrency.queue, 'max');
  assert.equal(parent.jobs.observer.needs, 'auto_compile-release');
  assert.deepEqual(parent.on.pull_request['branches-ignore'], ['e2e/**']);
  const ordinaryGroups = new Set();
  for (const role of ['auto_compile-release', 'observer', 'collect']) {
    const expected = 'formal-ai-release-fixture-check-' + role + '-${{ github.workflow }}-${{ github.ref }}';
    assert.deepEqual(parent.jobs[role].concurrency, {group: expected, 'cancel-in-progress': true});
    assert.notEqual(expected, privateGroup, 'ordinary checks cannot own the private caller lease');
    ordinaryGroups.add(expected);
  }
  assert.equal(ordinaryGroups.size, 3);
  const admission = parent.jobs.observer.steps.find(step => step.name === 'Observe authenticated caller admission before queueing contender');
  assert.equal(admission.run, 'node scripts/release-runtime-fixture.mjs stage-contender');
  assert.deepEqual(admission.env, {GH_TOKEN: '${{ github.token }}'});
  assert.equal(parent.jobs.observer.steps.some(step => step.run === 'node scripts/release-runtime-fixture.mjs download'), false);
  const liveWitness = child.jobs['lease-holder'].steps.find(step => step.name === 'Expose live callee witness before observer finishes');
  assert.equal(liveWitness.uses, 'actions/upload-artifact@v7');
  assert.equal(liveWitness.with.name, 'release-fixture-lease-start-${{ github.run_id }}-${{ github.run_attempt }}');
  assert.equal(liveWitness.with.path, '.release-fixture/lease-start.json');
  assert.equal(child.jobs['lease-holder'].if, 'inputs.active');
  assert.equal(child.jobs['artifact-consumer'].if, 'inputs.active');
  assert.equal(child.jobs['artifact-consumer'].needs, 'lease-holder');
  const download = child.jobs['artifact-consumer'].steps.find(step => step.name === 'Download single immutable caller artifact by actual ID');
  assert.equal(download.env.ARTIFACT_NAME, 'release-fixture-text-${{ github.run_id }}-${{ github.run_attempt }}');
  assert.deepEqual(downloadInputs(download), {
    'artifact-ids': '${{ inputs.artifact-id }}',
    path: '.release-fixture/consumer/.release-transfer/binary',
    'digest-mismatch': 'error'
  });
  assert.deepEqual(child.jobs['fixture-final'].needs, ['lease-holder', 'artifact-consumer']);
  assert.equal(child.jobs['fixture-final'].if, "${{ always() && (!inputs.active || (needs.lease-holder.result == 'success' && needs.artifact-consumer.result == 'success')) }}");
  for (const [name, output] of Object.entries({
    'fixture-source': 'fixture_source',
    'artifact-observation': 'artifact_observation',
    'fixture-marker': 'fixture_marker',
    'lease-observation': 'lease_observation'
  })) {
    assert.equal(child.on.workflow_call.outputs[name].value, '${{ jobs.fixture-final.outputs.' + output + ' }}');
    assert.equal(child.jobs['fixture-final'].outputs[output], '${{ steps.final.outputs.' + output + ' }}');
  }
  for (const [id, job] of Object.entries(child.jobs)) assert.equal(job.concurrency, undefined, 'no duplicate child lease ' + id);
  for (const workflow of [parent, child]) {
    for (const job of Object.values(workflow.jobs)) {
      assert.equal(job.environment, undefined);
      assert.equal(job.secrets, undefined);
      if (job.steps) {
        assert.equal(job['timeout-minutes'], 5);
        for (const step of job.steps) {
          if (step.uses === 'actions/checkout@v7') assert.deepEqual(step.with, {
            ref: '${{ github.sha }}',
            'persist-credentials': false
          });
          assert.ok(!step.run || /^node scripts\/release-runtime-fixture\.mjs (?:download|produce|consume|lease-start|wait-queue|observe-start|stage-contender|verify-start|finalize|contender|collect)$/.test(step.run));
          if (step.uses === 'actions/download-artifact@v8') for (const forbidden of ['github-token', 'repository', 'run-id', 'pattern', 'name']) assert.equal(step.with[forbidden], undefined);
        }
      }
    }
  }
}
test('runtime fixture uses read-only same-run IDs, actual outputs, inactive skipping and private caller-only leases', () => {
  validateGraph(caller(), callee());
  const source = helper();
  assert.ok(source.includes("import { seal, importBundle } from './release-stage-transfer.mjs'"));
  assert.ok(source.includes("method: 'GET'"));
  assert.ok(source.includes("https://api.github.com"));
  assert.ok(source.includes("compilerInvoked: false") && source.includes("productionAuthority: false"));
  assert.ok(!/rustc|cargo |rust-script|docker |version-and-commit|invoke-create-github-release|release-staged\.yml/.test(source), 'fixture cannot invoke production authority/native commands');
});
test('wrong run/group, missing ancestor or unqueued contender never become actual lease proof', () => {
  const expected = {
    group: 'formal-ai-release-fixture-123-1',
    run: '123'
  };
  const snapshot = {
    status: 200,
    body: {
      group_name: expected.group,
      group_members: [{
        run_id: 123,
        status: 'in_progress'
      }, {
        run_id: 123,
        status: 'pending',
        job_name: 'Fixture private queued contender',
        job_id: 456
      }]
    }
  };
  assert.equal(validateQueueSnapshot(snapshot, expected), true);
  for (const mutate of [item => item.status = 403,
     item => item.body.group_name = 'formal-ai-repository-writes',
     item => item.body.group_members[0].run_id = 124,
     item => item.body.group_members[0].status = 'pending',
     item => item.body.group_members[1].status = 'in_progress',
     item => item.body.group_members[1].job_name = 'unrelated',
     item => item.body.group_members[1].job_id = '456',
     item => item.body.group_members.pop()]) {

    const changed = structuredClone(snapshot);
    mutate(changed);
    assert.throws(() => validateQueueSnapshot(changed, expected));
  }
});
test('real active/inactive output contract rejects skipped success, wrong source and early marker', () => {
  const source = 'a'.repeat(40);
  const active = {
    source,
    artifact: 'VerifiedTextTar',
    marker: 'fixture-text-artifact-roundtrip'
  };
  const inactive = {
    source,
    artifact: 'SkippedInactive',
    marker: ''
  };
  validateFixtureOutputs(active, inactive, source);
  for (const mutate of [(a, i) => a.source = 'b'.repeat(40), (a, i) => i.source = 'b'.repeat(40), (a, i) => a.artifact = 'Unknown', (a, i) => a.marker = '', (a, i) => i.artifact = 'VerifiedTextTar', (a, i) => i.marker = 'fixture-text-artifact-roundtrip']) {
    const a = structuredClone(active),
      i = structuredClone(inactive);
    mutate(a, i);
    assert.throws(() => validateFixtureOutputs(a, i, source));
  }
});
test('credential routing, production group, nested lease and spoofed or ungated artifact graphs are refused', () => {
  const parent = caller(),
    child = callee();
  for (const mutate of [(p, c) => p.permissions.contents = 'write', (p, c) => p.jobs['active-fixture'].secrets = 'inherit', (p, c) => p.jobs['active-fixture'].concurrency.group = 'formal-ai-repository-writes', (p, c) => c.jobs['lease-holder'].concurrency = {
    group: p.jobs['active-fixture'].concurrency.group
  }
    ,
     (p,
     c) => p.jobs['fixture-contender'].needs.push('active-fixture'),
     (p,
     c) => p.jobs['active-fixture'].with['artifact-id'] = 'arbitrary',
     (p,
     c) => p.jobs['inactive-fixture'].with['artifact-id'] = 'arbitrary',
     (p,
     c) => c.jobs['artifact-consumer'].if = undefined,
     (p,
     c) => c.jobs['artifact-consumer'].steps.find(step => step.run === 'node scripts/release-runtime-fixture.mjs download').env.ARTIFACT_RUN = 'other-run',
     (p,
     c) => c.on.workflow_call.outputs['fixture-marker'].value = 'guessed',
     (p,
     c) => p.jobs['active-fixture'].uses = './.github/workflows/release-staged.yml']) {

    const p = structuredClone(parent),
      c = structuredClone(child);
    mutate(p, c);
    assert.throws(() => validateGraph(p, c));
  }
});

test('ordinary check groups preserve private lease isolation and e2e filtering', () => {
  for (const role of ['auto_compile-release', 'observer', 'collect']) {
    for (const change of [
      p => delete p.jobs[role].concurrency,
      p => p.jobs[role].concurrency.group = p.jobs['active-fixture'].concurrency.group,
      p => p.jobs[role].concurrency['cancel-in-progress'] = false,
      p => p.jobs[role].concurrency.group = 'formal-ai-repository-writes'
    ]) {
      const p = caller(); change(p); assert.throws(() => validateGraph(p, callee()));
    }
  }
  for (const value of [undefined, [], ['other/**']]) {
    const p = caller(); p.on.pull_request['branches-ignore'] = value;
    assert.throws(() => validateGraph(p, callee()));
  }
});
