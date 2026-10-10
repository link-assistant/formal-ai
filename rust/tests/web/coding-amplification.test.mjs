import assert from 'node:assert/strict';
import { test } from 'node:test';
import { measureCodingRun, sourceDigest, summarizeCodingRuns } from '../../../experiments/formal_ai_subagent/coding-amplification.mjs';

const change = (after, before = '') => ({path:'module.mjs',role:'production',language:'javascript',before,after});
let runSequence = 0;
const run = (changes, extra = {}) => ({runId:'fixture-'+String(++runSequence),taskKind:'coding',origin:'autonomous',task:'Add a sum function.',
  attemptInputs:['Add a sum function.'],repositoryContext:[],reviewedPatches:[],toolReceipts:[],changes,...extra});
const verified = async changes => ({passed:true,checks:[{name:'independent arithmetic acceptance',exitCode:0}],
  bindings:changes.map(item => ({path:item.path,before:sourceDigest(item.before),after:sourceDigest(item.after)}))});

test('a valid change can exceed concise input without counting explanation text', async()=>{
  const source='export function sum(first, second) { return first + second; }';
  const proof = async changes => {
    const module = await import('data:text/javascript;base64,' + Buffer.from(changes[0].after).toString('base64'));
    assert.equal(module.sum(7,-3),4); assert.equal(module.sum(-6,2),-4);
    return verified(changes);
  };
  const result=await measureCodingRun(run([change(source)]),proof);
  assert.equal(result.accepted,true); assert.ok(result.taskAmplification>1);
  assert.equal(result.modelTokens,null); assert.equal(result.monetarySavings,null);
});

test('failed or missing independent checks cannot produce validated output',async()=>{
  for(const proof of [{passed:true,checks:[],bindings:[]},{passed:true,checks:[{name:'assertions',exitCode:1}],bindings:[]}]){
    const result=await measureCodingRun(run([change('export const result = 7;')]),async()=>proof);
    assert.equal(result.accepted,false);assert.equal(result.autonomousNetCodeBytes,0);
  }
});

test('a passing check against different final source is rejected',async()=>{
  const result=await measureCodingRun(run([change('export const result = 7;')]),async changes=>{
    const proof=await verified(changes);proof.bindings[0].after=sourceDigest('export const result = 8;');return proof;
  });
  assert.equal(result.sourceBound,false);assert.equal(result.taskAmplification,0);
});

test('comments, whitespace, unchanged file copies and literal payload padding do not increase the proxy',async()=>{
  const before='export const value = "small";';
  const comment=await measureCodingRun(run([change(before+'\n// '+ 'padding '.repeat(2000),before)]),verified);
  const literal=await measureCodingRun(run([change('export const value = "'+'padding '.repeat(2000)+'";',before)]),verified);
  const unchanged=await measureCodingRun(run([change(before,before)]),verified);
  for(const result of [comment,literal,unchanged])assert.equal(result.validatedNetCodeBytes,0);
});

test('reviewed patch replay and generators remain outside autonomous authorship',async()=>{
  for(const extra of [{reviewedPatches:['export const result = 7;']},{origin:'reviewed-patch'},{origin:'generator'}]){
    const result=await measureCodingRun(run([change('export const result = 7;')],extra),verified);
    assert.ok(result.validatedNetCodeBytes>0);assert.equal(result.autonomousNetCodeBytes,0);
  }
});

test('generated mirrors, documentation and evidence cannot inflate code output',async()=>{
  const changes=['generated','evidence','documentation'].map((role,index)=>({path:role+index,role,before:'',after:'padding'.repeat(2000)}));
  const result=await measureCodingRun(run(changes),verified);
  assert.equal(result.validatedNetCodeBytes,0);assert.ok(result.excludedChangedBytes>0);
});

test('all retries, patch bytes, observed source and tool receipts remain charged',async()=>{
  const result=await measureCodingRun(run([], {attemptInputs:['Add a sum function.','Retry unchanged.'],
    repositoryContext:['observed module'],reviewedPatches:['reviewed repair'],toolReceipts:['failed command']}),verified);
  assert.equal(result.attempts,2);
  assert.equal(result.totalObservedInputBytes,Buffer.byteLength('Add a sum function.Retry unchanged.observed modulereviewed repairfailed command'));
});

test('a useful deletion and a short mathematical answer never fail for lack of growth',async()=>{
  const deletion=await measureCodingRun(run([change('', 'export const unused = 1;')]),verified);
  assert.equal(deletion.accepted,true);assert.ok(deletion.removedCodeBytes>0);assert.equal(deletion.taskAmplification,0);
  const math=await measureCodingRun(run([], {taskKind:'mathematics'}),verified);
  assert.equal(math.accepted,true);assert.equal(math.taskAmplification,null);
});

