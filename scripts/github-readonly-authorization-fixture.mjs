import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createGovernedGithubCommandProvider} from './governed-github-command-provider.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const workflowPath='.github/workflows/github-readonly-authorization-fixture.yml';
export function requireObservationProfile(workflow,jobId,run,attempt) {
 assert.ok(['actions-omitted','actions-readable'].includes(jobId),'unknown authorization profile');
 for(const value of [run,attempt])assert.match(value,/^[1-9][0-9]*$/u);
 const job=workflow.jobs[jobId];assert.ok(job);
 const permissions=jobId==='actions-omitted'?{contents:'read'}:{contents:'read',actions:'read'};
 assert.deepEqual(job.permissions,permissions);
 assert.equal(job.if,"github.event_name == 'pull_request'");
 assert.equal(job.timeoutMinutes??job['timeout-minutes'],5);
 const template='formal-ai-readonly-profile-'+jobId+'-\${{ github.run_id }}-\${{ github.run_attempt }}';
 assert.equal(job.concurrency.group,template);assert.equal(job.concurrency['cancel-in-progress'],false);
 assert.ok(job.steps.some(step=>step.run==='node scripts/github-readonly-authorization-fixture.mjs' && step.env?.GH_TOKEN==='\${{ github.token }}'));
 return {job,group:'formal-ai-readonly-profile-'+jobId+'-'+run+'-'+attempt};
}
export async function observeAuthorization() {
 assert.equal(process.env.GITHUB_ACTIONS,'true');assert.equal(process.env.GITHUB_EVENT_NAME,'pull_request');
 const run=process.env.GITHUB_RUN_ID,attempt=process.env.GITHUB_RUN_ATTEMPT;
 const sha=process.env.GITHUB_SHA;assert.match(sha,/^[a-f0-9]{40}$/u);
 const event=JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH,'utf8'));
 const repository=process.env.GITHUB_REPOSITORY;
 assert.equal(event.repository.full_name,repository);assert.equal(event.pull_request.head.repo.full_name,repository);
 const head=event.pull_request.head.sha;assert.match(head,/^[a-f0-9]{40}$/u);
 assert.equal(execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8',timeout:5000}).trim(),sha);
 const files=[workflowPath,'scripts/github-readonly-authorization-fixture.mjs','scripts/governed-github-command-provider.mjs'];
 const sources=files.map(file=>{
  assert.equal(fs.lstatSync(file).isSymbolicLink(),false);
  const entry=execFileSync('git',['ls-tree',sha,'--',file],{encoding:'utf8',timeout:5000});
  assert.match(entry,/^100644 blob [a-f0-9]{40}\t/u);
  const bytes=fs.readFileSync(file),selected=execFileSync('git',['show',sha+':'+file],{timeout:5000});
  assert.deepEqual(bytes,selected,'selected immutable source differs');return {path:file,sha256:hash(bytes)};
 });
 const require=createRequire(path.resolve('package.json'));const YAML=require('yaml');
 const profile=requireObservationProfile(YAML.parse(fs.readFileSync(workflowPath,'utf8')),process.env.GITHUB_JOB,run,attempt);
 const expected={repository,head,run,attempt,workflowPath,descendantName:profile.job.name,runnerName:process.env.RUNNER_NAME};
 assert.ok(expected.runnerName);assert.ok(process.env.GH_TOKEN?.trim());
 const provider=createGovernedGithubCommandProvider({expected,sourcePaths:[workflowPath,'.github/workflows/release-staged.yml'],writerGroup:profile.group});
 const directory=path.join('.release-fixture','readonly-'+process.env.GITHUB_JOB);fs.mkdirSync(directory,{recursive:true});
 const observations=[];let error=null;
 function record(route) {
  assert.ok(route,'route unavailable after prior refusal');const snapshot=provider.read(route);provider.issued(snapshot,route);
  const streams=provider.processStreams(snapshot),index=observations.length;
  fs.writeFileSync(path.join(directory,index+'.stdout'),streams.stdout);fs.writeFileSync(path.join(directory,index+'.stderr'),streams.stderr);
  observations.push(snapshot);assert.equal(snapshot.status,200,'authenticated read not successful');return snapshot;
 }
 try {record(provider.runPath);record(provider.registered().workflow);record(provider.registered().jobs);record(provider.registered().group);}
 catch(failure){error=failure.message;}
 const receipt={schema:'ReadonlyGithubAuthorizationObservationV1',repository,run,attempt,checkout:sha,head,job:process.env.GITHUB_JOB,sources,
  requestCount:observations.length,observations,error,status:error?'Unknown':'ObservedReadSuccess',productionAuthority:false,mainAuthorization:'Pending',ancestorAuthority:'Unknown',publication:false};
 fs.writeFileSync(path.join(directory,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 return receipt;
}
if(process.argv[1] && import.meta.url===pathToFileURL(fs.realpathSync(process.argv[1])).href)observeAuthorization().catch(error=>{console.error(error.message);process.exitCode=1;});
