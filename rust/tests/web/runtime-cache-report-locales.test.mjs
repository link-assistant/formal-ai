import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {installHost} from '../../../js/agentic/host.mjs';
import {parseLino} from '../../../js/server/lino.mjs';
import {renderResponse, localizedResponse} from '../../../js/agentic/crate/seed.mjs';
import {reportText} from '../../../js/agentic/crate/seed_reports.mjs';
import {emptyDerivation} from '../../../js/agentic/crate/derivation.mjs';
import {persistRuntimeDerivation, loadRuntimeDerivation}
  from '../../../js/server/runtime-cache.mjs';
import {runExplain} from '../../../js/server/derivation-store.mjs';

installHost({parseLino, readText: relative => fs.readFileSync(
  new URL('../../../' + relative, import.meta.url), 'utf8')});
const answer = 'answer_0123456789abcdef';
const values = [['answer_id', answer], ['answer', answer], ['directory', '/runtime/cache']];
const expected = [
  ['en', 'derivation_record_missing:answer_0123456789abcdef; directory=/runtime/cache',
    'derivation_record_ambiguous: use answer_0123456789abcdef@<record-digest>'],
  ['ru', 'derivation_record_missing:answer_0123456789abcdef; каталог=/runtime/cache',
    'derivation_record_ambiguous: используйте answer_0123456789abcdef@<record-digest>'],
  ['hi', 'derivation_record_missing:answer_0123456789abcdef; निर्देशिका=/runtime/cache',
    'derivation_record_ambiguous: answer_0123456789abcdef@<record-digest> का उपयोग करें'],
  ['zh', 'derivation_record_missing:answer_0123456789abcdef; 目录=/runtime/cache',
    'derivation_record_ambiguous: 请使用 answer_0123456789abcdef@<record-digest>'],
  ['es', 'derivation_record_missing:answer_0123456789abcdef; directorio=/runtime/cache',
    'derivation_record_ambiguous: utilice answer_0123456789abcdef@<record-digest>'],
];
for (const [language, missing, ambiguous] of expected) {
  test('actual runtime-cache report renderer preserves locale ' + language, () => {
    assert.equal(renderResponse('runtime_derivation_record_missing', language, values), missing);
    assert.equal(renderResponse('derivation_record_ambiguous', language, values), ambiguous);
    assert.equal(localizedResponse('runtime_derivation_record_missing', language),
      missing.replace(answer, '{answer_id}').replace('/runtime/cache', '{directory}'));
  });
}
test('English diagnostic emission and missing-report fallback retain their original semantics', () => {
  assert.equal(reportText('runtime_derivation_record_missing', values), expected[0][1]);
  assert.equal(reportText('derivation_record_ambiguous', values), expected[0][2]);
  assert.equal(reportText('missing_runtime_cache_report', values), 'missing_runtime_cache_report');
  assert.equal(renderResponse('missing_runtime_cache_report', 'en', values), null);
  assert.equal(localizedResponse('runtime_derivation_record_missing', 'unregistered'),
    'derivation_record_missing:{answer_id}; directory={directory}');
});
test('actual cache ambiguity and explain failure expose the seeded English diagnostics', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-runtime-report-'));
  try {
    const environment = {FORMAL_AI_CACHE_DIR: directory};
    const first = emptyDerivation(answer);
    const firstLocation = persistRuntimeDerivation(first, {env: environment});
    const secondLocation = persistRuntimeDerivation(
      {...first, search_queries: ['a distinct source trace']}, {env: environment});
    assert.notEqual(firstLocation.locator, secondLocation.locator);
    assert.throws(() => loadRuntimeDerivation(answer, {env: environment}), error => error.message === expected[0][2]);
    assert.deepEqual(loadRuntimeDerivation(firstLocation.locator, {env: environment}), first);
    const result = runExplain(['answer_ffffffffffffffff'], {env: environment, cwd: directory});
    assert.equal(result.code, 1);
    assert.equal(result.stdout, '');
    const message = 'derivation_record_missing:answer_ffffffffffffffff; directory='
      + path.join(directory, 'derivations');
    assert.equal(result.stderr, 'Error: ' + JSON.stringify(message) + '\n');
  } finally {
    fs.rmSync(directory, {recursive: true, force: true});
  }
});
