import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {verifiedGraphTree,treeQuery,createTreeReader} from '../../../scripts/lib/hive-delivery-trees.mjs';
import {githubReader,selectPullRequest} from '../../../scripts/admit-hive-original-delivery.mjs';
import {lateSourceCandidate,precheckPublishedCi,publishedChangedPaths} from '../../../scripts/lib/hive-delivery-release.mjs';
const treeRepository='owner/repository';
const graphLeaf=(name,mode=33188,type='blob')=>({name,nameRaw:Buffer.from(name).toString('base64'),mode,type,size:0,oid:'e'.repeat(40)});
function independentTree(entries){
 const ordered=[...entries].sort((left,right)=>Buffer.compare(Buffer.from(left.name+(left.type==='tree'?'/':'')),Buffer.from(right.name+(right.type==='tree'?'/':''))));
 const bytes=Buffer.concat(ordered.map(entry=>Buffer.concat([Buffer.from(entry.mode.toString(8)+' '+entry.name+'\0'),Buffer.from(entry.oid,'hex')])));
 const result=spawnSync('git',['hash-object','-t','tree','--stdin'],{input:bytes,encoding:'utf8',timeout:5000,maxBuffer:65536});
 assert.equal(result.status,0,result.stderr);return {oid:result.stdout.trim(),entries};
}
const answer=(values,cost=1)=>({data:{rateLimit:{cost},repository:{nameWithOwner:treeRepository,
 ...Object.fromEntries(values.map((value,index)=>['tree'+index,value]))}}});
