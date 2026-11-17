// Offline reproductions against unmodified, content-addressed upstream sources.
// The scheduler is a bounded model of the documented default success() rule,
// not evidence that a hosted workflow was dispatched.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync, mkdtempSync, mkdirSync, cpSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {Script} from 'node:vm';
import YAML from 'yaml';
import {evaluate as evaluateGate} from './fixtures/go-template/scripts/ci/check-pipeline-status.mjs';

const fixture=fileURLToPath(new URL('./fixtures/go-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}
const workflow=YAML.parse(readFileSync(join(fixture,'.github/workflows/release.yml'),'utf8'));

function manualResults(document,mode,overrides={}){
  const results={};
  const pending=new Set();
  const resultFor=id=>{
    if(Object.hasOwn(results,id))return results[id];
    assert.ok(!pending.has(id),'acyclic fixture graph');
    pending.add(id);
    const job=document.jobs[id];
    assert.ok(job,'known job '+id);
    const dependencies=typeof job.needs==='string'?[job.needs]:job.needs??[];
    const needs=Object.fromEntries(dependencies.map(name=>[name,{result:resultFor(name),outputs:{}}]));
    const success=()=>Object.values(needs).every(value=>value.result==='success');
    const source=String(job.if??'true').trim().replace(/^\$\{\{\s*|\s*\}\}$/gu,'');
    const explicitStatus=/\b(?:always|success|failure|cancelled)\s*\(/u.test(source);
    const condition=Boolean(new Script(source).runInNewContext({
      github:{event_name:'workflow_dispatch',ref:'refs/heads/main',event:{inputs:{release_mode:mode}}},
      needs,success,always:()=>true,cancelled:()=>false,failure:()=>!success(),
    },{timeout:100}));
    const admitted=condition&&(explicitStatus||success());
    results[id]=admitted?(overrides[id]??'success'):'skipped';
    pending.delete(id);
    return results[id];
  };
  for(const id of Object.keys(document.jobs))resultFor(id);
  return results;
}

test('Go manual release skips lint/build/release while the actual terminal gate accepts it',()=>{
  for(const mode of ['instant','changeset']){
    const results=manualResults(workflow,mode);
    for(const id of ['detect-changes','lint','build',`${mode}-release`])assert.equal(results[id],'skipped',id);
    assert.equal(results.test,'success','the explicit !cancelled() condition admits tests');
    assert.equal(results['pipeline-status'],'success');
    const gateNeeds=Object.fromEntries(workflow.jobs['pipeline-status'].needs.map(id=>[id,{result:results[id]}]));
    let output='';
    assert.equal(evaluateGate({needs:gateNeeds,stdout:{write:text=>output+=text},githubActions:false}),0);
    assert.equal(output,'Every required job succeeded or was skipped.\n');
  }
});

test('allowing manual change detection restores each selected release and still blocks failed prerequisites',()=>{
  const repaired=structuredClone(workflow);
  delete repaired.jobs['detect-changes'].if;
  for(const mode of ['instant','changeset']){
    const results=manualResults(repaired,mode);
    for(const id of ['detect-changes','lint','test','build',`${mode}-release`])assert.equal(results[id],'success',id);
    assert.equal(results[mode==='instant'?'changeset-release':'instant-release'],'skipped');
    for(const failed of ['lint','test','build']){
      const negative=manualResults(repaired,mode,{[failed]:'failure'});
      assert.equal(negative[`${mode}-release`],'skipped',failed+' must block publication');
      const needs=Object.fromEntries(repaired.jobs['pipeline-status'].needs.map(id=>[id,{result:negative[id]}]));
      assert.equal(evaluateGate({needs,jobs:()=>[],classify:()=>[],stdout:{write:()=>{}},githubActions:false}),1);
    }
  }
});

test('Go detector converts failed git into docs-only success; propagating errors restores fallback/failure', {timeout:15000},()=>{
  const directory=mkdtempSync(join(tmpdir(),'issue-1580-go-'));
  try{
    cpSync(join(fixture,'scripts'),join(directory,'scripts'),{recursive:true});
    const bin=join(directory,'bin');mkdirSync(bin);
    writeFileSync(join(bin,'git'),`#!${process.execPath}\n`+String.raw`
const command=process.argv[2];
const mode=process.env.FAKE_GIT_MODE;
if(command==='cat-file')process.exit(0);
if(command==='diff'){
  if(mode==='code'){process.stdout.write('pkg/sample.go\n');process.exit(0);}
  if(mode==='docs'){process.stdout.write('docs/guide.md\n');process.exit(0);}
  process.stderr.write('fatal: unavailable comparison commit\n');process.exit(128);
}
if(command==='ls-tree'&&mode==='first-commit'){process.stdout.write('pkg/sample.go\n');process.exit(0);}
process.stderr.write('fatal: unavailable repository\n');process.exit(128);
`,{mode:0o755});
    const script=join(directory,'scripts/detect-code-changes.mjs');
    const original=readFileSync(script,'utf8');
    const invoke=(mode,event='push')=>spawnSync(process.execPath,['--max-old-space-size=64',script],{
      cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:65536,
      env:{...process.env,PATH:bin+':'+process.env.PATH,GITHUB_EVENT_NAME:event,GITHUB_BASE_SHA:'a'.repeat(40),GITHUB_HEAD_SHA:'b'.repeat(40),GITHUB_OUTPUT:'',GITHUB_ACTIONS:'false',FAKE_GIT_MODE:mode},
    });
    for(const mode of ['missing-comparison','first-commit']){
      const before=invoke(mode);
      assert.equal(before.status,0,before.stderr);
      assert.match(before.stderr,/fatal: unavailable comparison commit/u);
      assert.match(before.stdout,/^any-code-changed=false$/mu);
      assert.match(before.stdout,/^go-changed=false$/mu);
    }
    const pullRequest=invoke('missing-comparison','pull_request');
    assert.equal(pullRequest.status,0,pullRequest.stderr);
    assert.match(pullRequest.stdout,/^any-code-changed=false$/mu);
    assert.match(pullRequest.stderr,/fatal: unavailable comparison commit/u);
    assert.match(invoke('code').stdout,/^any-code-changed=true$/mu);
    assert.match(invoke('docs').stdout,/^any-code-changed=false$/mu);
    assert.equal(original.split('return "";').length,2,'one swallowed error boundary');
    writeFileSync(script,original.replace('return "";','throw error;'));
    const missing=invoke('missing-comparison');
    assert.equal(missing.status,1,missing.stderr);
    assert.doesNotMatch(missing.stdout,/^any-code-changed=false$/mu,'unknown comparison cannot masquerade as docs-only');
    const missingPullRequest=invoke('missing-comparison','pull_request');
    assert.equal(missingPullRequest.status,1,missingPullRequest.stderr);
    assert.doesNotMatch(missingPullRequest.stdout,/^any-code-changed=false$/mu);
    const initial=invoke('first-commit');
    assert.equal(initial.status,0,initial.stderr);
    assert.match(initial.stdout,/^any-code-changed=true$/mu);
    assert.match(initial.stdout,/^go-changed=true$/mu);
    assert.match(invoke('code').stdout,/^any-code-changed=true$/mu);
    assert.match(invoke('docs').stdout,/^any-code-changed=false$/mu);
  }finally{rmSync(directory,{recursive:true,force:true});}
});

test('Go initial-commit detection loses real source files without a comparison parent', {timeout:15000},()=>{
  const directory=mkdtempSync(join(tmpdir(),'issue-1580-real-git-'));
  try{
    cpSync(join(fixture,'scripts'),join(directory,'scripts'),{recursive:true});
    const git=(...args)=>{
      const result=spawnSync('git',args,{cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:65536,
        env:{...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_AUTHOR_NAME:'Fixture',GIT_AUTHOR_EMAIL:'fixture@example.invalid',GIT_COMMITTER_NAME:'Fixture',GIT_COMMITTER_EMAIL:'fixture@example.invalid'}});
      assert.equal(result.status,0,result.stderr);
      return result.stdout;
    };
    git('init','--quiet');
    writeFileSync(join(directory,'main.go'),'package main\nfunc main() {}\n');
    git('add','main.go');
    git('-c','core.hooksPath=/dev/null','-c','commit.gpgsign=false','commit','--quiet','-m','initial fixture');
    const script=join(directory,'scripts/detect-code-changes.mjs');
    const invoke=()=>spawnSync(process.execPath,['--max-old-space-size=64',script],{
      cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:65536,
      env:{...process.env,GITHUB_EVENT_NAME:'push',GITHUB_BASE_SHA:'',GITHUB_HEAD_SHA:'',GITHUB_OUTPUT:'',GITHUB_ACTIONS:'false'},
    });
    const before=invoke();
    assert.equal(before.status,0,before.stderr);
    assert.match(before.stderr,/HEAD\^/u);
    assert.match(before.stdout,/^go-changed=false$/mu);
    const original=readFileSync(script,'utf8');
    writeFileSync(script,original.replace('return "";','throw error;'));
    const after=invoke();
    assert.equal(after.status,0,after.stderr);
    assert.match(after.stdout,/^go-changed=true$/mu);
    assert.match(after.stdout,/^any-code-changed=true$/mu);
  }finally{rmSync(directory,{recursive:true,force:true});}
});