test('failed self-coding attempts stay in the usually denominator',async()=>{
  const passing=await measureCodingRun(run([change('export const result = 7;')]),verified);
  const failed=await measureCodingRun(run([], {taskKind:'self-coding'}),async()=>({passed:false,checks:[],bindings:[]}));
  const secondFailure={...failed,runId:'separate-declared-failure'};
  const cohort=[passing,failed,secondFailure];
  const summary=summarizeCodingRuns(cohort,cohort.map(item=>item.runId));
  assert.equal(summary.coding.attempted,3);assert.equal(summary.coding.usuallyAmplifies,false);
  assert.equal(summary.selfCoding.accepted,0);assert.equal(summary.selfCoding.usuallyAmplifies,false);
});


test('a thrown independent verifier remains a failed measured attempt',async()=>{
  const result=await measureCodingRun(run([change('export const result = 7;')]),async()=>{throw new SyntaxError('actual invalid module');});
  assert.equal(result.accepted,false);assert.equal(result.autonomousNetCodeBytes,0);
  assert.deepEqual(result.verifierFailure,{name:'SyntaxError',message:'actual invalid module'});
  assert.equal(summarizeCodingRuns([result],[result.runId]).coding.attempted,1);
});

test('duplicate, missing and undeclared cohort runs cannot improve the usually count',async()=>{
  const result=await measureCodingRun(run([change('export const result = 7;')]),verified);
  assert.throws(()=>summarizeCodingRuns([result,result],[result.runId]),/duplicate run/);
  assert.throws(()=>summarizeCodingRuns([],[result.runId]),/retain every declared/);
  assert.throws(()=>summarizeCodingRuns([result],['other-declared-run']),/retain every declared/);
  assert.throws(()=>summarizeCodingRuns([result],[result.runId,result.runId]),/unique declared/);
});

test('missing original attempts and unsupported source languages are rejected',async()=>{
  await assert.rejects(measureCodingRun(run([], {attemptInputs:['replacement instructions']}),verified),/retain the original/);
  await assert.rejects(measureCodingRun(run([{...change('fn value() {}'),language:'rust'}]),verified),/supports JavaScript only/);
});


