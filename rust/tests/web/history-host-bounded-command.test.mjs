import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {nodeHistoryIo as historyIo} from '../../../js/agentic/crate/history_store.mjs';
import {nodeHistoryIo} from '../../../js/server/history-host.mjs';
const receipt=()=>({status:0,signal:null,error:undefined,timed_out:false,cleanup_complete:true,stdout:'original\n',stderr:''});
test('legacy childProcess injection cannot execute Git without a bounded runner',()=>{
 let calls=0;assert.throws(()=>historyIo({fs,path,childProcess:{execFileSync(){calls++;}}}),{name:'TypeError',message:'HistoryCommandRunnerRequired'});assert.equal(calls,0);
});
test('history checks caller-trusted runner receipt and exact finite command policy',()=>{
 let observation;
 const io=historyIo({fs,path,runCommand(...values){observation=values;return receipt();}});
 assert.equal(io.runGit('/observed/root',['show','HEAD:source.rs']),'original\n');
 assert.deepEqual(observation,['git',['show','HEAD:source.rs'],{cwd:'/observed/root',encoding:'utf8',timeout:30000,stdoutLimitBytes:64*1024*1024,stderrLimitBytes:16*1024*1024}]);
 for(const mutation of [{error:new Error('timeout')},{status:7},{signal:'SIGKILL'},{timed_out:true},{cleanup_complete:false},{stdout:new Uint8Array([1])},{stderr:null}]){
  const invalid=historyIo({fs,path,runCommand(){return {...receipt(),...mutation};}});assert.throws(()=>invalid.runGit('/root',[]),{message:'HistoryGitCommandFailed'});
 }
 for(const value of [null,undefined,{},Promise.resolve(receipt())])assert.throws(()=>historyIo({fs,path,runCommand:()=>value}).runGit('/root',[]),{message:'HistoryGitCommandFailed'});
});
test('actual Node adapter observes real Git output and refuses nonzero Git status',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-history-bounded-'));
 try{const io=nodeHistoryIo();const version=io.runGit(directory,['--version']);assert.match(version,/^git version /u);assert.throws(()=>io.runGit(directory,['rev-parse','HEAD']),{message:'HistoryGitCommandFailed'});}finally{fs.rmSync(directory,{recursive:true,force:true});}
});
