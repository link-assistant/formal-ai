// R1188-U2: the map of how this repository applies code-architecture-principles
// is generated from data/meta/architecture-principles.lino and checked.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { parsePrinciples, problems, renderPage } from '../../../scripts/render-architecture-principles.mjs';

const SOURCE = 'data/meta/architecture-principles.lino';
const PAGE = 'docs/architecture/principles.md';

test('every universal principle of the pinned README is mapped, in eight groups', () => {
  const map = parsePrinciples(readFileSync(SOURCE, 'utf8'));
  assert.equal(map.groups.length, 8);
  assert.equal(map.groups.reduce((sum, group) => sum + group.principles.length, 0), 51);
  assert.match(map.commit, /^[0-9a-f]{40}$/u);
});

test('the page is current and every cited gate and requirement row exists', () => {
  const map = parsePrinciples(readFileSync(SOURCE, 'utf8'));
  assert.deepEqual(problems(map), []);
  assert.equal(readFileSync(PAGE, 'utf8'), renderPage(map));
});

test('an unknown gate or requirement row is reported', () => {
  const map = parsePrinciples([
    'architecture-principles',
    '  group "Example"',
    '    principle "Example principle"',
    '      practice "Something."',
    '      enforced-by check-that-does-not-exist',
    '      gap R0-NOT-A-ROW',
  ].join('\n'));
  assert.deepEqual(problems(map), [
    'Example principle: no gate data/meta/ci-gates/check-that-does-not-exist.lino',
    'Example principle: no requirement row R0-NOT-A-ROW',
  ]);
});
