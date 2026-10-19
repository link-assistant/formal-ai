// Recompute from caller-authenticated retained engine inputs, never installed planner bytes.
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath}from 'node:url';
import {spawnSync}from 'node:child_process';
import {sha256Hex as digest}from '../agentic/crate/source_fetch.mjs';
import {authenticatedRuntimeArchive}from './archive-runtime.mjs';
import {collectRuntimeObservation}from './fresh-runtime.mjs';
import {regularEvidenceBytes,retainedEvidenceBytes,firstEvidenceRecord}from './verify-drive.mjs';
import {readBoundedFileBytes}from './bounded-file.mjs';
import {decodeTyped}from '../logging/typed-codec.mjs';

export function recomputeArchivedRun(options){
 const archive=authenticatedRuntimeArchive(options);
 const records=regularEvidenceBytes(options.directory,'chain.lino').toString('utf8').trimEnd().split('\n').map(decodeTyped);
 if(!firstEvidenceRecord(records,'caller_request'))throw Error('ArchivedRuntimeCallerRequestRequired');
 const genesis=firstEvidenceRecord(records,'logical_memory_genesis'),first=firstEvidenceRecord(records,'message_in');
 if(!genesis||!first)throw Error('ArchivedRuntimeGenesisAndInputRequired');
 const context=archive.context,owned=fileSystem.mkdtempSync(path.join(os.tmpdir(),'formal-ai-archived-replay-'));
 fileSystem.chmodSync(owned,0o700);
 try{
  const memoryFile=path.join(owned,'memory.lino');
  fileSystem.writeFileSync(memoryFile,retainedEvidenceBytes(options.directory,genesis.payload.evidence),{flag:'wx',mode:0o600});
  if(genesis.payload.revisionEvidence!==null)fileSystem.writeFileSync(memoryFile+'.provenance-revision.lino',
   retainedEvidenceBytes(options.directory,genesis.payload.revisionEvidence),{flag:'wx',mode:0o600});
  for(const[name,entry]of Object.entries(archive.bootstrap)){
   if(!entry||typeof entry!=='object'||!('bytes'in entry)||typeof entry.bytes!=='string')throw Error('ArchivedRuntimeBootstrapShape');
   fileSystem.writeFileSync(path.join(owned,name),Buffer.from(entry.bytes,'base64'),{flag:'wx',mode:0o400});
  }
  const hooks={};
  for(const[name,target]of [['archive-preload.js','archive-preload.cjs'],['archive-loader.mjs','archive-loader.mjs']]){
   const filename=fileURLToPath(new URL(name,import.meta.url)),bytes=readBoundedFileBytes(path.dirname(filename),path.basename(filename),256*1024);
   hooks[name]=digest(bytes);fileSystem.writeFileSync(path.join(owned,target),bytes,{flag:'wx',mode:0o400});
  }
  const input=JSON.parse(retainedEvidenceBytes(options.directory,first.payload.evidence).toString('utf8'));
  const replayInput=path.join(owned,'input.json');
  fileSystem.writeFileSync(replayInput,JSON.stringify({directory:options.directory,workspace:options.workspace,
   expectedAnchor:options.expectedAnchor,trustedPublicKey:options.trustedPublicKey,tools:input.tools}),{flag:'wx',mode:0o600});
  const transportDirectory=path.join(owned,'transport');fileSystem.mkdirSync(transportDirectory,{mode:0o700});
  const resultFile=path.join(owned,'result.json'),configurationFile=path.join(owned,'config.json');
  const originalRuntime=fileSystem.realpathSync(options.bootstrapDirectory??path.dirname(path.dirname(options.directory)));
  const configuration={repository:archive.repository,workspace:fileSystem.realpathSync(options.workspace),
   ...context.request,requests:context.requests,atomicRecordAppend:context.driver?.atomicRecordAppend??false,
   memoryFile,taskURL:options.expectedTaskURL,replayInput,transportDirectory,resultFile,
   runtimeExecutable:{path:fileSystem.realpathSync(process.execPath),version:process.version,platform:process.platform,architecture:process.arch},
   bootstrapArtifacts:context.bootstrapArtifacts,sourceArchive:archive};
  fileSystem.writeFileSync(configurationFile,JSON.stringify(configuration),{flag:'wx',mode:0o600});
  const child=spawnSync(process.execPath,['--max-old-space-size=512','--require',path.join(owned,'archive-preload.cjs'),
   '--require',path.join(owned,'bootstrap.cjs'),'--experimental-loader',path.join(owned,'archive-loader.mjs'),path.join(originalRuntime,'entry.cjs')],{
   cwd:archive.repository,encoding:'utf8',timeout:300000,maxBuffer:2*1024*1024,
   env:{...context.environment,FORMAL_AI_RUNTIME_CONFIG:configurationFile,FORMAL_AI_RUNTIME_BOOTSTRAP:path.join(owned,'bootstrap.cjs')}});
  const processReceipt={status:child.status,signal:child.signal,error:child.error?.message??null};
  if(child.status!==0)throw Object.assign(Error('ArchivedRuntimeProcessFailed'),{processReceipt,stderr:child.stderr});
  const result=JSON.parse(readBoundedFileBytes(owned,'result.json',2*1024*1024).toString('utf8'));
  if(result.result?.stop!=='recomputed'||result.result.recomputation?.outputCount<1)throw Error('ArchivedRuntimeRecomputationIncomplete');
  const observation=collectRuntimeObservation(transportDirectory),consumed=new Set();
  for(const source of observation.sources){
   const isRead=source.format.startsWith('repository-read-');
   const key=isRead?source.url+'\0'+source.format:source.url;
   const entry=isRead?archive.reads[key]:archive.modules[key];
   if(!entry||(!isRead&&entry.format!==source.format)||entry.bytes!==source.bytes||entry.size!==source.size)throw Error('ArchivedRuntimeObservedInputMismatch');
   consumed.add((isRead?'read:':'module:')+key);
  }
  const missing=[...Object.keys(archive.modules).map(key=>'module:'+key),...Object.keys(archive.reads).map(key=>'read:'+key)].filter(key=>!consumed.has(key));
  if(missing.length){
   // Diagnostic identities contain no filenames, archived bytes or caller values.
   const missingRuntimeInputs=missing.slice(0,64).map(key=>{
    const read=key.startsWith('read:'),identity=key.slice(read?5:7);
    const entry=read?archive.reads[identity]:archive.modules[identity];
    const representation=read?identity.slice(identity.lastIndexOf('\0')+1):entry.format;
    const format=['module','commonjs','json','repository-read-string','repository-read-buffer'].includes(representation)?representation:'unsupported';
    return{kind:read?'read':'module',format,sha256:entry.sha256,size:entry.size};
   });
   throw Object.assign(Error('ArchivedRuntimeExpectedInputNotConsumed'),{missing,missingRuntimeInputs,missingRuntimeInputCount:missing.length});
  }
  return {schema:'formal-ai-archived-recomputation-v1',recomputation:result.result.recomputation,processReceipt,
   consumedInputs:{modules:Object.keys(archive.modules).length,reads:Object.keys(archive.reads).length,actual:observation.sources.length},
   trustedTransportHooks:hooks,sourceIdentity:options.expectedSourceIdentity,runtimeSha256:options.expectedRuntimeSha256,
   scope:'authenticated-retained-js-engine-and-recorded-tool-inputs',externalEffects:'not-reexecuted',
   fullReplay:{verified:false,reason:'CallerMustComposeActualRepositoryAndCheckoutProof'},autonomousAuthorship:'not-established'};
 }finally{fileSystem.rmSync(owned,{recursive:true,force:true});}
}

// Authority is restricted to the authenticated runtime replay; repository evidence is composed separately.
export function verifyArchivedFreshReplay(options){
 const observed=recomputeArchivedRun(options);
 return {taskURL:options.expectedTaskURL,sourceIdentity:observed.sourceIdentity,
  runtime:{sha256:observed.runtimeSha256,version:process.version},recomputation:observed.recomputation,
  fullReplay:{verified:true,scope:'authenticated-retained-js-engine-logical-memory-and-recorded-tool-inputs',
   consumedInputs:observed.consumedInputs},repository:'not-checked',autonomousAuthorship:'not-established'};
}
