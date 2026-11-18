// Candidate-scope evidence only; this suite never loads a native grammar.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {isDeepStrictEqual} from 'node:util';
import {observedCargoSourceDirectories,observedManifestFacts} from '../../../js/agentic/source_manifest_facts.mjs';
const blocked=()=>{throw Error('LocalWasmExecutionForbidden');};
globalThis.WebAssembly={...WebAssembly,instantiate:blocked,compile:blocked,Module:blocked,Instance:blocked,instantiateStreaming:blocked,compileStreaming:blocked};
const fixtures=JSON.parse(readFileSync(new URL('../fixtures/cargo-source-directories.json',import.meta.url),'utf8'));
for(const entry of fixtures.cases)test(entry.name,()=>{
 const result=observedCargoSourceDirectories(entry.owner,entry.source);
 assert.equal(result.schema,'observed-cargo-source-directories/v1');assert.equal(result.manifest,entry.owner);
 assert.equal(result.scope,'cargo-lib-bin-candidate-search-scopes-only; no existence/target/runtime claim');
 if(entry.gap){assert.equal(result.complete,false);assert.deepEqual(result.directories,[]);assert.deepEqual(result.facts,[]);assert.deepEqual(result.gaps,[{reason:entry.gap}]);return;}
 const sourceIdentity=createHash('sha256').update(entry.source).digest('hex');assert.equal(result.sourceIdentity,sourceIdentity);
 assert.equal(result.complete,true);assert.deepEqual(result.gaps,[]);assert.deepEqual(result.directories,entry.directories);
 assert.deepEqual(result.facts.map(({kind,manifest,sourceIdentity:identity,...fact})=>{assert.equal(kind,'cargo-source-directory-candidate');assert.equal(manifest,entry.owner);assert.equal(identity,sourceIdentity);return fact;}),entry.facts);
});
test('UTF8 source cap and input types refuse without observation identity',()=>{
 for(const source of ['雪'.repeat(87382),null]){const result=observedCargoSourceDirectories('rust/Cargo.toml',source);assert.equal(result.complete,false);assert.equal(result.sourceIdentity,null);assert.deepEqual(result.gaps,[{reason:'CargoSourceBudgetExceeded'}]);}
});
test('existing manifest observations retain their shared contract',()=>{
 const original=JSON.parse(readFileSync(new URL('../fixtures/source-manifest-facts.json',import.meta.url),'utf8'));
 for(const entry of original.cases){
  if(entry.rejected){assert.throws(()=>observedManifestFacts(entry.owner,entry.source));continue;}
  const actual=observedManifestFacts(entry.owner,entry.source).map(({manifest,sourceIdentity,...fact})=>fact);
  if(entry.expected)assert.deepEqual(actual,entry.expected,entry.name);
  else for(const expected of entry.expectedSubset)assert.ok(actual.some(row=>isDeepStrictEqual(row,expected)),entry.name);
 }
});
