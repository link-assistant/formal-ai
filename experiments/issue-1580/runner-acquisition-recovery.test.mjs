import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,writeFileSync,chmodSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {recover,MAX_ATTEMPTS} from '../../scripts/rerun-unacquired-jobs.mjs';
const options={repository:'owner/repo',runId:9,attempt:1,defaultBranch:'main'};
const message='The job was not acquired by Runner of type hosted even after multiple attempts';
const missing={id:1,run_id:9,name:'Native contracts',status:'completed',conclusion:'cancelled',
  steps:[],runner_name:'',check_run_url:'https://api.github.com/repos/owner/repo/check-runs/101'};
const gate={...missing,id:2,name:'Pipeline Status',runner_name:'hosted',steps:[{name:'Status',conclusion:'failure'}]};
const passed={...gate,id:3,name:'Lint',conclusion:'success'};
const run={id:9,run_attempt:1,workflow_id:42,name:'CI/CD Pipeline',status:'completed',conclusion:'failure',
  head_repository:{full_name:'owner/repo'},repository:{full_name:'owner/repo'},
  head_branch:'main',event:'push',head_sha:'a'.repeat(40)};
function fixture(change={}) {
  const calls=[],writes=[];
  const state={run:structuredClone(run),jobs:[missing,gate,passed],annotations:[[{message:'unrelated'}],[{message}]],
    latest:{workflow_runs:[{id:9,head_sha:run.head_sha}]},...change};
  const api={read:(endpoint,paginated=false)=>{
    calls.push({endpoint,paginated});
    if(change.fail?.(endpoint))throw new Error('API unavailable');
    if(endpoint.includes('/check-runs/108/'))return [[]];
    if(endpoint.includes('/annotations?'))return state.annotations;
    if(endpoint.includes('/jobs?'))return [{jobs:state.jobs.slice(0,1)},{jobs:state.jobs.slice(1)}];
    if(endpoint.includes('/workflows/'))return state.latest;
    const reads=calls.filter(call=>call.endpoint==='repos/owner/repo/actions/runs/9').length;
    return reads>1&&state.secondRun?state.secondRun:state.run;
  },rerun:value=>writes.push(value)};
  return {state,calls,writes,api};
}
test('production recovery reads all job and annotation pages and reruns only failed jobs',()=>{
  const f=fixture();assert.equal(recover(options,f.api).action,'rerun');
  assert.deepEqual(f.writes,[{repository:'owner/repo',runId:9}]);
  for(const call of f.calls.filter(call=>/\/(jobs|annotations)\?/u.test(call.endpoint)))assert.equal(call.paginated,true);
  assert(f.calls.some(call=>call.endpoint.includes('/check-runs/101/annotations?')),
    'use check-run URL identity rather than guessing from job ID');
  assert.equal(f.calls.filter(call=>call.endpoint==='repos/owner/repo/actions/runs/9').length,2);
});
for(const [name,fields]of [
  ['actual test failure',{conclusion:'failure',steps:[{name:'Test'}]}],
  ['assigned runner',{runner_name:'hosted'}],
  ['ordinary cancellation',{}],
  ['unstarted unrelated failed job',{conclusion:'failure'}]
])test('recovery preserves '+name,()=>{
  const job={...missing,id:8,check_run_url:'https://api.github.com/repos/owner/repo/check-runs/108',...fields};
  const f=fixture({jobs:[missing,job]});
  assert.equal(recover(options,f.api).action,'blocked');assert.deepEqual(f.writes,[]);
});
test('absence of the acquisition annotation never schedules a rerun',()=>{
  const f=fixture({annotations:[[{message:'Cancelled by a newer commit'}]]});
  assert.equal(recover(options,f.api).action,'nothing');assert.deepEqual(f.writes,[]);
});
for(const attempt of [2,3,4])test('recovery attempt '+attempt+' is bounded',()=>{
  const f=fixture({run:{...run,run_attempt:attempt}});
  assert.equal(recover({...options,attempt},f.api).action,attempt<3?'rerun':'give-up');
  assert.equal(f.writes.length,attempt<3?1:0);assert.equal(MAX_ATTEMPTS,3);
});
test('superseded workflow source remains untouched',()=>{
  for(const latest of [{workflow_runs:[{id:10,head_sha:run.head_sha}]},
    {workflow_runs:[{id:9,head_sha:'b'.repeat(40)}]}]) {
    const f=fixture({latest});assert.equal(recover(options,f.api).action,'superseded');assert.deepEqual(f.writes,[]);
  }
});
test('dry run inspects real decisions without writing',()=>{
  const f=fixture();assert.equal(recover({...options,dryRun:true},f.api).action,'dry-run');assert.deepEqual(f.writes,[]);
});
for(const [name,change]of [
  ['fork',{run:{...run,head_repository:{full_name:'fork/repo'}}}],
  ['branch',{run:{...run,head_branch:'feature'}}],
  ['PR event',{run:{...run,event:'pull_request'}}],
  ['unrelated workflow',{run:{...run,name:'Other'}}],
  ['new attempt',{secondRun:{...run,run_attempt:2}}],
  ['API failure',{fail:()=>true}],
  ['annotation failure',{fail:endpoint=>endpoint.includes('/annotations?')}],
  ['newest observation failure',{latest:{workflow_runs:[]}}],
  ['incomplete jobs',{jobs:[]}],
  ['running job',{jobs:[{...missing,status:'in_progress'}]}],
  ['unknown result',{jobs:[{...missing,conclusion:'unknown'}]}],
  ['missing steps',{jobs:[{...missing,steps:undefined}]}],
  ['foreign annotation URL',{jobs:[{...missing,check_run_url:'https://api.github.com/repos/other/repo/check-runs/1'}]}],
  ['malformed annotation',{annotations:[[{message:null}]]}]
])test('unknown '+name+' fails closed',()=>{
  const f=fixture(change);assert.throws(()=>recover(options,f.api));assert.deepEqual(f.writes,[]);
});
test('production workflow uses default-branch code, bounded writer authority and no artifacts',()=>{
  const source=readFileSync('.github/workflows/recover-unacquired-runners.yml','utf8'),w=YAML.parse(source);
  assert.deepEqual(w.on.workflow_run,{workflows:['CI/CD Pipeline'],types:['completed']});
  assert.deepEqual(w.permissions,{contents:'read'});
  const job=w.jobs.recover;
  assert.equal(job['runs-on'],'ubuntu-24.04');assert.equal(job['timeout-minutes'],10);
  assert.equal(job.concurrency['cancel-in-progress'],false);
  assert(job.concurrency.group.includes('github.event.workflow_run.id'));
  assert.deepEqual(job.permissions,{contents:'read',actions:'write',checks:'read'});
  assert(job.if.includes("github.event.workflow_run.head_branch == github.event.repository.default_branch"));
  assert(job.if.includes("github.event.workflow_run.head_repository.full_name == github.repository"));
  const checkout=job.steps.find(step=>step.uses?.startsWith('actions/checkout@'));
  assert.equal(checkout.with.ref,'${{ github.event.repository.default_branch }}');
  assert.equal(checkout.with['persist-credentials'],false);
  assert.equal(job.steps.at(-1).run,'node --max-old-space-size=128 scripts/rerun-unacquired-jobs.mjs');
  assert.equal(job.steps.at(-1).env.GH_TOKEN,'${{ github.token }}');
  assert.doesNotMatch(source,/download-artifact|head_sha|head_commit/u);
});


