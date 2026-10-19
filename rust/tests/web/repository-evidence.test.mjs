import test, {before} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {execute} from '../../../experiments/js_dogfood/drive.mjs';
import {provenancePrimitives} from '../../../js/provenance/session.mjs';
import {blake3Hex} from '../../../js/crypto/blake3.mjs';
import {encodeTyped,decodeTyped} from '../../../js/logging/typed-codec.mjs';
import {openInstallationSigner} from '../../../js/provenance/installation-key.mjs';
import {observeRepositoryCommit,commitPrefix} from '../../../js/provenance/git-evidence.mjs';
import {verifyRepositoryEvidence} from '../../../js/provenance/repository-verifier.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
before(async()=>{await installNodeHost(new WorkerHost({WebAssembly:{
 compile(){throw Error('LocalProductWasmForbidden');},
 instantiate(){throw Error('LocalProductWasmForbidden');}
}}));});
const primitives=provenancePrimitives({encodeTyped,decodeTyped});
const blob=bytes=>bytes===null?null:{size:bytes.length,blake3:blake3Hex(bytes),
 gitBlobSha1:createHash('sha1').update(Buffer.from('blob '+bytes.length+'\0')).update(bytes).digest('hex')};
const quote=text=>"'"+text.replaceAll("'","'\\''")+"'";
function fixture(run){
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-git-evidence-')),work=path.join(root,'work');
 fs.mkdirSync(work);
 const git=args=>execFileSync('git',args,{cwd:work,encoding:'utf8',env:{...process.env,GIT_CONFIG_GLOBAL:os.devNull,GIT_CONFIG_SYSTEM:os.devNull},maxBuffer:1024*1024}).trim();
 try{
  git(['init','--quiet']);git(['config','user.name','Evidence Test']);git(['config','user.email','evidence@example.invalid']);
  git(['commit','--quiet','--allow-empty','-m','base']);
  const base=git(['rev-parse','HEAD']);
  const session=primitives.createSession({sessionId:'test-session',contextId:'executor-control',initialMemory:Buffer.from('initial'),runtimeId:process.version,sourceId:'test-only-supplied-observer'});
  const commits=[];
  for(let index=1;index<=3;index++){
   const file='value.mjs',full=path.join(work,file),before=fs.existsSync(full)?fs.readFileSync(full):null,content='export const value = '+index+';\n';
   const writePlan={tool:'write',arguments:JSON.stringify({filePath:file,content})};
   const output=session.append('message_out',{plan:writePlan});
   assert.equal(execute(work,writePlan),'');
   const edit=session.append('edit',{path:file,preimage:blob(before),postimage:blob(fs.readFileSync(full)),messageOut:output.hash,scope:'test-actual-driver-write'});
   const commandOutput=session.append('message_out',{operation:'git-commit'});
   const precommit=decodeTyped(session.lines().at(-1));
   const command='git add -- value.mjs && git commit --quiet -m '+quote('change '+index)+' -m '+quote('Formal-AI-Chain: '+commitPrefix(precommit));
   const result=execute(work,{tool:'bash',arguments:JSON.stringify({command})});
   assert.match(result,/Exit Code: 0/u);
   const commit=git(['rev-parse','HEAD']);commits.push(commit);
   session.append('commit',observeRepositoryCommit(work,commit,{messageOut:commandOutput.hash,editHashes:[edit.hash],precommitAnchor:precommit}));
  }
  session.append('terminal',{reason:'control-completed'});
  const lines=session.lines(),expectedAnchor=session.anchor(),signer=openInstallationSigner(path.join(root,'keys'),{excludedDirectory:work});
  const options={lines,expectedAnchor,trustedPublicKey:signer.publicKey,seal:signer.seal(lines,expectedAnchor),base,head:commits.at(-1),minShare:1};
  const reseal=transform=>{
   const records=lines.map(decodeTyped),genesis=records[0];
   const replacement=primitives.createSession({sessionId:genesis.sessionId,contextId:genesis.contextId,
    initialMemory:Buffer.from(genesis.payload.initialMemory.value,'base64'),runtimeId:genesis.payload.runtimeId,sourceId:genesis.payload.sourceId});
   for(const record of records.slice(1))replacement.append(record.kind,transform(record,records));
   const changedLines=replacement.lines(),changedAnchor=replacement.anchor();
   return {...options,lines:changedLines,expectedAnchor:changedAnchor,seal:signer.seal(changedLines,changedAnchor)};
  };
  return run({root,work,git,commits,options,reseal});
 }finally{fs.rmSync(root,{recursive:true,force:true});}
}
test('three actual driver writes and Git commits bind every changed blob and line to an anchored signed chain',()=>fixture(({work,commits,options})=>{
 const report=verifyRepositoryEvidence(work,options);
 assert.equal(report.commits.length,3);assert.deepEqual(report.commits.map(row=>row.commit),commits);
 assert.equal(report.allBlobsAttributed,true);assert.equal(report.files.length,3);
 assert.ok(report.files.every(row=>row.status==='causally-observed'));assert.equal(report.share,1);assert.equal(report.meetsMinimum,true);
 assert.equal(report.provenChangedLines,5);assert.equal(report.totalChangedLines,5);
 assert.equal(report.replay,'not-performed');assert.equal(report.autonomousAuthorship,'not-established');
}));
test('one subsequent foreign source byte cannot disappear from actual commit and line coverage',()=>fixture(({work,git,options})=>{
 fs.appendFileSync(path.join(work,'value.mjs'),' ');git(['add','--','value.mjs']);git(['commit','--quiet','-m','foreign-byte']);
 const report=verifyRepositoryEvidence(work,{...options,head:git(['rev-parse','HEAD'])});
 assert.equal(report.commits.length,4);assert.equal(report.files.at(-1).status,'foreign');
 assert.equal(report.files.at(-1).reason,'missing-commit-observation');assert.equal(report.allBlobsAttributed,false);
 assert.ok(report.share<1);assert.equal(report.meetsMinimum,false);
}));
test('deleted, reordered, altered chain records and an untrusted signer fail before attribution',()=>fixture(({work,options})=>{
 const missing=options.lines.slice();missing.splice(2,1);assert.throws(()=>verifyRepositoryEvidence(work,{...options,lines:missing}));
 const reordered=options.lines.slice();[reordered[2],reordered[3]]=[reordered[3],reordered[2]];assert.throws(()=>verifyRepositoryEvidence(work,{...options,lines:reordered}));
 const changed=options.lines.slice(),record=decodeTyped(changed[2]);record.payload.path='other.mjs';changed[2]=encodeTyped(record);assert.throws(()=>verifyRepositoryEvidence(work,{...options,lines:changed}));
 assert.throws(()=>verifyRepositoryEvidence(work,{...options,trustedPublicKey:'untrusted'}));
 assert.throws(()=>verifyRepositoryEvidence(work,{...options,minShare:1.01}));
 assert.throws(()=>verifyRepositoryEvidence(work,{...options,base:'HEAD~3'}));
}));
test('amending an attributed commit or its attribution trailer invalidates the original commit observation',()=>fixture(({work,git,options})=>{
 git(['commit','--quiet','--amend','-m','changed trailer','-m','Formal-AI-Chain: wrong@'+'0'.repeat(64)]);
 const report=verifyRepositoryEvidence(work,{...options,head:git(['rev-parse','HEAD'])});
 assert.equal(report.allBlobsAttributed,false);assert.equal(report.meetsMinimum,false);
 assert.equal(report.files.at(-1).status,'foreign');
}));
test('even a signed record cannot place an edit after the commit tool request it claims caused the commit',()=>fixture(({work,reseal})=>{
 const rebound=reseal((record,records)=>record.kind==='commit'&&record.payload.commit===records.filter(row=>row.kind==='commit').at(-1).payload.commit
  ?{...record.payload,messageOut:records.filter(row=>row.kind==='message_out').at(-2).hash}:record.payload);
 const report=verifyRepositoryEvidence(work,rebound);
 assert.equal(report.files.at(-1).status,'foreign');assert.equal(report.files.at(-1).reason,'commit-edit-window');assert.equal(report.meetsMinimum,false);
}));
test('a signed commit cannot reuse an earlier committed edit as a fresh executor write',()=>fixture(({work,reseal})=>{
 const rebound=reseal((record,records)=>record.kind==='commit'&&record.payload.commit===records.filter(row=>row.kind==='commit').at(-1).payload.commit
  ?{...record.payload,editHashes:[records.find(row=>row.kind==='edit').hash]}:record.payload);
 const report=verifyRepositoryEvidence(work,rebound);
 assert.equal(report.files.at(-1).status,'foreign');assert.equal(report.files.at(-1).reason,'commit-edit-window');assert.equal(report.meetsMinimum,false);
}));
