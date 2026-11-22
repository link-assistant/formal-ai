import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {gunzipSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
const root=fileURLToPath(new URL('./fixtures/shared-runner-pagination/',import.meta.url));
const manifest=JSON.parse(readFileSync(join(root,'manifest.json'),'utf8'));
function source(language,path){
 const row=manifest.sources.find(row=>row.language===language&&row.path===path);assert(row);
 const bytes=gunzipSync(readFileSync(join(root,row.fixture)));
 assert.equal(bytes.length,row.bytes);assert.equal(createHash('sha256').update(bytes).digest('hex'),row.sha256);
 assert.equal(createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex'),row.gitBlob);
 return bytes.toString();
}
test('all retained shared pagination sources keep their pinned content and licences',()=>{
 for(const row of manifest.sources)source(row.language,row.path);
 assert.equal(manifest.authorshipCredit,0);
 for(const language of ['cpp','csharp'])assert(source(language,'LICENSE').includes('https://unlicense.org'));
});
test('actual C++ helper misses annotation 101; its isolated pagination patch preserves all controls',()=>{
 source('cpp','scripts/unacquired_jobs.py');
 const observed=spawnSync('python3',[fileURLToPath(new URL('./cpp-runner-annotation-replay.py',import.meta.url))],{encoding:'utf8',maxBuffer:1024*1024});
 assert.equal(observed.status,0,observed.stderr);
 const rows=JSON.parse(observed.stdout);assert.equal(rows.length,12);
 for(const row of rows){
  if(!row.candidate&&row.case==='second-hundred-page')assert.deepEqual(row.actual,[]);
  else assert.deepEqual(row.actual,row.expected,row.case);
  if(row.candidate&&row.calls.length>1)assert(row.calls[1].includes('--paginate')&&row.calls[1].includes('--slurp'));
  if(row.case==='real-code-failure')assert.equal(row.calls.length,1);
 }
});
test('actual C# helper misses annotations 31 and 101; isolated full-page traversal preserves controls',async context=>{
 const original=source('csharp','scripts/runner-acquisition.mjs');
 const old=`  for (const job of jobs.filter(isRunnerless))
    annotations.set(
      job.id,
      await api(\`repos/\${repo}/check-runs/\${job.id}/annotations\`),
    );`;
 const replacement=`  for (const job of jobs.filter(isRunnerless)) {
    const found = [];
    for (let page = 1; page <= 10; page++) {
      const items = await api(\`repos/\${repo}/check-runs/\${job.id}/annotations?per_page=100&page=\${page}\`);
      found.push(...items);
      if (items.length < 100) break;
      if (page === 10) throw Error('annotation page bound exhausted');
    }
    annotations.set(job.id, found);
  }`;
 assert.equal(original.split(old).length,2);
 const candidate=process.env.ISSUE_1580_DISABLE_ANNOTATION_PAGINATION==='1'?original:original.replace(old,replacement);
 const directory=mkdtempSync(join(tmpdir(),'formal-shared-runner-'));
 context.after(()=>rmSync(directory,{recursive:true,force:true}));
 writeFileSync(join(directory,'fetch-with-retries.mjs'),source('csharp','scripts/fetch-with-retries.mjs'));
 for(const [name,text] of [['original',original],['candidate',candidate]]){
  const path=join(directory,name+'.mjs');writeFileSync(path,text);
  const module=await import(pathToFileURL(path).href);
  for(const [label,position,ran] of [['first',0,false],['last-default-page',29,false],['second-default-page',30,false],['second-hundred-page',100,false],['absent',null,false],['real-code-failure',100,true]]){
   const calls=[];const annotations=Array.from({length:101},()=>({message:'ordinary diagnostic'}));
   if(position!==null)annotations[position]={message:'The job was not acquired by Runner of type hosted even after multiple attempts'};
   const job={id:42,name:'finite runner-loss case',status:'completed',conclusion:ran?'failure':'cancelled',runner_name:ran?'actual runner':'',steps:ran?[{conclusion:'failure'}]:[]};
   const api=async path=>{
    calls.push(path);const url=new URL(path,'https://api.example.test/');
    if(url.pathname.endsWith('/jobs'))return {jobs:[job],total_count:1};
    assert(url.pathname.endsWith('/annotations'));const size=Number(url.searchParams.get('per_page')??30),page=Number(url.searchParams.get('page')??1);
    return annotations.slice((page-1)*size,page*size);
   };
   const result=await module.inspectAttempt(api,'owner/repo',7,1);
   const expected=position!==null&&!ran&&!(name==='original'&&position>=30)?[job]:[];
   assert.deepEqual(result.unacquired,expected,name+':'+label);
   if(ran)assert.equal(calls.length,1);
   if(name==='candidate'&&!ran)assert.equal(calls.length,3);
  }
 }
});
