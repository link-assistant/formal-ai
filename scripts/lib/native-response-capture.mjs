import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const BOUNDARIES=new Set(['UniversalSolver::solve_with_history_probability_store_and_intent_cache','FormalAiEngine::answer_with_memory']);
export function collectNativeResponseRecords(bytes,{caller,processId,execution,identity,expectedIdentity}) {
 assert.ok(Buffer.isBuffer(bytes));assert.ok(bytes.length>0,'missing native observations');
 assert.equal(bytes.at(-1),10,'native observation stream lacks terminal LF');
 assert.ok(Number.isInteger(processId)&&processId>0,'missing actual child process ID');
 assert.equal(execution.processId,processId);assert.equal(execution.exitCode,0);assert.equal(execution.signal,null);
 assert.equal(execution.error,undefined);assert.equal(execution.succeeded,true);
 assert.deepEqual(execution.problems,[]);assert.equal(execution.completed,1);
 assert.deepEqual(execution.summary,{passed:1,failed:0,ignored:0});
 assert.deepEqual(identity,expectedIdentity,'native producer identity differs from independent expected inputs');
 for(const key of ['source-commit','source-tree'])assert.match(identity[key],/^[a-f0-9]{40}$/u);
 for(const key of ['cargo-lock-sha256','native-inputs-sha256','test-inputs-sha256'])assert.match(identity[key],/^[a-f0-9]{64}$/u);
 for(const key of ['compiler','target','profile','features'])assert.ok(typeof identity[key]==='string'&&identity[key].length>0);
 for(const key of ['rust-flags','encoded-rust-flags'])assert.equal(typeof identity[key],'string');
 assert.ok(identity['cargo-profile-overrides']&&typeof identity['cargo-profile-overrides']==='object');
 const records=new TextDecoder('utf-8',{fatal:true}).decode(bytes).split('\n').slice(0,-1).map(line=>JSON.parse(line));
 const sequences=new Set();const bound=[],unbound=[];
 for(const record of records) {
  assert.equal(record.schema,'native-response-observation/v1');
  assert.ok(BOUNDARIES.has(record.entrypoint));assert.equal(record['process-id'],processId);
  assert.ok(Number.isSafeInteger(record.sequence)&&record.sequence>0);
  assert.ok(!sequences.has(record.sequence),'duplicate native observation sequence');sequences.add(record.sequence);
  assert.ok(record['caller-thread']===null||typeof record['caller-thread']==='string');
  assert.equal(typeof record.prompt,'string');assert.equal(typeof record['config-debug'],'string');assert.ok(record['config-debug'].length>0);
  assert.ok(Array.isArray(record.history));
  for(const turn of record.history){assert.ok(['user','assistant'].includes(turn.role));assert.equal(typeof turn.content,'string');}
  assert.ok(record.context&&typeof record.context==='object'&&!Array.isArray(record.context));
  assert.equal(typeof record.response?.answer,'string');assert.equal(typeof record.response.intent,'string');
  assert.ok(Object.hasOwn(record.response,'confidence'));assert.ok(record.response.confidence===null||typeof record.response.confidence==='number');
  assert.ok(Array.isArray(record.response.evidence_links)&&record.response.evidence_links.every(value=>typeof value==='string'));assert.equal(typeof record.response.links_notation,'string');
  assert.equal(typeof record.response.derivation_id,'string');
  const entry={...record,'answer-utf8-bytes':Buffer.byteLength(record.response.answer),
   'answer-sha256':createHash('sha256').update(record.response.answer).digest('hex')};
  (record['caller-thread']===caller?bound:unbound).push(entry);
 }
 assert.ok(bound.length>0,'no observation bound to actual original native test caller');
 bound.sort((a,b)=>a.sequence-b.sequence);unbound.sort((a,b)=>a.sequence-b.sequence);
 return {bound,unbound,'raw-bytes':bytes.length,'raw-sha256':createHash('sha256').update(bytes).digest('hex'),identity};
}
