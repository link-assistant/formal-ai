import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { literalSourceList, requiredCensusSelectors, checkedCensusSelectorRepair } from '../../../scripts/generate-census-source-selectors.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
test('actual native roots and all collector source profiles select census validation', () => {
  const text = fs.readFileSync(path.join(root, '.github/workflows/regenerate-self-ast-census.yml'), 'utf8');
  assert.equal(checkedCensusSelectorRepair(text, requiredCensusSelectors(root)), text);
});
test('repair preserves dispatch permissions jobs comments and existing source guards', () => {
  const original = 'on:\n  pull_request:\n    paths:\n      - "old/**"\n  workflow_dispatch: {}\npermissions:\n  contents: read\njobs: {}\n# original\n';
  const repaired = checkedCensusSelectorRepair(original, ['new/**']);
  assert.equal(repaired, original.replace('"old/**"', '"old/**"\n      - "new/**"'));
  assert.equal(checkedCensusSelectorRepair(repaired, ['new/**']), repaired);
});
test('unknown and duplicate source-list identities refuse instead of silently omitting inputs', () => {
  for (const source of ['const nativeRoots=[foreign];', 'const nativeRoots=["same","same"];', 'const nativeRoots=["../outside"];', 'const nativeRoots=["a"];function f(){const nativeRoots=["b"];}']) assert.throws(() => literalSourceList(source, 'nativeRoots'));
});
test('duplicate original guards and malformed authority shape refuse', () => {
  for (const source of ['on:\n  pull_request:\n    paths: [same, same]\n', 'on:\n  pull_request:\n    paths: unknown\n']) assert.throws(() => checkedCensusSelectorRepair(source, ['new/**']));
});
