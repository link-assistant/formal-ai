import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {
  measure, upstreamBlocks, readProjection, presentCommittedProjection,
} from '../../../scripts/translate-js-rust.mjs';

const hash = value => createHash('sha256').update(value).digest('hex');
const source = '/** @returns {number[]} */\nexport function values() { return [1, 2]; }';
const path = 'js/agentic/crate/synthetic-presentation.mjs';

// A synthetic upstream-result protocol fixture, never evidence of running the pinned translator.
function syntheticResult(code) {
  return {
    text: source,
    details: [null, 'import from ../outside.mjs: unsupported'],
    code: [
      `// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=${hash(source)} bytes=${Buffer.byteLength(source)}`,
      '',
      '// meta-language:prelude begin',
      '#![allow(unused)]',
      '// meta-language:prelude end',
      '',
      `// meta-language:translated JavaScript export_statement items=1 sha256=${hash(code)}`,
      ...source.split('\n').map(line => '// | ' + line),
      code,
      '',
      '// meta-language:carried JavaScript import_statement (unsupported)',
      "// | import { missing } from '../outside.mjs';",
      '',
    ].join('\n'),
  };
}

function assertProductionPath(result) {
  const measured = measure(path, result);
  const decoded = upstreamBlocks(measured.projection);
  const records = readProjection(measured.projection);
  assert.equal(decoded.header, upstreamBlocks(result.code).header);
  assert.equal(records.sha256, hash(source));
  assert.equal(records.bytes, Buffer.byteLength(source));
  assert.equal(records.translated, measured.translated);
  assert.equal(records.carried, measured.carried);
  assert.deepEqual(records.refusals, measured.refusals);
  assert.deepEqual(records.blockers, measured.blockers);
  assert.equal(decoded.blocks[0].text, upstreamBlocks(result.code).blocks[0].text);
  assert.equal(presentCommittedProjection(measured.projection), measured.projection);
  return {measured, decoded};
}

test('short synthetic upstream result follows measure and stays raw byte exact', () => {
  const code = 'pub fn values() -> Vec<f64> {\n    vec![1f64, 2f64]\n}';
  const {measured, decoded} = assertProductionPath(syntheticResult(code));
  assert.equal(decoded.blocks[0].presentation, null);
  assert.ok(measured.projection.includes(code));
  assert.equal(measured.translated, 1);
  assert.equal(measured.carried, 1);
});

test('long synthetic result becomes a recoverable derivative through measure', () => {
  const values = Array.from({length: 80}, (_, index) => `${index}f64`).join(', ');
  const code = `pub fn values() -> Vec<f64> { vec![${values}] }`;
  const {decoded} = assertProductionPath(syntheticResult(code));
  const block = decoded.blocks[0];
  assert.ok(block.presentation);
  assert.equal(block.presentation.raw, code);
  assert.equal(block.presentation.rawSHA256, hash(code));
  assert.notEqual(block.renderedText, block.text);
});

test('measure refuses upstream raw hash corruption before emitting a projection', () => {
  const result = syntheticResult('pub fn values() -> Vec<f64> { vec![1f64] }');
  result.code = result.code.replace('vec![1f64]', 'vec![2f64]');
  assert.throws(() => measure(path, result), /no code matching its hash/);
});

test('measure refuses opaque long source instead of certifying presentation', () => {
  const result = syntheticResult('foreign!(' + Array(100).fill('value').join(', ') + ');');
  assert.throws(() => measure(path, result), /Macro|Opaque|Unsupported/);
});
