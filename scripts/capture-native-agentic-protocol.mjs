#!/usr/bin/env node
// Retain complete actual native sessions; unavailable outcomes remain failures.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {buildIdentity,verifyReceipt} from './native-test-artifact.mjs';
const CONTRACT='docs/case-studies/pull-request-1188/native-protocol-capture-provenance.json';
export const bytesDigest=bytes=>createHash('sha256').update(bytes).digest('hex');
export function checkedCapture(bytes,legacy){
 const session=JSON.parse(bytes.toString('utf8'));
 assert.equal(session.task,legacy.task);
 assert.ok(Array.isArray(session.tools_advertised));
 assert.equal(new Set(session.tools_advertised).size,session.tools_advertised.length);
 assert.ok(session.tools_advertised.every(tool=>typeof tool==='string'));
 assert.ok(Array.isArray(session.steps));
 assert.ok(Number.isInteger(session.turns)&&session.turns>0);
 assert.equal(session.hit_turn_cap,false);
 assert.equal(typeof session.final_answer,'string');
 return session;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const [directory,binaryPath]=process.argv.slice(2);assert.ok(directory&&binaryPath);
 const cwd=process.cwd(),binary=resolve(binaryPath),identity=buildIdentity(cwd,'release','all');
 verifyReceipt(resolve('dist/tests'),identity,binary);mkdirSync(directory,{recursive:true});
 const contract=JSON.parse(readFileSync(CONTRACT,'utf8')),results=[];
 const retain=()=>writeFileSync(join(directory,'provenance.json'),JSON.stringify({version:1,identity,run:process.env.GITHUB_RUN_ID,attempt:process.env.GITHUB_RUN_ATTEMPT,job:process.env.GITHUB_JOB,captures:results},null,2)+'\n');
 retain();
 for(const record of contract.captures){
  const original=readFileSync(record.legacy);assert.equal(bytesDigest(original),record['legacy-sha256']);
  const legacy=JSON.parse(original),path=resolve(directory,record.id+'.json');
  const result=spawnSync(binary,['agent','--task',legacy.task,'--session-json',path],{cwd,encoding:'utf8',timeout:240000});
  writeFileSync(join(directory,record.id+'.stdout.log'),result.stdout??'');
  writeFileSync(join(directory,record.id+'.stderr.log'),result.stderr??String(result.error??''));
  const observed={id:record.id,'legacy-sha256':record['legacy-sha256'],exit:result.status,signal:result.signal};
  try{assert.equal(result.status,0);assert.equal(result.signal,null);if(result.error)throw result.error;
   const bytes=readFileSync(path),session=checkedCapture(bytes,legacy);
   observed['session-sha256']=bytesDigest(bytes);observed['tools-advertised']=session.tools_advertised;observed.status='captured';
  }catch(error){observed.status='failed';observed.error=String(error);}
  results.push(observed);retain();
 }
 assert.ok(results.every(result=>result.status==='captured'),'Every original task requires a complete actual native capture');
}
