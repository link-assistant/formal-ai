import {wellFormedText} from '../logging/text-lines.mjs';
// Private general provenance primitive. Supplied recipe: autonomous authorship credit 0.
// Integrity and supplied-key signature validity are not autonomous authorship or engine replay.
import {sourceSnapshot} from './snapshot.mjs';
import {sign,
 verify} from 'node:crypto';
import {blake3,
 blake3Hex} from '../crypto/blake3.mjs';
const utf8=s=>Buffer.from(s,
 'utf8');
const hex=s=>typeof s==='string'&&/^[0-9a-f]{64}$/.test(s);
function identifier(s){if(typeof s!=='string'||!wellFormedText(s)||!s.length)throw Error('identifier_required');
 return s;
 }
export function rawBytes(bytes){if(!(bytes instanceof Uint8Array))throw Error('bytes_required');
 return {type:'bytes-v1',
 encoding:'base64',
 value:Buffer.from(bytes).toString('base64')};
 }
export function exactInteger(value){if(typeof value!=='bigint')throw Error('bigint_required');
 return {type:'integer-v1',
 encoding:'decimal',
 value:value.toString()};
 }
/** Codec is injected from the shared logging owner; no competing typed-Lino implementation. */
export function provenancePrimitives({encodeTyped,
 decodeTyped}){
 if(typeof encodeTyped!=='function'||typeof decodeTyped!=='function')throw Error('canonical_codec_required');
 const canonical=value=>{const line=encodeTyped(value);
 if(encodeTyped(decodeTyped(line))!==line)throw Error('noncanonical_codec');
 return line;
 };
 const digest=value=>blake3Hex(utf8('formal-ai-provenance-record-v1\0'+canonical(value)));
 const parse=line=>{const x=decodeTyped(line);
 if(canonical(x)!==line)throw Error('noncanonical_record');
 return x;
 };
 const keys=(x,
 wanted)=>{if(!x||Object.keys(x).sort().join(',')!==wanted.slice().sort().join(','))throw Error('record_shape');
 };
 function anchor(value){keys(value,
 ['schema',
 'sessionId',
 'contextId',
 'head',
 'count',
 'genesisHash']);
 if(value.schema!=='formal-ai-session-anchor-v1'||!hex(value.head)||!hex(value.genesisHash)||!Number.isSafeInteger(value.count)||value.count<2)throw Error('anchor_invalid');
 identifier(value.sessionId);
 identifier(value.contextId);
 return value;
 }
 function verifyChain(lines,
 expected){
  anchor(expected);
 if(!Array.isArray(lines)||lines.length!==expected.count)throw Error('chain_count');
  let previous=null,
 genesisHash=null,
 terminal=false;
  for(let i=0;
 i<lines.length;
 i++){
   const row=parse(lines[i]);
 keys(row,
 ['schema',
 'sessionId',
 'contextId',
 'index',
 'prev',
 'kind',
 'payload',
 'hash']);
   if(row.schema!=='formal-ai-session-record-v1'||row.sessionId!==expected.sessionId||row.contextId!==expected.contextId||row.index!==i||row.prev!==previous||!hex(row.hash)||terminal)throw Error('chain_binding');
   identifier(row.kind);
 const {hash,
 ...body}=row;
 if(digest(body)!==hash)throw Error('chain_hash');
   if(i===0){if(row.kind!=='genesis')throw Error('genesis_required');
 keys(row.payload,
 ['initialMemory',
 'runtimeId',
 'sourceId']);
 identifier(row.payload.runtimeId);
 identifier(row.payload.sourceId);
 const m=row.payload.initialMemory;
 keys(m,
 ['type',
 'encoding',
 'value']);
 if(m.type!=='bytes-v1'||m.encoding!=='base64'||typeof m.value!=='string'||Buffer.from(m.value,
 'base64').toString('base64')!==m.value)throw Error('genesis_memory');
 genesisHash=hash;
 }
   else if(row.kind==='genesis')throw Error('duplicate_genesis');
   if(row.kind==='terminal'){keys(row.payload,
 ['reason']);
 identifier(row.payload.reason);
 terminal=true;
 }
   previous=hash;
  }
  if(!terminal||previous!==expected.head||genesisHash!==expected.genesisHash)throw Error('chain_completion');
 return Object.freeze({...expected});
 }
 function createSession({sessionId,
 contextId,
 initialMemory,
 runtimeId,
 sourceId,
 snapshotSource}){
  identifier(sessionId);
 identifier(contextId);
 identifier(runtimeId);
 identifier(sourceId);
 if(snapshotSource!==undefined&&typeof snapshotSource!=='function')throw Error('snapshot_source_required');
 const lines=[];
 let head=null,
 genesisHash=null,
 ended=false;
  function append(kind,
 payload){identifier(kind);
 if(ended||kind==='genesis'&&lines.length)throw Error('append_refused');
 if(snapshotSource&&(kind==='commit'||kind==='terminal')){const observed=snapshotSource();append('source_snapshot',sourceSnapshot(observed.files,observed.extensions));}
 if(kind==='terminal'){keys(payload,
 ['reason']);
 identifier(payload.reason);
 }
   const body={schema:'formal-ai-session-record-v1',
 sessionId,
 contextId,
 index:lines.length,
 prev:head,
 kind,
 payload};
 const hash=digest(body);
 const line=canonical({...body,
 hash});
 lines.push(line);
 head=hash;
 genesisHash??=hash;
 ended=kind==='terminal';
 return Object.freeze({line,
 hash,
 index:lines.length-1});
 }
  append('genesis',
 {initialMemory:rawBytes(initialMemory),
 runtimeId,
 sourceId});
  return Object.freeze({append,
 lines:()=>lines.slice(),
 anchor:()=>{if(!ended)throw Error('session_incomplete');
 return Object.freeze({schema:'formal-ai-session-anchor-v1',
 sessionId,
 contextId,
 head,
 count:lines.length,
 genesisHash});
 }});
 }
 function sealMessage(expected){anchor(expected);
 return blake3(utf8('formal-ai-provenance-seal-v1\0'+canonical(expected)));
 }
 function seal(lines,
 expected,
 privateKey){verifyChain(lines,
 expected);
 if(privateKey?.asymmetricKeyType!=='ed25519')throw Error('ed25519_key_required');
 return sign(null,
 sealMessage(expected),
 privateKey);
 }
 function verifySeal(lines,
 expected,
 signature,
 trustedPublicKey){verifyChain(lines,
 expected);
 if(trustedPublicKey?.asymmetricKeyType!=='ed25519'||!(signature instanceof Uint8Array)||signature.length!==64)throw Error('ed25519_seal_invalid');
 if(!verify(null,
 sealMessage(expected),
 trustedPublicKey,
 signature))throw Error('ed25519_seal_invalid');
 return true;
 }
 return Object.freeze({createSession,
 verifyChain,
 seal,
 verifySeal});
}
