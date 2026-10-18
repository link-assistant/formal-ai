// One canonical logical-memory transition encoding and evidence validator.
import {encodeTyped,decodeTyped}from '../logging/typed-codec.mjs';
import {blake3Hex}from '../crypto/blake3.mjs';
export function logicalMemoryDelta(transition){
 return Buffer.from(encodeTyped({schema:'formal-ai-logical-memory-delta-v1',kind:transition.kind,
  revision:transition.revision,messageHash:transition.messageHash,
  previousMemoryHash:transition.previousMemoryHash,memoryHash:transition.memoryHash}));
}
export function logicalMemoryStep(previousState,transition){
 const delta=logicalMemoryDelta(transition),deltaHash=blake3Hex(delta);
 return {delta,deltaHash,state:blake3Hex(Buffer.concat([Buffer.from(previousState,'hex'),Buffer.from(deltaHash,'hex')]))};
}
export function inspectLogicalMemoryEvidence(records,readEvidence){
 const genesisRecords=records.filter(record=>record.kind==='logical_memory_genesis');
 if(genesisRecords.length===0){
  if(records.some(record=>record.kind.startsWith('logical_memory_')))throw Error('logical_memory_genesis_missing');
  return null;
 }
 if(genesisRecords.length!==1)throw Error('logical_memory_genesis_duplicate');
 const genesis=genesisRecords[0],initialBytes=readEvidence(genesis.payload.evidence);
 const manifest=decodeTyped(Buffer.from(records[0].payload.initialMemory.value,'base64').toString('utf8'));
 if(manifest.schema!=='formal-ai-logical-memory-manifest-v1'
  ||manifest.memoryHash!==blake3Hex(initialBytes)||manifest.memoryHash!==genesis.payload.memoryHash
  ||manifest.size!==initialBytes.length||manifest.revision!==genesis.payload.revision
  ||manifest.taskURL!==genesis.payload.taskURL||!Number.isSafeInteger(manifest.revision)||manifest.revision<0)throw Error('logical_memory_initial_binding');
 if(genesis.payload.revisionEvidence!==undefined&&genesis.payload.revisionEvidence!==null){
  const bytes=readEvidence(genesis.payload.revisionEvidence),text=bytes.toString('utf8');
  if(!text.endsWith('\n'))throw Error('logical_memory_revision_incomplete');
  const observed=decodeTyped(text.slice(0,-1));
  if(observed.schema!=='formal-ai-memory-revision-v1'||observed.revision!==manifest.revision
   ||observed.memoryHash!==manifest.memoryHash)throw Error('logical_memory_revision_binding');
 }else if(manifest.revision!==0)throw Error('logical_memory_initial_revision_unavailable');
 let memoryHash=manifest.memoryHash,state=memoryHash,revision=manifest.revision,pending=null,terminal=null;
 let started=false;const transitions=[];
 for(const record of records){
  if(record===genesis){started=true;continue;}
  if(record.kind==='logical_memory_transition'){
   const transition=record.payload;
   if(!started||pending||terminal||!['message_in','message_out'].includes(transition.kind)
    ||!Number.isSafeInteger(transition.revision)||transition.revision!==revision+1
    ||transition.previousMemoryHash!==memoryHash||transition.previousState!==state)throw Error('logical_memory_transition_order');
   const bytes=readEvidence(transition.evidence),delta=readEvidence(transition.delta);
   if(blake3Hex(bytes)!==transition.memoryHash||!delta.equals(logicalMemoryDelta(transition))
    ||blake3Hex(delta)!==transition.deltaHash)throw Error('logical_memory_transition_bytes');
   const nextState=logicalMemoryStep(state,transition).state;
   if(transition.state!==nextState)throw Error('logical_memory_state');
   revision=transition.revision;memoryHash=transition.memoryHash;state=nextState;
   pending=record;transitions.push(record);continue;
  }
  if(record.kind==='message_in'||record.kind==='message_out'){
   if(!started||!pending||terminal||pending.payload.kind!==record.kind
    ||record.payload.memoryTransition!==pending.hash||record.payload.memoryHash!==memoryHash
    ||record.payload.state!==state||blake3Hex(readEvidence(record.payload.evidence))!==pending.payload.messageHash)throw Error('logical_memory_message_binding');
   pending=null;continue;
  }
  if(record.kind==='logical_memory_terminal'){
   if(!started||pending||terminal||record.payload.revision!==revision||record.payload.memoryHash!==memoryHash
    ||blake3Hex(readEvidence(record.payload.evidence))!==memoryHash)throw Error('logical_memory_terminal_binding');
   terminal=record;
  }
 }
 if(!terminal||pending)throw Error('logical_memory_terminal_missing');
 return {schema:'formal-ai-logical-memory-report-v1',taskURL:manifest.taskURL,
  initialMemoryHash:manifest.memoryHash,finalMemoryHash:memoryHash,initialRevision:manifest.revision,
  finalRevision:revision,state,transitions:transitions.length,initialEvidence:genesis.payload.evidence,
  memoryFile:genesis.payload.path};
}
