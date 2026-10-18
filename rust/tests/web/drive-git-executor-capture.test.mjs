import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash}from 'node:crypto';
import {WorkerHost}from '../../../js/server/worker-host.mjs';
import {installNodeHost}from '../../../js/agentic/node-host.mjs';
import {planChatStep}from '../../../js/agentic/planner.mjs';
import {shellCapture}from '../../../experiments/js_dogfood/shell-capture.mjs';
import {drive}from '../../../experiments/js_dogfood/drive.mjs';
import {withDriveCapture}from '../../../js/provenance/drive-capture.mjs';
import {repositoryGit,repositoryText}from '../../../js/provenance/git-evidence.mjs';
import {verifyRepositoryEvidence}from '../../../js/provenance/repository-verifier.mjs';
import {trustedInstallationKey}from '../../../js/provenance/installation-key.mjs';
import {retainedEvidenceBytes}from '../../../js/provenance/verify-drive.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
const git=(workspace,args)=>repositoryText(repositoryGit(workspace,args)).trimEnd();
test('real maintained planner and driver retain actual three commit process boundaries and foreign-byte refusal',async()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'executor-git-'));
 try{
  const workspace=path.join(directory,'work');fileSystem.mkdirSync(workspace);
  git(workspace,['init']);git(workspace,['config','user.name','Fixture']);git(workspace,['config','user.email','fixture@example.invalid']);
  fileSystem.writeFileSync(path.join(workspace,'base.txt'),'base\n');git(workspace,['add','base.txt']);git(workspace,['commit','-m','base']);
  const base=git(workspace,['rev-parse','HEAD']),keys=path.join(directory,'keys');
  await installNodeHost(new WorkerHost({WebAssembly:{
   compile(){throw Error('ProductWasmDisabled');},instantiate(){throw Error('ProductWasmDisabled');},
   instantiateStreaming(){throw Error('ProductWasmDisabled');}}},{provenance:true}));
  const observed=[];
  const result=await withDriveCapture({directory:path.join(directory,'proof'),signing:{directory:keys}},
   workspace,'Three explicitly declared fixture create requests and three independently owned Git commands',
   createHash('sha256').update(fileSystem.readFileSync(new URL('../../../experiments/js_dogfood/drive.mjs',import.meta.url))).digest('hex'),async()=>{
    for(let index=0;index<3;index++){
     const filename='heldout-'+index+'.mjs',content='export const observedValue = '+(37+index)+';';
     const created=await drive(planChatStep,workspace,'Create file '+filename+' containing '+content,{steps:12});
     assert.equal(created.stop,'final',JSON.stringify(created));assert.equal(fileSystem.readFileSync(path.join(workspace,filename),'utf8'),content);
     const command='git add -- '+filename+' && git commit -m "observed '+index+'" --trailer "Formal-AI-Chain: $FORMAL_AI_CHAIN_PREFIX"';
     const committed=await drive(planChatStep,workspace,'Run '+command,{steps:6,allowedCommands:[command]});
     assert.equal(committed.stop,'final',JSON.stringify(committed));observed.push(git(workspace,['rev-parse','HEAD']));
    }
    return{stop:'final'};
   });
  const evidence=result.provenance.directory,lines=fileSystem.readFileSync(path.join(evidence,'chain.lino'),'utf8').trimEnd().split('\n');
  const records=lines.map(decodeTyped),seal=decodeTyped(fileSystem.readFileSync(path.join(evidence,'seal.lino'),'utf8').trimEnd());
  const publicKey=trustedInstallationKey(keys),head=git(workspace,['rev-parse','HEAD']);
  const verified=verifyRepositoryEvidence(workspace,{lines,expectedAnchor:result.provenance.anchor,trustedPublicKey:publicKey,seal,base,head,minShare:1});
  assert.equal(verified.allBlobsAttributed,true,JSON.stringify(verified));assert.equal(verified.meetsMinimum,true);
  const commits=records.filter(record=>record.kind==='commit'),executions=records.filter(record=>record.kind==='commit_execution');
  assert.deepEqual(commits.map(record=>record.payload.commit),observed);assert.equal(executions.length,3);
  assert.equal(records.filter(record=>record.kind==='edit').length,3);
  for(const execution of executions){
   const payload=execution.payload,process=records.find(record=>record.hash===payload.process);
   assert.equal(process.kind,'tool_process');assert.equal(process.payload.status,0);assert.equal(process.payload.signal,null);
   const commit=records.find(record=>record.hash===payload.commitRecord);assert.equal(commit.payload.commit,payload.commit);
   assert.deepEqual(commit.payload.parents,payload.parents);assert(process.index<commit.index);
   const output=records.find(record=>record.hash===commit.payload.messageOut);assert.equal(output.kind,'message_out');
   assert(commit.payload.editHashes.every(hash=>records.find(record=>record.hash===hash).index<output.index));
   for(const field of ['stdout','stderr']){
    const stream=process.payload[field],bytes=retainedEvidenceBytes(evidence,stream.evidence);
    assert.equal(bytes.length,stream.bytes);assert.equal(createHash('sha256').update(bytes).digest('hex'),stream.sha256);
   }
  }
  fileSystem.appendFileSync(path.join(workspace,'heldout-0.mjs'),'\n// actual foreign bytes\n');
  git(workspace,['add','heldout-0.mjs']);git(workspace,['commit','-m','foreign']);
  const foreign=verifyRepositoryEvidence(workspace,{lines,expectedAnchor:result.provenance.anchor,trustedPublicKey:publicKey,seal,
   base,head:git(workspace,['rev-parse','HEAD']),minShare:1});
  assert.equal(foreign.allBlobsAttributed,false);assert.equal(foreign.meetsMinimum,false);
  assert.equal(result.provenance.authorship,'not-established');
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});

