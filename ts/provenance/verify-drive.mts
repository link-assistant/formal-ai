// Integrity of retained driver evidence, not PR attribution or computational replay.
import fs from 'node:fs';
import path from 'node:path';
import { provenancePrimitives } from './session.mjs';
import { blake3Hex } from '../crypto/blake3.mjs';
import { encodeTyped, decodeTyped } from '../logging/typed-codec.mjs';
import {verifyInstallationSeal} from './installation-key.mjs';
const p = provenancePrimitives({
  encodeTyped,
  decodeTyped
});
export function regularEvidenceBytes(directory, relative) {
  if (typeof relative !== 'string' || !relative || relative.includes('\\') || relative.split('/').some(x => !x || x === '.' || x === '..') || path.isAbsolute(relative)) throw Error('unsafe_evidence_path');
  let target = fs.realpathSync(directory);
  for (const part of relative.split('/')) {
    target = path.join(target, part);
    if (fs.lstatSync(target).isSymbolicLink()) throw Error('evidence_symlink');
  }
  if (!fs.statSync(target).isFile()) throw Error('evidence_not_file');
  return fs.readFileSync(target);
}
export function retainedEvidenceBytes(directory,evidence){
      if(evidence?.format==='chunks-v1'){
        if(!Number.isSafeInteger(evidence.size)||evidence.size<=256*1024||evidence.size>64*1024*1024||!Array.isArray(evidence.chunks))throw Error('invalid_evidence_chunks');
        if(evidence.chunks.length!==Math.ceil(evidence.size/(256*1024)))throw Error('invalid_evidence_chunks');
        const chunks=evidence.chunks.map((chunk,index)=>{
          const expected=index===evidence.chunks.length-1?evidence.size-index*256*1024:256*1024;
          if(chunk.format!==undefined||chunk.size!==expected)throw Error('invalid_evidence_chunks');
          return retainedEvidenceBytes(directory,chunk);
        });
        const bytes=Buffer.concat(chunks);
        if(bytes.length!==evidence.size||blake3Hex(bytes)!==evidence.blake3)throw Error('evidence_hash');
        return bytes;
      }
      if (!/^\d{6}\.lino$/u.test(evidence.file) || !Number.isSafeInteger(evidence.size) || evidence.size < 0) throw Error('invalid_evidence_binding');
      const text = regularEvidenceBytes(directory, evidence.file).toString('utf8');
      if (!text.endsWith('\n')) throw Error('incomplete_evidence_line');
      const encoded = text.slice(0, -1),
        value = decodeTyped(encoded);
      if (encodeTyped(value) !== encoded || value.type !== 'bytes-v1' || value.encoding !== 'base64' || typeof value.value !== 'string') throw Error('invalid_evidence_bytes');
      const bytes = Buffer.from(value.value, 'base64');
      if (bytes.toString('base64') !== value.value || bytes.length !== evidence.size || blake3Hex(bytes) !== evidence.blake3) throw Error('evidence_hash');
      return bytes;
}
export function verifyDriveEvidence(directory, workspace, expectedAnchor, {trustedPublicKey}={}) {
  const anchor = expectedAnchor ?? decodeTyped(regularEvidenceBytes(directory, 'anchor.lino').toString('utf8').trim());
  const chain = regularEvidenceBytes(directory, 'chain.lino').toString('utf8');
  if (!chain.endsWith('\n')) throw Error('incomplete_chain_line');
  const lines = chain.slice(0, -1).split('\n');
  p.verifyChain(lines, anchor);
  const records = lines.map(decodeTyped),
    outputs = new Set(),
    thinking = new Set(),
    latestEdits = new Map(),plannerRecords=new Map(),responses=new Map();
  let missingThinkingFields = 0,logIndex=0;
  const logFiles=new Map();
  for (const record of records) {
    const evidence = record.payload?.evidence;
    if (evidence) {
      retainedEvidenceBytes(directory,evidence);
    }
    if(record.kind==='structured_log'){
      const log=record.payload.log;
      if(!log||!/^serve-\d+\.lino$/u.test(log.file)||log.index!==logIndex++||log.algorithm!=='blake3')throw Error('structured_log_binding');
      if(!logFiles.has(log.file)){
        const bytes=regularEvidenceBytes(directory,log.file),text=bytes.toString('utf8');
        if(!text.endsWith('\n'))throw Error('structured_log_incomplete');
        logFiles.set(log.file,{lines:text.slice(0,-1).split('\n'),references:0});
      }
      const retained=logFiles.get(log.file),line=retained.lines[retained.references++];
      if(typeof line!=='string'||Buffer.byteLength(line)!==log.size||blake3Hex(Buffer.from(line,'utf8'))!==log.blake3)throw Error('structured_log_hash');
      const value=decodeTyped(line);
      if(encodeTyped(value)!==line||value.schema!=='formal-ai-diagnostic-v1'
        ||value.metadata?.session_id!==record.sessionId||value.sequence!==log.index+1)throw Error('structured_log_record');
    }
    if(record.kind==='engine_genesis'&&record.payload.status==='captured'){
      const genesis=JSON.parse(retainedEvidenceBytes(directory,record.payload.evidence).toString('utf8'));
      if(genesis.schema!=='formal-ai-worker-genesis-v1'||!Array.isArray(genesis.sources)||!Array.isArray(genesis.inputs))throw Error('engine_genesis_schema');
      const paths=new Set();
      for(const source of genesis.sources){
        if(typeof source.path!=='string'||paths.has(source.path))throw Error('engine_genesis_source_identity');
        paths.add(source.path);
      }
      for(const entry of [...genesis.sources,...genesis.inputs]){
        if(entry.textState==='not-delivered'){
          if(entry.evidence!==undefined)throw Error('engine_genesis_undelivered_text');
          continue;
        }
        const body=JSON.parse(retainedEvidenceBytes(directory,entry.evidence).toString('utf8'));
        if(typeof body.text!=='string')throw Error('engine_genesis_text');
      }
    }
    if(record.kind==='planner_thinking'){
      const observation=JSON.parse(retainedEvidenceBytes(directory,record.payload.evidence).toString('utf8'));
      if(!['formal-ai-planner-observation-v1','formal-ai-planner-failure-v1'].includes(observation.schema)
        ||!Array.isArray(observation.eventLog)||!observation.eventLog.length)throw Error('planner_event_log_missing');
      plannerRecords.set(record.hash,observation);
    }
    if(record.kind==='planner_response')responses.set(record.hash,record.payload.evidence);
    if (record.kind === 'message_out') outputs.add(record.hash);
    if (record.kind === 'thinking') {
      if (!outputs.has(record.payload.messageOut) || thinking.has(record.payload.messageOut)) throw Error('thinking_binding');
      thinking.add(record.payload.messageOut);
      if(record.payload.plannerThinking!==undefined&&record.payload.plannerThinking!==null){
        const observed=plannerRecords.get(record.payload.plannerThinking),response=responses.get(record.payload.plannerResponse);
        if(!observed||!response||JSON.stringify(observed.projected)!==retainedEvidenceBytes(directory,response).toString('utf8'))throw Error('planner_thinking_binding');
      }
      if (record.payload.status === 'unavailable') missingThinkingFields++;
    }
    if (record.kind === 'edit') {
      if (!outputs.has(record.payload.messageOut)) throw Error('edit_message_binding');
      latestEdits.set(record.payload.path, record.payload);
    }
  }
  if (thinking.size !== outputs.size) throw Error('thinking_missing');
  for(const retained of logFiles.values())if(retained.references!==retained.lines.length)throw Error('structured_log_unbound');
  const signature=trustedPublicKey?verifyInstallationSeal(lines,anchor,decodeTyped(regularEvidenceBytes(directory,'seal.lino').toString('utf8').trimEnd()),trustedPublicKey):{signature:'not-verified',trust:'not-configured'};
  const files = [...latestEdits].sort((a, b) => Buffer.compare(Buffer.from(a[0]), Buffer.from(b[0]))).map(([file, edit]) => {
    try {
      const bytes = regularEvidenceBytes(workspace, file);
      return {
        path: file,
        status: blake3Hex(bytes) === edit.postimage.blake3 ? 'matches-observation' : 'foreign'
      };
    } catch {
      return {
        path: file,
        status: 'foreign'
      };
    }
  });
  return {
    schema: 'formal-ai-driver-evidence-report-v1',
    integrity: true,
    anchorTrust: expectedAnchor ? 'caller-pinned' : 'self-supplied',
    files,
    missingThinkingFields,
    structuredLogs:logIndex,
    plannerObservations:plannerRecords.size,
    installation:signature,
    scope: 'driver-dialogue-and-observed-write-edit-bytes',
    authorship: 'not-established',
    replay: 'not-performed',
    prDiffCoverage: 'not-checked'
  };
}
