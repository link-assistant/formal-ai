import assert from 'node:assert/strict';
import { test } from 'node:test';
import { sourceDigest } from '../../../experiments/formal_ai_subagent/coding-amplification.mjs';
import { categoryRules, declareTask, measureTaskRelation, nearUniversalObjective, summarizeTaskRelations }
  from '../../../experiments/formal_ai_subagent/task-relations.mjs';

let sequence = 0;
const task = (category, taskKind, original) => declareTask({runId:'relation-'+String(++sequence), category, taskKind, task:original});
const run = (declaration, extra={}) => ({...declaration,origin:'autonomous',attemptInputs:[declaration.task],
  repositoryContext:[],reviewedPatches:[],toolReceipts:[],changes:[],...extra});
const proof = (changes,response) => ({passed:true,checks:[{name:'independent behavioral acceptance',exitCode:0}],
  bindings:changes.map(change=>({path:change.path,before:sourceDigest(change.before),after:sourceDigest(change.after)})),
  responseSHA256:typeof response==='string'?sourceDigest(response):null});

test('actual arithmetic and boolean decision responses compress independently checked tasks',async()=>{
  for(const [category,kind,original,response,expected] of [
    ['calculation','mathematics','Calculate 17 multiplied by 23.','391',17*23],
    ['decision','other','Is 97 a prime number? Answer yes or no.','yes',true],
  ]){
    const declaration=task(category,kind,original);
    const measured=await measureTaskRelation(declaration,run(declaration,{response}),async(changes,actual)=>{
      if(category==='calculation')assert.equal(Number(actual),expected);
      else {let prime=true;for(let divisor=2;divisor*divisor<=97;divisor++)if(97%divisor===0)prime=false;
        assert.equal(prime,expected);assert.equal(actual,prime?'yes':'no');}
      return proof(changes,actual);
    });
    assert.equal(measured.accepted,true);assert.equal(measured.observedRelation,'output-smaller');
    assert.equal(measured.matchesExpectation,true);assert.ok(measured.outputPerOriginalTask<1);
  }
});

test('a failing calculation and a response digest for different bytes cannot satisfy compression',async()=>{
  const declaration=task('calculation','mathematics','Calculate 17 multiplied by 23.');
  const wrong=await measureTaskRelation(declaration,run(declaration,{response:'392'}),async(changes,response)=>{
    assert.equal(Number(response),17*23);return proof(changes,response);
  });
  assert.equal(wrong.accepted,false);assert.equal(wrong.observedRelation,'unknown');assert.equal(wrong.matchesExpectation,false);
  const drift=await measureTaskRelation(declaration,run(declaration,{response:'391'}),async changes=>proof(changes,'392'));
  assert.equal(drift.responseBound,false);assert.equal(drift.accepted,false);
});

test('feature expansion uses executed code and independently bound lexical changes',async()=>{
  const declaration=task('feature-implementation','self-coding','Add sum.');
  const source='export function sum(first, second) { return first + second; }';
  const changes=[{path:'sum.mjs',role:'production',language:'javascript',before:'',after:source}];
  const result=await measureTaskRelation(declaration,run(declaration,{changes}),async observed=>{
    const module=await import('data:text/javascript;base64,'+Buffer.from(observed[0].after).toString('base64'));
    assert.equal(module.sum(7,-3),4);assert.equal(module.sum(-6,2),-4);return proof(observed);
  });
  assert.equal(result.expectedRelation,'output-larger');assert.equal(result.matchesExpectation,true);
  assert.equal(result.outputMeasure,'autonomous-validated-net-code-proxy');
});

test('deletion and repair remain acceptable without an expansion floor',async()=>{
  const declaration=task('deletion','coding','Remove the unused export.');
  const changes=[{path:'unused.mjs',role:'production',language:'javascript',before:'export const unused = 1;',after:''}];
  const result=await measureTaskRelation(declaration,run(declaration,{changes}),proof);
  assert.equal(result.accepted,true);assert.ok(result.removedCodeBytes>0);assert.equal(result.matchesExpectation,null);
  assert.equal(categoryRules.repair.expectedRelation,'unrestricted');assert.equal(categoryRules.proof.expectedRelation,'output-larger');
});

