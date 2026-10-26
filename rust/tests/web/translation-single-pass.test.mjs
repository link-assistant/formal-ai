import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { combinedTranslationModuleURL, parseSinglePassContract, SINGLE_PASS_CONTRACT_FILE } from '../../../scripts/lib/translation-single-pass.mjs';
import { parseLedger } from '../../../scripts/translate-js-rust.mjs';

const hash = source => createHash('sha256').update(source).digest('hex');
const source = `import { suffix } from './dependency.mjs';
let calls = 0;
function translateModule(source, from, to, options = {}) {
  calls++;
  return { translation: Object.freeze({code: source + suffix, from, to}),
    signatures: [{name: options.imports.ref[0].name, type: 'number'}] };
}
export function selfTranslate(source, from, to, options) {
  return translateModule(source, from, to, options).translation;
}
export function selfTranslationSignatures(source, from, options) {
  return translateModule(source, from, 'Rust', options).signatures;
}
export function callCount() { return calls; }
`;

test('one fresh pinned core result equals both public wrapper results with relative imports preserved', async t => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-single-pass-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  writeFileSync(join(directory, 'dependency.mjs'), 'export const suffix = ":same dependency";');
  const original = pathToFileURL(join(directory, 'self-translation.js')).href;
  const api = await import(combinedTranslationModuleURL(source, original, hash(source)));
  const options = {imports: {ref: [{name: 'bound_import'}]}};
  const combined = api.formalAICombinedSelfTranslation('fresh source', 'JavaScript', 'Rust', options);
  assert.equal(api.callCount(), 1);
  assert.deepEqual(combined.translation, api.selfTranslate('fresh source', 'JavaScript', 'Rust', options));
  assert.deepEqual(combined.signatures, api.selfTranslationSignatures('fresh source', 'JavaScript', options));
  assert.equal(api.callCount(), 3);
  assert.equal(combined.translation.code, 'fresh source:same dependency');
});

test('a changed pinned source refuses before any module execution', () => {
  assert.throws(() => combinedTranslationModuleURL(source + '\nthrow Error("must not execute");',
    'file:///owned/self-translation.js', hash(source)), /Pinned upstream/);
  assert.throws(() => combinedTranslationModuleURL(source, 'https://untrusted.invalid/module.js', hash(source)));
  for (const altered of [source.replace('function translateModule(', 'function renamed('),
    source + '\nconst origin = import.meta.url;', source + '\nimport("./dynamic.mjs");']) {
    assert.throws(() => combinedTranslationModuleURL(altered, 'file:///owned/self-translation.js', hash(altered)));
  }
  assert.throws(() => combinedTranslationModuleURL('x'.repeat(262145), 'file:///owned/module.js', 'a'.repeat(64)));
});

test('adapter metadata is exact, complete and pinned to the current upstream ledger', () => {
  const text = readFileSync(SINGLE_PASS_CONTRACT_FILE, 'utf8');
  const contract = parseSinglePassContract(text);
  assert.equal(contract.commit, parseLedger(readFileSync('data/meta/js-rust-translation.lino', 'utf8')).commit);
  for (const invalid of [text + '\n  commit ' + contract.commit,
    text.replace('link-foundation/meta-language', 'untrusted/repository'),
    text.replace('js/src/self-translation.js', '../outside.js'),
    text.replace(contract.sha256, 'not-a-hash'), text + '\n  unknown value']) {
    assert.throws(() => parseSinglePassContract(invalid));
  }
});
