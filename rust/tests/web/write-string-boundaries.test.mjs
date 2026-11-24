// Preserve Rust character matching while inspecting only the requested boundary.
import test from 'node:test';
import assert from 'node:assert/strict';
import {charIn, isWhitespace, trim, trimEndMatches, trimStartMatches} from '../../../js/agentic/write_str.mjs';

test('boundary trimming preserves Unicode characters, predicate order and unmatched content', () => {
  const seen = [];
  assert.equal(trimStartMatches('😀🐍body🐍😀', character => {
    seen.push(character); return character === '😀' || character === '🐍';
  }), 'body🐍😀');
  assert.deepEqual(seen, ['😀', '🐍', 'b']);
  seen.length = 0;
  assert.equal(trimEndMatches('😀🐍body🐍😀', character => {
    seen.push(character); return character === '😀' || character === '🐍';
  }), '😀🐍body');
  assert.deepEqual(seen, ['😀', '🐍', 'y']);
  assert.equal(trim('\u2003\t😀body😀\r\n'), '😀body😀');
  assert.equal(trim('\ufeffbody\ufeff'), '\ufeffbody\ufeff');
  for (const text of ['', 'body', ' \n\t', '😀', '\ud800tail', 'head\udc00', '😀\udc00', '\ud800😀']) {
    for (const predicate of [isWhitespace, charIn('😀'), () => true, () => false]) {
      const characters = Array.from(text);
      let start = 0, end = characters.length;
      while (start < characters.length && predicate(characters[start])) start++;
      while (end > 0 && predicate(characters[end - 1])) end--;
      assert.equal(trimStartMatches(text, predicate), characters.slice(start).join(''));
      assert.equal(trimEndMatches(text, predicate), characters.slice(0, end).join(''));
    }
  }
});

test('trimming a short boundary does not enumerate the unchanged source body', () => {
  const text = 'body😀'.repeat(512);
  const original = Object.getOwnPropertyDescriptor(String.prototype, Symbol.iterator);
  let inspected = 0;
  try {
    Object.defineProperty(String.prototype, Symbol.iterator, {...original, value: function* () {
      for (const character of Reflect.apply(original.value, this, [])) {
        inspected++; yield character;
      }
    }});
    assert.equal(trimStartMatches('\t' + text, isWhitespace), text);
    assert.equal(trimEndMatches(text + '\n', isWhitespace), text);
    assert.equal(trim(text), text);
  } finally {
    Object.defineProperty(String.prototype, Symbol.iterator, original);
  }
  assert(inspected <= 8, 'short boundary trimming enumerated ' + inspected + ' characters from the unchanged body');
});