test('actual command checks paginated GitHub observations before the one bounded write', {timeout:10000},()=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-acquisition-command-'));
  try {
    const tool=join(directory,'gh'),event=join(directory,'event.json'),control=join(directory,'control.json'),
      trace=join(directory,'trace.jsonl');
    writeFileSync(event,JSON.stringify({workflow_run:run,repository:{default_branch:'main'}}));
    const fake=[
      '#!/usr/bin/env python3','import json,os,sys',
      "state=json.load(open(os.environ['ACQUISITION_CONTROL']))",
      "with open(os.environ['ACQUISITION_TRACE'],'a') as stream:stream.write(json.dumps(sys.argv[1:])+'\\n')",
      'args=sys.argv[1:]',
      "if args[:2]==['run','rerun']:sys.exit(0)",
      'endpoint=args[-1]',
      "if '/annotations?' in endpoint:",
      " if state.get('failAnnotations'):sys.exit(23)",
      " print(json.dumps([[{'message':'unrelated'}],[{'message':state['message']}]]))",
      "elif '/jobs?' in endpoint:print(json.dumps([{'jobs':state['jobs'][:1]},{'jobs':state['jobs'][1:]}]))",
      "elif '/workflows/' in endpoint:print(json.dumps({'workflow_runs':[{'id':9,'head_sha':state['run']['head_sha']}]}))",
      "else:print(json.dumps(state['run']))"
    ].join('\n')+'\n';
    writeFileSync(tool,fake);chmodSync(tool,0o755);
    for(const [name,state,status,writes]of [
      ['recognized',{run,jobs:[missing,gate,passed],message},0,1],
      ['unknown annotations',{run,jobs:[missing,gate],message,failAnnotations:true},1,0],
      ['attempt limit',{run:{...run,run_attempt:3},jobs:[missing],message},1,0]
    ]) {
      writeFileSync(control,JSON.stringify(state));writeFileSync(trace,'');
      writeFileSync(event,JSON.stringify({workflow_run:state.run,repository:{default_branch:'main'}}));
      const result=spawnSync(process.execPath,['--max-old-space-size=128',resolve('scripts/rerun-unacquired-jobs.mjs')],
        {encoding:'utf8',env:{...process.env,PATH:directory+':'+process.env.PATH,GITHUB_EVENT_NAME:'workflow_run',
          GITHUB_EVENT_PATH:event,GITHUB_REPOSITORY:'owner/repo',ACQUISITION_CONTROL:control,ACQUISITION_TRACE:trace}});
      assert.equal(result.status,status,name+': '+result.stderr);
      const calls=readFileSync(trace,'utf8').trim().split('\n').map(JSON.parse);
      assert.equal(calls.filter(args=>args[0]==='run').length,writes,name);
      for(const args of calls.filter(args=>/\/(jobs|annotations)\?/u.test(args.at(-1)))) {
        assert(args.includes('--paginate'));assert(args.includes('--slurp'));
      }
      if(writes)assert.deepEqual(calls.at(-1),['run','rerun','9','--failed','--repo','owner/repo']);
    }
  }finally{rmSync(directory,{recursive:true,force:true});}
});
