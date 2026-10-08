// R1188-U5 / T1017-T1028: module grouping retains every mapped test and fixture.
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, posix, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { loadRenameMap, resolveRenameChains } from '../../../experiments/formal_ai_subagent/rename-by-rule.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');

test('later area moves resolve historical destinations, and cycles refuse', () => {
  const moves = [{ from: 'old.rs', to: 'named.rs', tree: 'names' },
    { from: 'named.rs', to: 'area/named.rs', tree: 'areas' }];
  assert.deepEqual(resolveRenameChains(moves).map(({ from, to }) => [from, to]),
    [['old.rs', 'area/named.rs'], ['named.rs', 'area/named.rs']]);
  assert.deepEqual(resolveRenameChains([moves[0]], moves)[0].to, 'area/named.rs');
  assert.throws(() => resolveRenameChains([{ from: 'a', to: 'b' }, { from: 'b', to: 'a' }]), /cycle/u);
});

test('named shards join the map and exclude their own rule source from substitution', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-map-shards-'));
  try {
    mkdirSync(join(directory, 'data/meta/rename-map'), { recursive: true });
    writeFileSync(join(directory, 'data/meta/rename-map.lino'),
      'rename-map\n  exclude "history/"\n  include "data/meta/rename-map/areas.lino"\n');
    writeFileSync(join(directory, 'data/meta/rename-map/areas.lino'),
      'rename-map\n  tree areas\n    rename\n      from "old.rs"\n      to "area/old.rs"\n      reason "module area"\n');
    const map = loadRenameMap(directory);
    assert.equal(map.renames.length, 1);
    assert.equal(map.renames[0].tree, 'areas');
    assert.ok(map.exclude.includes('data/meta/rename-map.lino'));
    assert.ok(map.exclude.includes('data/meta/rename-map/areas.lino'));
    assert.ok(map.exclude.includes('history/'));
    writeFileSync(join(directory, 'data/meta/rename-map/areas.lino'),
      'rename-map\n  include "data/meta/rename-map.lino"\n');
    assert.throws(() => loadRenameMap(directory), /include cycle/u);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('each moved unit test remains registered once in its declared area and keeps resolvable fixtures', () => {
  const map = loadRenameMap(root);
  const moves = map.renames.filter((move) => move.tree === 'unit-tests-by-area');
  assert.ok(moves.length > 0);
  const rootModules = readFileSync(join(root, 'rust/tests/unit/mod.rs'), 'utf8');
  const areas = new Map();
  for (const move of moves) {
    assert.equal(existsSync(join(root, move.from)), false, move.from);
    assert.equal(existsSync(join(root, move.to)), true, move.to);
    const area = posix.dirname(move.to);
    areas.set(area, (areas.get(area) ?? 0) + 1);
    const name = posix.basename(move.to, '.rs');
    const areaModules = readFileSync(join(root, area, 'mod.rs'), 'utf8');
    assert.equal(areaModules.split(`mod ${name};`).length - 1, 1, move.to);
    assert.equal(rootModules.split(`mod ${name};`).length - 1,
      name === posix.basename(area).replaceAll('-', '_') ? 1 : 0, move.from);
    const source = readFileSync(join(root, move.to), 'utf8');
    for (const match of source.matchAll(/(?:include_str!|include_bytes!)\s*\(\s*"([^"\n]+)"/gu)) {
      assert.ok(existsSync(resolve(root, area, match[1])), `${move.to}: ${match[1]}`);
    }
  }
  for (const [area, count] of areas) {
    assert.ok(count >= 5, area);
    const path = `${posix.basename(area)}/mod.rs`;
    assert.equal(rootModules.split(`#[path = "${path}"]`).length - 1, 1, area);
  }
});
