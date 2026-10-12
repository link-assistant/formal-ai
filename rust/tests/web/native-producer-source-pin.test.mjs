import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,rmSync} from 'node:fs';
import {createRequire} from 'node:module';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
const require=createRequire(import.meta.url);
const {parse}=require('yaml');
const workflow=parse(readFileSync(new URL('../../../.github/workflows/native-response-observations.yml',import.meta.url),'utf8'));
const steps=workflow.jobs.observe.steps;
test('native response checkout is bound to the actual caller commit',()=>{
  assert.equal(workflow.permissions.contents,'read');
  assert.equal(steps[0].env.SOURCE_COMMIT,'${{ inputs.source-commit }}');
  assert.equal(steps[0].env.WORKFLOW_COMMIT,'${{ github.sha }}');
  assert.equal(steps[1].with.ref,'${{ github.sha }}');
  assert.equal(steps[1].with['persist-credentials'],false);
  assert.equal(steps[2].env.SOURCE_COMMIT,'${{ inputs.source-commit }}');
  assert.ok(steps[2].run.includes('git rev-parse HEAD'));
});
test('actual producer source precheck rejects a foreign or malformed commit before checkout',()=>{
  const directory=mkdtempSync(join(tmpdir(),'native-producer-source-pin-'));
  try {
    const commit='a'.repeat(40);
    const cases=[
      [commit,commit,true],
      ['b'.repeat(40),commit,false],
      ['',commit,false],
      ['a'.repeat(39),commit,false],
      ['A'.repeat(40),'A'.repeat(40),false],
      [commit+';true',commit,false],
      [commit,'',false],
    ];
    for(const [source,caller,success] of cases){
      const receipt=spawnSync('bash',['-c',steps[0].run],{
        cwd:directory,
        env:{...process.env,SOURCE_COMMIT:source,WORKFLOW_COMMIT:caller},
        encoding:'utf8',
      });
      assert.equal(receipt.status===0,success,JSON.stringify({source,caller,status:receipt.status}));
      assert.equal(receipt.signal,null);
    }
  } finally {rmSync(directory,{recursive:true,force:true});}
});
test('source authority does not alter native capture operands or budgets',()=>{
  assert.equal(workflow.jobs.observe['timeout-minutes'],30);
  const installation=steps.find(step=>step.name==='Install actual locked native fixture dependencies');
  assert.equal(installation.run,'bun install --frozen-lockfile --ignore-scripts');
  const capture=steps.find(step=>step.name==='Observe unchanged source-selected native test programs');
  assert.ok(capture.run.includes('scripts/capture-native-responses.mjs dist/tests dist/formal-ai'));
  assert.ok(capture.run.endsWith('data/meta/native-response-capture-cases.json release all'));
});
