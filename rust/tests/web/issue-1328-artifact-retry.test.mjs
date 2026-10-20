import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
const root = new URL('../../../', import.meta.url);
const action = YAML.parse(readFileSync(new URL('.github/actions/upload-artifact-with-retry/action.yml', root), 'utf8'));
const workflow = YAML.parse(readFileSync(new URL('.github/workflows/release-runtime-fixture.yml', root), 'utf8'));
const callee = YAML.parse(readFileSync(new URL('.github/workflows/release-runtime-fixture-callee.yml', root), 'utf8'));
test('artifact recovery preserves the pinned upload operation, original inputs and a finite enclosing budget', () => {
  const attempts = action.runs.steps.filter(step => step.uses);
  assert.equal(attempts.length, 3);
  for (const attempt of attempts) {
    assert.equal(attempt.uses, 'actions/upload-artifact@v7');
    assert.equal(attempt['timeout-minutes'], undefined, 'composite action metadata does not accept step timeouts');
    assert.deepEqual(attempt.with, attempts[0].with);
    assert.equal(attempt.with.overwrite, undefined, 'never replace an existing immutable artifact');
  }
  assert.equal(attempts[0]['continue-on-error'], true);
  assert.equal(attempts[1]['continue-on-error'], true);
  assert.notEqual(attempts[2]['continue-on-error'], true);
  assert.match(attempts[1].if, /first.outcome == 'failure'/);
  assert.match(attempts[2].if, /second.outcome == 'failure'/);
  assert.ok(action.runs.steps.some(step => /exit 1/u.test(step.run ?? '')));
  const producer = workflow.jobs['auto_compile-release'];
  assert.equal(producer.outputs.artifact_id, '${{ steps.upload.outputs.artifact-id }}');
  assert.equal(producer.steps.find(step => step.id === 'upload').uses, './.github/actions/upload-artifact-with-retry');
  assert.equal(producer.steps.find(step => step.id === 'upload')['timeout-minutes'],4);
  assert.ok(4 < producer['timeout-minutes']);
});
test('all public receipt outputs require a successful original action and have no fabricated ID fallback', () => {
  for (const [name, output] of Object.entries(action.outputs)) {
    const branches = output.value.slice(3, -2).trim().split(' || ');
    assert.equal(branches.length, 3);
    for (const [index, id] of ['first','second','third'].entries()) {
      assert.equal(branches[index], `(steps.${id}.outcome == 'success' && steps.${id}.outputs.${name})`);
    }
  }
});

test('the observed failing lease-start upload retries inside its original caller lease and deadline', () => {
  const lease = callee.jobs['lease-holder'];
  const witness = lease.steps.find(step => step.name === 'Expose live callee witness before observer finishes');
  assert.equal(witness.uses, './.github/actions/upload-artifact-with-retry');
  assert.equal(witness['timeout-minutes'], 1);
  assert.ok(witness['timeout-minutes'] < lease['timeout-minutes']);
  assert.deepEqual(witness.with, {name:'release-fixture-lease-start-${{ github.run_id }}-${{ github.run_attempt }}', path:'.release-fixture/lease-start.json', 'if-no-files-found':'error', 'retention-days':3});
  assert.equal(callee.concurrency, undefined);
  assert.equal(lease.concurrency, undefined);
  assert.equal(lease.if, 'inputs.active');
  assert(workflow.on.pull_request.paths.includes('.github/actions/upload-artifact-with-retry/**'));
});