test('category declarations bind original task, task kind and complete cohort',async()=>{
  const declaration=task('decision','other','Is 97 prime?');
  await assert.rejects(measureTaskRelation(declaration,run(declaration,{task:'Replacement task',response:'yes'}),proof),/original task/);
  assert.throws(()=>declareTask({...declaration,taskKind:'coding'}),/fit the task kind/);
  await assert.rejects(measureTaskRelation({...declaration,category:'explanation'},run(declaration,{response:'yes'}),proof),/declaration changed/);
  const result=await measureTaskRelation(declaration,run(declaration,{response:'yes'}),proof);
  assert.throws(()=>summarizeTaskRelations([{...result,category:'explanation'}],[declaration]),/declared category/);
  assert.throws(()=>summarizeTaskRelations([result,result],[declaration]),/duplicate measured/);
  assert.throws(()=>summarizeTaskRelations([],[declaration]),/missing declared/);
});

test('near-universal reporting retains failures and requires a meaningful declared cohort',async()=>{
  assert.deepEqual(nearUniversalObjective,{minimumTasks:20,acceptedMatchingShare:0.95});
  const declarations=[],measurements=[];
  for(let index=0;index<20;index++){
    const declaration=task('calculation','mathematics','Calculate 17 multiplied by 23.');declarations.push(declaration);
    measurements.push(await measureTaskRelation(declaration,run(declaration,{response:index===0?'wrong':'391'}),async(changes,response)=>{
      assert.equal(response,'391');return proof(changes,response);
    }));
  }
  const summary=summarizeTaskRelations(measurements,declarations).calculation;
  assert.equal(summary.attempted,20);assert.equal(summary.accepted,19);assert.equal(summary.acceptedMatchingShare,0.95);
  assert.equal(summary.meetsNearUniversalObjective,true);
  assert.equal(summarizeTaskRelations(measurements.slice(1),declarations.slice(1)).calculation.meetsNearUniversalObjective,null);
});

test('an exact large integer can legitimately expand a calculation request',async()=>{
  const declaration=task('calculation','mathematics','Calculate 2^128.');
  const response=(2n**128n).toString();
  const result=await measureTaskRelation(declaration,run(declaration,{response}),async(changes,actual)=>{
    assert.equal(BigInt(actual),1n<<128n);return proof(changes,actual);
  });
  assert.equal(result.accepted,true);assert.equal(result.observedRelation,'output-larger');
  assert.equal(result.matchesExpectation,false);assert.ok(result.outputPerOriginalTask>1);
});


test('reviewed feature work remains accepted but cannot satisfy autonomous category expansion',async()=>{
 const declaration=task('feature-implementation','self-coding','Add sum.');
 const source='export function sum(first, second) { return first + second; }';
 const changes=[{path:'sum.mjs',role:'production',language:'javascript',before:'',after:source}];
 const result=await measureTaskRelation(declaration,run(declaration,{origin:'reviewed-patch',reviewedPatches:[source],changes}),async(actual)=>{
  const module=await import('data:text/javascript,'+encodeURIComponent(actual[0].after));assert.equal(module.sum(2,3),5);return proof(actual);
 });
 assert.equal(result.accepted,true);assert.equal(result.outputBytes,0);assert.equal(result.matchesExpectation,false);
 assert.equal(result.modelTokens,null);assert.equal(result.monetarySavings,null);
});
test('independent extractive compression acceptance keeps missing usage Unknown',async()=>{
 const declaration=task('extraction','other','Extract only the launch year from this source: the satellite launched in 1998 and retired in 2004.');
 const response='1998';
 const result=await measureTaskRelation(declaration,run(declaration,{response}),async(changes,actual)=>{
  assert.equal(actual,'1998');return proof(changes,actual);
 });
 assert.equal(result.accepted,true);assert.equal(result.matchesExpectation,true);assert.equal(result.usageStatus,'Unknown');
 const wrong=await measureTaskRelation(declaration,run(declaration,{response:'2004'}),async(changes,actual)=>{
  assert.equal(actual,'1998');return proof(changes,actual);
 });
 assert.equal(wrong.accepted,false);assert.equal(wrong.matchesExpectation,false);
});
