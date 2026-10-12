import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import {structuralRecoveryPlan, consumeRecoveryDecision, observeReleaseRecovery,
  validateReleaseSnapshot} from '../../../scripts/release-recovery-decision.mjs';
const definition = JSON.parse(fs.readFileSync(new URL('../../../.github/release-advertisement.json', import.meta.url)));
const plan = structuralRecoveryPlan(definition, '1.2.3', []);
test('advertised assets derive from maintained source contract and absence is structural only', () => {
  assert.ok(plan.expected.length > 20);
  assert.deepEqual(plan.potentiallyMissing, plan.expected);
  assert.equal(plan.publicationComplete, false);
  assert.equal(plan.authenticated, false);
});
test('three-product presence cannot establish all-target completion', () => {
  const result = structuralRecoveryPlan(definition, '1.2.3', [{id:1,name:'core-crate',size:1,state:'uploaded',digest:null}]);
  assert.deepEqual(result.potentiallyMissing, plan.expected);
  assert.equal(result.registryEvidence, 'Unavailable');
});
test('all names present still lack registry/runtime/signing authority', () => {
  const assets = plan.expected.map((name,i)=>({id:i+1,name,size:1,state:'uploaded',digest:null}));
  const result = structuralRecoveryPlan(definition, '1.2.3', assets);
  assert.deepEqual(result.potentiallyMissing, []);
  assert.equal(result.publicationComplete, false);
});
for(const [name,rows] of [
 ['duplicate id',[{id:1,name:'a',size:1,state:'uploaded',digest:null},{id:1,name:'b',size:1,state:'uploaded',digest:null}]],
 ['duplicate name',[{id:1,name:'a',size:1,state:'uploaded',digest:null},{id:2,name:'a',size:1,state:'uploaded',digest:null}]],
 ['unsafe path',[{id:1,name:'../a',size:1,state:'uploaded',digest:null}]],
 ['unfinished asset',[{id:1,name:'a',size:1,state:'new',digest:null}]],
 ['false digest',[{id:1,name:'a',size:1,state:'uploaded',digest:'false'}]]
]) test('metadata refuses '+name,()=>assert.throws(()=>structuralRecoveryPlan(definition,'1.2.3',rows)));
test('unknown/copied source cannot invoke network observer', () => {
 assert.throws(()=>observeReleaseRecovery({repository:'owner/repository'}));
 assert.throws(()=>validateReleaseSnapshot({}, {}, []));
});
test('caller JSON cannot mint selective repair or complete authorization', () => {
 assert.throws(()=>consumeRecoveryDecision({}, {authenticated:true, publicationComplete:true, missingAssets:[]}));
});

test('source-derived target set is deeply immutable', () => {
 assert.equal(Object.isFrozen(plan.expected), true);
 assert.equal(Object.isFrozen(plan.potentiallyMissing), true);
 assert.throws(()=>plan.expected.push('foreign-target'));
 assert.throws(()=>plan.potentiallyMissing.push('foreign-target'));
});