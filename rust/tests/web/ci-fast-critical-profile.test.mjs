import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';

const read = file => YAML.parse(fs.readFileSync(file, 'utf8'));
const coverage = read('.github/workflows/coverage.yml');
const release = read('.github/workflows/release.yml');
const agent = read('.github/workflows/agent-cli-e2e.yml');
function evaluate(expression, context) {
  const source = expression.trim().replace(/^\$\{\{\s*/u, '').replace(/\s*\}\}$/u, '')
    .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu, (_, key) => '[' + JSON.stringify(key) + ']');
  return new Script(source).runInNewContext({cancelled: () => false, ...context}, {timeout: 100});
}
function coverageContext(enabled, event = 'workflow_dispatch') {
  return {
    github: {event_name: event},
    needs: {
      'coverage-policy': {outputs: {run: String(enabled)}},
      'detect-changes': {outputs: {'rs-changed': 'true', 'toml-changed': 'true', 'any-code-changed': 'true', 'workflow-changed': 'true'}},
      'coverage-build': {result: 'success'},
    },
  };
}

test('default coverage profile declares deferral without awarding coverage credit', () => {
  const step = coverage.jobs['coverage-policy'].steps[0];
  assert.equal(coverage.on.workflow_dispatch.inputs['full-coverage'].default, false);
  assert.equal(evaluate(step.env.FULL_COVERAGE, {inputs: {}, vars: {}}), false);
  assert.match(step.run, /No coverage measurement or coverage credit is claimed/u);
  for (const job of ['coverage-build', 'coverage', 'browser-coverage-shard', 'browser-coverage']) {
    assert.equal(evaluate(coverage.jobs[job].if, coverageContext(false)), false, job);
  }
});

test('both explicit restore controls select the complete unchanged coverage plans', () => {
  const expression = coverage.jobs['coverage-policy'].steps[0].env.FULL_COVERAGE;
  assert.equal(evaluate(expression, {inputs: {'full-coverage': true}, vars: {}}), true);
  assert.equal(evaluate(expression, {inputs: {}, vars: {FORMAL_AI_FULL_COVERAGE: 'run'}}), true);
  for (const job of ['coverage-build', 'coverage', 'browser-coverage-shard', 'browser-coverage']) {
    assert.equal(evaluate(coverage.jobs[job].if, coverageContext(true)), true, job);
  }
  assert.equal(coverage.jobs['coverage-shard'].strategy.matrix.shard.length, 32);
  assert.equal(coverage.jobs['browser-coverage-shard'].strategy.matrix.shard.length, 16);
  for (const [job,dependency] of [['coverage','coverage-build coverage-shard'], ['browser-coverage','browser-coverage-shard']]) {
    assert(coverage.jobs[job].steps.some(step => step.env?.JOBS === dependency && step.run?.includes('check-shard-results.sh')));
    assert(coverage.jobs[job].steps.some(step => step.run?.includes('check-coverage-ratchet.rs')));
  }
  const context = coverageContext(true);
  for (const result of ['failure','cancelled','skipped']) {
    context.needs['coverage-build'].result = result;
    assert.equal(evaluate(coverage.jobs['coverage-shard'].if, context), false);
  }
});

test('failed artifact producers cannot start test, project or image consumers', () => {
  for (const id of ['test','test-agent-cli-e2e','box-language-projects','docker-build','docker-slim']) {
    const context = {
      github: {event_name: 'pull_request'},
      needs: {'detect-changes': {outputs: {'any-code-changed': 'true'}}, 'build-artifacts': {result: 'success'}},
    };
    assert.equal(evaluate(release.jobs[id].if, context), true, id);
    for (const result of ['failure','cancelled','skipped']) {
      context.needs['build-artifacts'].result = result;
      assert.equal(evaluate(release.jobs[id].if, context), false, id + ' / ' + result);
    }
  }
});

test('ordinary Linux test inventory and critical agent coding/reporting remain required', () => {
  const lanes = release.jobs.test.strategy.matrix.include;
  assert.equal(lanes.filter(lane => lane['test-suite'] === 'full').length, 8);
  assert.equal(lanes.filter(lane => lane['test-suite'] === 'specification').length, 4);
  for (const name of [
    'literal payload containing an edit phrase', 'file-creation routes to write not read',
    'existing issue reference does not file a duplicate', 'generated programs use harness commands',
    'compile and execute a resumed substitution export', 'report action executes gh',
    'filed report stays readable and bounded', 'confirmed complete-context reports',
    'report multiselect and issue body', 'auto-learn client execution architecture',
    'ordinary output project with CI and verification', 'Replay Hive Mind self-coding inner loop',
  ]) {
    const step = agent.jobs['agent-cli-e2e'].steps.find(item => item.name?.includes(name));
    assert(step, name);
    assert(!step.if.includes('inputs.full-replay'), name + ' must run without recorded replay');
  }
  assert.match(release.jobs['test-agent-cli-e2e'].with['full-replay'], /FORMAL_AI_FULL_AGENT_REPLAY/u);
  assert.match(release.jobs['test-agent-cli-e2e'].with['full-replay'], /agentic-routing-changed/u);
});

test('dependency validation dispatch is read-only unless creation is explicit', () => {
  const workflow = read('.github/workflows/dependencies-latest.yml');
  assert.equal(workflow.on.workflow_dispatch.inputs['create-pull-request'].default, false);
  const step = workflow.jobs.update.steps.find(item => item.name === 'Open one pull request with everything that moved');
  for (const [event, requested, expected] of [['workflow_dispatch', false, false], ['workflow_dispatch', true, true], ['schedule', false, true]]) {
    assert.equal(evaluate(step.if, {github: {event_name: event}, inputs: {'create-pull-request': requested}}), expected);
  }
});

test('automatic real-client admission ignores feature branches while retaining main readiness tails', () => {
  const workflow = read('.github/workflows/hive-original-delivery.yml');
  assert.deepEqual(workflow.on.workflow_run.branches, ['main']);
  assert(workflow.on.workflow_dispatch.inputs['pull-request']);
  const expression = workflow.jobs.admission.if;
  assert.equal(evaluate(expression, {github: {event_name: 'workflow_dispatch'}}), true);
  for (const [event, branch, name, conclusion, expected] of [
    ['push','main','Coverage','success',true],
    ['workflow_run','main','Verify Public Release Downloads','success',true],
    ['pull_request','feature','Coverage','success',false],
    ['schedule','main','dependencies-latest','success',false],
    ['push','main','Coverage','failure',false],
    ['workflow_run','main','Desktop Release','success',false],
  ]) {
    assert.equal(evaluate(expression, {github: {event_name:'workflow_run',event:{workflow_run:{event,head_branch:branch,name,conclusion}}}}), expected);
  }
});
