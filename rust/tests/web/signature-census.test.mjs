import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {signatureCensus, scanSignatureSymbols, stableSourceIdentifier}
  from '../../../scripts/lib/signature-census.mjs';

test('native baseline fixture is reproduced exactly for general signature modules', () => {
  const fixture = JSON.parse(fs.readFileSync(
    new URL('../fixtures/signature-census-native-baseline.json', import.meta.url), 'utf8'));
  assert.equal(fixture.identity.artifactId, 11988455797);
  for (const sample of fixture.samples) {
    assert.equal(signatureCensus(sample.modulePath, sample.source), sample.document);
  }
});

test('visibility, modifiers, macros, constants and inclusive spans follow the Rust scanner', () => {
  const source = 'pub(in crate::scope) async unsafe extern "C" fn launch() {\n'
    + ' let text = "}"; // not the end\n}\n'
    + 'pub const fn value() {}\nstatic mut COUNTER: u8 = 0;\n'
    + 'macro_rules! first { () => {} }\nmacro_rules!second { () => {} }\n'
    + 'type Alias = u8;\nmod child;\n';
  assert.deepEqual(scanSignatureSymbols(source), [
    {kind: 'function', name: 'launch', startLine: 1, endLine: 3},
    {kind: 'function', name: 'value', startLine: 4, endLine: 4},
    {kind: 'static', name: 'COUNTER', startLine: 5, endLine: 5},
    {kind: 'macro', name: 'first', startLine: 6, endLine: 6},
    {kind: 'macro', name: 'second', startLine: 7, endLine: 7},
    {kind: 'type', name: 'Alias', startLine: 8, endLine: 8},
    {kind: 'module', name: 'child', startLine: 9, endLine: 9},
  ]);
});

test('comments, escaped strings, character braces and lifetimes retain lexical behavior', () => {
  const source = '/* fn phantom() {\n } */\nfn visible<\'a>() {\n'
    + ' let quote = "\\\"}";\n let brace = \'}\';\n}\n'
    + 'fn unfinished() {\n';
  assert.deepEqual(scanSignatureSymbols(source), [
    {kind: 'function', name: 'visible', startLine: 3, endLine: 6},
    {kind: 'function', name: 'unfinished', startLine: 7, endLine: 7},
  ]);
});

test('UTF8 identity and Rust lines semantics preserve exact bytes', () => {
  assert.equal(stableSourceIdentifier(''), 'source_module_cbf29ce484222325');
  assert.notEqual(stableSourceIdentifier('é'), stableSourceIdentifier('e\u0301'));
  assert.match(signatureCensus('src/empty.rs', ''), /  line_count 0\n/);
  assert.match(signatureCensus('src/lines.rs', 'mod item;\r\n'), /  line_count 1\n/);
  assert.match(signatureCensus('src/lines.rs', 'mod item;\n\n'), /  line_count 2\n/);
  assert.deepEqual(scanSignatureSymbols('fn café() {}\n'), [
    {kind: 'function', name: 'café', startLine: 1, endLine: 1},
  ]);
});

test('signature helper refuses full AST and invalid inputs without pretending to parse', () => {
  assert.throws(() => signatureCensus('src/agentic_coding/any.rs', ''), /full_ast/);
  assert.throws(() => signatureCensus('src/any.rs', '', {fidelity: 'full_ast'}), /full_ast/);
  assert.throws(() => signatureCensus('src/any.rs', '\ud800'), /well_formed/);
  assert.throws(() => signatureCensus('src/any.rs', null), /well_formed/);
  assert.throws(() => signatureCensus('src/any.rs\nforeign', ''), /module_path/);
});
