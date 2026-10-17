import assert from 'node:assert/strict';
import test from 'node:test';
import {createHash} from 'node:crypto';
import {presentRustCode, encodeRustPresentation, decodeRustPresentation, validatePresentationScope}
  from '../../../scripts/lib/rust-projection-presentation.mjs';
import {upstreamBlocks, renderProjection, readProjection} from '../../../scripts/translate-js-rust.mjs';
import {lex} from '../../../scripts/self-translation/lexer.mjs';
const hash = source => createHash('sha256').update(source).digest('hex');
const raw = 'pub fn values() -> Vec<i32> { vec![' + Array.from({length: 160}, (_, index) => index).join(', ') + '] }';
const prefix = '// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=abc bytes=10';
const marker = source => '// meta-language:translated JavaScript export_statement items=1 sha256=' + hash(source);
const upstream = source => [prefix, marker(source), '// | export function values() {}', source].join('\n');

test('long source is reversible with exact tokens, adjacency and independent upstream digest', () => {
  const result = presentRustCode(raw);
  assert(result.edits.length > 0);
  assert.notEqual(result.rendered, raw);
  assert.deepEqual(lex(result.rendered, 'Rust').map(row => [row.type, row.text]),
    lex(raw, 'Rust').map(row => [row.type, row.text]));
  const encoded = encodeRustPresentation(raw);
  const decoded = decodeRustPresentation(encoded.split('\n'), 0, hash(raw));
  assert.equal(decoded.raw, raw);
  assert.equal(decoded.rendered, result.rendered);
  assert.equal(decoded.rawSHA256, hash(raw));
});

test('short original block stays byte-identical and the original public counts remain equal', () => {
  const short = 'pub const VALUE: f64 = 7.0;';
  assert.equal(encodeRustPresentation(short), short);
  const parsed = upstreamBlocks(upstream(short));
  const rendered = renderProjection(parsed);
  assert(rendered.includes(parsed.blocks[0].text));
  assert.equal(upstreamBlocks(rendered).blocks[0].text, parsed.blocks[0].text);
  assert.equal(rendered.includes('presentation:v1'), false);
  const longParsed = upstreamBlocks(upstream(raw)), longRendered = renderProjection(longParsed);
  assert.equal(upstreamBlocks(longRendered).blocks[0].text, longParsed.blocks[0].text);
  assert.deepEqual(readProjection(longRendered), readProjection(upstream(raw)));
  assert.equal(renderProjection(upstreamBlocks(longRendered)), longRendered);
});

test('every derivative part and canonical encoding is checked, not trusted receipt metadata', () => {
  const encoded = encodeRustPresentation(raw);
  const changed = [encoded.replace('rawBytes=', 'unknownBytes='),
    encoded.replace('pub fn values()', 'pub fn changed()'),
    encoded.replace(/presentation-raw ([A-Za-z0-9])/u, 'presentation-raw !'),
    encoded.replace('presentation-raw end', 'presentation-raw missing'),
    encoded.replace('renderedSHA256=', 'extra=1 renderedSHA256=')];
  for (const source of changed) assert.throws(() => decodeRustPresentation(source.split('\n'), 0, hash(raw)));
  assert.throws(() => decodeRustPresentation(encoded.split('\n'), 0, '0'.repeat(64)));
});

test('opaque literals, comments, macros and source-position contracts refuse rather than guessed formatting', () => {
  assert.throws(() => presentRustCode('r#"opaque"#'), /OpaqueRustPrefixedLiteral/);
  assert.throws(() => presentRustCode('/* nested /* opaque */ */'), /OpaqueNestedRustComment/);
  assert.throws(() => presentRustCode(raw.replace('vec!', 'unknown!')), /OpaqueEditedMacro/);
  for (const source of ['macro_rules! vec { () => {} }', 'line!()', 'use foreign::vec;', 'use foreign::*;']) {
    assert.throws(() => validatePresentationScope(source));
  }
  assert.throws(() => presentRustCode('pub const VALUE: i32 = ' + 'long'.repeat(100) + ';'), /NoSafeReadablePresentation/);
});

test('fresh upstream projection keeps already-readable opaque Unicode char bytes unchanged', () => {
  const short = "pub const VALUE: char = '\\u{85}';";
  const parsed = upstreamBlocks(upstream(short));
  const rendered = renderProjection(parsed);
  assert.equal(upstreamBlocks(rendered).blocks[0].text, parsed.blocks[0].text);
  assert.equal(rendered.includes('presentation:v1'), false);
});

test('procedural and foreign attributes never inherit token-only presentation approval', () => {
  for (const prefix of ['#[foreign]\n', '#![foreign]\n', '#![allow(custom::path)]\n']) {
    assert.throws(() => presentRustCode(prefix + raw), /OpaqueRustAttribute/);
  }
  const original = '#![allow(unused, non_snake_case)]\n' + raw;
  const projected = presentRustCode(original);
  assert(projected.edits.length > 0);
  assert(projected.rendered.startsWith('#![allow(unused, non_snake_case)]\n'));
});


test('builtin derives on generated data preserve exact presentation tokens', () => {
  for (const declaration of ['pub enum Choice { Yes, No }', 'pub struct Point { x: i32 }']) {
    const source = '#[derive(Clone, Debug, PartialEq, Eq)]\n' + declaration + '\n' + raw;
    assert.doesNotThrow(() => validatePresentationScope(source));
    const rendered = presentRustCode(source).rendered;
    assert.deepEqual(lex(rendered, 'Rust').map(row => [row.type, row.text]),
      lex(source, 'Rust').map(row => [row.type, row.text]));
  }
});

test('derive support refuses custom attributes, malformed targets and shadowed builtin identities', () => {
  for (const attribute of ['#[derive()]', '#[derive(Foreign)]', '#[derive(custom::Clone)]',
    '#[derive(Clone, Clone)]', '#[derive(Clone,)]', '#[cfg(any())]', '#[derive(Clone) extra]',
    '#![derive(Clone)]', '#[derive(Clone)] #[cfg(any())]']) {
    assert.throws(() => validatePresentationScope(attribute + '\npub struct Value;'));
  }
  for (const declaration of ['fn target() {}', 'type Value = i32;', 'enum Value;', 'static Value: i32 = 0;']) {
    assert.throws(() => validatePresentationScope('#[derive(Clone)]\n' + declaration));
  }
  for (const shadow of ['use custom::Clone;', 'use custom::Thing as Clone;',
    'use custom::{Clone};', 'trait Clone {}', 'struct Clone;', 'type Clone = i32;',
    'mod Clone {}', 'fn shadow<Clone>() {}', 'fn shadow<T, Clone>() {}']) {
    assert.throws(() => validatePresentationScope(shadow + '\n#[derive(Clone)] struct Value;'), /ShadowedBuiltinDerive/);
  }
});
