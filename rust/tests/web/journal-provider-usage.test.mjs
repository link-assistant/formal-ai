import test from 'node:test';
import assert from 'node:assert/strict';
import {
mkdtempSync,
writeFileSync,
readFileSync,
rmSync}
 from 'node:fs';
import {
tmpdir}
 from 'node:os';
import {
join}
 from 'node:path';
import {
aggregateJournalUsage}
from'../../../experiments/formal_ai_subagent/journal-provider-usage.mjs';
import{
sourceDigest}
from'../../../experiments/formal_ai_subagent/coding-amplification.mjs';
// Synthetic instrumentation fixtures are not vendor observations or performance samples.
function setup(){
const dir=mkdtempSync(join(tmpdir(),
'journal-usage-control-'));
const item=(runId,
taskKind,
category)=>({
runId,
taskKind,
category,
task:'Original '+runId,
taskSHA256:sourceDigest('Original '+runId),
expectedRelation:category==='repair'?'unrestricted':'output-larger'}
);
const manifest={
schemaVersion:1,
cohortId:'synthetic-controls-only',
cases:[item('ordinary',
'coding',
'feature-implementation'),
item('repair',
'self-coding',
'repair')]}
;
const records=[{
kind:'admitted',
sequence:0,
previousDigest:null,
cohortId:manifest.cohortId,
manifestSHA256:sourceDigest(JSON.stringify(manifest)),
manifest}
];
 const append=(kind,
runId,
attemptId,
accepted)=>records.push({
kind,
sequence:records.length,
previousDigest:sourceDigest(JSON.stringify(records.at(-1))),
manifestSHA256:records[0].manifestSHA256,
runId,
attemptId,
taskSHA256:manifest.cases.find(item=>item.runId===runId).taskSHA256,
...(kind==='finished'?{
result:{
accepted}
}
:{
}
)}
);
 append('started',
'ordinary',
'first');
append('finished',
'ordinary',
'first',
false);
append('started',
'ordinary',
'retry');
append('finished',
'ordinary',
'retry',
true);
append('started',
'repair',
'first');
 let receipts=[];
const binding=path=>({
path,
sha256:sourceDigest(readFileSync(path))}
);
 const receipt=(runId,
attemptId,
id,
input=7,
output=3)=>{
const path=join(dir,
runId+'-'+attemptId+'-'+id+'.json'),
producer=join(dir,
'producer.mjs');
writeFileSync(producer,
'// synthetic producer identity only\n');
const record={
schemaVersion:1,
kind:'normalized-provider-usage',
receiptId:id,
runId,
attemptId,
provider:'synthetic-provider',
model:'synthetic-model',
requestId:id,
usage:{
inputTokens:input,
outputTokens:output}
}
;
writeFileSync(path,
JSON.stringify(record));
return {
...record,
capture:binding(path),
producer:binding(producer)}
;
}
;
 const run=()=>{
const journalPath=join(dir,
'journal.jsonl');
writeFileSync(journalPath,
records.map(JSON.stringify).join('\n')+'\n');
const journal=binding(journalPath),
indexPath=join(dir,
'index.json');
writeFileSync(indexPath,
JSON.stringify({
schemaVersion:1,
kind:'captured-provider-receipt-index',
cohortId:manifest.cohortId,
journalSHA256:journal.sha256,
receipts}
));
return aggregateJournalUsage(journal,
binding(indexPath));
}
;
 return {
dir,
manifest,
records,
receipt,
run,
set receipts(value){
receipts=value}
,
get receipts(){
return receipts}
,
cleanup:()=>rmSync(dir,
{
recursive:true,
force:true}
)}
;
}
const control=(name,
body)=>test(name,
()=>{
const f=setup();
try{
body(f)}
finally{
f.cleanup()}
}
);
control('every source-owned case and failed unfinished retry remains with Unknown absent capture',
f=>{
const r=f.run();
assert.equal(r.cases.length,
2);
assert.equal(r.cases[0].attempts.length,
2);
assert.equal(r.cases[0].attempts[0].accepted,
false);
assert.equal(r.cases[1].attempts[0].finishedSequence,
null);
assert.equal(r.cases[0].attempts[0].usageStatus,
'Unknown');
assert.equal(r.fullModelUsageStatus,
'Unknown');
assert.equal(r.actualCost,
null);
assert.equal(r.partitions[1].expectedRelation,
'unrestricted');
}
);
control('captured retry totals remain provider-model scoped and never prove full usage',
f=>{
f.receipts=[f.receipt('ordinary',
'first',
'a'),
f.receipt('ordinary',
'retry',
'b',
11,
5)];
const r=f.run();
assert.equal(r.cases[0].models[0].counters.inputTokens,
18);
assert.equal(r.cases[0].models[0].counters.outputTokens,
8);
assert.equal(r.cases[1].attempts[0].usageStatus,
'Unknown');
assert.equal(r.fullModelUsageStatus,
'Unknown');
assert.equal(r.semanticUsefulness,
'Unknown');
}
);
control('omitted retry receipt stays Unknown rather than inheriting first capture',
f=>{
f.receipts=[f.receipt('ordinary',
'first',
'a')];
const r=f.run();
assert.equal(r.cases[0].attempts[0].usageStatus,
'Captured');
assert.equal(r.cases[0].attempts[1].usageStatus,
'Unknown');
}
);
control('unknown or cross-run attempt receipts refuse',
f=>{
f.receipts=[f.receipt('repair',
'retry',
'a')];
assert.throws(f.run,
/undeclared/);
}
);
control('duplicates across cases cannot inflate receipt totals',
f=>{
f.receipts=[f.receipt('ordinary',
'first',
'a'),
f.receipt('repair',
'first',
'a')];
assert.throws(f.run,
/mismatch|duplicate/);
}
);
control('negative invented counters refuse instead of conversion from bytes',
f=>{
f.receipts=[f.receipt('ordinary',
'first',
'a',
-1,
4)];
assert.throws(f.run,
/nonnegative/);
}
);
control('missing token fields remain Unknown while preserving original receipt identity',
f=>{
f.receipts=[f.receipt('ordinary',
'first',
'a',
null,
null)];
const r=f.run();
assert.equal(r.cases[0].attempts[0].usageStatus,
'Unknown');
assert.equal(r.cases[0].models[0].counters.inputTokens,
null);
}
);
control('case relabeling against the frozen task kind cannot change category relation',
f=>{
f.manifest.cases[0].category='calculation';
f.records[0].manifestSHA256=sourceDigest(JSON.stringify(f.manifest));
assert.throws(f.run,
/category|task kind/);
}
);
control('journal sequence and chained source identities cannot be bypassed',
f=>{
f.records[1].sequence=999;
assert.throws(f.run,
/sequence|chain/);
}
);
control('finished attempts require an original unique start',
f=>{
f.records[1].kind='finished';
f.records[1].result={
accepted:true}
;
assert.throws(f.run,
/prior start/);
}
);

control('distinct provider-model safe counters never enter an unused mixed-model total',
f=>{
const first=f.receipt('ordinary',
'first',
'limit',
Number.MAX_SAFE_INTEGER,
0),
second=f.receipt('ordinary',
'retry',
'one',
1,
0);
const raw=JSON.parse(readFileSync(second.capture.path,
'utf8'));
raw.model='different-model';
writeFileSync(second.capture.path,
JSON.stringify(raw));
second.model=raw.model;
second.capture.sha256=sourceDigest(readFileSync(second.capture.path));
f.receipts=[first,
second];
const r=f.run();
assert.deepEqual(r.cases[0].models.map(model=>model.counters.inputTokens),
[Number.MAX_SAFE_INTEGER,
1]);
assert.equal(r.fullModelUsageStatus,
'Unknown');
}
);
control('same-provider-model overflow still refuses after receipt-local normalization',
f=>{
f.receipts=[f.receipt('ordinary',
'first',
'limit',
Number.MAX_SAFE_INTEGER,
0),
f.receipt('ordinary',
'retry',
'one',
1,
0)];
assert.throws(f.run,
/safe integer/);
}
);
