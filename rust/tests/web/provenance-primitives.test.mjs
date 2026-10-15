// Official vector fixture and source-owned general provenance regressions. No production keys.
import os from 'node:os';
 import path from 'node:path';
 import {pathToFileURL} from 'node:url';
 import {spawnSync} from 'node:child_process';
import test from 'node:test';
 import assert from 'node:assert/strict';
 import fs from 'node:fs';
 import {generateKeyPairSync} from 'node:crypto';
import {blake3,
 blake3Hex} from '../../../js/crypto/blake3.mjs';
 import {provenancePrimitives,
 rawBytes,
 exactInteger} from '../../../js/provenance/session.mjs';
import {encodeTyped,
 decodeTyped} from '../../../js/server/structured-log.mjs';
const vectors=JSON.parse(fs.readFileSync(new URL('../fixtures/blake3-1.8.2/test_vectors.json',
 import.meta.url)));
for(const c of vectors.cases)test(`official unkeyed XOF vector ${c.input_len}`,
 ()=>{const input=Uint8Array.from({length:c.input_len},
 (_,
 i)=>i%251);
 assert.equal(Buffer.from(blake3(input,
 {length:c.hash.length/2})).toString('hex'),
 c.hash);
 assert.equal(blake3Hex(input),
 c.hash.slice(0,
 64));
 assert.equal(blake3(input,
 {length:0}).length,
 0);
 });
const p=provenancePrimitives({encodeTyped,
 decodeTyped});
function session(){const s=p.createSession({sessionId:'session α',
 contextId:'context β',
 initialMemory:new Uint8Array([0,
 255,
 128,
 13,
 10]),
 runtimeId:'runtime-bytes-id',
 sourceId:'source-tree-id'});
 s.append('message',
 {bytes:rawBytes(new Uint8Array([255,
 0,
 10])),
 large:exactInteger(90071992547409931234n),
 text:'é\n💠'});
 s.append('terminal',
 {reason:'completed'});
 return s;
 }
test('deterministic canonical chain retains raw bytes, large integers and exact unicode',
 ()=>{const a=session(),
 b=session();
 assert.deepEqual(a.lines(),
 b.lines());
 assert.deepEqual(p.verifyChain(a.lines(),
 a.anchor()),
 a.anchor());
 const row=decodeTyped(a.lines()[1]);
 assert.equal(row.payload.bytes.value,
 '/wAK');
 assert.equal(row.payload.large.value,
 '90071992547409931234');
 assert.equal(row.payload.text,
 'é\n💠');
 assert.equal(encodeTyped({b:2,
 a:1}),
 encodeTyped({a:1,
 b:2}));
 assert.notEqual(blake3Hex(Buffer.from('é')),
 blake3Hex(Buffer.from('e\u0301')));
 });
test('tamper, reorder, truncation, extension and foreign anchors refuse',
 ()=>{const s=session(),
 lines=s.lines(),
 a=s.anchor();
 for(const change of [lines.slice(0,
 -1),
 [...lines,
 lines[2]],
 [lines[1],
 lines[0],
 lines[2]],
 [lines[0],
 lines[1].replace('message',
 'messagf'),
 lines[2]]])assert.throws(()=>p.verifyChain(change,
 a));
 for(const change of [{head:'0'.repeat(64)},
 {genesisHash:'0'.repeat(64)},
 {sessionId:'foreign'},
 {contextId:'foreign'},
 {count:2}])assert.throws(()=>p.verifyChain(lines,
 {...a,
 ...change}));
 assert.throws(()=>p.verifyChain(lines));
 assert.throws(()=>s.append('message',
 {}));
 });
test('noncanonical record encoding and incomplete sessions refuse',
 ()=>{const s=session();
 const lines=s.lines();
 assert.throws(()=>p.verifyChain(lines.map(l=>' '+l),
 s.anchor()));
 const open=p.createSession({sessionId:'x',
 contextId:'y',
 initialMemory:new Uint8Array(),
 runtimeId:'r',
 sourceId:'s'});
 assert.throws(()=>open.anchor());
 assert.throws(()=>open.append('genesis',
 {}));
 });
test('Ed25519 ephemeral-only seal validates against caller-supplied independent anchor and key',
 ()=>{const s=session(),
 a=s.anchor(),
 keys=generateKeyPairSync('ed25519'),
 wrong=generateKeyPairSync('ed25519');
 const sig=p.seal(s.lines(),
 a,
 keys.privateKey);
 assert.equal(p.verifySeal(s.lines(),
 a,
 sig,
 keys.publicKey),
 true);
 const damaged=Uint8Array.from(sig);
 damaged[0]^=1;
 assert.throws(()=>p.verifySeal(s.lines(),
 a,
 damaged,
 keys.publicKey));
 assert.throws(()=>p.verifySeal(s.lines(),
 a,
 sig,
 wrong.publicKey));
 assert.throws(()=>p.verifySeal(s.lines().slice(0,
 -1),
 a,
 sig,
 keys.publicKey));
 assert.throws(()=>p.verifySeal(s.lines(),
 {...a,
 contextId:'wrong'},
 sig,
 keys.publicKey));
 assert.throws(()=>p.verifySeal(s.lines(),
 a,
 sig.subarray(1),
 keys.publicKey));
 });