import { normalizeUsageReceipts } from '../../../experiments/formal_ai_subagent/coding-amplification.mjs';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
function usageFixture(overrides = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'usage-instrumentation-'));
  const producerPath = join(directory, 'producer.mjs');
  writeFileSync(producerPath, '// Instrumentation fixture, not an actual provider capture.\n');
  const record = {schemaVersion:1,kind:'normalized-provider-usage',receiptId:'receipt-a',runId:'usage-run',
    attemptId:'attempt-a',provider:'instrumentation-only',model:'control',requestId:'request-a',
    usage:{inputTokens:100,outputTokens:40,cachedInputTokens:20,reasoningTokens:10},
    counterSemantics:{cachedInput:'included-in-input',reasoning:'included-in-output'},...overrides};
  const path = join(directory, 'capture.json'); writeFileSync(path, JSON.stringify(record));
  const binding = file => ({path:file,sha256:sourceDigest(readFileSync(file))});
  const receipt = Object.fromEntries(['receiptId','runId','attemptId','provider','model','requestId'].map(name=>[name,record[name]]));
  receipt.capture=binding(path);receipt.producer=binding(producerPath);
  const input = run([], {runId:'usage-run',attemptIds:['attempt-a'],usageReceipts:[receipt]});
  return {record,receipt,input,path,producerPath,cleanup:()=>rmSync(directory,{recursive:true,force:true})};
}
test('source-bound usage counts token subsets once and never guesses prices or savings',async()=>{
 const f=usageFixture();try {
  const result=await measureCodingRun(f.input,verified);
  assert.deepEqual(result.modelTokens,{inputTokens:100,outputTokens:40,cachedInputTokens:20,reasoningTokens:10,totalTokens:140,scope:'captured-provider-receipts-only'});
  assert.equal(result.actualCost,null);assert.equal(result.monetarySavings,null);assert.equal(result.usageProvenanceRequiresReview,true);
  assert.equal(result.totalObservedInputBytes,Buffer.byteLength(f.input.task),'unobserved capture bytes are not charged as solver context');
 }finally{f.cleanup();}
});
test('absent and partial actual usage stay Unknown rather than zero or byte-derived',()=>{
 assert.equal(normalizeUsageReceipts(run([])).modelTokens,null);
 const f=usageFixture({usage:{inputTokens:100}});try {
  const result=normalizeUsageReceipts(f.input);assert.equal(result.modelTokens,null);assert.equal(result.usageStatus,'Unknown');
  assert.equal(result.usageReceipts[0].counters.inputTokens,100);assert.equal(result.usageReceipts[0].counters.outputTokens,null);
 }finally{f.cleanup();}
});
test('negative fractional unsafe and unsupported token fields are refused',()=>{
 for(const usage of [{inputTokens:-1,outputTokens:1},{inputTokens:1.5,outputTokens:1},{inputTokens:Number.MAX_SAFE_INTEGER+1,outputTokens:1},{inputTokens:1,outputTokens:1,inventedTokens:3}]) {
  const f=usageFixture({usage});try{assert.throws(()=>normalizeUsageReceipts(f.input),/token|fields/);}finally{f.cleanup();}
 }
});
test('cached and reasoning counters require explicit included subset semantics',()=>{
 for(const extra of [{usage:{inputTokens:10,outputTokens:5,cachedInputTokens:11}},{usage:{inputTokens:10,outputTokens:5,reasoningTokens:6}},{counterSemantics:{}}]) {
  const f=usageFixture(extra);try{assert.throws(()=>normalizeUsageReceipts(f.input),/subset/);}finally{f.cleanup();}
 }
});
test('capture digest drift and producer drift cannot enter token totals',()=>{
 const f=usageFixture();try {
  writeFileSync(f.path,'{}');assert.throws(()=>normalizeUsageReceipts(f.input),/drift/);
 }finally{f.cleanup();}
 const g=usageFixture();try {
  writeFileSync(g.producerPath,'drift');assert.throws(()=>normalizeUsageReceipts(g.input),/drift/);
 }finally{g.cleanup();}
});
test('cross-run cross-attempt model and request mismatches are refused',()=>{
 for(const mutate of [f=>{f.input.runId='another';},f=>{f.input.attemptIds=['another'];},f=>{f.receipt.model='another';},f=>{f.receipt.requestId='another';}]) {
  const f=usageFixture();try{mutate(f);assert.throws(()=>normalizeUsageReceipts(f.input),/identity|another/);}finally{f.cleanup();}
 }
});
test('duplicate captures and missing attempt identities cannot cheapen accounting',()=>{
 const f=usageFixture();try {
  assert.throws(()=>normalizeUsageReceipts({...f.input,usageReceipts:[f.receipt,f.receipt]}),/duplicate/);
  assert.throws(()=>normalizeUsageReceipts({...f.input,attemptIds:[]}),/attempt/);
 }finally{f.cleanup();}
});
test('malformed and unsupported captured schemas refuse normalization',()=>{
 const f=usageFixture();try {
  writeFileSync(f.path,'{');f.receipt.capture.sha256=sourceDigest(readFileSync(f.path));assert.throws(()=>normalizeUsageReceipts(f.input),/complete JSON/);
 }finally{f.cleanup();}
 const g=usageFixture({schemaVersion:2});try{assert.throws(()=>normalizeUsageReceipts(g.input),/schema/);}finally{g.cleanup();}
});
test('ordinary coding summary is separated while existing combined and selfcoding fields remain',async()=>{
 const ordinary=await measureCodingRun(run([change('export const answer = 42;')]),verified);
 const self=await measureCodingRun(run([],{taskKind:'self-coding'}),verified);
 const summary=summarizeCodingRuns([ordinary,self],[ordinary.runId,self.runId]);
 assert.equal(summary.coding.attempted,2);assert.equal(summary.ordinaryCoding.attempted,1);assert.equal(summary.selfCoding.attempted,1);
 assert.equal(summary.monetarySavings,null);
});
test('same provider request cannot be counted twice across cohort records',async()=>{
 const f=usageFixture();try {
  const first=await measureCodingRun(f.input,verified);
  const second={...first,runId:'distinct-task'};
  assert.throws(()=>summarizeCodingRuns([first,second],[first.runId,second.runId]),/duplicate usage/);
 }finally{f.cleanup();}
});


test('failed acceptance still retains actual captured resource counters',async()=>{
 const f=usageFixture();try {
  const result=await measureCodingRun(f.input,async()=>({passed:false,bindings:[],checks:[{name:'independent failure',exitCode:1}]}));
  assert.equal(result.accepted,false);assert.equal(result.autonomousNetCodeBytes,0);assert.equal(result.modelTokens.totalTokens,140);
 }finally{f.cleanup();}
});
test('distinct receipt IDs cannot duplicate a provider request across distinct runs',async()=>{
 const f=usageFixture(),g=usageFixture({receiptId:'receipt-b',runId:'second-run'});g.input.runId='second-run';
 try {
  const first=await measureCodingRun(f.input,verified),second=await measureCodingRun(g.input,verified);
  assert.notEqual(first.usageReceipts[0].receiptId,second.usageReceipts[0].receiptId);
  assert.throws(()=>summarizeCodingRuns([first,second],[first.runId,second.runId]),/duplicate usage/);
 }finally{f.cleanup();g.cleanup();}
});
