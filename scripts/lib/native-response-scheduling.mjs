import assert from 'node:assert/strict';
import {executionBatches} from './coverage-execution.mjs';
import {durationLookup,DEFAULT_SECONDS} from './ci-speed-shards.mjs';

/** Keep source-bound original identities while planning isolated actual processes. */
export function planNativeResponseCaptures(cases,listing,executables,recorded,fallbackSeconds=DEFAULT_SECONDS) {
 assert.ok(Array.isArray(cases)&&cases.length>0,'selected original cases required');
 assert.ok(recorded instanceof Map,'recorded duration map required');
 assert.ok(Number.isFinite(fallbackSeconds)&&fallbackSeconds>=0,'invalid fallback duration');
 const byItem=new Map(),ids=new Set();
 for(const [selectionIndex,row] of cases.entries()) {
  assert.ok(typeof row.id==='string'&&row.id.length>0,'original source identity required');
  assert.ok(typeof row.target==='string'&&row.target.length>0,'registered target required');
  assert.ok(typeof row.caller==='string'&&row.caller.length>0,'qualified caller required');
  assert.ok(!row.target.includes('\t')&&!row.caller.includes('\t'),'invalid qualified caller key');
  assert.ok(!ids.has(row.id),'duplicate original source identity');ids.add(row.id);
  const item=row.target+'\t'+row.caller;
  assert.ok(!byItem.has(item),'ambiguous original qualified caller');
  byItem.set(item,{...row,selectionIndex});
 }
 const secondsOf=durationLookup(recorded,fallbackSeconds);
 const batches=executionBatches([...byItem.keys()],listing,executables,secondsOf,1);
 return batches.map((batch,dispatchIndex)=>{
  assert.equal(batch.names.length,1,'one unchanged original per process');
  const row=byItem.get(batch.target+'\t'+batch.names[0]);
  return {...row,dispatchIndex,weight:{seconds:batch.seconds,
   origin:recorded.has(row.caller)?'recorded':'fallback',fallbackSeconds},batch};
 });
}
