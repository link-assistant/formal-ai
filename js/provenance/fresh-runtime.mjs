// Fresh real planner custody. Source observations are never a claim of replay/autonomy.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {blake3Hex} from '../crypto/blake3.mjs';
import {resolveEvidenceDirectory}from './drive-capture.mjs';
const repository=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
function readBoundedRuntimeFile(directory,relative,maximum){
 const parts=relative.split('/');let filename=fs.realpathSync(directory);
 for(let index=0;index<parts.length;index++){
  filename=path.join(filename,parts[index]);const state=fs.lstatSync(filename);
  if(state.isSymbolicLink())throw Error('RuntimeArchiveSymlink');
  if(index<parts.length-1&&!state.isDirectory())throw Error('RuntimeArchiveDirectory');
 }
 if(typeof fs.constants.O_NOFOLLOW!=='number')throw Error('RuntimeArchiveNoFollowUnavailable');
 const descriptor=fs.openSync(filename,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
 try{
  const before=fs.fstatSync(descriptor,{bigint:true});
  if(!before.isFile()||before.size>BigInt(maximum)||before.size<0n)throw Error('RuntimeArchiveSizeOrType');
  const length=Number(before.size),bytes=Buffer.alloc(length);let offset=0;
  while(offset<length){
   const count=fs.readSync(descriptor,bytes,offset,length-offset,null);
   if(count===0)throw Error('RuntimeArchiveTruncated');offset+=count;
  }
  const after=fs.fstatSync(descriptor,{bigint:true}),current=fs.lstatSync(filename,{bigint:true});
  if(current.isSymbolicLink()||current.dev!==before.dev||current.ino!==before.ino
    ||after.size!==before.size||after.mtimeNs!==before.mtimeNs||after.ctimeNs!==before.ctimeNs
    ||current.size!==before.size||current.mtimeNs!==before.mtimeNs||current.ctimeNs!==before.ctimeNs)throw Error('RuntimeArchiveChanged');
  return bytes;
 }finally{fs.closeSync(descriptor);}
}
export function collectRuntimeObservation(directory){
 const sources=[],events=[];
 let journals=0;
 let total=0;
 for(const name of fs.readdirSync(directory).filter(name=>/^cjs-[0-9]+\.jsonl$/u.test(name)).sort()){
  journals++;
  const text=readBoundedRuntimeFile(directory,name,2*1024*1024).toString('utf8');
  if(Buffer.byteLength(text)>2*1024*1024||!text.endsWith('\n'))throw Error('RuntimeJournalIncomplete');
  const rows=[];for(const line of text.slice(0,-1).split('\n'))rows.push(JSON.parse(line));
  for(let index=0;index<rows.length;index++){
   const row=rows[index];if(row.sequence!==index)throw Error('RuntimeJournalSequence');
   if(row.kind!=='source'){events.push({journal:name,...row});continue;}
   if(!/^objects\/[0-9a-f]{64}\.src$/u.test(row.file))throw Error('RuntimeSourcePath');
   const bytes=readBoundedRuntimeFile(directory,row.file,Math.min(4*1024*1024,32*1024*1024-total));
   total+=bytes.length;if(total>32*1024*1024||bytes.length!==row.size)throw Error('RuntimeSourceBudgetOrSize');
   if(crypto.createHash('sha256').update(bytes).digest('hex')!==row.sha256)throw Error('RuntimeSourceDigest');
   sources.push({url:row.url,format:row.format,boundary:row.boundary,blake3:blake3Hex(bytes),size:bytes.length,
    bytes:bytes.toString('base64')});
  }
 }
 if(journals===0)throw Error('RuntimeJournalMissing');
 return {schema:'formal-ai-loaded-runtime-v1',scope:'actual-node-load-and-compile-inputs',sources,events,
  runtime:process.version,trust:'bootstrap-observation-requires-independent-trust',
  fullReplay:{verified:false,reason:'PinnedFreshReplayNotYetPerformed'}};
}
export function captureFreshPlannerRun(options=null){
 if(!options||typeof options!=='object')throw Error('FreshRuntimeRequestInvalid');
 const {directory,workspace,prompt,steps=12,signing}=options;
 if(typeof directory!=='string'||typeof workspace!=='string'||typeof prompt!=='string'
   ||!Number.isSafeInteger(steps)||steps<1||steps>100)throw Error('FreshRuntimeRequestInvalid');
 const work=fs.realpathSync(workspace);
 const parent=resolveEvidenceDirectory(directory,work,'FreshRuntimeEvidenceInsideWorkspace');
 fs.mkdirSync(parent,{recursive:true,mode:0o700});
 const owned=fs.mkdtempSync(path.join(parent,'runtime-'));fs.chmodSync(owned,0o700);
 const transportDirectory=path.join(owned,'transport');fs.mkdirSync(transportDirectory,{mode:0o700});
 const copies=[['runtime-journal.js','bootstrap.cjs'],['runtime-loader.mjs','loader.mjs'],['fresh-planner-main.mjs','main.mjs']];
 const bootstrap=[];
 for(const[source,target]of copies){
  const bytes=fs.readFileSync(path.join(repository,'js/provenance',source));
  const file=path.join(owned,target);fs.writeFileSync(file,bytes,{flag:'wx',mode:0o400});
  bootstrap.push({source:'js/provenance/'+source,file:target,blake3:blake3Hex(bytes),size:bytes.length});
 }
 const resultFile=path.join(owned,'result.json'),configFile=path.join(owned,'config.json');
 const configuration={repository,workspace:work,prompt,steps,transportDirectory,resultFile,
  provenance:{directory:path.join(owned,'evidence'),...(signing?{signing}:{})}};
 fs.writeFileSync(configFile,JSON.stringify(configuration),{flag:'wx',mode:0o600});
 const child=spawnSync(process.execPath,['--require',path.join(owned,'bootstrap.cjs'),
  '--experimental-loader',path.join(owned,'loader.mjs'),path.join(owned,'main.mjs')],{
  cwd:repository,encoding:'utf8',timeout:300000,maxBuffer:2*1024*1024,
  env:{...process.env,NODE_OPTIONS:'',NODE_TEST_CONTEXT:undefined,
   FORMAL_AI_RUNTIME_CONFIG:configFile,FORMAL_AI_RUNTIME_BOOTSTRAP:path.join(owned,'bootstrap.cjs')}});
 fs.writeFileSync(path.join(owned,'stdout.log'),child.stdout??'',{mode:0o600});
 fs.writeFileSync(path.join(owned,'stderr.log'),child.stderr??'',{mode:0o600});
 const processReceipt={status:child.status,signal:child.signal,error:child.error?.message??null};
 fs.writeFileSync(path.join(owned,'process.json'),JSON.stringify(processReceipt),{mode:0o600});
 if(child.status!==0)throw Object.assign(Error('FreshRuntimeProcessFailed'),{directory:owned,processReceipt});
 for(const entry of bootstrap){if(blake3Hex(fs.readFileSync(path.join(owned,entry.file)))!==entry.blake3)throw Error('FreshRuntimeBootstrapChanged');}
 const observed=JSON.parse(fs.readFileSync(resultFile,'utf8'));
 const final=collectRuntimeObservation(transportDirectory);
 const manifest={schema:'formal-ai-fresh-runtime-custody-v1',bootstrap,processReceipt,
  loadedSources:final.sources.length,engineProfile:observed.engineProfile,
  fullReplay:{verified:false,reason:'PinnedFreshReplayNotYetPerformed'},authorship:'not-established'};
 fs.writeFileSync(path.join(owned,'manifest.json'),JSON.stringify(manifest),{mode:0o600});
 return {directory:owned,...observed,custody:manifest};
}
