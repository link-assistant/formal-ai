import assert from 'node:assert/strict';
import {test} from 'node:test';
import {sourceQualifiedAliasReferences} from '../../../experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs';
const move={from:'pkg/src/domain/catalog/brief.rs',to:'pkg/src/domain/catalog/description.rs'};
const root='pub(crate) mod domain;\npub use domain::catalog as external_catalog;\n';
const consumer='// 😀 astral comment\nuse sample_library::external_catalog::brief::entry;\n';
function sources(source=root, module='pub mod catalog;') {
 return new Map([['pkg/Cargo.toml','[lib]\nname = "sample_library"\n'],
 ['pkg/src/lib.rs',source],['pkg/src/domain/mod.rs',module]]);
}
function project(text=consumer, context=sources()) {
 return sourceQualifiedAliasReferences(text,'pkg/tests/check.rs',[move],context);
}
test('actual source path and Cargo namespace bind UTF16 token offsets',()=>{
 const refs=project();assert.equal(refs.length,1);
 assert.equal(consumer.slice(refs[0].start,refs[0].end),'brief');
 assert.equal(consumer.slice(0,refs[0].start)+refs[0].to+consumer.slice(refs[0].end),consumer.replace('::brief::','::description::'));
});
test('alias names and modules derive entirely from selected declarations',()=>{
 const refs=project(consumer.replaceAll('external_catalog','other_export'),sources(root.replace('external_catalog','other_export')));
 assert.equal(refs.length,1);
});
test('comments containing synthetic declarations do not mint aliases',()=>{
 assert.equal(project(consumer,sources('pub(crate) mod domain;\n// pub use domain::catalog as external_catalog;')).length,0);
});
test('quoted alias-like source remains data',()=>{
 assert.equal(project('const LABEL: &str = "sample_library::external_catalog::brief::entry";').length,0);
});
for (const [name,context] of [
 ['attribute-conditioned alias',sources(root.replace('pub use','#[cfg(feature="optional")] pub use'))],
 ['attribute-conditioned module',sources(root,'#[cfg(feature="optional")] pub mod catalog;')],
 ['duplicate alias',sources(root+'pub use domain::catalog as external_catalog;')],
 ['private alias',sources(root.replace('pub use','use'))],
 ['macro namespace',sources('create_modules!();\n'+root)],
 ['unknown module',sources(root,'pub mod another_catalog;')],
 ['missing physical module',new Map([['pkg/Cargo.toml','[lib]\nname="sample_library"\n'],['pkg/src/lib.rs',root]])],
 ['missing Cargo namespace',new Map([['pkg/src/lib.rs',root],['pkg/src/domain/mod.rs','pub mod catalog;']])],
]) test(name+' refuses',()=>assert.throws(()=>project(consumer,context)));
for (const [name,text] of [
 ['nested use','fn action(){ '+consumer+' }'],
 ['library binding shadow','use sample_library::external_catalog::brief::entry;\nfn action(sample_library: usize) {}'],
 ['grouped imports','use sample_library::external_catalog::{brief};'],
 ['import alias shadow','use other_library as sample_library;\n'+consumer],
 ['consumer macro','create_imports!();\n'+consumer],
 ['conditional consumer','#[cfg(feature="optional")] '+consumer],
]) test(name+' refuses',()=>assert.throws(()=>project(text)));
test('foreign namespace does not gain declaration ownership',()=>{
 assert.equal(project(consumer.replace('sample_library','foreign_library')).length,0);
});


test('private exported target refuses',()=>assert.throws(()=>project(consumer,sources(root,'mod catalog;'))));
 test('duplicate library tables refuse',()=>{
  const context=sources();context.set('pkg/Cargo.toml','[lib]\nname="sample_library"\n[lib]\nname="sample_library"\n');
  assert.throws(()=>project(consumer,context));
 });

test('exported alias colliding with physical root module refuses',()=>{
  assert.throws(()=>project(consumer,sources(root+'pub mod external_catalog;')));
 });
 test('exported alias colliding with root type binding refuses',()=>{
  assert.throws(()=>project(consumer,sources(root+'pub struct external_catalog {}')));
 });

test('Cargo library path must bind the selected physical root',()=>{
  const context=sources();context.set('pkg/Cargo.toml','[lib]\nname="sample_library"\npath="src/foreign.rs"\n');
  assert.throws(()=>project(consumer,context));
 });

test('explicit library source preserves CRLF and horizontal field spacing',()=>{
  const context=sources();context.set('pkg/Cargo.toml','[lib]\r\nname="sample_library"\r\n\tpath = "src/lib.rs"  # actual library\r\n\r\n[[bin]]\r\nname="different"\r\n');
  assert.equal(project(consumer,context).length,1);
 });

for (const suffix of [' unexpected;', '::;', '::entry extra;', ' as renamed;', ', other;']) {
 test('complete alias import refuses unconsumed or ambiguous suffix '+suffix,()=>{
  assert.throws(()=>project('use sample_library::external_catalog::brief'+suffix));
 });
}
test('complete alias import permits terminal module and longer item paths',()=>{
 for(const suffix of [';', '::entry;', '::nested::entry;']){
  const source='use sample_library::external_catalog::brief'+suffix;
  const references=project(source);assert.equal(references.length,1);
  assert.equal(source.slice(references[0].start,references[0].end),'brief');
 }
});
