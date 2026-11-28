// Actual Git and output-file controls; no compiler or product build.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, writeFileSync, readFileSync, mkdirSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {execFileSync, spawnSync} from 'node:child_process';
import {Script} from 'node:vm';
import YAML from 'yaml';
const script = resolve('scripts/confirm-javascript-candidate.mjs');
const workflow = YAML.parse(readFileSync('.github/workflows/release.yml','utf8'));

test('candidate receipt requires the actual clean event commit and a writable output', {timeout:15000}, t => {
  const directory=mkdtempSync(join(tmpdir(),'formal-javascript-candidate-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const repo=join(directory,'repo');mkdirSync(repo);
  const git=args=>execFileSync('git',args,{cwd:repo,encoding:'utf8'});
  git(['init','--quiet']);writeFileSync(join(repo,'source.txt'),'before\n');git(['add','source.txt']);
  git(['-c','user.name=Candidate Test','-c','user.email=candidate@example.invalid','commit','--quiet','-m','fixture']);
  const sha=git(['rev-parse','HEAD']).trim();
  const output=join(directory,'output');
  const run=(expected=sha,target=output)=>spawnSync(process.execPath,[script],{cwd:repo,
    encoding:'utf8',env:{...process.env,GITHUB_SHA:expected,GITHUB_OUTPUT:target}});
  const refused=(expected,target)=>{
    writeFileSync(output,'prior=retained\n');const result=run(expected,target);
    assert.notEqual(result.status,0,result.stdout+result.stderr);
    assert.equal(result.signal,null);assert.equal(readFileSync(output,'utf8'),'prior=retained\n');
    assert.doesNotMatch(result.stdout,/JAVASCRIPT_CANDIDATE_CONFIRMED/u);
  };
  for(const expected of ['', 'main', sha.slice(0,12), sha+'\ntested-sha=forged', 'f'.repeat(40)])refused(expected,output);
  writeFileSync(join(repo,'source.txt'),'modified\n');refused(sha,output);
  git(['add','source.txt']);refused(sha,output);git(['reset','--hard','--quiet','HEAD']);
  writeFileSync(join(repo,'extra.js'),'untracked\n');refused(sha,output);rmSync(join(repo,'extra.js'));
  refused(sha,'');refused(sha,directory);
  writeFileSync(output,'prior=retained\n');const accepted=run();
  assert.equal(accepted.status,0,accepted.stderr);
  assert.equal(readFileSync(output,'utf8'),'prior=retained\ntested-sha='+sha+'\n');
  assert.equal(accepted.stdout,'JAVASCRIPT_CANDIDATE_CONFIRMED '+sha+'\n');
});

test('native jobs require successful JavaScript checks for the same event commit', () => {
  const source=workflow.jobs['fast-pr-lint'], native=workflow.jobs['native-type-check'];
  assert.deepEqual(native.needs,['detect-changes','fast-pr-lint']);
  assert.equal(source.outputs['tested-sha'],'${{ steps.candidate.outputs.tested-sha }}');
  assert.deepEqual(source.steps.at(-1),{name:'Confirm checked JavaScript candidate',id:'candidate',
    run:'node scripts/confirm-javascript-candidate.mjs'});
  assert(source.steps.slice(0,-1).some(step=>step.run?.includes('node scripts/check-fast-governance.mjs')));
  const expression=native.if.trim().replace(/^\$\{\{\s*|\s*\}\}$/gu,'')
    .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu,(_,key)=>'['+JSON.stringify(key)+']');
  const sha='a'.repeat(40);
  const selected=({event='pull_request',ref='refs/pull/1/merge',result='success',tested=sha,cancelled=false}={})=>
    Boolean(new Script(expression).runInNewContext({cancelled:()=>cancelled,
      github:{event_name:event,ref,sha},needs:{'fast-pr-lint':{result,outputs:{'tested-sha':tested}}}}, {timeout:100}));
  assert.equal(selected(),true);
  assert.equal(selected({event:'workflow_dispatch',ref:'refs/heads/fix/candidate'}),true);
  for(const result of ['failure','cancelled','skipped','queued','',undefined])
    assert.equal(selected({result:result??'unknown'}),false,String(result));
  for(const tested of ['', 'b'.repeat(40),sha.slice(0,12),null])assert.equal(selected({tested}),false,String(tested));
  assert.equal(selected({cancelled:true}),false);
  assert.equal(selected({event:'push',ref:'refs/heads/main'}),false);
  assert.equal(selected({event:'workflow_dispatch',ref:'refs/heads/main'}),false);
});