test('GraphQL tree identity matches independent Git across raw Unicode, directory ordering and every supported mode',()=>{
 const value=independentTree([graphLeaf('z'),graphLeaf('prefix',16384,'tree'),graphLeaf('prefix.ext'),graphLeaf('😀'),
  graphLeaf('é'),graphLeaf('\uE000'),graphLeaf('run',33261),graphLeaf('link',40960),graphLeaf('submodule',57344,'commit')]);
 const result=verifiedGraphTree({...value,entries:[...value.entries].reverse()},value.oid);
 assert.equal(result.sha,value.oid);assert.equal(result.truncated,false);assert.equal(result.tree.length,9);
 const namesOnly={...value,entries:value.entries.map(({size,...entry})=>entry)};
 assert.equal(verifiedGraphTree(namesOnly,value.oid).sha,value.oid, 'complete Git identity needs no blob size expansion');
 assert.equal(verifiedGraphTree(independentTree([]),'4b825dc642cb6eb9a060e54bf8d69288fbee4904').tree.length,0);
});
test('missing, duplicated, substituted and malformed entries cannot authenticate a complete tree',()=>{
 const value=independentTree([graphLeaf('a'),graphLeaf('b')]);
 for(const mutate of [x=>x.entries.pop(),x=>x.entries.push({...x.entries[0]}),x=>x.entries[0].oid='f'.repeat(40),
  x=>x.entries[0].name='../outside',x=>x.entries[0].name='bad/name',x=>x.entries[0].name='bad\0name',
  x=>x.entries[0].nameRaw=Buffer.from('different').toString('base64'),x=>x.entries[0].nameRaw='!!!!',
  x=>x.entries[0].nameRaw='/w==',x=>x.entries[0].type='tree',x=>x.entries[0].mode=1,x=>x.entries[0].size=-1,
  x=>x.entries=null,x=>x.oid='f'.repeat(40),x=>x.entries=Array(100001).fill(x.entries[0])]){
  const changed=structuredClone(value);mutate(changed);assert.throws(()=>verifiedGraphTree(changed,value.oid));
 }
});
test('readonly query authority admits at most four exact owned tree IDs',()=>{
 assert.match(treeQuery(treeRepository,['a'.repeat(40)]),/^query\{/u);
 for(const [repository,ids]of [['foreign/repo/extra',['a'.repeat(40)]],[treeRepository,[]],
  [treeRepository,Array(5).fill('a'.repeat(40))],[treeRepository,['a'.repeat(40),'a'.repeat(40)]],
  [treeRepository,['HEAD']]])assert.throws(()=>treeQuery(repository,ids));
});
test('actual GithubReader batches fresh trees and preserves transport limits with separate cost accounting',async()=>{
 const fixtures=Array.from({length:8},(_,index)=>independentTree([graphLeaf('file'+index)]));
 const byId=new Map(fixtures.map(value=>[value.oid,value]));let calls=0,active=0,maximum=0;
 const reader=githubReader({repository:treeRepository,transport:async(program,args,options)=>{
  assert.equal(program,'gh');assert.equal(args[3],'graphql');assert.equal(args[4],'-f');
  assert(options.timeout>0&&options.timeout<=20000);assert.equal(options.stdoutLimitBytes,4*1024*1024);
  assert.equal(options.stderrLimitBytes,65536);const query=args[5];assert(query.startsWith('query=query{'));assert(!query.includes(' size '),'no expensive per-blob size expansion');
  const ids=[...query.matchAll(/object\(oid:"([a-f0-9]{40})"\)/gu)].map(match=>match[1]);assert(ids.length<=4);
  calls++;active++;maximum=Math.max(maximum,active);await new Promise(resolve=>setTimeout(resolve,2));active--;
  return {status:0,signal:null,cleanup_complete:true,stdout:Buffer.from(JSON.stringify(answer(ids.map(id=>byId.get(id))))) };
 }});
 const values=await Promise.all(fixtures.map(value=>reader.readTree(value.oid)));
 assert.deepEqual(values.map(value=>value.sha),fixtures.map(value=>value.oid));assert.equal(calls,2);assert(maximum<=4);
 assert.equal(reader.statistics().restRequests,0);assert.equal(reader.statistics().graphqlRequests,2);
 assert.equal(reader.statistics().graphqlPoints,2);await reader.readTree(fixtures[0].oid);assert.equal(calls,2);
});
test('published changed-path traversal uses verified typed trees while commit roots stay fresh REST',async()=>{
 const old=independentTree([graphLeaf('before')]),updated=independentTree([graphLeaf('after')]);
 const before='a'.repeat(40),after='b'.repeat(40);const fixtures=new Map([[old.oid,old],[updated.oid,updated]]);
 const reader=githubReader({repository:treeRepository,transport:async(program,args)=>{
  let response;
  if(args[3]==='graphql'){
   const ids=[...args[5].matchAll(/object\(oid:"([a-f0-9]{40})"\)/gu)].map(match=>match[1]);
   response=answer(ids.map(id=>fixtures.get(id)));
  }else{
   const endpoint=args.at(-1);assert(endpoint.includes('/git/commits/'));
   const sha=endpoint.endsWith(before)?before:after;response={sha,tree:{sha:sha===before?old.oid:updated.oid}};
  }
  return {status:0,signal:null,cleanup_complete:true,stdout:Buffer.from(JSON.stringify(response))};
 }});
 const result=await publishedChangedPaths(reader,treeRepository,before,after);
 assert.deepEqual(result.files,['after','before']);assert.equal(result.trees.length,2);
 assert.equal(reader.statistics().restRequests,2);assert.equal(reader.statistics().graphqlPoints,1, 'old and new trees share one bounded query');
});
test('GraphQL errors, wrong repository, missing objects and unexpected costs stop without retry',async()=>{
 const value=independentTree([graphLeaf('a')]);
 for(const mutation of [x=>x.errors=[{message:'failed'}],x=>x.data.repository.nameWithOwner='foreign/repo',
  x=>x.data.repository.tree0=null,x=>x.data.rateLimit.cost=2]){
  let calls=0;const reader=createTreeReader(treeRepository,async()=>{calls++;const response=answer([value]);mutation(response);return response;});
  await assert.rejects(reader.readTree(value.oid));await assert.rejects(reader.readTree(value.oid));assert.equal(calls,1);
 }
 let calls=0;const reader=createTreeReader(treeRepository,async()=>{calls++;throw Error('physical network failure');});
 await assert.rejects(reader.readTree(value.oid));await assert.rejects(reader.readTree(value.oid));assert.equal(calls,1);
});
test('GraphQL point exhaustion never issues a retry or a129th physical query',async()=>{
 let calls=0;const values=Array.from({length:129},(_,index)=>independentTree([graphLeaf('point'+index)]));
 const reader=createTreeReader(treeRepository,async query=>{
  calls++;const id=query.match(/object\(oid:"([a-f0-9]{40})"\)/u)[1];return answer([values.find(value=>value.oid===id)]);
 });
 for(const value of values.slice(0,128))await reader.readTree(value.oid);
 await assert.rejects(reader.readTree(values[128].oid),/point budget/u);assert.equal(calls,128);assert.equal(reader.statistics().points,128);
});
test('failed API reads expose bounded status diagnostics without echoing response bodies',async()=>{
 for(const [status,stderr,error,timed_out,expected]of [
  [1,'gh: HTTP 502\n',undefined,false,/"httpStatus":502/u],
  [1,'gh: HTTP 502: Bad Gateway\n',undefined,false,/"httpStatus":502/u],
  [null,'',Object.assign(Error('timeout'),{code:'ETIMEDOUT'}),true,/"errorCode":"ETIMEDOUT"/u]]){
  const reader=githubReader({repository:treeRepository,transport:async()=>({status,signal:null,error,timed_out,
   cleanup_complete:true,stdout:Buffer.from('private-diagnostic-content'),stderr:Buffer.from(stderr)})});
  await assert.rejects(reader.read('repos/'+treeRepository+'/metadata'),failure=>{
   assert.match(failure.message,expected);assert.match(failure.message,/"stdoutSha256":"[a-f0-9]{64}"/u);
   assert(!failure.message.includes('private-diagnostic-content'));return true;
  });
 }
});
test('clean empty REST read failure retries the identical owned request once with private diagnostics hashed',async()=>{
 const observed=[];let calls=0;const argumentsSeen=[];
 const endpoint='repos/'+treeRepository+'/commits/'+'a'.repeat(40)+'/pulls?per_page=100&page=1';
 const stderr=Buffer.from('private diagnostic transport');
 const reader=githubReader({repository:treeRepository,diagnostics:record=>observed.push(record),
  transport:async(program,args,options)=>{
   calls++;argumentsSeen.push(args);assert.equal(program,'gh');
   assert(options.timeout>0&&options.timeout<=20000);assert.equal(options.stderrLimitBytes,65536);
   return calls===1?{status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr}:
    {status:0,signal:null,cleanup_complete:true,stdout:Buffer.from('[]')};
  }});
 assert.deepEqual(await reader.pages(endpoint),[]);assert.equal(calls,2);assert.deepEqual(argumentsSeen[0],argumentsSeen[1]);
 assert.equal(observed.length,1);assert.equal(observed[0].operation,'commit-pull-requests');
 assert.equal(observed[0].attempt,1);assert.equal(observed[0].maximumAttempts,2);assert.equal(observed[0].retrying,true);
 assert.equal(observed[0].stderrBytes,stderr.length);assert.match(observed[0].requestSha256,/^[a-f0-9]{64}$/u);
 assert(!JSON.stringify(observed).includes(stderr.toString()));assert(!JSON.stringify(observed).includes(endpoint));
 assert.equal(reader.statistics().requests,2);assert.equal(reader.statistics().restRequests,2);
});
test('repeated clean REST failures stop after two physical requests and preserve both diagnostics',async()=>{
 for(const stderr of ['unknown CLI read failure\n','gh: HTTP 503: Service Unavailable\n']){
  let calls=0;const observed=[];const reader=githubReader({repository:treeRepository,diagnostics:x=>observed.push(x),
   transport:async()=>{calls++;return {status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr:Buffer.from(stderr)};}});
  await assert.rejects(reader.read('repos/'+treeRepository+'/metadata'),/GitHub read failed/u);
  assert.equal(calls,2);assert.deepEqual(observed.map(x=>[x.attempt,x.retrying]),[[1,true],[2,false]]);
  assert.equal(observed[0].requestSha256,observed[1].requestSha256);assert.equal(reader.statistics().requests,2);
 }
});
test('fourth-read admission failure recovers through the real selector without admitting an absent or changed source',async()=>{
 const head='a'.repeat(40),prefix='repos/'+treeRepository;
 const trigger={id:41,status:'completed',path:'.github/workflows/coverage.yml',head_sha:head,
  repository:{full_name:treeRepository},head_repository:{full_name:treeRepository}};
 const pull={number:1,state:'open',merged:false,merged_at:null,
  head:{sha:head,ref:'task',repo:{full_name:treeRepository}},
  base:{sha:'b'.repeat(40),ref:'main',repo:{full_name:treeRepository}}};
 for(const mode of ['absent','current','changed']){
  let calls=0,failed=false;const observed=[];
  const reader=githubReader({repository:treeRepository,diagnostics:x=>observed.push(x),transport:async(program,args)=>{
   calls++;const endpoint=args.at(-1);let value;
   if(endpoint===prefix+'/metadata')value={};
   else if(endpoint===prefix+'/actions/runs/41')value=trigger;
   else if(endpoint===prefix+'/commits/'+head+'/pulls?per_page=100&page=1'){
    if(!failed){failed=true;assert.equal(calls,4);return {status:1,signal:null,cleanup_complete:true,
     stdout:Buffer.alloc(0),stderr:Buffer.alloc(29,120)};}
    value=mode==='absent'?[]:[pull];
   }else{assert.equal(endpoint,prefix+'/pulls/1');value=mode==='changed'?{...pull,head:{...pull.head,sha:'c'.repeat(40)}}:pull;}
   return {status:0,signal:null,cleanup_complete:true,stdout:Buffer.from(JSON.stringify(value))};
  }});
  await reader.read(prefix+'/metadata');await reader.read(prefix+'/metadata');
  const selected=selectPullRequest({repository:treeRepository,eventName:'workflow_run',event:{workflow_run:{id:41}},reader});
  if(mode==='changed')await assert.rejects(selected,/stale pull-request head/u);
  else assert.equal((await selected)?.source??null,mode==='current'?head:null);
  assert.equal(calls,mode==='absent'?5:6);assert.equal(observed.length,1);
  assert.equal(observed[0].request,4);assert.equal(observed[0].stderrBytes,29);assert.equal(observed[0].retrying,true);
 }
});
test('REST denial, partial body, timeout, interrupted process and incomplete cleanup never retry',async()=>{
 const base={status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr:Buffer.alloc(0)};
 for(const change of [
  {stderr:Buffer.from('gh: HTTP 401: Bad credentials\n')},
  {stderr:Buffer.from('gh: HTTP 403: Forbidden\n')},
  {stderr:Buffer.from('gh: HTTP 429: Too Many Requests\n')},
  {stdout:Buffer.from('{"private":"partial response"}'),stderr:Buffer.from('gh: HTTP 503: Service Unavailable\n')},
  {timed_out:true},{signal:'SIGTERM'},{cleanup_complete:false},
  {error:Object.assign(Error('private process failure'),{code:'EACCES'})}
 ]){
  let calls=0;const observed=[];const reader=githubReader({repository:treeRepository,diagnostics:x=>observed.push(x),
   transport:async()=>{calls++;return {...base,...change};}});
  await assert.rejects(reader.read('repos/'+treeRepository+'/metadata'),/GitHub read failed/u);
  assert.equal(calls,1);assert.equal(observed[0].retrying,false);
  assert(!JSON.stringify(observed).includes('private'));
 }
});
test('GraphQL physical failures still consume exactly one query with no retry',async()=>{
 let calls=0;const reader=githubReader({repository:treeRepository,diagnostics:()=>{},transport:async()=>{
  calls++;return {status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr:Buffer.from('gh: HTTP 503: Service Unavailable\n')};
 }});
 await assert.rejects(reader.readTree('a'.repeat(40)),/GitHub read failed/u);
 await assert.rejects(reader.readTree('a'.repeat(40)),/GitHub read failed/u);
 assert.equal(calls,1);assert.equal(reader.statistics().graphqlRequests,1);
});
test('a REST retry shares the original absolute deadline and cannot issue request513',async()=>{
 const saved=Date.now;let now=0;Date.now=()=>now;
 try{
  const timeouts=[];const reader=githubReader({repository:treeRepository,diagnostics:()=>{},transport:async(program,args,options)=>{
   timeouts.push(options.timeout);
   if(timeouts.length===1){now=149990;return {status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0)};}
   return {status:0,signal:null,cleanup_complete:true,stdout:Buffer.from('{}')};
  }});
  assert.deepEqual(await reader.read('repos/'+treeRepository+'/metadata'),{});assert.deepEqual(timeouts,[20000,10]);
 }finally{Date.now=saved;}
 let calls=0;const observed=[];const reader=githubReader({repository:treeRepository,diagnostics:x=>observed.push(x),transport:async()=>{
  calls++;return calls===512?{status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0)}:
   {status:0,signal:null,cleanup_complete:true,stdout:Buffer.from('{}')};
 }});
 for(let index=0;index<511;index++)await reader.read('repos/'+treeRepository+'/metadata');
 await assert.rejects(reader.read('repos/'+treeRepository+'/metadata'),/GitHub read failed/u);
 assert.equal(calls,512);assert.equal(reader.statistics().requests,512);assert.equal(observed[0].retrying,false);
 await assert.rejects(reader.read('repos/'+treeRepository+'/metadata'),/bounded admission API requests/u);assert.equal(calls,512);
});
test('REST and GraphQL share the existing finite physical request budget',async()=>{
 const value=independentTree([graphLeaf('budget')]);let calls=0;
 const reader=githubReader({repository:treeRepository,transport:async(program,args)=>{
  calls++;return {status:0,signal:null,cleanup_complete:true,
   stdout:Buffer.from(JSON.stringify(args[3]==='graphql'?answer([value]):{}))};
 }});
 await reader.readTree(value.oid);
 for(let index=0;index<511;index++)await reader.read('repos/'+treeRepository+'/metadata');
 await assert.rejects(reader.read('repos/'+treeRepository+'/metadata'),/bounded admission API requests/u);
 assert.equal(calls,512);assert.equal(reader.statistics().graphqlRequests,1);
 assert.equal(reader.statistics().restRequests,511);
});
test('actual reader retains deadline, per-response and aggregate byte refusals',async()=>{
 const saved=Date.now;let now=0,calls=0;Date.now=()=>now;
 try{
  const reader=githubReader({repository:treeRepository,transport:async()=>{calls++;throw Error('must not run');}});
  now=150001;await assert.rejects(reader.readTree('a'.repeat(40)),/deadline/u);assert.equal(calls,0);
 }finally{Date.now=saved;}
 const response=Buffer.alloc(4*1024*1024+1,32);
 const reader=githubReader({repository:treeRepository,transport:async()=>({status:0,signal:null,cleanup_complete:true,stdout:response})});
 await assert.rejects(reader.bytes('repos/'+treeRepository+'/metadata'),/response bytes/u);
 const full=Buffer.alloc(4*1024*1024,32);
 const aggregate=githubReader({repository:treeRepository,transport:async()=>({status:0,signal:null,cleanup_complete:true,stdout:full})});
 for(let index=0;index<16;index++)await aggregate.bytes('repos/'+treeRepository+'/metadata');
 await assert.rejects(aggregate.bytes('repos/'+treeRepository+'/metadata'),/aggregate bytes/u);
});
test('late source candidate denial never promotes a parent into authority',()=>{
 const source='a'.repeat(40),parent='b'.repeat(40),foreign='c'.repeat(40),commit={sha:source,parents:[{sha:parent}]};
 assert.equal(lateSourceCandidate(source,commit,source),true);
 assert.equal(lateSourceCandidate(source,commit,parent),true);
 assert.equal(lateSourceCandidate(source,commit,foreign),false);
 assert.equal(lateSourceCandidate(source,{...commit,parents:[{sha:parent},{sha:foreign}]},parent),false);
 assert.throws(()=>lateSourceCandidate(source,{...commit,sha:foreign},parent));
 assert.throws(()=>lateSourceCandidate(source,commit,'invalid'));
});
test('cheap fresh CI precheck only defers observed pending work; full required-workflow census remains later',async()=>{
 const source='a'.repeat(40),run={id:1,run_attempt:1,head_sha:source,path:'.github/workflows/release.yml',
  repository:{full_name:treeRepository},head_repository:{full_name:treeRepository},event:'push',status:'in_progress',conclusion:null};
 let calls=0;const reader={pages:async(endpoint,member)=>{calls++;assert.equal(member,'workflow_runs');assert(endpoint.endsWith(source));return [run];}};
 assert.deepEqual(await precheckPublishedCi(reader,treeRepository,source,run.path),{admitted:false,reason:'other CI pending'});
 run.status='completed';run.conclusion='success';assert.equal((await precheckPublishedCi(reader,treeRepository,source,run.path)).admitted,true);
 assert.equal(calls,2);assert.equal((await precheckPublishedCi({pages:async()=>[]},treeRepository,source,run.path)).admitted,false);
 await assert.rejects(precheckPublishedCi({pages:async()=>[{...run,head_repository:{full_name:'foreign/repo'}}]},treeRepository,source,run.path));
 await assert.rejects(precheckPublishedCi({pages:async()=>[{...run,head_sha:'b'.repeat(40)}]},treeRepository,source,run.path));
 const code=readFileSync(new URL('../../../scripts/lib/hive-delivery-release.mjs',import.meta.url),'utf8');
 assert(code.indexOf('const earlyChecked = await precheckPublishedCi')<code.indexOf('const changedPaths = await publishedChangedPaths'));
 assert(code.indexOf('const checkedDecision = ciAdmissionDecision')>code.indexOf('const expected = applicableWorkflows'));
});
