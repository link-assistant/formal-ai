import test from 'node:test';
import assert from 'node:assert/strict';
import {createWorkerContext,evaluate,plain} from './support/browser-runtime.mjs';
const helperNames=["factSeedRoot","factChildren","factChildText","factList","factCapturedEntities","factRealizationRules","factSurface","factDefinite","factAcronym","factInflect","factForm","factRealize","factSourceUrl","factLabelSurfaces"];
for(const reverse of [false,true])test('canonical helper identities and raw mutations survive '+(reverse?'reverse':'normal')+' module ordering',async()=>{
 const worker=createWorkerContext({}, {reorderModules:reverse?modules=>modules.reverse():null});
 // Reading an alias before seed loading instantiates no cached fact result.
 assert.equal(evaluate(worker,'factSeedRoot'),evaluate(worker,"crateModule('crate/fact_derivation.mjs').factSeedRoot"));
 await evaluate(worker,'loadSeed()');
 for(const name of helperNames)assert.equal(evaluate(worker,name),evaluate(worker,"crateModule('crate/fact_derivation.mjs')."+name),name);
 const records=plain(evaluate(worker,'derivedCaptureFacts(SEED_RAW)'));
 assert(records.length>0);
 const differential=plain(evaluate(worker,`(()=>{
  const raw={...SEED_RAW};
  const direct=()=>crateModule('crate/fact_derivation.mjs').deriveFactsFromTexts(
   seedRawText(raw,FACT_CAPTURES_FILE),seedRawText(raw,FACT_REALIZATION_FILE),factRelationProperty);
  const baseline=derivedCaptureFacts(raw);
  const captures=Object.keys(raw).find(key=>seedFileBaseName(key)===FACT_CAPTURES_FILE);
  const realization=Object.keys(raw).find(key=>seedFileBaseName(key)===FACT_REALIZATION_FILE);
  raw[captures]='';
  const absent=derivedCaptureFacts(raw);
  const absentDirect=direct();
  raw[captures]=SEED_RAW[captures].replaceAll('P36','P999999999');
  const changed=derivedCaptureFacts(raw);
  const changedDirect=direct();
  raw[captures]=SEED_RAW[captures];raw[realization]='';
  const noRules=derivedCaptureFacts(raw);
  return {baseline,absent,absentDirect,changed,changedDirect,noRules,
   restored:derivedCaptureFacts(SEED_RAW)};
 })()`));
 assert.deepEqual(differential.baseline,records);
 assert.deepEqual(differential.absent,[]);
 assert.deepEqual(differential.absentDirect,[]);
 assert.deepEqual(differential.noRules,[]);
 assert.deepEqual(differential.changed,differential.changedDirect);
 assert(differential.changed.length<records.length);
 assert.deepEqual(differential.restored,records);
});
test('canonical factory and worker retain all legacy helper property bindings',async()=>{
 const worker=createWorkerContext();await evaluate(worker,'loadSeed()');
 for(const name of helperNames){
  const descriptor=evaluate(worker,'Object.getOwnPropertyDescriptor(self,'+JSON.stringify(name)+')');
  assert.equal(descriptor.configurable,true);assert.equal(descriptor.enumerable,true);
  assert.equal(typeof descriptor.get,'function');assert.equal(typeof descriptor.set,'function');
 }
 const original=evaluate(worker,'factAcronym');
 evaluate(worker,'factAcronym = () => "replacement"');
 assert.equal(evaluate(worker,'factAcronym("anything")'),'replacement');
 assert.equal(typeof original,'function');
 assert.notEqual(evaluate(worker,'factAcronym'),evaluate(worker,"crateModule('crate/fact_derivation.mjs').factAcronym"));
});
