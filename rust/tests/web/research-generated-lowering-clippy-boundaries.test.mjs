import test from 'node:test';
import assert from 'node:assert/strict';
import {lowerGeneratedRust} from '../../../scripts/lib/research-rust-lowering.mjs';
import {sourceMaintenanceOwnershipKernel} from '../../../js/agentic/portable/source_maintenance_state.mjs';
const options = {ownedValueModel: 'pinned-meta-language-owned-values/v1', pure: true};
test('UTF8 suffix lowering preserves exact case and Unicode bytes', () => {
  for (const suffix of ['.mts', '.cts', '.ts', '.mjs', '.cjs', '.js', '🌊.JS']) {
    const source = 'pub fn suffix(value: String) -> bool { value.ends_with(' + JSON.stringify(suffix) + ') }';
    const result = lowerGeneratedRust(source, options);
    const expectedPattern = suffix === '🌊.JS' ? '&[240, 159, 140, 138, 46, 74, 83]' : 'b' + JSON.stringify(suffix);
    assert(result.source.includes('.as_bytes().ends_with(' + expectedPattern + ')'));
    assert(result.operations.includes('utf8-string-suffix'));
    for (const prefix of ['', 'file', 'π/', '🌊']) {
      for (const ending of [suffix, suffix.toUpperCase(), suffix + '/', '']) {
        const value = prefix + ending;
        const bytes = new TextEncoder().encode(value), tail = new TextEncoder().encode(suffix);
        const observed = bytes.length >= tail.length && tail.every((byte, index) => byte === bytes[bytes.length - tail.length + index]);
        assert.equal(observed, value.endsWith(suffix));
      }
    }
  }
  const dynamic = 'pub fn suffix(value: String, suffix: String) -> bool { value.ends_with(suffix.as_str()) }';
  assert(!lowerGeneratedRust(dynamic, options).operations.includes('utf8-string-suffix'));
});
test('terminality sees final standalone blocks after semicolon-free conditional statements', () => {
  for (const tail of ['{ continue; }', '{ let next = 1f64; { crate::observe(next); continue; } }']) {
    const source = 'pub fn step(flag: bool, other: bool) { loop { if flag { if other { continue; } ' + tail + ' } else { crate::observe(); continue; } } }';
    const result = lowerGeneratedRust(source, options);
    assert(result.operations.includes('terminal-branch'));
    assert(!result.source.includes('else'), result.source);
    assert(result.source.includes('crate::observe()'));
  }
  const fallthrough = 'pub fn step(flag: bool, other: bool) { loop { if flag { if other { continue; } { crate::observe(); } } else { crate::other(); continue; } } }';
  assert(lowerGeneratedRust(fallthrough, options).source.includes('else'));
});
test('final unary constructors move unboxed owned payloads but arbitrary calls and reuse refuse', () => {
  const prefix = 'pub fn unpack(value: Box<ResearchValue>) -> ResearchValue { let value = *value; ';
  const result = lowerGeneratedRust(prefix + 'ResearchValue::Wrapped(Box::new(value.clone())) }', options);
  assert(!result.source.includes('.clone()'));
  assert(result.source.includes('ResearchValue::Wrapped(Box::new(value))'));
  assert(result.operations.includes('terminal-owned-rebinding'));
  for (const tail of ['crate::observe(value.clone())', 'ResearchValue::Wrapped(value.clone(), value.clone())', 'crate::observe(value.clone()); value.clone()']) {
    assert(lowerGeneratedRust(prefix + tail + ' }', options).source.includes('.clone()'));
  }
});

test('existing Rust literal decoder binds escaped byte patterns and unsupported escapes remain unlowered', () => {
  for (const [literal, pattern] of [['"\\t"', '&[9]'], ['"\\u{3c0}.JS"', '&[207, 128, 46, 74, 83]']]) {
    const result = lowerGeneratedRust('pub fn suffix(value: String) -> bool { value.ends_with(' + literal + ') }', options);
    assert(result.source.includes('.as_bytes().ends_with(' + pattern + ')'));
  }
  for (const literal of ['"\\q"', '"\\u{d800}"', 'r"raw"']) {
    const result = lowerGeneratedRust('pub fn suffix(value: String) -> bool { value.ends_with(' + literal + ') }', options);
    assert(!result.operations.includes('utf8-string-suffix'));
    assert(result.source.includes('.ends_with(' + literal + ')'), 'unsupported literal is never relabelled as compiled proof');
  }
});

test('canonical ownership construction retains exact declarative spacing order and empty-loop refusals', () => {
  const rows = [
    ['read source from', ['read'], ['source'], ['from'], true],
    ['revise layout in', ['revise'], ['layout'], ['in'], true],
    ['整える 配置 内', ['整える'], ['配置'], ['内'], true],
    ['read  from', ['read'], [''], ['from'], true],
    ['read source FROM', ['read'], ['source'], ['from'], false],
    ['read  source from', ['read'], ['source'], ['from'], false],
    ['read source from', [], ['source'], ['from'], false],
    ['read source from', ['read'], [], ['from'], false],
    ['read source from', ['read'], ['source'], [], false]
  ];
  for (const [prefix, actions, properties, prepositions, expected] of rows) {
    assert.equal(sourceMaintenanceOwnershipKernel(true, 0, 'typescript', prefix, '', true,
      actions, properties, prepositions, [], [true]), expected);
  }
});
