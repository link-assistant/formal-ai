import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
const workflow = YAML.parse(readFileSync(new URL('../../../.github/workflows/self-authored-pull-request.yml', import.meta.url), 'utf8'));
const action = YAML.parse(readFileSync(new URL('../../../.github/actions/author-with-formal-ai/action.yml', import.meta.url), 'utf8'));

test('release routes use only published or completed declared main producers', () => {
  assert.deepEqual(workflow.on.release, {types: ['published']});
  assert.equal(workflow.on.workflow_run, undefined);
  assert.deepEqual(workflow.on.workflow_call.inputs, {'production-run-id': {type: 'string', required: true}, 'production-run-attempt': {type: 'string', required: true}});
  assert.deepEqual(workflow.on.workflow_call.secrets, Object.fromEntries(['AUTOMATION_APP_ID', 'AUTOMATION_APP_PRIVATE_KEY', 'AUTOMATION_TOKEN'].map(name => [name, {required: false}])));
  assert.deepEqual(workflow.permissions, {});
  assert.equal(workflow.jobs.author['timeout-minutes'], 30);
  assert.deepEqual(workflow.jobs.author.permissions, {contents: 'write', 'pull-requests': 'write', issues: 'write', actions: 'write'});
  assert.deepEqual(workflow.concurrency, {group: 'self-authored-pull-request', 'cancel-in-progress': false});
  assert.deepEqual(workflow.jobs.author.concurrency, {group: 'formal-ai-repository-writes', queue: 'max'});
});

test('release context precedes build and both authoring and cache require active classification', () => {
  const steps = workflow.jobs.author.steps;
  const context = steps.find(step => step.id === 'release-context');
  const author = steps.find(step => step.id === 'author');
  const cache = steps.find(step => step.uses === './.github/actions/cache-cargo-registry');
  const gate = "github.event_name != 'release' && inputs.production-run-id == '' || steps.release-context.outputs.active == 'true'";
  assert.ok(steps.indexOf(context) < steps.indexOf(cache));
  assert.ok(steps.indexOf(context) < steps.indexOf(author));
  assert.equal(author.if, gate); assert.equal(cache.if, gate);
  assert.equal(context.run, 'node scripts/resolve-self-authored-release.mjs');
  assert.equal(context.env.CURRENT_RUN_ATTEMPT, '${{ github.run_attempt }}');
  assert.equal(context.env.PRODUCER_RUN_ATTEMPT, '${{ inputs.production-run-attempt }}');
  assert.equal(context.env.CURRENT_WORKFLOW_REF, '${{ github.workflow_ref }}');
  assert.equal(context.env.CURRENT_RUN_ID, '${{ github.run_id }}');
  assert.equal(author.with['base-branch'], '${{ steps.release-context.outputs.branch }}');
  assert.equal(author.with['starting-ref'], '${{ steps.release-context.outputs.branch }}');
  assert.equal(author.with['formal-ai-source'], 'source');
});

test('optional restoration conserves ordinary event fallback and never creates a release', () => {
  assert.equal(action.inputs['starting-ref'].default, '');
  assert.equal(action.inputs['starting-ref'].required, false);
  const restore = action.runs.steps.find(step => step.name === 'Return to the branch the run checked out');
  assert.equal(restore.env.STARTING_REF, '${{ inputs.starting-ref || github.head_ref || github.ref_name }}');
  assert.equal(restore.if, 'always()');
  assert.equal(workflow.jobs.author.if, "github.event_name != 'issues' || github.event.label.name == 'formal-ai-solve'");
  assert.equal(workflow.jobs.author.steps.find(step => step.name === "Dispatch the bot pull request's checks").if,
    "steps.automation.outputs.layer == 'default' && steps.author.outputs.authored == 'true'");
  assert.ok(workflow.jobs.author.steps.every(step => !String(step.run ?? '').includes('gh release')));
});
