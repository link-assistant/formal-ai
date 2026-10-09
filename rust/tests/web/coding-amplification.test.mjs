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
