import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {nativeProvenanceManifest,nativeProvenanceDependencies,resolveNativeProvenanceDependencies} from '../../../scripts/resolve-provenance-native-dependencies.mjs';
const source=fs.readFileSync(new URL('../../Cargo.toml',import.meta.url),'utf8');
test('native provenance dependency plan is complete, target scoped and repeatable without executing Cargo',()=>{
 const output=nativeProvenanceManifest(source);
 for(const [name,value]of Object.entries(nativeProvenanceDependencies))assert.ok(output.includes(name+' = '+value+'\n'));
 assert.equal(nativeProvenanceManifest(output),output);
 assert.ok(output.indexOf('blake3 =')>output.indexOf('[target.'));
 assert.ok(output.indexOf('blake3 =')<output.indexOf('[features]'));
 assert.ok(output.includes('provenance-gpu = ["dep:wgpu", "dep:pollster"]'));
});
test('ambiguous target sections or conflicting existing dependencies are refused',()=>{
 assert.throws(()=>nativeProvenanceManifest(source+'\n[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\n'));
 assert.throws(()=>nativeProvenanceManifest(source.replace('[dependencies]\n','[dependencies]\nblake3 = "0.1"\n')));
 const complete=nativeProvenanceManifest(source);
 assert.throws(()=>nativeProvenanceManifest(complete.replace('provenance-gpu = ["dep:wgpu", "dep:pollster"]','provenance-gpu = []')));
});
test('local execution is refused before filesystem or Cargo effects',()=>{
 const previous=process.env.GITHUB_ACTIONS;
 try{delete process.env.GITHUB_ACTIONS;assert.throws(()=>resolveNativeProvenanceDependencies('/nonexistent-root','/nonexistent-output'),/hosted-CI-only/u);}
 finally{if(previous===undefined)delete process.env.GITHUB_ACTIONS;else process.env.GITHUB_ACTIONS=previous;}
});
