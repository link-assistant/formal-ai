import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {readBoundedFileBytes,readBoundedFilePath} from '../../../js/provenance/bounded-file.mjs';
import {runLogsCli,MAXIMUM_LOG_FILE_BYTES} from '../../../js/server/logs-cli.mjs';
function fixture(run){const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-bounded-logs-'));try{return run(directory);}finally{fs.rmSync(directory,{recursive:true,force:true});}}
test('oversized metadata refuses a payload read and actual logs CLI uses that bound',()=>fixture(directory=>{
 const file=path.join(directory,'large.jsonl');const descriptor=fs.openSync(file,'w');fs.ftruncateSync(descriptor,MAXIMUM_LOG_FILE_BYTES+1);fs.closeSync(descriptor);
 let reads=0;const io={...fs,readSync(){reads++;throw Error('payload read forbidden');}};
 assert.throws(()=>readBoundedFilePath(file,MAXIMUM_LOG_FILE_BYTES,{io}),/SizeOrType/);assert.equal(reads,0);
 const original=fs.readSync;fs.readSync=io.readSync;
 try{assert.deepEqual(runLogsCli([file]),{code:2,stdout:'',stderr:'log_conversion_refused\n'});assert.equal(reads,0);}finally{fs.readSync=original;}
}));
test('growth and replacement races are refused without reading beyond the admitted allocation',()=>fixture(directory=>{
 for(const mode of ['growth','replacement']){
  const file=path.join(directory,mode);fs.writeFileSync(file,'original');let readBytes=0;
  const io={...fs,readSync(...args){const count=fs.readSync(...args);readBytes+=count;if(mode==='growth')fs.appendFileSync(file,'growth');else{fs.renameSync(file,file+'.old');fs.writeFileSync(file,'original');}return count;}};
  assert.throws(()=>readBoundedFilePath(file,100,{io}),/Changed/);assert.equal(readBytes,8);
 }
}));
test('partial reads make bounded progress while zero or invalid reads fail and close the descriptor',()=>fixture(directory=>{
 const file=path.join(directory,'small');fs.writeFileSync(file,'abcdef');let closed=0;
 const partial={...fs,readSync(descriptor,buffer,offset,length,position){return fs.readSync(descriptor,buffer,offset,Math.min(2,length),position);}};
 assert.equal(readBoundedFilePath(file,6,{io:partial}).toString(),'abcdef');
 for(const count of [0,-1,7,NaN]){
  const io={...fs,readSync(){return count;},closeSync(descriptor){closed++;fs.closeSync(descriptor);}};
  assert.throws(()=>readBoundedFilePath(file,6,{io}),/Truncated/);
 }
 assert.equal(closed,4);
}));
test('public log symlinks retain stable-target support while private archives retain no-follow protection',()=>fixture(directory=>{
 const file=path.join(directory,'target'),link=path.join(directory,'public.jsonl');fs.writeFileSync(file,'{"event":"ready"}\n');fs.symlinkSync(file,link);
 assert.equal(runLogsCli([link,'--format','json']).code,0);
 assert.throws(()=>readBoundedFileBytes(directory,'public.jsonl',100),/Symlink/);
 const io={...fs,constants:{O_RDONLY:fs.constants.O_RDONLY}};
 assert.equal(readBoundedFilePath(link,100,{allowSymlink:true,io}).toString(),'{"event":"ready"}\n');
 assert.throws(()=>readBoundedFilePath(file,100,{io}),/NoFollowUnavailable/);
}));
test('real conversion, redaction, explicit format and injected legacy readers preserve behavior',()=>fixture(directory=>{
 const secret='bounded-log-private-token',file=path.join(directory,'small.jsonl');fs.writeFileSync(file,JSON.stringify({event:'ready',password:secret,message:secret})+'\n');
 const env={GH_TOKEN:secret};const json=runLogsCli([file,'--format','json'],{env});assert.equal(json.code,0);assert.ok(!json.stdout.includes(secret));assert.ok(json.stdout.includes('[REDACTED]'));
 const lino=runLogsCli([file,'--format','lino'],{env});assert.equal(lino.code,0);const typed=path.join(directory,'small.lino');fs.writeFileSync(typed,lino.stdout);assert.deepEqual(JSON.parse(runLogsCli([typed,'--format','json'],{env}).stdout),JSON.parse(json.stdout));
 assert.equal(runLogsCli(['synthetic.jsonl','--format','json'],{readFile:()=>'{"event":"ready"}\n'}).code,0);
 for(const maximum of [-1,NaN,0.5,Infinity])assert.throws(()=>readBoundedFilePath(file,maximum),/InvalidLimit/);
}));
