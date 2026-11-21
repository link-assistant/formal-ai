// Keep source pins usable as complete Git SHA1 identities.
import assert from 'node:assert/strict';
import {readdirSync, readFileSync} from 'node:fs';
import test from 'node:test';

test('reviewed template ledgers and the timeout fixture use complete Git commit identities', () => {
  const directory = new URL('../../docs/case-studies/issue-1580/templates/', import.meta.url);
  let checked = 0;
  for (const name of readdirSync(directory).filter(name => name.endsWith('.json'))) {
    const value = JSON.parse(readFileSync(new URL(name, directory), 'utf8'));
    if (value && !Array.isArray(value) && value.commit) {
      assert.match(value.commit, /^[a-f0-9]{40}$/, name + ' commit must be a complete SHA1');
      checked++;
    }
  }
  assert.ok(checked >= 14, 'all retained current and historical review ledgers are included');
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/rust-timeout-documentation/manifest.json', import.meta.url), 'utf8'));
  assert.match(fixture.reviewedPin, /^[a-f0-9]{40}$/);
  assert.match(fixture.currentHead, /^[a-f0-9]{40}$/);
  assert.equal(fixture.reviewedPin, fixture.currentHead);
});
