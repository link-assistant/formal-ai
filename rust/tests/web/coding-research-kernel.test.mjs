import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { tapVerificationDecision } from '../../../js/agentic/portable/coding_research_kernel.mjs';
import { verificationReceiptGap } from '../../../js/agentic/coding_research.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { generateCodingResearch, SOURCE, RUST, META, TYPESCRIPT } from '../../../scripts/generate-coding-research.mjs';
const read = relative => fs.readFileSync(new URL('../../../' + relative, import.meta.url), 'utf8');
const source = read(SOURCE);
const vectors = JSON.parse(read('rust/tests/fixtures/coding-research/tap-kernel.json'));
test('canonical terminal decisions reject incomplete or unaccounted TAP counts', () => {
  for (const row of vectors) assert.equal(tapVerificationDecision(...row.args), row.expected, row.id);
});
test('generated Rust, Links IR and TypeScript match canonical JavaScript exactly', () => {
  const outputs = generateCodingResearch(source);
  for (const target of [RUST, META, TYPESCRIPT]) assert.equal(outputs[target], read(target), target);
});
test('canonical mutation changes generated IR and actual Rust decision code', () => {
  const changed = generateCodingResearch(source.replace('passed + skipped !== tests', 'passed !== tests'));
  const original = generateCodingResearch(source);
  assert.notEqual(changed[META], original[META]);
  assert.notEqual(changed[RUST].slice(changed[RUST].indexOf('#![allow')), original[RUST].slice(original[RUST].indexOf('#![allow')));
  assert.throws(() => generateCodingResearch(source + '\nexport function hidden(value) { return value.foo(); }'));
});
test('production receipt adapter delegates decisions and refuses empty/duplicate terminal evidence', async () => {
  await installNodeHost(new WorkerHost({ WebAssembly: {
    instantiate() { throw Error('LocalWasmExecutionForbidden'); },
    compile() { throw Error('LocalWasmExecutionForbidden'); },
  } }));
  const receipt = output => JSON.stringify({ output, exit_code: 0 });
  assert.equal(verificationReceiptGap(receipt('# tests 1\n# pass 1\n# fail 0\n')), null);
  for (const output of ['', '# tests 2\n# pass 1\n# fail 0\n',
    '# tests 1\n# tests 1\n# pass 1\n# fail 0\n']) {
    assert.equal(verificationReceiptGap(receipt(output)), 'VerificationReceiptIncomplete');
  }
});

test('signed inclusive range emission preserves chained guards and canonical score boundaries', async () => {
  const { sourceCaptureTrustAccepted } = await import('../../../js/agentic/portable/coding_research_kernel.mjs');
  const output = generateCodingResearch(source)[RUST];
  assert.ok(output.includes('(-9_007_199_254_740_991.0..=9_007_199_254_740_991.0).contains(&score)'));
  assert.ok(!output.includes('score >= -9_007_199_254_740_991.0 && score <= 9_007_199_254_740_991.0'));
  for (const score of [NaN, Infinity, -Infinity, -9007199254740992, -9007199254740991,
    -1, -0, 0, 1, 9007199254740991, 9007199254740992]) {
    assert.equal(sourceCaptureTrustAccepted(true, 2, true, score, true, 1),
      score >= -9007199254740991 && score <= 9007199254740991);
    assert.equal(sourceCaptureTrustAccepted(false, 2, true, score, true, 1), false);
    assert.equal(sourceCaptureTrustAccepted(true, 2, false, score, true, 1), false);
  }
  const declaration = '/** @param {boolean} observed @param {number} value @returns {boolean} */\n';
  const bounded = declaration + 'export function observedBounded(observed, value) { return observed && value >= -12 && value <= 7; }';
  const rendered = generateCodingResearch(bounded)[RUST];
  assert.ok(rendered.includes('observed && (-12.0..=7.0).contains(&value)'));
  const otherVariable = declaration + 'export function observedBounded(observed, value) { return value >= -12 && observed <= 7; }';
  assert.throws(() => generateCodingResearch(otherVariable));
  const guardedDeclaration = '/** @param {boolean} first @param {boolean} second @param {number} value @returns {boolean} */\n';
  const guarded = guardedDeclaration + 'export function guardedRange(first, second, value) { return (first || second) && value >= -12 && value <= 7; }';
  assert.ok(generateCodingResearch(guarded)[RUST].includes('(first || second) && (-12.0..=7.0).contains(&value)'));
  const unequalDeclaration = '/** @param {number} value @param {number} other @returns {boolean} */\n';
  const unequal = unequalDeclaration + 'export function unequalBounds(value, other) { return value >= -12 && other <= 7; }';
  assert.ok(!generateCodingResearch(unequal)[RUST].includes('.contains('));
});