test('actual failed shell receipts and every commit of a multi-commit process remain distinct from edit authority',async()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'executor-git-negative-'));
 try{
  const workspace=path.join(directory,'work');fileSystem.mkdirSync(workspace);
  git(workspace,['init']);git(workspace,['config','user.name','Fixture']);git(workspace,['config','user.email','fixture@example.invalid']);
  fileSystem.writeFileSync(path.join(workspace,'base.txt'),'base\n');git(workspace,['add','base.txt']);git(workspace,['commit','-m','base']);
  const base=git(workspace,['rev-parse','HEAD']),keys=path.join(directory,'keys');
  const failed="printf 'OUT\\n'; printf 'ERR\\n' >&2; exit 7";
  const multiple='printf first > first.txt && git add first.txt && git commit -m first --trailer "Formal-AI-Chain: $FORMAL_AI_CHAIN_PREFIX"'
   +' && printf second > second.txt && git add second.txt && git commit -m second --trailer "Formal-AI-Chain: $FORMAL_AI_CHAIN_PREFIX"';
  const result=await withDriveCapture({directory:path.join(directory,'proof'),signing:{directory:keys}},workspace,
   'Two caller-owned shell commands; no write/edit tool observations','fixture-negative',async()=>{
    const failure=await drive(planChatStep,workspace,'Run '+failed,{steps:1,allowedCommands:[failed],tools:['bash']});
    assert.equal(failure.stop,'steps');
    const commits=await drive(planChatStep,workspace,'Run '+multiple,{steps:1,allowedCommands:[multiple],tools:['bash']});
    assert.equal(commits.stop,'steps');return{stop:'steps'};
   });
  const evidence=result.provenance.directory,lines=fileSystem.readFileSync(path.join(evidence,'chain.lino'),'utf8').trimEnd().split('\n');
  const records=lines.map(decodeTyped),processes=records.filter(record=>record.kind==='tool_process');
  assert.equal(processes.length,2);assert.equal(processes[0].payload.status,7);assert.equal(processes[1].payload.status,0);
  assert.equal(retainedEvidenceBytes(evidence,processes[0].payload.stdout.evidence).toString('utf8'),'OUT\n');
  assert.equal(retainedEvidenceBytes(evidence,processes[0].payload.stderr.evidence).toString('utf8'),'ERR\n');
  const actual=git(workspace,['rev-list','--reverse',base+'..HEAD']).split('\n');assert.equal(actual.length,2);
  const commits=records.filter(record=>record.kind==='commit');assert.deepEqual(commits.map(record=>record.payload.commit),actual);
  assert.equal(records.filter(record=>record.kind==='repository_transition').length,1);
  assert.equal(records.filter(record=>record.kind==='commit_execution').length,2);
  assert.equal(records.filter(record=>record.kind==='edit').length,0);
  const seal=decodeTyped(fileSystem.readFileSync(path.join(evidence,'seal.lino'),'utf8').trimEnd());
  const report=verifyRepositoryEvidence(workspace,{lines,expectedAnchor:result.provenance.anchor,
   trustedPublicKey:trustedInstallationKey(keys),seal,base,head:git(workspace,['rev-parse','HEAD']),minShare:1});
  assert.equal(report.allBlobsAttributed,false);assert.equal(report.meetsMinimum,false);assert.equal(report.provenChangedLines,0);
  const marker=path.join(workspace,'observer-must-not-run');
  assert.throws(()=>shellCapture('printf wrong > '+JSON.stringify(marker),{cwd:workspace,onProcessObservation:null}),/shell_process_observer_required/u);
  assert.equal(fileSystem.existsSync(marker),false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
