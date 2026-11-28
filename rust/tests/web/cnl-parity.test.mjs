// The JavaScript twin of rust/tests/standalone/cnl_parity.rs (issue #1584): every
// committed per-language cross-runtime fixture must equal what the JavaScript
// engine produces now, so the Rust digests and dialogue turns it pins are the
// current behavior. Regenerate with `node experiments/issue-1584/parity-fixture.mjs --write`.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {engineLanguages, fixtureLanguages, formatFixture, parityFixture, parityPath} from '../../../experiments/issue-1584/parity-fixture.mjs';

test('every loaded grammar has a committed parity fixture', () => {
  assert.deepEqual(fixtureLanguages(), engineLanguages());
});

for (const language of fixtureLanguages()) {
  test(`${language}: the CNL parity fixture matches the JavaScript engine`, {timeout: 300_000}, () => {
    const committed = JSON.parse(readFileSync(parityPath(language), 'utf8'));
    const current = parityFixture(language);
    assert.deepEqual(committed.projections, current.projections);
    assert.deepEqual(committed.dialogue, current.dialogue);
    assert.equal(readFileSync(parityPath(language), 'utf8'), formatFixture(current));
  });
}
