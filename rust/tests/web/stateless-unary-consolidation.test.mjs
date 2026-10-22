import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {parse} from '@babel/parser';
import {unaryOperation, planUnaryConsolidation, applyUnaryConsolidation, writeCompleteUnaryInventory} from '../../../scripts/consolidate-stateless-unary.mjs';
import {lower, trim as rustTrim} from '../../../js/agentic/crate/rust_str.mjs';
import {callbackText,callbackValue,callbackTrim,callbackNonempty,callbackHasLength} from '../../../js/agentic/callback_values.mjs';
import {lower as publicLower} from '../../../js/agentic/write_str.mjs';
const expression = text => parse('const callback = '+text+';', {sourceType:'module'}).program.body[0].declarations[0].init;
const fixture = () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'unary-source-control-')));
  fs.mkdirSync(path.join(root,'js/agentic'),{recursive:true});
  fs.writeFileSync(path.join(root,'js/agentic/primitive.mjs'), 'export function lower(value) { return value.toLowerCase(); }\n');
  fs.writeFileSync(path.join(root,'js/agentic/consumer.mjs'), "// preserve this comment\nexport function normalize(values) { return values.map(value => value.toLowerCase()); }\n");
  const contracts = {lower:{module:'js/agentic/primitive.mjs',exportName:'lower'}};
  return {root,contracts,consumer:path.join(root,'js/agentic/consumer.mjs'),cleanup:()=>fs.rmSync(root,{recursive:true,force:true})};
};
test('exact stateless shapes exclude captures, optional/computed access, extra calls and async effects',()=>{
  for(const [text,expected] of [['x=>x.toLowerCase()','lower'],['x=>x.trim()','trim'],['x=>x.text','text'],['x=>x.value','value'],["x=>x!==''",'nonempty'],['x=>x.length>0','positiveLength']]) assert.equal(unaryOperation(expression(text)),expected);
  for(const text of ['x=>x.name===name','x=>lower.includes(x)','async x=>x.text','(x=defaultValue)=>x.text',"x=>x['text']",'x=>x?.text','x=>x.toLowerCase(extra)','x=>{return x.text;}']) assert.equal(unaryOperation(expression(text)),null,text);
});
test('existing public lowercase API shares one canonical binding with Unicode and exception behavior',()=>{
  assert.equal(lower,publicLower);
  const original = value => value.toLowerCase();
  for(const value of ['ASCII', 'İ', 'ß', 'Σ', '\uFEFF\u2003', '', undefined, null, 7, Object.create(null)]) {
    const observe = fn => {try{return {value:fn(value)};}catch(error){return {error:error.constructor};}};
    assert.deepEqual(observe(lower),observe(original));
  }
});
test('shared member calls preserve getter count, receiver identity and thrown original objects',()=>{
  const failure = new Error('declared failure');
  let calls = 0;
  const value = {get toLowerCase(){calls++;return function(){assert.equal(this,value);throw failure;};}};
  assert.throws(()=>lower(value),error=>error===failure);
  assert.equal(calls,1);
  const original = item => item.toLowerCase();
  assert.throws(()=>original(value),error=>error===failure);
  assert.equal(calls,2);
});
test('source plan preserves all preimages and rejects serialized authority and later source drift',()=>{
  const f=fixture();try {
    const plan=planUnaryConsolidation(f.root,['js/agentic/consumer.mjs'],f.contracts);
    assert.equal(plan.changes.length,1);assert.equal(plan.changes[0].edits.length,1);
    assert.match(plan.changes[0].after,/preserve this comment/u);
    assert.equal(Object.isFrozen(plan.changes[0]),true);
    assert.throws(()=>applyUnaryConsolidation(JSON.parse(JSON.stringify(plan))),/UnaryPlanRequired/u);
    fs.appendFileSync(f.consumer,'// changed after observation\n');
    const drifted=fs.readFileSync(f.consumer,'utf8');
    assert.throws(()=>applyUnaryConsolidation(plan),/UnarySourceDrift/u);
    assert.equal(fs.readFileSync(f.consumer,'utf8'),drifted);
  }finally{f.cleanup();}
});
test('actual callback consolidation keeps observable outputs on supported standard-array consumers',async()=>{
  const f=fixture();try {
    const plan=planUnaryConsolidation(f.root,['js/agentic/consumer.mjs'],f.contracts);
    const result=applyUnaryConsolidation(plan);assert.deepEqual(result,{files:1,callbacks:1,requiresIndependentConsumerChecks:true});
    const {pathToFileURL}=await import('node:url');
    const transformed=await import(pathToFileURL(f.consumer).href);
    const values=['HELLO','İ','ß','Σ'];
    assert.deepEqual(transformed.normalize(values),values.map(value=>value.toLowerCase()));
    assert.deepEqual(plan.changes[0].edits[0].captures,[]);
    assert.equal(plan.completeSemanticEquivalence,false);
  }finally{f.cleanup();}
});
test('helper body drift, import collisions and noncallback declarations remain refused or deferred',()=>{
  const f=fixture();try {
    fs.writeFileSync(f.consumer,'const canonicalUnaryLower=7; export const result=["X"].map(x=>x.toLowerCase());\n');
    assert.throws(()=>planUnaryConsolidation(f.root,['js/agentic/consumer.mjs'],f.contracts),/UnaryImportBindingCollision/u);
    fs.writeFileSync(f.consumer,'export const callback=x=>x.toLowerCase();\n');
    const deferred=planUnaryConsolidation(f.root,['js/agentic/consumer.mjs'],f.contracts);
    assert.equal(deferred.changes.length,0);assert.equal(deferred.deferred.length,1);
    fs.writeFileSync(path.join(f.root,'js/agentic/primitive.mjs'),'export function lower(value) { return String(value).toLowerCase(); }\n');
    assert.throws(()=>planUnaryConsolidation(f.root,['js/agentic/consumer.mjs'],f.contracts),/UnaryHelperBodyMismatch/u);
  }finally{f.cleanup();}
});
test('complete inventory streams every raw/independent group and proof, with digest-bound manifests',()=>{
  const f=fixture(),destination=f.root+'-inventory';try {
    const group={fingerprint:'declared',sites:[{file:'js/agentic/consumer.mjs',start:0,end:1}]};
    const report={files:1,callables:1,sources:[{file:'declared',sha256:'abc'}],clones:[group],independentClones:[group],proofs:[{accepted:false}],gaps:[],rawCloneGroups:1,independentCloneGroups:1,accepted:false};
    const manifest=writeCompleteUnaryInventory(f.root,destination,{scan:()=>report});
    assert.equal(manifest.complete,true);assert.equal(manifest.accepted,false);
    assert.equal(manifest.members.length,5);
    for(const member of manifest.members){const bytes=fs.readFileSync(path.join(destination,member.path));assert.equal(bytes.length,member.bytes);assert.equal(createHash('sha256').update(bytes).digest('hex'),member.sha256);}
    assert.equal(fs.readFileSync(path.join(destination,'raw-groups.jsonl'),'utf8'),JSON.stringify(group)+'\n');
  }finally{f.cleanup();fs.rmSync(destination,{recursive:true,force:true});}
});
test('nonfinite inventory bounds refuse before IO and truncated output cannot gain a complete manifest',()=>{
  const f=fixture(),destination=f.root+'-refusal';try {
    let scans=0;
    assert.throws(()=>writeCompleteUnaryInventory(f.root,destination,{maximumBytes:Infinity,scan:()=>{scans++;}}),/FiniteInventoryBudgetRequired/u);
    assert.equal(scans,0);assert.equal(fs.existsSync(destination),false);
    const report={sources:[{large:'x'.repeat(30)}],clones:[],independentClones:[],proofs:[],gaps:[]};
    assert.throws(()=>writeCompleteUnaryInventory(f.root,destination,{maximumBytes:10,scan:()=>report}),/CompleteInventoryBudgetExceeded/u);
    assert.equal(fs.existsSync(path.join(destination,'manifest.json')),false);
  }finally{f.cleanup();fs.rmSync(destination,{recursive:true,force:true});}
});

