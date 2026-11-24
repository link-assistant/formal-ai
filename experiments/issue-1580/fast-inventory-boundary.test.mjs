// A valid new suite must not invalidate the entire reviewed PR inventory.
import assert from 'node:assert/strict';
import test from 'node:test';
import {selectFastTests} from '../../scripts/list-fast-pull-request-tests.mjs';

test('fast inventory accepts 257 distinct available suites while rejecting invalid source inventories', () => {
  const files = Array.from({length:257}, (_, index) => 'rust/tests/web/fixture-' + index + '.test.mjs');
  const available = new Set(files);
  const manifest = files => ({schema:'fast-pr-javascript-tests/v1', files});
  assert.deepEqual(selectFastTests(manifest(files), file => available.has(file)), files);
  for (const invalid of [[], null, 'not-an-array', [...files, files[0]],
    [...files, 'rust/tests/web/missing.test.mjs'], [...files, '../outside.test.mjs']])
    assert.throws(() => selectFastTests(manifest(invalid), file => available.has(file)));
});
