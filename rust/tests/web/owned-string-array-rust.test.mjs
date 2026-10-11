import assert from 'node:assert/strict';
import test from 'node:test';
import { emitOwnedStringArrayRust } from '../../../scripts/self-translation/owned-string-array-rust.mjs';

test('owned string bindings clone on ordered push and join borrows explicit separator',
 () => {
  const source = "function buildText() { const value = 'x'; const items = ['a']; items.push(value, value); return items.join('-'); }";
  const result = emitOwnedStringArrayRust(source);
  assert.equal(result.state,
 'SourceEmitted');
  assert.equal(result.source,
 source);
  assert.equal(result.code,
 "pub fn build_text() -> String {\n    let value: String = \"x\".to_owned();\n    let mut items: Vec<String> = vec![\"a\".to_owned()];\n    let _ = {\n        items.push(value.clone());\n        items.push(value.clone());\n        items.len() as u32\n    };\n    items.join(\"-\")\n}\n");
  assert.equal(result.operationMap.length,
 result.ir.operations.length);
  assert.deepEqual(result.operationMap.map(entry => entry.sourceSpan),
 result.ir.operations.map(entry => entry.span));
  assert.equal(result.native,
 'Pending');
  assert.equal(result.admitted,
 false);
  assert.ok(Object.isFrozen(result.ir));
});

test('push return count and zero argument push never lower to unit',
 () => {
  for (const argumentsText of ["'x', 'y'",
 '']) {
    const result = emitOwnedStringArrayRust('function countItems() { const items = []; return items.push(' + argumentsText + '); }');
    assert.equal(result.state,
 'SourceEmitted');
    assert.match(result.code,
 /-> u32/);
    assert.match(result.code,
 /items\.len\(\) as u32/);
    assert.deepEqual(result.naturalBound,
 [0,
 1024]);
  }
});

test('element length is independent of Unicode string byte or code unit lengths',
 () => {
  const result = emitOwnedStringArrayRust("function countItems() { const items = ['😀', 'é']; return items.length; }");
  assert.equal(result.state,
 'SourceEmitted');
  assert.match(result.code,
 /vec!\["😀"\.to_owned\(\), "é"\.to_owned\(\)\]/);
  assert.match(result.code,
 /items\.len\(\) as u32/);
  assert.doesNotMatch(result.code,
 /mut items/);
});

test('literal spelling conserves controls quotes slashes and scalar Unicode',
 () => {
  const result = emitOwnedStringArrayRust("function textValue() { return 'a\\0\\n\\r\\t\\\\\\\"😀'; }");
  assert.equal(result.state,
 'SourceEmitted');
  assert.match(result.code,
 /\\0\\n\\r\\t/);
  assert.ok(result.code.includes('😀'));
  assert.equal(Buffer.from(result.code).toString('utf8'),
 result.code);
});

test('original source only refuses caller IR callbacks and malformed Unicode',
 () => {
  for (const input of [{state: 'Evaluated',
 ir: {}},
 null,
 () => 'source',
 ['source'],
    "function textValue() { return '\ud800'; }",
 "function textValue() { return '\udc00'; }"]) {
    const result = emitOwnedStringArrayRust(input);
    assert.equal(result.state,
 'Unsupported');
    assert.equal(result.code,
 null);
  }
});

test('shared aliases external effects callbacks holes and array escapes remain refused',
 () => {
  for (const source of [
    "function textValue() { const a = []; const b = a; return b.length; }",
    "function textValue() { const a = []; return a; }",
    "function textValue() { return outside.push('x'); }",
    "function textValue() { const a = ['x']; return a.map(x => x); }",
    "function textValue() { const a = [,]; return a.length; }",
    "function textValue() { const a = []; a[0] = 'x'; return a.length; }",
    "function textValue() { const a = []; return a.join(); }",
  ]) assert.equal(emitOwnedStringArrayRust(source).state,
 'Unsupported');
});

