import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {captureFreshPlannerRun}from '../../../js/provenance/fresh-runtime.mjs';
import {verifyDriveEvidence,retainedEvidenceBytes}from '../../../js/provenance/verify-drive.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
import {blake3Hex}from '../../../js/crypto/blake3.mjs';
import {inspectLogicalMemoryEvidence}from '../../../js/provenance/memory-evidence.mjs';
test('fresh actual planner binds canonical persisted memory at every real input and output',async()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'logical-memory-live-'));
 try{
  const workspace=path.join(directory,'workspace');fileSystem.mkdirSync(workspace);
  const memoryFile=path.join(directory,'memory.lino'),taskURL='https://example.invalid/tasks/real-js-fixture';
  const observed=captureFreshPlannerRun({directory:path.join(directory,'evidence'),workspace,memoryFile,taskURL,
   prompt:'Create file heldout.mjs containing export const measure = 37;',steps:12});
  assert.equal(observed.result.stop,'final');
  const executableHash=crypto.createHash('sha256');
  for await(const bytes of fileSystem.createReadStream(process.execPath))executableHash.update(bytes);
  assert.equal(observed.custody.runtimeExecutable.sha256,executableHash.digest('hex'));
  assert.equal(observed.custody.runtimeExecutable.version,process.version);
  assert.equal(observed.custody.runtimeExecutable.architecture,process.arch);

  assert.equal(fileSystem.readFileSync(path.join(workspace,'heldout.mjs'),'utf8'),'export const measure = 37;');
  const proof=observed.result.provenance,report=verifyDriveEvidence(proof.directory,workspace,proof.anchor);
  assert.equal(report.logicalMemory.taskURL,taskURL);
  assert(report.logicalMemory.transitions>=2);
  assert.equal(report.logicalMemory.finalMemoryHash,blake3Hex(fileSystem.readFileSync(memoryFile)));
  assert.equal(fileSystem.existsSync(memoryFile+'.provenance-owner.lino'),false);
  const records=fileSystem.readFileSync(path.join(proof.directory,'chain.lino'),'utf8').trimEnd().split('\n').map(decodeTyped);
  const read=evidence=>retainedEvidenceBytes(proof.directory,evidence);
  const transitions=records.filter(record=>record.kind==='logical_memory_transition');
  assert.equal(transitions.length,records.filter(record=>['message_in','message_out'].includes(record.kind)).length);
  for(const record of transitions)assert.equal(blake3Hex(read(record.payload.evidence)),record.payload.memoryHash);
  const tampered=structuredClone(records),transition=tampered.find(record=>record.kind==='logical_memory_transition');
  transition.payload.state='0'.repeat(64);
  assert.throws(()=>inspectLogicalMemoryEvidence(tampered,read),/logical_memory_state/);
  const omitted=records.filter(record=>record!==transitions[0]);
  assert.throws(()=>inspectLogicalMemoryEvidence(omitted,read),/logical_memory_message_binding/);
  assert.equal(observed.fullReplay.verified,false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
