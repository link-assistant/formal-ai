import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {compactJsonLeaves} from '../../scripts/lib/compact-json-leaves.mjs';

// Independent character walk: whitespace inside strings remains authoritative.
function lexicalContent(source) {
  let quoted = false, escaped = false, result = '';
  for (const character of source) {
    if (quoted || !/\s/u.test(character)) result += character;
    if (quoted && escaped) escaped = false;
    else if (quoted && character === String.fromCharCode(92)) escaped = true;
    else if (character === '"') quoted = !quoted;
  }
  return result;
}

test('leaf formatting preserves exact numbers, duplicate keys, strings and order', {timeout: 10000}, () => {
  const source = '{\n  "row": {\n    "n": 9007199254740993,\n    "n": 1.00e+2,\n    "s": "a [} \\" \\u1234"\n  }\n}\n';
  const actual = compactJsonLeaves(source);
  assert(actual.split('\n').length < source.split('\n').length);
  assert.equal(lexicalContent(actual), lexicalContent(source));
  assert.deepEqual(JSON.parse(actual), JSON.parse(source));
  assert(actual.includes('9007199254740993'));
  assert(actual.includes('1.00e+2'));
});

test('a leaf that exceeds its opening-line width remains expanded', () => {
  const source = JSON.stringify({long_key: {alpha: 'long value', beta: 2}}, null, 2) + '\n';
  assert.equal(compactJsonLeaves(source, 20), source);
  const actual = compactJsonLeaves(source, 100);
  assert(actual.includes('"long_key": {"alpha": "long value", "beta": 2}'));
  assert(actual.split('\n').every(line => line.length <= 100));
});

test('nested parents stay expanded and repeated formatting is stable', () => {
  const source = JSON.stringify({rows: [{a: true, b: null}, ['é', false]], empty: {}}, null, 2) + '\n';
  const actual = compactJsonLeaves(source);
  assert(actual.includes('"rows": [\n'));
  assert.equal(compactJsonLeaves(actual), actual);
  assert.equal(lexicalContent(actual), lexicalContent(source));
  assert.deepEqual(JSON.parse(actual), JSON.parse(source));
});

test('malformed documents and unbounded or fractional widths are refused', () => {
  for (const source of ['{"a":}', '{"a": 01}', '[NaN]', '[] trailing']) {
    assert.throws(() => compactJsonLeaves(source), SyntaxError);
  }
  for (const width of [0, -1, Infinity, NaN, 1.5]) {
    assert.throws(() => compactJsonLeaves('{}', width), RangeError);
  }
});

test('maintained JSON documents have readable headroom and stable leaf formatting', {timeout: 10000}, () => {
  for (const path of ['data/parity/issue-1584-cnl-infeasible.json',
    'rust/tests/fixtures/meta-executor-parity.json', 'rust/tests/fixtures/source-fetch-provenance.json']) {
    const source = readFileSync(new URL('../../' + path, import.meta.url), 'utf8');
    assert(source.trimEnd().split('\n').length < 1350, path);
    assert.equal(compactJsonLeaves(source), source, path);
    if (path.startsWith('data/parity/')) {
      assert.equal(compactJsonLeaves(JSON.stringify(JSON.parse(source), null, 2) + '\n'), source, path);
    }
    assert.equal(lexicalContent(compactJsonLeaves(source)), lexicalContent(source), path);
  }
});
