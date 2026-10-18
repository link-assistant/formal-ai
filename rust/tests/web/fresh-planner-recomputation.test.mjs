import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {captureFreshPlannerRun}from '../../../js/provenance/fresh-runtime.mjs';
import {verifyDriveEvidence,retainedEvidenceBytes}from '../../../js/provenance/verify-drive.mjs';
import {trustedInstallationKey}from '../../../js/provenance/installation-key.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
import {verifyPinnedFreshReplay}from '../../../js/provenance/fresh-verifier.mjs';
import {blake3Hex}from '../../../js/crypto/blake3.mjs';
test('a genuinely fresh planner recomputes every actual output and persisted memory state without replaying tool effects',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'fresh-planner-recompute-'));
 try{
  const workspace=path.join(directory,'workspace');fileSystem.mkdirSync(workspace);
  const prompt='Create file unseen.mjs containing export const signal = 43;';
  const originalMemory=path.join(directory,'original-memory.lino'),keys=path.join(directory,'keys');
  const original=captureFreshPlannerRun({directory:path.join(directory,'capture'),workspace,prompt,
   memoryFile:originalMemory,taskURL:'https://example.invalid/tasks/recompute-held-out',steps:12,signing:{directory:keys}});
  assert.equal(original.result.stop,'final');
  const proof=original.result.provenance,publicKey=trustedInstallationKey(keys);
  const verified=verifyDriveEvidence(proof.directory,workspace,proof.anchor,{trustedPublicKey:publicKey});
  const records=fileSystem.readFileSync(path.join(proof.directory,'chain.lino'),'utf8').trimEnd().split('\n').map(decodeTyped);
  const initial=records.find(record=>record.kind==='logical_memory_genesis');
  const first=records.find(record=>record.kind==='message_in');
  const logicalBytes=retainedEvidenceBytes(proof.directory,initial.payload.evidence);
  const replayMemory=path.join(directory,'replay-memory.lino');fileSystem.writeFileSync(replayMemory,logicalBytes,{mode:0o600});
  if(initial.payload.revisionEvidence!==null)fileSystem.writeFileSync(replayMemory+'.provenance-revision.lino',
   retainedEvidenceBytes(proof.directory,initial.payload.revisionEvidence),{mode:0o600});
  const bundleFile=path.join(directory,'replay-input.json');
  fileSystem.writeFileSync(bundleFile,JSON.stringify({directory:proof.directory,expectedAnchor:proof.anchor,
   workspace,trustedPublicKey:publicKey,tools:JSON.parse(retainedEvidenceBytes(proof.directory,first.payload.evidence)).tools}),{mode:0o600});
  const delivered=path.join(workspace,'unseen.mjs'),before=fileSystem.statSync(delivered,{bigint:true});
  const originalHash=blake3Hex(fileSystem.readFileSync(originalMemory));
  const replay=captureFreshPlannerRun({directory:path.join(directory,'replay'),workspace,prompt,
   memoryFile:replayMemory,taskURL:verified.logicalMemory.taskURL,replayInput:bundleFile,steps:12});
  assert.equal(replay.result.stop,'recomputed');
  const result=replay.result.recomputation;
  assert(result.outputCount>0);assert.equal(result.outputCount,result.inputCount);
  assert.equal(result.comparisons.length,result.outputCount);assert.equal(result.engineGenesis,'matched');
  assert.equal(result.finalMemoryHash,verified.logicalMemory.finalMemoryHash);
  assert.equal(result.state,verified.logicalMemory.state);
  assert.equal(blake3Hex(fileSystem.readFileSync(replayMemory)),verified.logicalMemory.finalMemoryHash);
  assert.equal(blake3Hex(fileSystem.readFileSync(originalMemory)),originalHash);
  const after=fileSystem.statSync(delivered,{bigint:true});assert.equal(after.ino,before.ino);
  assert.equal(after.mtimeNs,before.mtimeNs);assert.equal(after.ctimeNs,before.ctimeNs);
  assert.equal(fileSystem.readFileSync(delivered,'utf8'),'export const signal = 43;');
  assert.equal(fileSystem.existsSync(replayMemory+'.provenance-owner.lino'),false);
  assert.equal(replay.fullReplay.verified,false);
  assert.equal(replay.result.fullReplay.verified,false);
  const request={directory:proof.directory,workspace,expectedAnchor:proof.anchor,trustedPublicKey:publicKey,
   expectedRuntimeSha256:original.custody.runtimeExecutable.sha256,expectedSourceIdentity:records[0].payload.sourceId,
   expectedTaskURL:verified.logicalMemory.taskURL};
  assert.throws(()=>verifyPinnedFreshReplay({...request,expectedRuntimeSha256:'0'.repeat(64)}),/PinnedReplayRuntimeMismatch/u);
  assert.throws(()=>verifyPinnedFreshReplay({...request,expectedSourceIdentity:'0'.repeat(64)}),/PinnedReplaySourceIdentityMismatch/u);
  assert.throws(()=>verifyPinnedFreshReplay({...request,expectedTaskURL:'https://example.invalid/wrong'}),/PinnedReplayTaskMismatch/u);
  assert.throws(()=>verifyPinnedFreshReplay({...request,trustedPublicKey:null}),/PinnedReplayIndependentTrustRequired/u);
  const pinned=verifyPinnedFreshReplay(request);
  assert.equal(pinned.fullReplay.verified,true);assert.equal(pinned.repository,'not-checked');
  assert.equal(pinned.recomputation.outputCount,result.outputCount);
  assert.equal(pinned.recomputation.state,result.state);assert(pinned.sourceCount>0);
  assert.equal(fileSystem.statSync(delivered,{bigint:true}).ctimeNs,before.ctimeNs);
  assert.equal(blake3Hex(fileSystem.readFileSync(originalMemory)),originalHash);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
