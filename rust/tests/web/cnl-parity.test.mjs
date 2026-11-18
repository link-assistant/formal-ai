// The JavaScript twin of rust/tests/unit/cnl_parity.rs (issue #1584): the
// committed cross-runtime fixture must equal what the JavaScript engine produces
// now, so the Rust digests and dialogue turns it pins are the current behavior.
// Regenerate with `node experiments/issue-1584/parity-fixture.mjs --write`.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {formatFixture, PARITY_PATH, parityFixture} from '../../../experiments/issue-1584/parity-fixture.mjs';

test('the CNL parity fixture matches the JavaScript engine', {timeout: 300_000}, () => {
  const committed = JSON.parse(readFileSync(PARITY_PATH, 'utf8'));
  const current = parityFixture();
  assert.deepEqual(committed.languages, current.languages);
  assert.deepEqual(committed.projections, current.projections);
  for (const language of current.languages) assert.deepEqual(committed.dialogue[language], current.dialogue[language], language);
  assert.equal(readFileSync(PARITY_PATH, 'utf8'), formatFixture(current));
});
