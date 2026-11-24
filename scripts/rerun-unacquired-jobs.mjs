// The Go template's conservative acquisition classification and bounded retry
// policy, adapted to Formal AI's default-branch CI and checked GitHub objects.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const repositoryPattern=/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u;
const integer=value=>Number.isSafeInteger(value)&&value>0;
const unsuccessful=new Set(['failure','cancelled','timed_out']);
const acquisition='was not acquired by Runner';
const gates=new Set(['Pipeline Status']);
export const MAX_ATTEMPTS=3;
function validateRepository(repository) {
  assert(repositoryPattern.test(repository)&&repository.split('/').every(part=>!['.','..'].includes(part)),
    'exact repository identity required');
}
function pages(value,key) {
  assert(Array.isArray(value)&&value.length>0,'complete paginated API response required');
  return value.flatMap(page=>{
    const entries=key?page?.[key]:page;
    assert(Array.isArray(entries),'every API page must contain an inventory');
    return entries;
  });
}
function checkEndpoint(repository,job) {
  const url=new URL(job.check_run_url);
  assert(url.origin==='https://api.github.com'&&!url.username&&!url.password&&!url.search&&!url.hash,
    'exact check-run URL required');
  const prefix='/repos/'+repository+'/check-runs/';
  assert(url.pathname.startsWith(prefix)&&/^[1-9][0-9]*$/u.test(url.pathname.slice(prefix.length)),
    'check-run URL must belong to the observed repository');
  return url.pathname.slice(1)+'/annotations?per_page=100';
}
export function classifyJobs(jobs,annotations) {
  assert(Array.isArray(jobs)&&jobs.length>0,'nonempty job inventory required');
  const identifiers=new Set();
  return jobs.map(job=>{
    assert(integer(job.id)&&!identifiers.has(job.id),'unique job identity required');
    identifiers.add(job.id);
    assert(typeof job.name==='string'&&job.name.length>0&&job.status==='completed',
      'complete terminal job inventory required');
    assert(['success','skipped','neutral',...unsuccessful].includes(job.conclusion),'known job outcome required');
    if(!unsuccessful.has(job.conclusion))return {job,kind:'successful-or-skipped'};
    assert(Array.isArray(job.steps)&&(typeof job.runner_name==='string'||job.runner_name===null),
      'runner and step observations required');
    if(job.steps.length||job.runner_name)return {job,kind:'failed'};
    const messages=annotations(job);
    assert(Array.isArray(messages)&&messages.every(message=>typeof message==='string'),
      'complete annotation messages required');
    return {job,kind:messages.some(message=>message.includes(acquisition))?'not-acquired':'not-started'};
  });
}
export function decide(classified,attempt) {
  assert(integer(attempt),'positive attempt required');
  if(!classified.some(row=>row.kind==='not-acquired'))return 'nothing';
  if(classified.some(row=>['failed','not-started'].includes(row.kind)&&!gates.has(row.job.name)))return 'blocked';
  return attempt<MAX_ATTEMPTS?'rerun':'give-up';
}
function validateRun(run,{repository,runId,attempt,defaultBranch}) {
  assert(run?.id===runId&&run.run_attempt===attempt,'exact run and attempt required');
  assert(run.status==='completed'&&run.conclusion==='failure','completed failed run required');
  assert(run.head_repository?.full_name===repository&&run.repository?.full_name===repository,
    'same-repository run required');
  assert(run.head_branch===defaultBranch&&['push','workflow_dispatch'].includes(run.event),
    'default-branch push or dispatch required');
  assert(integer(run.workflow_id)&&/^[a-f0-9]{40}$/u.test(run.head_sha),'immutable workflow source required');
  assert(run.name==='CI/CD Pipeline','only the observed pipeline may be recovered');
}
export function recover(options,api) {
  const {repository,runId,attempt,defaultBranch}=options;
  validateRepository(repository);
  assert(integer(runId)&&integer(attempt),'positive run and attempt required');
  assert(typeof defaultBranch==='string'&&defaultBranch.length>0&&!defaultBranch.startsWith('-'),
    'default branch required');
  const endpoint='repos/'+repository+'/actions/runs/'+runId;
  const run=api.read(endpoint);validateRun(run,options);
  const jobs=pages(api.read(endpoint+'/attempts/'+attempt+'/jobs?per_page=100',true),'jobs');
  for(const job of jobs)assert(job.run_id===runId,'job belongs to the exact run');
  const classified=classifyJobs(jobs,job=>pages(api.read(checkEndpoint(repository,job),true))
    .map(annotation=>{assert(typeof annotation.message==='string','annotation message required');return annotation.message;}));
  const action=decide(classified,attempt);
  if(action!=='rerun')return {action,jobs:classified.length};
  const latestEndpoint='repos/'+repository+'/actions/workflows/'+run.workflow_id+'/runs?branch='+
    encodeURIComponent(defaultBranch)+'&event='+encodeURIComponent(run.event)+'&per_page=1';
  const latest=api.read(latestEndpoint)?.workflow_runs;
  assert(Array.isArray(latest)&&latest.length===1,'newest workflow observation required');
  if(latest[0].id!==runId||latest[0].head_sha!==run.head_sha)return {action:'superseded',jobs:classified.length};
  const current=api.read(endpoint);validateRun(current,options);
  assert(current.head_sha===run.head_sha&&current.workflow_id===run.workflow_id,
    'source identity changed during recovery');
  if(!options.dryRun)api.rerun({repository,runId});
  return {action:options.dryRun?'dry-run':'rerun',jobs:classified.length,attempt,maxAttempts:MAX_ATTEMPTS};
}
export function githubClient() {
  const deadline=Date.now()+180000;
  const invoke=args=>{
    const remaining=deadline-Date.now();assert(remaining>0,'recovery observation deadline exceeded');
    return execFileSync('gh',args,{encoding:'utf8',timeout:Math.min(15000,remaining),maxBuffer:8*1024*1024});
  };
  return {read:(endpoint,paginate=false)=>JSON.parse(invoke(['api',...(paginate?['--paginate','--slurp']:[]),endpoint])),
    rerun:({repository,runId})=>invoke(['run','rerun',String(runId),'--failed','--repo',repository])};
}
export function main(environment=process.env) {
  const event=JSON.parse(readFileSync(environment.GITHUB_EVENT_PATH,'utf8'));
  const run=event.workflow_run;
  assert(environment.GITHUB_EVENT_NAME==='workflow_run','workflow_run observation required');
  const options={repository:environment.GITHUB_REPOSITORY,runId:run?.id,attempt:run?.run_attempt,
    defaultBranch:event.repository?.default_branch,dryRun:environment.FORMAL_AI_RECOVERY_DRY_RUN==='true'};
  const result=recover(options,githubClient());
  // API names and annotation text are never interpolated into Actions commands.
  console.log(JSON.stringify({runnerRecovery:result}));
  if(result.action==='give-up'){
    console.error('::error::Hosted runner acquisition exhausted three attempts; rerun the failed pipeline later.');
    return 1;
  }
  return 0;
}
if(process.argv[1]&&pathToFileURL(resolve(process.argv[1])).href===import.meta.url) {
  try {process.exitCode=main();}
  catch(error){console.error('Runner recovery refused: '+JSON.stringify(error.message));process.exitCode=1;}
}
