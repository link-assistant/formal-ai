import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,symlinkSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {readNativeTestModuleGraph,bindOriginalSourceCase} from '../../../scripts/lib/native-test-module-graph.mjs';
function fixture(t,files){
 const root=mkdtempSync(join(tmpdir(),'native-registration-'));
 t.after(()=>rmSync(root,{recursive:true,force:true}));
 const all={'rust/Cargo.toml':'[[test]]\nname = "unit"\npath = "tests/unit/mod.rs"\n',...files};
 for(const [path,body] of Object.entries(all)){const target=join(root,path);mkdirSync(dirname(target),{recursive:true});writeFileSync(target,body);}
 return root;
}
const body='#[test]\nfn actual_case() { assert_eq!(1,1); }\n';
test('initial executable Rust shebang is lexical trivia while crate attributes remain attributes',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'#!/usr/bin/env rust-script\r\n#![allow(dead_code)]\n'+body});
 assert.equal(bindOriginalSourceCase(readNativeTestModuleGraph(root),'rust/tests/unit/mod.rs::actual_case').caller,'actual_case');
});
test('literal path registration binds original source under declared alias',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'#[path="actual.rs"] mod named_alias;','rust/tests/unit/actual.rs':body});
 const row=bindOriginalSourceCase(readNativeTestModuleGraph(root),'rust/tests/unit/actual.rs::actual_case');
 assert.equal(row.caller,'named_alias::actual_case');assert.equal(row.target,'unit');assert.equal(row.trace[1].kind,'path');assert.equal(row.sourceSHA.length,64);assert.equal(row.suppliedRuntimeListingMatches,false);
});
test('conventional nested module and inline module names preserve full caller',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'mod family;','rust/tests/unit/family/mod.rs':'mod leaf;','rust/tests/unit/family/leaf.rs':'mod inner { '+body+' }'});
 assert.equal(bindOriginalSourceCase(readNativeTestModuleGraph(root),'rust/tests/unit/family/leaf.rs::actual_case').caller,'family::leaf::inner::actual_case');
});
test('non-mod.rs crate root resolves default children beside crate root',t=>{
 const root=fixture(t,{'rust/Cargo.toml':'[[test]]\nname="integration"\npath="tests/harness.rs"\n','rust/tests/harness.rs':'mod leaf;','rust/tests/leaf.rs':body});
 assert.equal(bindOriginalSourceCase(readNativeTestModuleGraph(root),'rust/tests/leaf.rs::actual_case').caller,'leaf::actual_case');
});
test('comments, raw string decoys, and function-local declarations do not register',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'// mod ghost;\nconst SAMPLE: &str = r###"#[test] fn fake() {} mod ghost;"###;\nfn helper(){ mod local { #[test] fn fake() {} } }\n'+body});
 const graph=readNativeTestModuleGraph(root);assert.deepEqual(graph.functions.map(row=>row.function),['actual_case']);assert.equal(graph.unsupported.length,0);
});
test('ignored and unregistered source tests cannot bind',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'#[test] #[ignore] fn skipped() {}','rust/tests/unit/orphan.rs':body});
 const graph=readNativeTestModuleGraph(root);assert.throws(()=>bindOriginalSourceCase(graph,'rust/tests/unit/mod.rs::skipped'),/no unique registration/);assert.throws(()=>bindOriginalSourceCase(graph,'rust/tests/unit/orphan.rs::actual_case'),/no unique registration/);
});
test('both conventional module filenames refuse ambiguity',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'mod leaf;','rust/tests/unit/leaf.rs':body,'rust/tests/unit/leaf/mod.rs':body});
 const graph=readNativeTestModuleGraph(root);assert.equal(graph.functions.length,0);assert.equal(graph.unsupported[0].reason,'ambiguous module filenames');
});
test('two aliases for same original test refuse source binding',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'#[path="leaf.rs"] mod left; #[path="leaf.rs"] mod right;','rust/tests/unit/leaf.rs':body});
 assert.throws(()=>bindOriginalSourceCase(readNativeTestModuleGraph(root),'rust/tests/unit/leaf.rs::actual_case'),/no unique registration/);
});
test('same full caller from different registered sources refuses ambiguity',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'#[path="left.rs"] mod alias; #[path="right.rs"] mod alias;','rust/tests/unit/left.rs':body,'rust/tests/unit/right.rs':body});
 assert.throws(()=>bindOriginalSourceCase(readNativeTestModuleGraph(root),'rust/tests/unit/left.rs::actual_case'),/ambiguous full native caller/);
});
test('missing module and conditional path are explicit unsupported edges',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'mod missing; #[cfg_attr(feature="alternate",path="other.rs")] mod conditional;'});
 const graph=readNativeTestModuleGraph(root);assert.equal(graph.functions.length,0);assert.deepEqual(graph.unsupported.map(row=>row.reason),['unmapped module filenames','ambiguous or unsupported conditional/inline path attribute']);
});
test('opaque include macro prevents source-only certification of sibling and child tests',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'mod leaf; include!("unknown.rs"); '+body,'rust/tests/unit/leaf.rs':body});
 const graph=readNativeTestModuleGraph(root);assert.equal(graph.functions.length,2);for(const path of ['rust/tests/unit/mod.rs','rust/tests/unit/leaf.rs'])assert.throws(()=>bindOriginalSourceCase(graph,path+'::actual_case'),/opaque item macro/);
});
test('symlink module source is rejected rather than following a different identity',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':'mod leaf;','rust/tests/unit/actual.rs':body});symlinkSync('actual.rs',join(root,'rust/tests/unit/leaf.rs'));
 assert.throws(()=>readNativeTestModuleGraph(root),/symlink module source/);
});
test('listing input requires exact unique target and caller but is not a runtime attestation',t=>{
 const root=fixture(t,{'rust/tests/unit/mod.rs':body}),graph=readNativeTestModuleGraph(root),id='rust/tests/unit/mod.rs::actual_case',row=bindOriginalSourceCase(graph,id);
 for(const listing of [[],[row.runtimeKey,row.runtimeKey],['integration\\tactual_case']])assert.throws(()=>bindOriginalSourceCase(graph,id,listing),/does not uniquely confirm/);
 const confirmed=bindOriginalSourceCase(graph,id,[row.runtimeKey]);assert.equal(confirmed.suppliedRuntimeListingMatches,true);assert.equal(Object.hasOwn(confirmed,'currentNativeRuntimeListingVerified'),false);
});
