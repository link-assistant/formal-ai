// Execute the actual detector shell with job-local fake runtimes; never build Rust.
import assert from 'node:assert/strict';
import test from 'node:test';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {Script} from 'node:vm';
import YAML from 'yaml';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'../..');
const jobs=[
  ['coverage.yml','detect-changes'],
  ['release.yml','detect-changes'],
  ['layered-ci.yml','changes']
].map(([file,id])=>({file,id,job:YAML.parse(readFileSync(join(root,'.github/workflows',file),'utf8')).jobs[id]}));
const nativeInstall=step=>step.run==='bash scripts/install-rust-script.sh';
const detector=step=>step.run?.includes('node scripts/ci-pull-request-admission.mjs detect');
const guard=(value,event,ref)=>value===undefined?true:Boolean(new Script(value)
  .runInNewContext({github:{event_name:event,ref}},{timeout:100}));
function run(job,event,ref){
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-job-tools-'));
  try{
    const bin=join(directory,'bin'),scripts=join(directory,'scripts'),trace=join(directory,'trace');
    mkdirSync(bin);mkdirSync(scripts);writeFileSync(trace,'');
    const put=(path,text)=>writeFileSync(path,text,{mode:0o755});
    put(join(bin,'bash'),'#!/bin/sh\nexec /bin/bash "$@"\n');
    put(join(bin,'node'),'#!/bin/sh\nprintf "node %s\\n" "$*" >> "$PROBE_TRACE"\n');
    // The installer only exposes a fake runtime inside this fresh job sandbox.
    put(join(scripts,'install-rust-script.sh'),[
      'set -eu',
      'printf "install\\n" >> "$PROBE_TRACE"',
      "printf '%s\\n' '#!/bin/sh' 'printf \"rust-script %s\\\\n\" \"$*\" >> \"$PROBE_TRACE\"' > \"$PROBE_BIN/rust-script\"",
      '/bin/chmod +x "$PROBE_BIN/rust-script"',''
    ].join('\n'));
    let result;
    for(const step of job.steps){
      if(!step.run || !(nativeInstall(step)||detector(step)||step.run.includes('rust-script scripts/detect-code-changes-tests.rs')))continue;
      if(!guard(step.if,event,ref))continue;
      result=spawnSync('/bin/bash',['--noprofile','--norc','-e','-o','pipefail','-c',step.run],{
        cwd:directory,encoding:'utf8',timeout:2000,maxBuffer:64*1024,
        env:{PATH:bin,PROBE_BIN:bin,PROBE_TRACE:trace,GITHUB_EVENT_NAME:event,GITHUB_REF:ref}
      });
      if(result.error || result.status!==0)break;
    }
    return {status:result?.status,error:result?.error,stderr:result?.stderr,trace:readFileSync(trace,'utf8').trim().split('\n').filter(Boolean)};
  }finally{rmSync(directory,{recursive:true,force:true});}
}
test('detector event guards install each native runtime in its own job before shell use',{timeout:15000},()=>{
  const cases=[
    ['pull_request','refs/pull/1581/merge',false],
    ['pull_request','refs/heads/main',false],
    ['workflow_dispatch','refs/heads/branch',false],
    ['workflow_dispatch','refs/heads/main',true],
    ['push','refs/heads/main',true],
    ['push','refs/tags/v1.2.3',true]
  ];
  for(const {file,job}of jobs)for(const [event,ref,native]of cases){
    const result=run(job,event,ref),label=file+' '+event+' '+ref;
    assert.equal(result.error,undefined,label);
    assert.equal(result.status,0,label+': '+result.stderr);
    assert.equal(result.trace.at(-1),native?'rust-script scripts/detect-code-changes.rs':'node scripts/ci-pull-request-admission.mjs detect',label);
    assert.equal(result.trace.includes('install'),native,label);
  }
});
test('actual shell rejects missing, disabled and late native installation independently in every job',{timeout:15000},()=>{
  for(const {file,job}of jobs){
    for(const mutation of ['missing','disabled','late']){
      const draft=structuredClone(job),index=draft.steps.findIndex(nativeInstall);
      assert(index>=0,file);
      if(mutation==='missing')draft.steps.splice(index,1);
      if(mutation==='disabled')draft.steps[index].if='false';
      if(mutation==='late')draft.steps.push(...draft.steps.splice(index,1));
      const result=run(draft,'push','refs/heads/main');
      assert.equal(result.error,undefined,file+' '+mutation);
      assert.equal(result.status,127,file+' '+mutation+': '+result.stderr);
      assert.match(result.stderr,/rust-script: command not found/u);
    }
  }
});
