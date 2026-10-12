// Captured receipt accounting only: source integrity cannot establish provider authenticity.
import {
readFileSync,
lstatSync}
 from 'node:fs';
import {
sourceDigest,
normalizeUsageReceipts}
 from './coding-amplification.mjs';
import {
declareTask}
 from './task-relations.mjs';
const fail=message=>{
throw new TypeError(message);
}
;
const counters=['inputTokens',
'outputTokens',
'cachedInputTokens',
'reasoningTokens'];
function readBound(binding,
maximum=8388608){
 if(!binding||typeof binding.path!=='string'||!/^[a-f0-9]{64}$/u.test(binding.sha256))fail('exact physical capture binding required');
 const info=lstatSync(binding.path);
if(!info.isFile()||info.size>maximum)fail('regular bounded capture required');
 const bytes=readFileSync(binding.path);
if(bytes.length>maximum||sourceDigest(bytes)!==binding.sha256)fail('capture source drift or bound');
 return bytes;
}
function sum(items){
 return Object.fromEntries(counters.map(name=>{
  if(!items.length||items.some(item=>item[name]===null))return [name,
null];
  const value=items.reduce((total,
item)=>total+item[name],
0);
if(!Number.isSafeInteger(value))fail('captured total exceeds safe integer');
return [name,
value];
 }
));
}
/** Bind a closed journal and a closed receipt-index capture; never trust byte/token proxies. */
export function aggregateJournalUsage(journalBinding,
receiptIndexBinding){
 const bytes=readBound(journalBinding),
text=bytes.toString('utf8');
 if(!Buffer.from(text).equals(bytes)||!text.endsWith('\n'))fail('complete UTF8 journal required');
 const lines=text.slice(0,
-1).split('\n');
if(!lines.length||lines.length>10000)fail('journal record bound');
 const records=lines.map(line=>JSON.parse(line)),
first=records[0];
 if(first?.kind!=='admitted'||first.sequence!==0||first.previousDigest!==null||!first.manifest||first.manifestSHA256!==sourceDigest(JSON.stringify(first.manifest))||typeof first.cohortId!=='string'||first.cohortId!==first.manifest.cohortId)fail('invalid admission identity');
 const manifest=first.manifest;
if(manifest.schemaVersion!==1||!Array.isArray(manifest.cases)||!manifest.cases.length||manifest.cases.length>200)fail('bounded admitted cases required');
 const cases=new Map();
 for(const item of manifest.cases){
  const declared=declareTask(item);
  if(cases.has(item.runId)||item.taskSHA256!==sourceDigest(item.task)||item.expectedRelation!==declared.expectedRelation)fail('admitted case identity or category mismatch');
  cases.set(item.runId,
{
declaration:declared,
attempts:new Map()}
);
 }
 records.forEach((record,
index)=>{
  if(record.sequence!==index||record.previousDigest!==(index?sourceDigest(lines[index-1]):null))fail('journal sequence or chain mismatch');
  if(!index)return;
  const owner=cases.get(record.runId);
  if(!owner||record.manifestSHA256!==first.manifestSHA256||record.taskSHA256!==sourceDigest(owner.declaration.task)||typeof record.attemptId!=='string'||!record.attemptId)fail('undeclared journal case or attempt');
  if(record.kind==='started'){
   if(owner.attempts.has(record.attemptId))fail('duplicate journal attempt');
   owner.attempts.set(record.attemptId,
{
attemptId:record.attemptId,
startedSequence:index,
finishedSequence:null,
accepted:null}
);
  }
else if(record.kind==='finished'){
   const attempt=owner.attempts.get(record.attemptId);
   if(!attempt||attempt.finishedSequence!==null||typeof record.result?.accepted!=='boolean')fail('finish requires unique prior start and disposition');
   attempt.finishedSequence=index;
attempt.accepted=record.result.accepted;
  }
else fail('unsupported journal record');
 }
);
 const indexBytes=readBound(receiptIndexBinding,
1048576),
index=JSON.parse(indexBytes.toString('utf8'));
 if(index?.schemaVersion!==1||index.kind!=='captured-provider-receipt-index'||index.cohortId!==first.cohortId||index.journalSHA256!==journalBinding.sha256||!Array.isArray(index.receipts)||index.receipts.length>1000)fail('source-bound receipt index required');
 const globalReceipts=new Set(),
globalRequests=new Set();
 for(const receipt of index.receipts){
  const owner=cases.get(receipt?.runId);
  if(!owner||!owner.attempts.has(receipt?.attemptId))fail('receipt refers to undeclared run or retry');
 }
 const measured=[];
 for(const [runId,
owner]of cases){
  const attempts=[...owner.attempts.values()],
receipts=index.receipts.filter(receipt=>receipt.runId===runId);
  const normalized={
usageReceipts:receipts.flatMap(receipt=>normalizeUsageReceipts({
runId,
attemptIds:attempts.map(item=>item.attemptId),
attemptInputs:attempts.map(()=>owner.declaration.task),
usageReceipts:[receipt]}
).usageReceipts)}
;
  for(const receipt of normalized.usageReceipts){
   const requestKey=JSON.stringify([receipt.provider,
receipt.requestId]);
   if(globalReceipts.has(receipt.receiptId)||globalRequests.has(requestKey))fail('duplicate receipt or provider request across cases');
   globalReceipts.add(receipt.receiptId);
globalRequests.add(requestKey);
  }
  const models=new Map();
  for(const receipt of normalized.usageReceipts){
const key=JSON.stringify([receipt.provider,
receipt.model]);
if(!models.has(key))models.set(key,
[]);
models.get(key).push(receipt);
}
  measured.push({
runId,
taskKind:owner.declaration.taskKind,
category:owner.declaration.category,
expectedRelation:owner.declaration.expectedRelation,
declarationDigest:owner.declaration.digest,
   attempts:attempts.map(attempt=>{
const owned=normalized.usageReceipts.filter(receipt=>receipt.attemptId===attempt.attemptId);
return {
...attempt,
receiptIds:owned.map(receipt=>receipt.receiptId),
usageStatus:owned.length&&owned.every(receipt=>receipt.counters.inputTokens!==null&&receipt.counters.outputTokens!==null)?'Captured':'Unknown'}
;
}
),
   models:[...models.values()].map(receipts=>({
provider:receipts[0].provider,
model:receipts[0].model,
receipts:receipts.length,
counters:sum(receipts.map(receipt=>receipt.counters))}
)),
   usageReceipts:normalized.usageReceipts,
fullModelUsageStatus:'Unknown',
actualCost:null,
qualityRequiresIndependentReview:true}
);
 }
 const partitions=new Map();
 for(const item of measured){
const key=JSON.stringify([item.taskKind,
item.category]);
if(!partitions.has(key))partitions.set(key,
[]);
partitions.get(key).push(item);
}
 return {
schemaVersion:1,
cohortId:first.cohortId,
journal:journalBinding,
receiptIndex:receiptIndexBinding,
admissionSHA256:sourceDigest(lines[0]+'\n'),
manifestSHA256:first.manifestSHA256,
  cases:measured,
partitions:[...partitions.values()].map(items=>({
taskKind:items[0].taskKind,
category:items[0].category,
expectedRelation:items[0].expectedRelation,
cases:items.length,
attempts:items.reduce((count,
item)=>count+item.attempts.length,
0),
failedAttempts:items.reduce((count,
item)=>count+item.attempts.filter(attempt=>attempt.accepted===false).length,
0),
unfinishedAttempts:items.reduce((count,
item)=>count+item.attempts.filter(attempt=>attempt.finishedSequence===null).length,
0)}
)),
  scope:'all source-journaled cases and observed retries; captured provider receipts only',
fullModelUsageStatus:'Unknown',
observedInputCompleteness:'Unknown',
actualCost:null,
monetarySavings:null,
providerAuthenticityRequiresReview:true,
semanticUsefulness:'Unknown',
autonomousRepairCredit:0}
;
}
