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
  return new Script(source).runInNewContext({cancelled: () => false, vars: {}, inputs: {}, ...context}, {timeout: 100});
}
function coverageContext(enabled, event = 'workflow_dispatch') {
  return {
    github: {event_name: event, ref:'refs/heads/main'},
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
  for (const id of ['test','test-agent-cli-e2e','box-language-projects']) {
    const context = {
      github: {event_name: 'push', ref:'refs/heads/main'}, vars: {FORMAL_AI_FULL_INTEGRATION:'run'},
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
    for (const publish of ['true', 'false']) {
      assert.equal(evaluate(step.if, {
        github: {event_name: event}, inputs: {'create-pull-request': requested},
        steps: {publication: {outputs: {publish}}},
      }), expected && publish === 'true');
    }
  }
});

test('automatic real-client admission ignores feature branches while retaining main readiness tails', () => {
  const workflow = read('.github/workflows/hive-original-delivery.yml');
  assert.deepEqual(workflow.on.workflow_run.branches, ['main']);
  assert(workflow.on.workflow_dispatch.inputs['pull-request']);
  const expression = workflow.jobs.admission.if;
  assert.equal(evaluate(expression, {github: {event_name: 'workflow_dispatch'}, vars: {}, inputs: {'real-github-e2e': true}}), true);
  for (const [event, branch, name, conclusion, expected] of [
    ['push','main','Coverage','success',true],
    ['workflow_run','main','Verify Public Release Downloads','success',true],
    ['pull_request','feature','Coverage','success',false],
    ['schedule','main','dependencies-latest','success',false],
    ['push','main','Coverage','failure',false],
    ['workflow_run','main','Desktop Release','success',false],
  ]) {
    assert.equal(evaluate(expression, {vars: {FORMAL_AI_REAL_GITHUB_E2E: 'true'}, github: {event_name:'workflow_run',event:{workflow_run:{event,head_branch:branch,name,conclusion}}}}), expected);
  }
});


test('corpus verdict preserves every shard obligation with bounded script-only acquisition', async () => {
  const workflow=read('.github/workflows/benchmark-corpus-gate.yml'),job=workflow.jobs['corpus-gate'];
  const {createHash}=await import('node:crypto');
  const digest=value=>createHash('sha256').update(value).digest('hex');
  assert.deepEqual(job.needs,['corpus-build','corpus-shard']);
  assert.equal(job.if,"${{ (github.event_name != 'pull_request' && github.event_name != 'pull_request_target' && (github.ref == 'refs/heads/main' || github.event_name == 'release' || (github.event_name == 'workflow_run' && github.event.workflow_run.head_branch == 'main'))) && (!cancelled()) }}");
  assert.equal(job['timeout-minutes'],5);
  assert.equal(job.steps.length,2);
  const checkout=job.steps[0],verdict=job.steps[1];
  assert.equal(checkout.uses,'actions/checkout@v7');assert.equal(checkout['timeout-minutes'],2);
  assert.equal(checkout.with['persist-credentials'],false);assert.equal(checkout.with['fetch-depth'],1);
  assert.equal(checkout.with.filter,'blob:none');assert.equal(checkout.with['sparse-checkout-cone-mode'],false);
  assert.equal(Object.hasOwn(checkout.with,'ref'),false,'original event-source checkout selection remains unchanged');
  assert.deepEqual(checkout.with['sparse-checkout'].trim().split(/\r?\n/u),['/scripts/check-shard-results.sh']);
  assert.deepEqual(checkout.env,{GIT_CONFIG_COUNT:2,GIT_CONFIG_KEY_0:'http.lowSpeedLimit',GIT_CONFIG_VALUE_0:1024,GIT_CONFIG_KEY_1:'http.lowSpeedTime',GIT_CONFIG_VALUE_1:20});
  assert.equal(verdict.name,'Every corpus shard answered its prompts');assert.equal(verdict['timeout-minutes'],1);
  assert.equal(verdict.run,'bash scripts/check-shard-results.sh');
  assert.deepEqual(verdict.env,{"NEEDS_JSON":"${{ toJSON(needs) }}","JOBS":"corpus-build corpus-shard","CHECK":"Benchmark corpus gate"});
  // Source pins preserve the original actual quorum/refusal validator and the full build/shard gates.
  assert.equal(digest(fs.readFileSync('scripts/check-shard-results.sh')),"80c490a21334417bc9e2835761b8cf37e89a2c422307ac2a4bf6da86a5ddb168");
  assert.equal(digest(JSON.stringify(workflow.jobs['corpus-build'])),"c9bfdd565b02f95162e23624c78baec6c1a567c2f4851047c27a86f0881adf5d");
  assert.equal(digest(JSON.stringify(workflow.jobs['corpus-shard'])),"d158657d57b95d845031893a4258b3f7ce2220031616c99583b07e57d28afdf2");
  const {execFileSync}=await import('node:child_process'),{tmpdir}=await import('node:os'),{join}=await import('node:path');
  const directory=fs.mkdtempSync(join(tmpdir(),'corpus-verdict-selection-'));
  try{
    const env={...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_TERMINAL_PROMPT:'0'};
    execFileSync('git',['init','-q',directory],{env,timeout:5000,stdio:['ignore','pipe','pipe']});
    fs.mkdirSync(join(directory,'.git/info'),{recursive:true});
    fs.writeFileSync(join(directory,'.git/info/sparse-checkout'),checkout.with['sparse-checkout']);
    const names=['scripts/check-shard-results.sh','scripts/other.sh','package.json','rust/tests/corpus.rs','data/benchmarks/prompts.json','.github/workflows/benchmark-corpus-gate.yml','archive/payload.gz'];
    const selected=execFileSync('git',['sparse-checkout','check-rules','--no-cone','--rules-file',join(directory,'.git/info/sparse-checkout'),'-z'],{cwd:directory,env,input:Buffer.from(names.join('\0')+'\0'),timeout:5000,maxBuffer:1024*1024}).toString().split('\0').filter(Boolean);
    assert.deepEqual(selected,['scripts/check-shard-results.sh']);
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
