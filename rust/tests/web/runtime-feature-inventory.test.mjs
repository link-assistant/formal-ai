import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {buildRuntimeFeatureInventory,FEATURE_SOURCES,objectMembers,equalityChoices,browserProtocol} from '../../../scripts/lib/runtime-feature-inventory.mjs';
import {checkRuntimeFeatureInventory,renderRuntimeFeatureInventory} from '../../../scripts/generate-runtime-feature-inventory.mjs';
const root=new URL('../../../',import.meta.url);
const read=file=>fs.readFileSync(new URL(file,root),'utf8');
test('every shared route remains visible and corpus declarations do not establish runtime parity',()=>{
 const report=buildRuntimeFeatureInventory(read),routes=report.features.filter(row=>row.domain==='http-route');
 assert.ok(routes.length>0);assert.equal(report.behavioralParity,'unmeasured');assert.equal(report.discovery.complete,false);
 for(const row of routes){assert.equal(row.behavioralParity,'unmeasured');assert.equal(row.declaredEvidence[0].kind,'corpus_declared');}
 assert.deepEqual(report.runtimeObservedEvidence,[]);
});
test('native command denominator is visibly unresolved rather than inferred from JS names',()=>{
 const report=buildRuntimeFeatureInventory(read);assert.equal(report.discovery.nativeCli,'pending_actual_clap_schema');
 assert.ok(report.gaps.some(row=>row.domain==='native-cli'));for(const row of report.features.filter(row=>row.domain==='cli-command'))assert.equal(row.runtimes.rust.status,'unmeasured');
});
test('a missing or new route cannot disappear from the generated denominator',()=>{
 const report=buildRuntimeFeatureInventory(file=>file===FEATURE_SOURCES.routes?read(file)+'\n  route heldout_new_route\n    method GET\n    path /heldout\n':read(file));
 assert.ok(report.features.some(row=>row.id==='http:heldout_new_route'));assert.ok(report.gaps.some(row=>row.id==='http:heldout_new_route'));
});
test('table extraction refuses dynamic, missing and ambiguous authoritative members',()=>{
 assert.deepEqual(objectMembers('const ACTIONS={run:{},resume:{}}','ACTIONS'),['run','resume']);
 for(const source of ['const ACTIONS={...other}','const unrelated={}','const ACTIONS={};function f(){const ACTIONS={}}'])assert.throws(()=>objectMembers(source,'ACTIONS'));
});
test('browser message ownership and dynamic response shapes fail closed',()=>{
 const source='self.onmessage=event=>{const data=event.data;if(data.kind==="probe")postMessage({kind:"answer"});}';
 assert.deepEqual(browserProtocol(source).requests,['probe']);assert.throws(()=>browserProtocol(source+'self.onmessage=()=>{}'));
 assert.throws(()=>browserProtocol('self.onmessage=()=>{postMessage(dynamic)}'));
});
test('source changes force inventory regeneration without promoting evidence',()=>{
 const expected=renderRuntimeFeatureInventory(read);assert.equal(checkRuntimeFeatureInventory(read,expected),true);
 assert.equal(checkRuntimeFeatureInventory(file=>read(file)+'\n',expected),false);
});
test('CLI comparisons are actual AST selectors, not matching comment text',()=>{
 assert.deepEqual(equalityChoices('// argv[0] === "fake"\nif(argv[0]==="logs"){}','argv'),['logs']);
});

test('declarative feature rows stay one per line and generated-data tampering fails check',()=>{
 const expected=renderRuntimeFeatureInventory(read),inventory=JSON.parse(expected);
 assert.equal(expected.split('\n').filter(line=>line.startsWith('    {"id":')).length,inventory.features.length);
 assert.ok(expected.split('\n').length<1500);
 assert.equal(checkRuntimeFeatureInventory(read,expected.replace('"behavioralParity":"unmeasured"','"behavioralParity":"verified"')),false);
});
