// Replay the real hosted selector and runner without compiling product sources.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,readdirSync,rmSync,writeFileSync} from 'node:fs';
import {join,relative} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {Script} from 'node:vm';
import YAML from 'yaml';
const read=file=>readFileSync(file,'utf8');
function evaluate(value,event,ref) {
  if(typeof value!=='string')return value;
  return new Script(value.replace(/^\$\{\{\s*|\s*\}\}$/gu,''))
    .runInNewContext({github:{event_name:event,ref},fromJSON:JSON.parse},{timeout:100});
}
test('PR, feature dispatch and main plan the identical complete default JavaScript suite',{timeout:30000},t=>{
  const job=YAML.parse(read('.github/workflows/layered-ci.yml')).jobs.js;
  const plan=job.steps.find(step=>step.id==='javascript-plan');
  const execute=job.steps.find(step=>step.name==='Run this shard of the JavaScript suites');
  const expected=readdirSync('rust/tests/web').filter(name=>name.endsWith('.test.mjs')).map(name=>'rust/tests/web/'+name).sort();
  assert(expected.includes('rust/tests/web/pull-request-1188-teach-f.test.mjs'));
  const directory=mkdtempSync(join(tmpdir(),'complete-javascript-plan-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  for(const [event,ref] of [['pull_request','refs/pull/1581/merge'],['workflow_dispatch','refs/heads/feature'],['push','refs/heads/main']]){
    const shards=Array.from(evaluate(job.strategy.matrix.shard,event,ref));
    assert.deepEqual(shards,[1,2,3,4,5,6,7,8]);
    assert.equal(evaluate(plan.env.SHARDS,event,ref),8);
    assert.equal(evaluate(execute.env.SHARDS,event,ref),8);
    assert.equal(plan.env.FAST_PR_PROFILE,undefined);
    assert.equal(execute.env.FAST_PR_PROFILE,undefined);
    const actual=[];
    for(const shard of shards){
      const output=join(directory,'outputs');writeFileSync(output,'');
      const result=spawnSync('bash',['-c',plan.run],{encoding:'utf8',timeout:10000,maxBuffer:65536,
        env:{...process.env,RUNNER_TEMP:directory,GITHUB_OUTPUT:output,SHARD:String(shard),SHARDS:'8'}});
      assert.equal(result.error,undefined,String(result.error));
      assert.equal(result.status,0,result.stdout+result.stderr);
      const selected=read(join(directory,'javascript-shard.txt')).trim().split('\n');
      actual.push(...selected);
      assert.equal(read(output),`python-required=${selected.includes('rust/tests/web/issue-1580-template-observations.test.mjs')}\n`);
    }
    assert.deepEqual(actual.sort(),expected,event+' must neither omit nor duplicate suites');
  }
});
test('the shared ordered runner reports two failures and still finishes the later passing file',{timeout:15000},t=>{
  const directory=mkdtempSync(join(tmpdir(),'javascript-continue-failures-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const files=['first-failure','second-failure','later-success'].map((name,index)=>{
    const file=join(directory,name+'.test.mjs');
    writeFileSync(file,`import test from 'node:test';\ntest('${name}',()=>{${index<2?`throw new Error('${name}-diagnostic');`:''}});\n`);
    return file;
  });
  const env={...process.env};delete env.NODE_TEST_CONTEXT;
  const result=spawnSync(process.execPath,['--max-old-space-size=64','scripts/run-ordered-javascript-tests.mjs','--concurrency','1',...files],{
    encoding:'utf8',timeout:10000,maxBuffer:65536,env});
  assert.equal(result.error,undefined,String(result.error));
  assert.equal(result.status,1,result.stdout+result.stderr);
  for(const [index,file] of files.entries()){
    const prefix='[test-file] '+(index<2?'failed ':'completed ')+relative(process.cwd(),file)+' ';
    const rows=result.stderr.split('\n').filter(line=>line.startsWith(prefix));
    assert.equal(rows.length,1,result.stderr);
    assert(Number.isFinite(JSON.parse(rows[0].slice(prefix.length)).duration_ms));
  }
  for(const name of ['first-failure-diagnostic','second-failure-diagnostic','later-success'])assert(result.stdout.includes(name),result.stdout);
});
