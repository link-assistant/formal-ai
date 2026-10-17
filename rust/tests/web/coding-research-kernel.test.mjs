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
