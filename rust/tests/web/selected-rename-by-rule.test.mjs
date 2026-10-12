import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync,
  rmSync, symlinkSync, realpathSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { selectedRenamePlan } from '../../../experiments/formal_ai_subagent/selected-rename-by-rule.mjs';
const sourceRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const digest = value => createHash('sha256').update(value).digest('hex');
function fixture(run) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'selected-rename-')));
  const dependencies = [
    'experiments/formal_ai_subagent/rename-by-rule.mjs',
    'scripts/measure-abbreviations.mjs', 'scripts/lib/links-notation-names.mjs',
    'data/meta/notation-rules.lino', 'data/meta/abbreviation-rules.lino',
    'data/meta/rename-map.lino',
  ];
  const values = new Map();
  function add(path) {
    if (values.has(path)) return;
    const content = readFileSync(join(sourceRoot, path));
    values.set(path, content);
    for (const match of content.toString().matchAll(
      /^\s*include "([^"]+)"\s*$/gmu)) add(match[1]);
  }
  for (const path of dependencies) add(path);
  values.set('rust/src/documentation/doc.rs', Buffer.from('// α😀\npub fn value() -> u8 { 7 }\n'));
  values.set('rust/src/documentation/mod.rs', Buffer.from('pub mod doc;\n'));
  for (const [path, bytes] of values) {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), bytes);
  }
  const inventory = [...values.keys()].sort();
  const manifest = { inventory, inventorySha256: digest(JSON.stringify(inventory)),
    prefix: 'rust/src/documentation', sources: [...values].map(([path, bytes]) =>
      ({ path, sha256: digest(bytes) })) };
  try { run(root, manifest); } finally { rmSync(root, { recursive: true, force: true }); }
}
function rebindInventory(manifest) {
  manifest.inventorySha256 = digest(JSON.stringify(manifest.inventory));
}
test('dictionary rename preserves bytes, module references and exact inverse', () => fixture((root, manifest) => {
  const plan = selectedRenamePlan(root, manifest);
  assert.deepEqual(plan.moves.map(move => [move.from, move.to]), [
    ['rust/src/documentation/doc.rs', 'rust/src/documentation/document.rs'],
  ]);
  const body = plan.rows.find(row => row.beforePath.endsWith('/doc.rs'));
  assert.equal(body.after, '// α😀\npub fn value() -> u8 { 7 }\n');
  assert.equal(body.inverse, body.before);
  assert.equal(plan.rows.find(row => row.beforePath.endsWith('/mod.rs')).after,
    'pub mod document;\n');
  assert.equal(plan.completeness, 'SelectedReferencesOnly');
  assert.equal(plan.namespaceRaceProof, false);
  assert.deepEqual(selectedRenamePlan(root, manifest), plan);
}));
test('source drift refuses', () => fixture((root, manifest) => {
  writeFileSync(join(root, 'rust/src/documentation/doc.rs'), 'changed');
  assert.throws(() => selectedRenamePlan(root, manifest), /Source drift/u);
}));
test('inventory hash drift refuses', () => fixture((root, manifest) => {
  manifest.inventory.push('other.rs');
  assert.throws(() => selectedRenamePlan(root, manifest), /Inventory drift/u);
}));
test('duplicate inventory refuses', () => fixture((root, manifest) => {
  manifest.inventory.push(manifest.inventory[0]); rebindInventory(manifest);
  assert.throws(() => selectedRenamePlan(root, manifest), /Duplicate inventory/u);
}));
test('destination collision refuses', () => fixture((root, manifest) => {
  manifest.inventory.push('rust/src/documentation/document.rs'); rebindInventory(manifest);
  assert.throws(() => selectedRenamePlan(root, manifest), /collision/u);
}));
test('missing moved source refuses', () => fixture((root, manifest) => {
  manifest.sources = manifest.sources.filter(row => !row.path.endsWith('/doc.rs'));
  assert.throws(() => selectedRenamePlan(root, manifest), /Missing moved source/u);
}));
test('unbound transformation dependency refuses', () => fixture((root, manifest) => {
  manifest.sources = manifest.sources.filter(row => !row.path.endsWith('/rename-by-rule.mjs'));
  assert.throws(() => selectedRenamePlan(root, manifest), /Unbound transformation/u);
}));
test('symlink source refuses', () => fixture((root, manifest) => {
  const path = join(root, 'rust/src/documentation/doc.rs');
  rmSync(path); symlinkSync('mod.rs', path);
  assert.throws(() => selectedRenamePlan(root, manifest), /Aliased selected/u);
}));
test('invalid UTF-8 source refuses before hashing', () => fixture((root, manifest) => {
  writeFileSync(join(root, 'rust/src/documentation/doc.rs'), Buffer.from([65, 255, 66]));
  assert.throws(() => selectedRenamePlan(root, manifest), /Invalid UTF-8/u);
}));
test('unsafe source paths refuse', () => fixture((root, manifest) => {
  manifest.sources[0].path = '../outside.rs';
  assert.throws(() => selectedRenamePlan(root, manifest), /Unsafe selected/u);
}));

test('selected dependency bytes must match the imported transformer origin', () => fixture((root, manifest) => {
  const path = 'experiments/formal_ai_subagent/rename-by-rule.mjs';
  const changed = Buffer.concat([
    readFileSync(join(root, path)), Buffer.from('\n// different selected transformer origin\n'),
  ]);
  writeFileSync(join(root, path), changed);
  manifest.sources.find(row => row.path === path).sha256 = digest(changed);
  assert.throws(() => selectedRenamePlan(root, manifest), /Transformation origin mismatch/u);
}));

test('physical destination collision refuses even when inventory omits it', () => fixture((root, manifest) => {
  writeFileSync(join(root, 'rust/src/documentation/document.rs'), 'preserve existing source\n');
  assert.throws(() => selectedRenamePlan(root, manifest), /Physical destination collision/u);
  assert.equal(readFileSync(join(root, 'rust/src/documentation/document.rs'), 'utf8'),
    'preserve existing source\n');
}));