test('fresh source lowering is deterministic and reports all operation boundaries',
 () => {
  const source = "function textValue() { const a = []; a.push('x'); return a.join(''); }";
  const first = emitOwnedStringArrayRust(source);
  const second = emitOwnedStringArrayRust(source);
  assert.deepEqual(first,
 second);
  assert.notEqual(first.ir,
 second.ir);
  for (const entry of first.operationMap) {
    assert.equal(first.code.split('\n').slice(entry.startLine - 1, entry.endLine).join('\n'),
 '    ' + entry.text);
  }
});

test('generated Rust literal spelling decodes to complete checked string values', async () => {
  const {lex} = await import('../../../scripts/self-translation/lexer.mjs');
  function decoded(text) {
    let result = '';
    for (let index = 1; index < text.length - 1; index++) {
      const character = text[index];
      if (character !== '\\') {
        result += character;
        continue;
      }
      const escape = text[++index];
      const table = {'0': '\0', n: '\n', r: '\r', t: '\t', '"': '"', '\\': '\\'};
      if (Object.hasOwn(table, escape)) {
        result += table[escape];
        continue;
      }
      assert.equal(escape, 'u');
      assert.equal(text[++index], '{');
      let hex = '';
      while (text[++index] !== '}') hex += text[index];
      assert.match(hex, /^[0-9a-f]+$/);
      result += String.fromCodePoint(parseInt(hex, 16));
    }
    return result;
  }
  for (const source of [
    "function answer() { const a = ['😀', 'é']; return a.join('::'); }",
    "function answer() { return 'a\\0\\n\\r\\t\\\\\\\"😀'; }",
    "function answer() { return '\u0001\u007f'; }",
    "function answer() { const a = []; return a.push('x', 'y'); }",
  ]) {
    const result = emitOwnedStringArrayRust(source);
    assert.equal(result.state, 'SourceEmitted');
    const expected = [];
    const collect = expression => {
      if (expression.kind === 'string') expected.push(expression.value);
      for (const argument of expression.args ?? []) collect(argument);
    };
    for (const operation of result.ir.operations) {
      for (const value of operation.values ?? []) collect(value);
      if (operation.value) collect(operation.value);
    }
    const actual = lex(result.code, 'Rust').filter(token => token.type === 'string')
      .map(token => decoded(token.text));
    assert.deepEqual(actual, expected);
  }
});

test('multiple arrays lower to distinct fresh allocations with immutable string clone arguments', () => {
  const source = "function answer() { const s = 'x'; const first = ['a']; const second = ['b']; first.push(s, s); return second.join(''); }";
  const result = emitOwnedStringArrayRust(source);
  assert.equal(result.state, 'SourceEmitted');
  assert.equal(result.ir.bindings.filter(binding => binding.type === 'OwnedDenseStringArray').length, 2);
  assert.match(result.code, /let mut first: Vec<String>/);
  assert.match(result.code, /let second: Vec<String>/);
  assert.match(result.code, /first\.push\(s\.clone\(\)\);\s+first\.push\(s\.clone\(\)\)/);
  assert.match(result.code, /second\.join\(""\)/);
});

test('binding collisions canonical names and full output bounds refuse before source emission', () => {
  const sources = [
    "function answer() { const answer = []; return answer.length; }",
    "function answer() { const a = []; const a = []; return a.length; }",
    "function answer() { const type = []; return type.length; }",
    "function answer() { const String = []; return String.length; }",
    'function answer() { const a = [' + Array(1025).fill("'x'").join(',') + ']; return a.length; }',
    "function answer() { const a = ['" + 'x'.repeat(16384) + "']; return a.push('y'); }",
    "function answer() { const a = ['" + 'x'.repeat(8192) + "','" + 'y'.repeat(8192) + "']; return a.join('-'); }",
  ];
  for (const source of sources) {
    const result = emitOwnedStringArrayRust(source);
    assert.equal(result.state, 'Unsupported');
    assert.equal(result.code, null);
  }
});
