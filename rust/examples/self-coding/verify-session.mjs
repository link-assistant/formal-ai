#!/usr/bin/env node
// Current creation evidence reuses production validation; historical captures stay immutable.
import assert from 'node:assert/strict';
import {readFileSync,openSync,readSync,closeSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {checkedCapture,captureVerificationStdout} from '../../../scripts/capture-native-agentic-protocol.mjs';
import {installNativeCaptureObservationHost,nativeLiteralCaptureObservation} from '../../../scripts/lib/native-capture-observation.mjs';

const legacy=JSON.parse(readFileSync(new URL('../../../docs/case-studies/issue-651/self-coding-run/session.json',import.meta.url),'utf8'));
export const task='Create file self-coding-result.txt containing self-coding=passed';
export const target='self-coding-result.txt';
export const content='self-coding=passed';

export function verifySelfCodingSession(bytes,physicalBytes){
 assert.ok(bytes.length<=1048576,"Session exceeds bounded proof input");
 assert.equal(legacy.task,task);
 const write=legacy.steps.filter(step=>step.tool==='write_file').at(-1);
 assert.equal(write.arguments.path,target);
 assert.equal(write.arguments.content,content);
 const verification=legacy.steps.filter(step=>step.tool==='run_command').at(-1);
 const session=checkedCapture(bytes,legacy);
 assert.ok(nativeLiteralCaptureObservation(session,legacy,{
  command:verification.arguments.command,stdout:captureVerificationStdout(verification),
 }),'Current replay requires a successful exact Write followed by a fresh owned complete observation');
 if(physicalBytes!==undefined)assert.deepEqual(Buffer.from(physicalBytes),Buffer.from(content),'Physical target must retain every authored byte');
 return session;
}

function boundedInput(path,limit){
 const fd=path==='-'?0:openSync(path,'r'),buffer=Buffer.alloc(limit+1);
 let length=0;
 try{
  while(length<buffer.length){
   const count=readSync(fd,buffer,length,Math.min(4096,buffer.length-length),null);
   if(count===0)break;
   length+=count;
  }
  assert.ok(length<=limit,'Proof input exceeds its byte bound');
  return buffer.subarray(0,length);
 }finally{if(fd!==0)closeSync(fd);}
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 try{
  await installNativeCaptureObservationHost();
  const [sessionPath,physicalPath]=process.argv.slice(2);
  assert.ok(sessionPath,'A current session is required');
  verifySelfCodingSession(boundedInput(sessionPath,1048576),
   physicalPath===undefined?undefined:boundedInput(physicalPath,Buffer.byteLength(content)));
 }catch(error){
  process.stderr.write(('SelfCodingProofRejected: '+String(error.message)).slice(0,1024)+'\n');
  process.exitCode=1;
 }
}
