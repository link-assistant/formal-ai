import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {parse} from '@babel/parser';
import path from 'node:path';
import os from 'node:os';
import traversal from '@babel/traverse';
import {callbackProperty} from '../../../js/agentic/callback_property.mjs';
import {planUnaryConsolidation,applyUnaryConsolidation} from '../../../scripts/consolidate-stateless-unary.mjs';
const traverse=traversal.default??traversal;
const workspace=fs.mkdtempSync(path.join(os.tmpdir(),'formal-property-contract-'));
test.after(()=>fs.rmSync(workspace,{recursive:true,force:true}));
fs.mkdirSync(path.join(workspace,'js/agentic'),{recursive:true});
fs.writeFileSync(path.join(workspace,'js/agentic/callback_property.mjs'),'export '+callbackProperty.toString()+'\n');
const contracts={property:{module:'js/agentic/callback_property.mjs',exportName:'callbackProperty'}};
test('independent whole-request stateless unary callback coverage',()=>{
 const owner='js/agentic/coding_research_observation.mjs';
 const original=fs.readFileSync(new URL('../../../'+owner,import.meta.url),'utf8');
 fs.writeFileSync(path.join(workspace,owner),original);
 const plan=planUnaryConsolidation(workspace,[owner],contracts);
 assert.equal(plan.deferred.length,0);
 applyUnaryConsolidation(plan);
 const source=fs.readFileSync(path.join(workspace,owner),'utf8');
 const remaining=[],captured=[];
 traverse(parse(source,{sourceType:'module'}),{ArrowFunctionExpression(p){
  if(p.node.params.length!==1||p.node.params[0].type!=='Identifier')return;
  const local=p.scope.getBinding(p.node.params[0].name);let captures=false;
  p.traverse({ReferencedIdentifier(reference){if(reference.scope.getBinding(reference.node.name)!==local)captures=true;}});
  (captures?captured:remaining).push(source.slice(p.node.start,p.node.end));
 }});
 assert.deepEqual(remaining,[]);assert.equal(captured.length,1);
 assert.match(captured[0],/declaration/u);
 assert(source.includes('// Lossless lexical/catalog observations; canonical kernels make research decisions.'));
 assert.equal(source.includes('token => token.kind'),false);
});
test('generic factory preserves arbitrary member names, getter receiver and one read',()=>{
 for(const property of ['kind','arbitraryField','étiquette','constructor','__proto__']){
  let count=0;const object={};Object.defineProperty(object,property,{get(){count++;assert.equal(this,object);return property;}});
  assert.equal(callbackProperty(property)(object),property);assert.equal(count,1);
 }
 for(const value of [null,undefined])assert.throws(()=>callbackProperty('whatever')(value),TypeError);
 const error=new Error('getter');assert.throws(()=>callbackProperty('field')({get field(){throw error;}}),value=>value===error);
});
test('operation derives arbitrary identifier fields from AST, never task names',()=>{
 for(const property of ['arbitraryField','étiquette','constructor']){
  const relative='js/agentic/fixture_'+property+'.mjs';fs.writeFileSync(workspace+'/'+relative,'export const values = items.map(entry => entry.'+property+');\n');
  const plan=planUnaryConsolidation(workspace,[relative],contracts);
  assert.equal(plan.changes.length,1);assert.equal(plan.deferred.length,0);
  assert(plan.changes[0].after.includes('canonicalUnaryProperty('+JSON.stringify(property)+')'));
 }
});
test('computed optional captured and comment-bearing projections remain unresolved',()=>{
 for(const expression of ['entry => entry[field]','entry => entry?.field','entry => external.field','entry => entry./* keep */field']){
  const relative='js/agentic/refusal.mjs';fs.writeFileSync(workspace+'/'+relative,'export const values = items.map('+expression+');\n');
  const plan=planUnaryConsolidation(workspace,[relative],contracts);
  assert.equal(plan.changes.length,0);assert.equal(plan.deferred.length,1,expression);
 }
});
test('factory source identity and exact AST body remain required',()=>{
 const helper=workspace+'/js/agentic/callback_property.mjs',original=fs.readFileSync(helper,'utf8');
 const relative='js/agentic/drift.mjs';fs.writeFileSync(workspace+'/'+relative,'export const values = items.map(entry => entry.field);\n');
 const plan=planUnaryConsolidation(workspace,[relative],contracts);
 try{
  fs.writeFileSync(helper,original.replace('value[property]','value.other'));
  assert.throws(()=>applyUnaryConsolidation(plan),/UnaryHelperDrift/u);
  assert.throws(()=>planUnaryConsolidation(workspace,[relative],contracts),/UnaryHelperBodyMismatch/u);
 }finally{fs.writeFileSync(helper,original);}
});

test('applied emitted consumer preserves literal values getter dispatch errors and comments',async()=>{
 const {pathToFileURL}=await import('node:url');
 const relative='js/agentic/runtime-consumer.mjs';
 const file=path.join(workspace,relative);
 const comment='// preserved independent consumer comment';
 fs.writeFileSync(file,comment+'\nexport function collect(values){return values.map(entry=>entry.selected); }\n');
 const plan=planUnaryConsolidation(workspace,[relative],contracts);
 assert.equal(plan.changes.length,1);assert.equal(plan.deferred.length,0);
 applyUnaryConsolidation(plan);
 assert(fs.readFileSync(file,'utf8').includes(comment));
 const {collect}=await import(pathToFileURL(file).href);
 let reads=0;
 const receiver={get selected(){reads++;assert.equal(this,receiver);return 'literal-result';}};
 assert.deepEqual(collect([receiver,{selected:42}]),['literal-result',42]);
 assert.equal(reads,1);
 const sentinel=new Error('independent getter sentinel');
 let failedReads=0;
 const failingReceiver={get selected(){failedReads++;assert.equal(this,failingReceiver);throw sentinel;}};
 assert.throws(()=>collect([failingReceiver]),error=>error===sentinel);
 assert.equal(failedReads,1);
 assert.throws(()=>collect([null]),TypeError);
});
