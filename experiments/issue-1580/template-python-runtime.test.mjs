// Hosted template observations must execute instead of passing via runtime skips.
import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
import {selectFastTests} from '../../scripts/list-fast-pull-request-tests.mjs';
import {readTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';
import {durationLookup, planShards} from '../../scripts/lib/ci-speed-shards.mjs';

const root=fileURLToPath(new URL('../../',import.meta.url));
const observed='rust/tests/web/issue-1580-template-observations.test.mjs';
const source=fileURLToPath(new URL('./template-python-fragment-integrity.test.mjs',import.meta.url));

test('missing or old Python fails hosted observations while remaining optional locally',{timeout:30000},t=>{
  const directory=mkdtempSync(join(tmpdir(),'template-python-runtime-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  for(const version of [null,'3.12.9']){
    if(version)writeFileSync(join(directory,'python3'),`#!${process.execPath}\nprocess.stdout.write('Python ${version}\\n');\n`,{mode:0o755});
    for(const hosted of [false,true]){
      const env={...process.env,PATH:directory,GITHUB_ACTIONS:String(hosted)};
      delete env.NODE_TEST_CONTEXT;
      const result=spawnSync(process.execPath,['--max-old-space-size=64','--test','--test-reporter=tap',source],{
        cwd:root,encoding:'utf8',timeout:10000,maxBuffer:65536,
        env,
      });
      assert.equal(result.error,undefined,String(result.error));
      assert.equal(result.status,hosted?1:0,result.stdout+result.stderr);
      assert.match(result.stdout,hosted?/# skipped 0/u:/# skipped 4/u);
      if(hosted)assert.match(result.stdout,/Python 3[.]14\+ is required/u);
    }
  }
});

test('the selected JavaScript shard provisions Python before executing the pinned observations',{timeout:30000},t=>{
  const job=YAML.parse(readFileSync(join(root,'.github/workflows/layered-ci.yml'),'utf8')).jobs.js;
  const plan=job.steps.find(step=>step.id==='javascript-plan');
  assert.ok(plan,'plan the selected test files before runtime provisioning');
  const setup=job.steps.find(step=>step.uses==='actions/setup-python@v7');
  assert.ok(setup,'install a runtime that parses the unchanged upstream source');
  assert.equal(setup.with['python-version'],'3.15.2');
  assert.equal(setup.if,"steps.javascript-plan.outputs.python-required == 'true'");
  const run=job.steps.find(step=>step.name==='Run this shard of the JavaScript suites');
  assert.ok(job.steps.indexOf(plan)<job.steps.indexOf(setup));
  assert.ok(job.steps.indexOf(setup)<job.steps.indexOf(run));
  assert.match(run.run,/mapfile -t files < "\$RUNNER_TEMP\/javascript-shard[.]txt"/u);
  const directory=mkdtempSync(join(tmpdir(),'template-runtime-shards-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const durations=join(root,'data/meta/javascript-test-durations.lino');
  const fallback=Number(readFileSync(durations,'utf8').match(/default-seconds ([0-9.]+)/u)[1]);
  for(const shards of [2,8]){
    const files=shards===2
      ? selectFastTests(JSON.parse(readFileSync(join(root,'data/benchmarks/fast-pull-request-javascript-tests.json'),'utf8')))
      : readdirSync(join(root,'rust/tests/web')).filter(name=>name.endsWith('.test.mjs')).map(name=>'rust/tests/web/'+name);
    const selected=planShards(files,shards,durationLookup(readTestDurations(durations),fallback)).shards;
    const owners=selected.flatMap((files,index)=>files.includes(observed)?[index+1]:[]);
    assert.equal(owners.length,1,`${shards} shards execute the Python observations exactly once`);
    const other=selected.findIndex(files=>!files.includes(observed))+1;
    // Exercise both provisioning branches without launching the entire matrix.
    for(const shard of [owners[0],other]){
      const output=join(directory,'output');writeFileSync(output,'');
      const result=spawnSync('bash',['-c',plan.run],{cwd:root,encoding:'utf8',timeout:10000,maxBuffer:65536,
        env:{...process.env,RUNNER_TEMP:directory,GITHUB_OUTPUT:output,SHARD:String(shard),SHARDS:String(shards),FAST_PR_PROFILE:String(shards===2)}});
      assert.equal(result.error,undefined,String(result.error));
      assert.equal(result.status,0,result.stdout+result.stderr);
      const selected=readFileSync(join(directory,'javascript-shard.txt'),'utf8').trim().split('\n');
      const required=selected.includes(observed);
      assert.equal(readFileSync(output,'utf8'),`python-required=${required}\n`);
    }
  }
});
