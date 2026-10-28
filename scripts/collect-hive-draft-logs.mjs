// Read-only post-run recovery survives a cancelled draft runner. Logs are data, never commands.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {runGh,redact} from './collect-ci-failures.mjs';
export async function collectDraftJobLogs({repository,runId,attempt,head,output,transport=runGh}) {
 assert(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository));
 for(const number of [runId,attempt])assert(Number.isSafeInteger(number)&&number>0);
 assert(/^[a-f0-9]{40}$/.test(head));
 fs.mkdirSync(output,{recursive:false,mode:0o700});
 const manifest={schema:'hive-draft-job-logs/v1',repository,runId,attempt,head,jobs:[],unavailable:[],
  complete:false,privacy:'GitHub masking plus credential and personal-data redaction',modelRerun:false};
 const deadline=Date.now()+180000;let bytes=0,requests=0;
 async function request(endpoint,log=false){
  assert(++requests<=33&&Date.now()<deadline,'collection deadline/request budget exhausted');
  const remaining=134217728-bytes;assert(remaining>0,'aggregate log budget exhausted');
  const limit=Math.min(log?67108864:1048576,remaining);
  const result=await transport(['api','repos/'+repository+'/'+endpoint],{
   captureRaw:log,timeoutMs:Math.min(20000,deadline-Date.now()),maxBytes:limit,maxDownloadBytes:limit});
  bytes+=result.bytesReceived??Buffer.byteLength(result.text??'');
  if(log)return result;
  assert(result.ok&&!result.truncated&&!result.downloadTruncated&&result.streamComplete!==false,'complete API response unavailable');
  return JSON.parse(result.text);
 }
 try {
  const run=await request('actions/runs/'+runId+'/attempts/'+attempt);
  assert.equal(run.id,runId);assert.equal(run.run_attempt,attempt);assert.equal(run.head_sha,head);
  assert.equal(run.repository?.full_name,repository);assert.equal(run.path,'.github/workflows/hive-mind-draft.yml');
  assert.equal(run.status,'completed');assert(['issues','workflow_dispatch'].includes(run.event));
  manifest.conclusion=run.conclusion;
  const page=await request('actions/runs/'+runId+'/attempts/'+attempt+'/jobs?per_page=100');
  assert(Array.isArray(page.jobs)&&page.jobs.length===page.total_count&&page.jobs.length>0&&page.jobs.length<=30,'complete bounded job census required');
  assert.equal(new Set(page.jobs.map(job=>job.id)).size,page.jobs.length);
  for(const job of page.jobs){
   try{
    assert.equal(job.run_id,runId);assert.equal(job.status,'completed');assert(Number.isSafeInteger(job.id)&&job.id>0);
    const row={id:job.id,name:redact(job.name),status:job.status,conclusion:job.conclusion,steps:job.steps?.map(step=>({
     name:redact(step.name),number:step.number,status:step.status,conclusion:step.conclusion,startedAt:step.started_at,completedAt:step.completed_at}))??null};
    manifest.jobs.push(row);
    if(job.conclusion==='skipped'){row.log='not-executed';continue;}
    const observed=await request('actions/jobs/'+job.id+'/logs',true);
    const complete=observed.ok&&!observed.truncated&&!observed.downloadTruncated&&observed.streamComplete!==false;
    const content=Buffer.from(redact(observed.text??'')),name='job-'+job.id+'.log';
    fs.writeFileSync(path.join(output,name),content,{flag:'wx',mode:0o600});
    row.log={path:name,bytes:content.length,sha256:createHash('sha256').update(content).digest('hex'),complete};
    if(!complete)manifest.unavailable.push({jobId:job.id,reason:'partial_job_log',
     timedOut:observed.timedOut===true,truncated:observed.truncated===true||observed.downloadTruncated===true});
   }catch{manifest.unavailable.push({jobId:job.id,reason:'complete_job_log_unavailable'});}
  }
 }catch{manifest.unavailable.push({reason:'complete_source_and_job_census_unavailable'});}
 manifest.complete=manifest.unavailable.length===0;
 manifest.scope='Complete available GitHub job console logs; runner-local dialog/session artifacts remain separate.';
 fs.writeFileSync(path.join(output,'job-logs-manifest.json'),JSON.stringify(manifest,null,2)+'\n',{flag:'wx'});
 return manifest;
}
if(process.argv[1]&&fs.realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
 const event=JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH,'utf8')),run=event.workflow_run;
 assert(run&&run.repository?.full_name===process.env.GITHUB_REPOSITORY,'foreign workflow run');
 const result=await collectDraftJobLogs({repository:process.env.GITHUB_REPOSITORY,runId:run.id,attempt:run.run_attempt,
  head:run.head_sha,output:process.env.DRAFT_JOB_LOG_OUTPUT});
 console.log(JSON.stringify({complete:result.complete,jobs:result.jobs.length,unavailable:result.unavailable}));
 if(!result.complete)process.exitCode=1;
}