test('ECMAScript trim, strict emptiness and positive length retain separate original contracts',()=>{
  assert.equal(callbackTrim('\uFEFF value \uFEFF'),'value');
  assert.notEqual(callbackTrim('\u0085value\u0085'),rustTrim('\u0085value\u0085'));
  for(const value of ['', 'x', 0, false, null, undefined, {}, []])assert.equal(callbackNonempty(value),value!=='');
  for(const value of ['', 'x', [], [0], {length:0}, {length:'2'}])assert.equal(callbackHasLength(value),value.length>0);
});
test('field projection helpers retain getters, original receivers and exact thrown objects',()=>{
  let reads=0;const failure={declared:'failure'};
  const value={get text(){reads++;assert.equal(this,value);return 'kept';},get value(){reads++;throw failure;}};
  assert.equal(callbackText(value),'kept');assert.equal(reads,1);
  assert.throws(()=>callbackValue(value),error=>error===failure);assert.equal(reads,2);
});

test('comments inside callback source remain attached instead of being discarded',()=>{
 const f=fixture();try{fs.writeFileSync(f.consumer,'export const values=["A"].map(value=> /* keep rationale */ value.toLowerCase());\n');
 const plan=planUnaryConsolidation(f.root,['js/agentic/consumer.mjs'],f.contracts);
 assert.equal(plan.changes.length,0);assert.equal(plan.deferred[0].reason,'CommentInsideCallbackRequiresOwnedTransfer');
 }finally{f.cleanup();}
});