test('reject implicit or lossy input coercions',
 ()=>{for(const x of ['abc',
 null,
 {},
 123])assert.throws(()=>blake3(x));
 for(const length of [-1,
 NaN,
 1.5,
 1048577])assert.throws(()=>blake3(new Uint8Array(),
 {length}));
 assert.throws(()=>rawBytes('abc'));
 assert.throws(()=>exactInteger(9007199254740992));
 const s=p.createSession({sessionId:'s',
 contextId:'c',
 initialMemory:new Uint8Array(),
 runtimeId:'r',
 sourceId:'s'});
 for(const bad of [NaN,
 Infinity,
 -0,
 9007199254740992,
 undefined,
 1n,
 '\ud800'])assert.throws(()=>s.append('event',
 {bad}));
 assert.equal(s.lines().length,
 1);
 });

test('schema validation rejects malformed records even with recomputed hashes and an updated anchor',
 ()=>{
 const fresh=()=>{const s=p.createSession({sessionId:'s',
 contextId:'c',
 initialMemory:new Uint8Array(),
 runtimeId:'r',
 sourceId:'src'});
 s.append('terminal',
 {reason:'completed'});
 return s.lines().map(decodeTyped);
 };
 const rehash=rows=>{let prev=null;
 const lines=rows.map((row,
 index)=>{const {hash,
 ...old}=row,
 body={...old,
 index,
 prev};
 prev=blake3Hex(Buffer.from('formal-ai-provenance-record-v1\0'+encodeTyped(body)));
 return encodeTyped({...body,
 hash:prev});
 });
 return {lines,
 anchor:{schema:'formal-ai-session-anchor-v1',
 sessionId:rows[0].sessionId,
 contextId:rows[0].contextId,
 count:rows.length,
 genesisHash:decodeTyped(lines[0]).hash,
 head:prev}};
 };
 for(const alter of [r=>r[0].kind='message',
 r=>r[0].payload.initialMemory.value='!==',
 r=>r[0].payload.initialMemory.encoding='hex',
 r=>r[0].payload.runtimeId='',
 r=>r[0].payload.extra='foreign',
 r=>r[1].kind='message',
 r=>r[1].payload.reason='',
 r=>r[1].payload.extra=1]){const rows=fresh();
 alter(rows);
 const forged=rehash(rows);
 assert.throws(()=>p.verifyChain(forged.lines,
 forged.anchor));
 }
 const base=rehash(fresh());
 for(const altered of [base.lines[0]+' ',
 base.lines[0].replace('formal-ai-log-v1',
 'formal-ai-log-v2'),
 base.lines[0].replace('(number 0)',
 '(number 00)')]){assert.notEqual(altered,
 base.lines[0]);
 assert.throws(()=>p.verifyChain([altered,
 base.lines[1]],
 base.anchor));
 }
});

test('independent compression rotation fault disagrees with the official vector oracle',
 ()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),
 'formal-ai-blake3-fault-'));
 try{
  const source=fs.readFileSync(new URL('../../../js/crypto/blake3.mjs',
 import.meta.url),
 'utf8');
  const needle='rotr(s[d]^s[a],16)';
 assert.ok(source.includes(needle),
 'fault must affect the actual compression implementation');
  const file=path.join(directory,
 'mutated-blake3.mjs');
 fs.writeFileSync(file,
 source.replaceAll(needle,
 'rotr(s[d]^s[a],15)'));
  const vector=vectors.cases.find(c=>c.input_len===0);
 assert.ok(vector);
  const program=`import {blake3} from ${JSON.stringify(pathToFileURL(file).href)}; console.log(Buffer.from(blake3(new Uint8Array(),{length:${vector.hash.length/2}})).toString('hex'));`;
  const child=spawnSync(process.execPath,
 ['--input-type=module',
 '--eval',
 program],
 {encoding:'utf8',
 timeout:10000,
 maxBuffer:65536});
  assert.equal(child.error,
 undefined);
 assert.equal(child.status,
 0,
 child.stderr);
 assert.equal(child.signal,
 null);
 const actual=child.stdout.trim();
 assert.match(actual,
 /^[0-9a-f]+$/);
 assert.equal(actual.length,
 vector.hash.length);
 assert.notEqual(actual,
 vector.hash,
 'official vector must expose a compression fault');
 }finally{fs.rmSync(directory,
 {recursive:true,
 force:true});
 }
});
