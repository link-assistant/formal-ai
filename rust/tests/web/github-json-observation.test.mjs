import test from 'node:test';
import assert from 'node:assert/strict';
import {readGithubJson,GithubObservationUnavailable} from '../../../scripts/resolve-package-release.mjs';
import {mkdtempSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const endpoint='repos/link-assistant/formal-ai/actions/runs/41084627272/attempts/1/jobs?per_page=20&page=1';
const success={status:0,signal:null,cleanup_complete:true,stdout:Buffer.from('{"total_count":0,"jobs":[]}'),stderr:Buffer.alloc(0)};
const failure=code=>({...success,status:1,stdout:Buffer.alloc(0),stderr:Buffer.from('gh: Server error (HTTP '+code+')')});
function scenario(results,extra={}){
 const calls=[],pauses=[],reports=[];
 const run=()=>readGithubJson(endpoint,{environment:{GH_TOKEN:'fixture-secret'},
  transport:(file,args,options)=>{calls.push({file,args,options});assert(results.length>0);return results.shift();},
  pause:value=>pauses.push(value),report:value=>reports.push(value),...extra});
 return {run,calls,pauses,reports};
}
test('captured jobs HTTP 504 and monitor HTTP 502 recover within a three-read bound',()=>{
 const fixture=scenario([failure(504),failure(502),success]);
 assert.deepEqual(fixture.run(),{total_count:0,jobs:[]});assert.equal(fixture.calls.length,3);
 assert.deepEqual(fixture.pauses,[2000,4000]);assert.equal(fixture.reports.length,2);
 for(const call of fixture.calls){assert.equal(call.file,'gh');assert.deepEqual(call.args,['api',endpoint]);
  assert.equal(call.options.timeout,30000);assert.equal(call.options.maxBuffer,2097152);}
});
test('persistent upstream failure stops after exactly three reads',()=>{
 const fixture=scenario([failure(504),failure(504),failure(504)]);
 assert.throws(fixture.run,error=>error instanceof GithubObservationUnavailable&&error.detail.attempt===3);
 assert.equal(fixture.calls.length,3);assert.deepEqual(fixture.pauses,[2000,4000]);
});
test('authorization, absence, malformed JSON and uncertain process cleanup never retry',()=>{
 for(const value of [failure(401),failure(403),failure(404),
  {...success,stdout:Buffer.from('truncated')},
  {...failure(504),cleanup_complete:false},{...failure(504),error:{code:'ETIMEDOUT'}},
  {...failure(504),timed_out:true},{...failure(504),signal:'SIGKILL'}]){
  const fixture=scenario([value]);assert.throws(fixture.run);assert.equal(fixture.calls.length,1);assert.equal(fixture.pauses.length,0);
 }
});
test('official production hostname and smaller response budget remain explicit',()=>{
 const fixture=scenario([success],{hostname:'github.com',timeout:10000,maxBuffer:1048576});fixture.run();
 assert.deepEqual(fixture.calls[0].args,['api','--hostname','github.com',endpoint]);
 assert.equal(fixture.calls[0].options.env.GH_HOST,'github.com');
 assert.equal(fixture.calls[0].options.maxBuffer,1048576);
 assert.equal(fixture.calls[0].options.timeout,10000);
});
test('unknown endpoints, invalid UTF-8 and oversized responses fail closed',()=>{
 for(const value of ['https://foreign.invalid','-X POST','repos/owner/repo\nextra'])
  assert.throws(()=>readGithubJson(value));
 for(const stdout of [Buffer.from([0xff]),Buffer.alloc(1048577)]){
  const fixture=scenario([{...success,stdout}],{maxBuffer:1048576});assert.throws(fixture.run);
 }
});
test('retry diagnostics preserve the status and redact selected credentials',()=>{
 const fixture=scenario([{...failure(504),stderr:Buffer.from('fixture-secret HTTP 504')},success]);
 fixture.run();assert(!JSON.stringify(fixture.reports).includes('fixture-secret'));
 assert.equal(fixture.reports[0].attempt,1);assert.match(fixture.reports[0].stderr,/\*\*\* HTTP 504/u);
});
test('real post-publication CLI defers only exhausted server reads and performs no authoring',
 {skip:process.platform==='win32'?'POSIX fake gh executable; portable transport and authority tests still run':false},()=>{
 const directory=mkdtempSync(join(tmpdir(),'self-author-observation-'));
 try{
  const output=join(directory,'output'),summary=join(directory,'summary'),calls=join(directory,'calls');
  writeFileSync(join(directory,'gh'),'#!/usr/bin/env node\n'+
   'require("node:fs").appendFileSync(process.env.FIXTURE_CALLS,JSON.stringify(process.argv.slice(2))+"\\n");'+
   'console.error("fixture-secret HTTP "+process.env.FIXTURE_HTTP);process.exitCode=1;\n',{mode:0o755});
  for(const code of ['504','403']){
   for(const path of [output,summary,calls])writeFileSync(path,'');
   const child=spawnSync(process.execPath,['scripts/resolve-self-authored-release.mjs'],{
    cwd:fileURLToPath(new URL('../../../',import.meta.url)),encoding:'utf8',timeout:15000,
    env:{...process.env,PATH:directory+':'+process.env.PATH,GH_TOKEN:'fixture-secret',FIXTURE_CALLS:calls,FIXTURE_HTTP:code,
     REPOSITORY:'fixture/repository',EVENT:'push',PRODUCER_RUN_ID:'41',PRODUCER_RUN_ATTEMPT:'1',CURRENT_RUN_ID:'41',
     CURRENT_RUN_ATTEMPT:'1',REFERENCE:'refs/heads/main',CURRENT_WORKFLOW_REF:'fixture/repository/.github/workflows/release.yml@refs/heads/main',
     GITHUB_OUTPUT:output,GITHUB_STEP_SUMMARY:summary}});
   const reads=readFileSync(calls,'utf8').trim().split('\n').map(JSON.parse);
   assert(reads.every(args=>args[0]==='api'&&args[1]==='repos/fixture/repository/actions/runs/41'));
   assert(!child.stdout.includes('fixture-secret')&&!child.stderr.includes('fixture-secret'));
   if(code==='504'){
    assert.equal(child.status,0,child.stderr);assert.equal(reads.length,3);
    assert.equal(readFileSync(output,'utf8'),'active=false\nbranch=\nrelease_id=\n');
    assert.match(child.stdout,/"observation":"Deferred"/u);
    assert.match(readFileSync(summary,'utf8'),/no self-development credit is awarded/u);
   }else{
    assert.equal(child.status,1);assert.equal(reads.length,1);assert.equal(readFileSync(output,'utf8'),'');
   }
  }
 }finally{rmSync(directory,{recursive:true,force:true});}
});

// Captured Windows failure shape: the direct read ended on its timeout signal before publication effects.
const windowsTimeout = {
 status:null,signal:'SIGTERM',cleanup_complete:false,timed_out:true,
 stdout:Buffer.alloc(0),stderr:Buffer.alloc(0),
 error:{code:'ETIMEDOUT',name:'Error',message:'spawnSync gh ETIMEDOUT fixture-secret',syscall:'spawnSync gh'}
};
test('opt-in captured Windows GET timeout recovers on the second or third bounded read',()=>{
 for(const failures of [1,2]){
  const fixture=scenario([...Array(failures).fill(windowsTimeout),success],{retryTimeouts:true});
  assert.deepEqual(fixture.run(),{total_count:0,jobs:[]});
  assert.equal(fixture.calls.length,failures+1);
  assert.deepEqual(fixture.pauses,[2000,4000].slice(0,failures));
  for(const call of fixture.calls)assert.deepEqual([call.file,call.args],['gh',['api',endpoint]]);
  for(const report of fixture.reports){
   assert.equal(report.status,null);assert.equal(report.signal,'SIGTERM');
   assert.equal(report.timedOut,true);assert.equal(report.cleanupComplete,false);
   assert.equal(report.error,'ETIMEDOUT');assert.equal(report.errorDetails.syscall,'spawnSync gh');
   assert(!JSON.stringify(report).includes('fixture-secret'));
  }
 }
});
test('opt-in timeout exhaustion preserves the last actual error after exactly three GETs',()=>{
 const last={...windowsTimeout,signal:'SIGKILL',
  error:{...windowsTimeout.error,message:'last fixture-secret timeout',errno:17},
  stdout:Buffer.from('last observed stdout'),stderr:Buffer.from('last observed stderr')};
 const fixture=scenario([windowsTimeout,windowsTimeout,last],{retryTimeouts:true});
 assert.throws(fixture.run,error=>{
  assert(error instanceof GithubObservationUnavailable);
  assert.equal(error.detail.attempt,3);assert.equal(error.detail.status,null);
  assert.equal(error.detail.signal,'SIGKILL');assert.equal(error.detail.cleanupComplete,false);
  assert.equal(error.detail.error,'ETIMEDOUT');assert.equal(error.detail.errorDetails.errno,17);
  assert.equal(error.detail.errorDetails.message,'last *** timeout');
  assert.equal(error.detail.stdout,'last observed stdout');assert.equal(error.detail.stderr,'last observed stderr');
  return true;
 });
 assert.equal(fixture.calls.length,3);assert.deepEqual(fixture.pauses,[2000,4000]);
});
test('one absolute deadline clamps subprocess and pause and prevents further reads',()=>{
 let clock=100;
 const fixture=scenario([windowsTimeout],{retryTimeouts:true,deadlineMilliseconds:150,now:()=>clock,
  transport:(file,args,options)=>{
   fixture.calls.push({file,args,options});clock+=20;return windowsTimeout;
  },pause:milliseconds=>{fixture.pauses.push(milliseconds);clock+=milliseconds;}});
 assert.throws(fixture.run,error=>error instanceof GithubObservationUnavailable
  &&error.detail.reason==='deadline-expired'&&error.detail.attempt===1
  &&error.detail.error==='ETIMEDOUT'&&error.detail.cleanupComplete===false);
 assert.equal(fixture.calls.length,1);assert.equal(fixture.calls[0].options.timeout,50);
 assert.deepEqual(fixture.pauses,[30]);
 const expired=scenario([success],{retryTimeouts:true,deadlineMilliseconds:150,now:()=>clock});
 assert.throws(expired.run,error=>error instanceof GithubObservationUnavailable&&error.detail.attempt===0);
 assert.equal(expired.calls.length,0);
});
test('a shared deadline also rejects late success without accepting metadata',()=>{
 let clock=100;
 const fixture=scenario([],{retryTimeouts:true,deadlineMilliseconds:150,now:()=>clock,
  transport:(file,args,options)=>{fixture.calls.push({file,args,options});clock=150;return success;}});
 assert.throws(fixture.run,error=>error instanceof GithubObservationUnavailable
  &&error.detail.reason==='deadline-expired'&&error.detail.status===0&&error.detail.attempt===1);
 assert.equal(fixture.calls.length,1);assert.equal(fixture.pauses.length,0);
});
test('timeout opt-in never retries nontransient, contradictory or unsafe observations',()=>{
 for(const value of [failure(401),failure(403),failure(404),
  {...success,stdout:Buffer.from('malformed')},{...success,stdout:Buffer.from([0xff])},
  {...success,stdout:Buffer.alloc(2097153)},
  {...windowsTimeout,status:1},{...windowsTimeout,signal:null},{...windowsTimeout,signal:'SIGUSR1'},
  {...windowsTimeout,timed_out:false},{...windowsTimeout,error:{code:'ECONNRESET'}},
  {...failure(504),cleanup_complete:false},
  ...[401,403,404].map(code=>({...windowsTimeout,stderr:Buffer.from('HTTP '+code)}))]){
  const fixture=scenario([value],{retryTimeouts:true});
  assert.throws(fixture.run);assert.equal(fixture.calls.length,1);assert.equal(fixture.pauses.length,0);
  assert.deepEqual(fixture.calls[0].args,['api',endpoint]);
 }
 for(const unsafe of ['-X POST','https://foreign.invalid','repos/owner/repo\n-X DELETE']){
  let calls=0;
  assert.throws(()=>readGithubJson(unsafe,{retryTimeouts:true,transport:()=>{calls++;return success;}}));
  assert.equal(calls,0);
 }
});
test('retry settings and injected clocks require finite bounded values before transport',()=>{
 for(const extra of [
  {retryTimeouts:'true'},{retryTimeouts:1},{deadlineMilliseconds:NaN},{deadlineMilliseconds:Infinity},
  {deadlineMilliseconds:-1},{deadlineMilliseconds:1.5},{now:null},{now:()=>NaN},{now:()=>Infinity},
  {now:()=>-1},{now:()=>1.5},
 ]){
  const fixture=scenario([success],extra);
  assert.throws(fixture.run);assert.equal(fixture.calls.length,0);
 }
});
