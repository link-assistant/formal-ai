// Exact private driver evidence. No autonomous authorship, native-memory or replay claim.
import {activeDriveCapture,withEvidenceContext} from './capture-context.mjs';
export {activeDriveCapture} from './capture-context.mjs';
import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomUUID } from 'node:crypto';
import { provenancePrimitives, rawBytes } from './session.mjs';
import { blake3Hex } from '../crypto/blake3.mjs';
import { encodeTyped, decodeTyped } from '../logging/typed-codec.mjs';
import {createLogger} from '../server/structured-log.mjs';
import {openInstallationSigner} from './installation-key.mjs';
const primitives = provenancePrimitives({
  encodeTyped,
  decodeTyped
});
const wire = value => Buffer.from(JSON.stringify(value), 'utf8');
const digest = bytes => blake3Hex(bytes);
export function resolveEvidenceDirectory(directory,workspace,
 reason='provenance_evidence_must_be_outside_workspace'){
 let ancestor=path.resolve(directory);const suffix=[];
 for(let depth=0;;depth++){
  if(depth>256)throw Error('provenance_directory_unresolvable');
  try{fs.lstatSync(ancestor);break;}catch(error){if(error.code!=='ENOENT')throw error;}
  suffix.unshift(path.basename(ancestor));const parent=path.dirname(ancestor);
  if(parent===ancestor)throw Error('provenance_directory_unresolvable');ancestor=parent;
 }
 const parent=path.join(fs.realpathSync(ancestor),...suffix),working=fs.realpathSync(workspace);
 if(parent===working||parent.startsWith(working+path.sep))throw Error(reason);
 return parent;
}
export async function withDriveCapture(options, workspace, prompt, sourceId, run) {
  const parent=resolveEvidenceDirectory(options.directory,workspace),working=fs.realpathSync(workspace);
  fs.mkdirSync(parent, {
    recursive: true,
    mode: 0o700
  });
  if(options.signing!==undefined&&(!options.signing||typeof options.signing.directory!=='string'))throw Error('provenance_signing_directory_required');
  if(options.signing){
    const keys=path.resolve(options.signing.directory);
    if(keys===working||keys.startsWith(working+path.sep))throw Error('provenance_key_must_be_outside_workspace');
  }
  const signer=options.signing?openInstallationSigner(options.signing.directory,{excludedDirectory:working}):null;
  const directory = fs.mkdtempSync(path.join(parent, 'session-'));
  fs.chmodSync(directory, 0o700);
  const sessionIdentifier = randomUUID();
  const initial = wire({
    scope: 'driver-dialogue-only',
    prompt
  });
  const session = primitives.createSession({
    sessionId: sessionIdentifier,
    contextId: 'js-dogfood-drive',
    initialMemory: initial,
    runtimeId: process.version,
    sourceId
  });
  const chain = path.join(directory, 'chain.lino');
  fs.writeFileSync(chain, session.lines().join('\n') + '\n', {
    mode: 0o600,
    flag: 'wx'
  });
  let memoryHash = digest(initial),
    state = memoryHash,
    output = null,
    payloadIndex = 0;
  const appendRaw = (kind, payload) => {
    const record = session.append(kind, payload);
    fs.appendFileSync(chain, record.line + '\n');
    return record.hash;
  };
  let logIndex=0;
  const logger=createLogger({level:'debug',directory,env:options.environment??process.env,
    recordObserver(line){
      appendRaw('structured_log',{log:{file:'serve-'+process.pid+'.lino',index:logIndex++,
        algorithm:'blake3',blake3:digest(Buffer.from(line,'utf8')),size:Buffer.byteLength(line)}});
    }});
  const diagnostic=(kind,metadata)=>{
    const result=logger.log('debug','provenance.'+kind,{component:'provenance',session_id:sessionIdentifier,...metadata});
    if(!result.delivered?.includes('file'))throw Error('provenance_log_sink_failed');
  };
  const append=(kind,payload)=>{
    if(kind==='terminal')diagnostic(kind,{reason:payload.reason});
    const recorded=appendRaw(kind,payload);
    if(kind!=='terminal')diagnostic(kind,{event_hash:recorded});
    return recorded;
  };
  const retainAnchor=()=>{
    const anchor=session.anchor();
    fs.writeFileSync(path.join(directory,'anchor.lino'),encodeTyped(anchor)+'\n',{mode:0o600,flag:'wx'});
    if(signer)fs.writeFileSync(path.join(directory,'seal.lino'),encodeTyped(signer.seal(session.lines(),anchor))+'\n',{mode:0o600,flag:'wx'});
    return anchor;
  };
  diagnostic('genesis',{runtime_id:process.version,source_id:sourceId});
  function retainBytes(bytes) {
    if(bytes.length>64*1024*1024)throw Error('retained_evidence_size_limit');
    if(bytes.length>256*1024){
      const chunks=[];
      for(let offset=0;offset<bytes.length;offset+=256*1024)chunks.push(retainBytes(bytes.subarray(offset,offset+256*1024)));
      return {format:'chunks-v1',chunks,blake3:digest(bytes),size:bytes.length};
    }
    const name=String(payloadIndex++).padStart(6,'0')+'.lino';
    fs.writeFileSync(path.join(directory,name),encodeTyped(rawBytes(bytes))+'\n',{mode:0o600,flag:'wx'});
    return {file:name,blake3:digest(bytes),size:bytes.length};
  }
  function retain(value) {return retainBytes(wire(value));}
  let plannerThinking=null,plannerResponse=null,retainedGenesis=null;
  const capture = {
    active: true,
    engineGenesis(observe){
      if(retainedGenesis!==null)return;
      const observation=observe();
      if(observation===null){retainedGenesis=append('engine_genesis',{status:'unavailable'});return;}
      const {sources,inputs,...metadata}=observation;
      const retainText=entry=>{
        const {text,...identity}=entry;
        return typeof text==='string'?{...identity,evidence:retain({text})}:identity;
      };
      retainedGenesis=append('engine_genesis',{status:'captured',
        evidence:retain({...metadata,sources:sources.map(retainText),inputs:inputs.map(retainText)})});
    },
    planningObservation(observation){
      plannerThinking=append('planner_thinking',{evidence:retain(observation),scope:'source-owned-planner-event-log'});
    },
    runtimeObservation(observation){
      const sources=observation.sources.map(source=>{
        const {bytes,...identity}=source;
        return {...identity,evidence:retain({bytes})};
      });
      append('runtime_sources',{evidence:retain({...observation,sources}),scope:'reported-node-loader-and-compile-boundaries'});
    },
    engineObservation(observation){
      append('engine_solve',{evidence:retain(observation),scope:'actual-javascript-worker-solve'});
    },
    input(messages, tools) {
      plannerThinking=null;plannerResponse=null;
      const evidence = retain({
        messages,
        tools
      });
      const previousMemoryHash = memoryHash;
      memoryHash = evidence.blake3;
      const deltaHash = digest(wire({
        previousMemoryHash,
        currentMemoryHash: memoryHash
      }));
      state = digest(Buffer.concat([Buffer.from(state, 'hex'), Buffer.from(deltaHash, 'hex')]));
      append('message_in', {
        scope: 'driver-dialogue-only',
        evidence,
        previousMemoryHash,
        memoryHash,
        deltaHash,
        state
      });
    },
    plannerResponse(plan) {
      plannerResponse=append('planner_response', {
        evidence: retain(plan)
      });
    },
    output(plan) {
      output = append('message_out', {
        scope: 'driver-dispatched-plan',
        evidence: retain(plan),
        memoryHash,
        state
      });
      const hasTrace = plannerThinking!==null || plan !== null && typeof plan === 'object' && (Object.hasOwn(plan, 'reasoning') || Object.hasOwn(plan, 'reasoning_content') || Object.hasOwn(plan, 'derivation'));
      append('thinking', {
        messageOut: output,
        status: hasTrace ? 'captured-fields' : 'unavailable',
        plannerThinking,plannerResponse,
        evidence: retain({
          reasoning: plan?.reasoning ?? null,
          reasoning_content: plan?.reasoning_content ?? null,
          derivation: plan?.derivation ?? null
        })
      });
    },
    before(file) {
      if (!output) throw Error('message_out_required');
      try {
        const status = fs.lstatSync(file);
        if (!status.isFile() || status.isSymbolicLink()) return {
          refused: true
        };
        return {
          bytes: fs.readFileSync(file),
          messageOut: output
        };
      } catch (error) {
        if (error.code === 'ENOENT') return {
          bytes: null,
          messageOut: output
        };
        throw error;
      }
    },
    after(file, relative, before) {
      if (before.refused) return;
      let after;
      try {
        if (!fs.lstatSync(file).isFile() || fs.lstatSync(file).isSymbolicLink()) return;
        after = fs.readFileSync(file);
      } catch (error) {
        if (error.code === 'ENOENT') return;
        throw error;
      }
      if (before.bytes !== null && before.bytes.equals(after)) return;
      const blob = bytes => bytes === null ? null : {
        blake3: digest(bytes),
        gitBlobSha1: createHash('sha1').update(Buffer.from('blob ' + bytes.length + '\0')).update(bytes).digest('hex'),
        size: bytes.length
      };
      append('edit', {
        path: relative,
        preimage: blob(before.bytes),
        postimage: blob(after),
        messageOut: before.messageOut,
        scope: 'observed-driver-write-or-edit'
      });
    },
 };
  try {
    const result = await withEvidenceContext(capture, run);
    append('terminal', {
      reason: result.stop ?? 'unknown'
    });
    const anchor = retainAnchor();
    return {
      ...result,
      provenance: {
        directory,
        anchor,
        scope: 'driver-dialogue-and-direct-write-edit-observations',
        authorship: 'not-established',
        replay: 'not-performed',
        signing:signer?{status:'sealed',keyId:signer.keyId,trust:'not-inferred'}:{status:'unsigned'}
      }
    };
  } catch (error) {
    try {
      append('failure', {
        name: String(error?.name ?? 'ThrownValue'),
        messageHash: digest(Buffer.from(String(error?.message ?? error)))
      });
      append('terminal', {
        reason: 'exception'
      });
      retainAnchor();
    } catch {/* A storage failure leaves incomplete evidence; never replace the original failure. */}
    throw error;
  } finally {
    capture.active = false;
    logger.close();
  }
}
