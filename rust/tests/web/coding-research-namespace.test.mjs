import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {relocateGeneratedRustNamespace,pruneUnreachablePreludeFunctions} from '../../../scripts/generate-coding-research-program.mjs';
import {lex} from '../../../scripts/self-translation/lexer.mjs';
const namespace='isolated_program';
test('arbitrary emitted symbols and runtime calls relocate through the inferred symbol table',()=>{
  const definitions=['pub mod runtime { pub fn next(x: f64) -> f64 { x } }',
    'pub fn held_out(x: f64) -> f64 { crate::runtime::next(x) }',
    'pub fn other(x: f64) -> f64 { crate::held_out(x) }'];
  const result=relocateGeneratedRustNamespace(definitions,namespace);
  assert.deepEqual(result.symbols,['held_out','other','runtime']);
  assert.equal(result.references,2);
  assert(result.definitions[1].includes('crate::isolated_program::runtime::next(x)'));
  assert(result.definitions[2].includes('crate::isolated_program::held_out(x)'));
});
test('literal and comment crate paths are preserved byte for byte',()=>{
  const source='// crate::unowned is data\npub fn literal() -> String { String::from("crate::unowned") }';
  const result=relocateGeneratedRustNamespace([source],namespace);
  assert.deepEqual(result.definitions,[source]);
  assert.equal(result.references,0);
});
test('unknown external roots and duplicate declarations refuse instead of creating aliases',()=>{
  assert.throws(()=>relocateGeneratedRustNamespace(['pub fn value() { crate::unowned(); }'],namespace),
    /UnknownGeneratedNamespaceReference/u);
  assert.throws(()=>relocateGeneratedRustNamespace(['pub fn value() {}','pub fn value() {}'],namespace),
    /DuplicateGeneratedRootDeclaration/u);
  assert.throws(()=>relocateGeneratedRustNamespace(['pub fn value() {}'],'invalid::namespace'),
    /GeneratedNamespaceIdentifierRequired/u);
});
test('opaque Rust Unicode and hexadecimal character spans remain exact while lifetimes stay distinct',()=>{
  const source="pub fn opaque() { let a = '\\u{85}'; let b = '\\u{feff}'; let c = '\\x41'; }";
  const characters=lex(source,'Rust').filter(token=>token.type==='char').map(token=>token.text);
  assert.deepEqual(characters,["'\\u{85}'","'\\u{feff}'","'\\x41'"]);
  assert.deepEqual(relocateGeneratedRustNamespace([source],namespace).definitions,[source]);
  assert(lex("fn borrowed<'a>(value: &'a str) {}",'Rust').some(token=>token.type==='char'&&token.text==="'a"));
  assert.throws(()=>lex("pub fn broken() { let a = '\\u{zz}'; }",'Rust'));
});
test('closed pure prelude reachability retains called dependencies and removes unreachable helpers',()=>{
  const prelude='pub mod runtime { pub fn reachable() { dependency(); } pub fn dependency() {} pub fn unused() {} }';
  const result=pruneUnreachablePreludeFunctions([prelude],['pub fn entry() { crate::runtime::reachable(); }']);
  assert.deepEqual(result.removed,['runtime::unused']);
  assert(result.preludes[0].includes('pub fn reachable'));
  assert(result.preludes[0].includes('pub fn dependency'));
  assert(!result.preludes[0].includes('pub fn unused'));
  const cold=pruneUnreachablePreludeFunctions([prelude],['pub fn entry() {}']);
  assert.deepEqual(cold.removed,['runtime::dependency','runtime::reachable','runtime::unused']);
});
test('unreachable cycles and string-only mentions do not become executable roots',()=>{
  const prelude='pub mod runtime { pub fn first() { second(); } pub fn second() { first(); } }';
  assert.equal(pruneUnreachablePreludeFunctions([prelude],['pub fn entry() { "first"; }']).removed.length,2);
  assert.deepEqual(pruneUnreachablePreludeFunctions([prelude],['pub fn entry() { crate::runtime::second(); }']).removed,[]);
});
test('opaque prelude modules are never partially pruned',()=>{
  for(const source of ['pub mod runtime { pub struct State; pub fn unused() {} }',
    'pub mod runtime { pub fn opaque() { unknown!(); } }']) {
    assert.deepEqual(pruneUnreachablePreludeFunctions([source],[]),{preludes:[source],removed:[]});
  }
});
test('the production library exposes the generated module without a root wildcard namespace',()=>{
  const lib=fs.readFileSync(new URL('../../src/lib.rs',import.meta.url),'utf8');
  assert(lib.includes('pub mod coding_research_generated;'));
  assert(!lib.includes('use coding_research_generated::*'));
});
