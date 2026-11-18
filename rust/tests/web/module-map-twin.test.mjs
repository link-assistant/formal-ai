// Issue #1191: the module-map property that only main's native unit test
// (rust/tests/unit/issue_954_module_map.rs) checked, run in the fast PR lane
// through the JavaScript twin scripts/generate-module-map.mjs.
//
// Recorded regression: PR #1559 added `pub mod standing_scope;` to lib.rs with
// a green PR run; main run 43730270231 then failed
// `the_module_map_equals_the_librs_mod_list`. The replay below removes that
// row again and requires the twin to fail exactly as the Rust test did.

import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  BEGIN, END, committedSection, coverage, main, mappedNames, rendered, rows, sectionFor, tables, tidy,
} from '../../../scripts/generate-module-map.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const SCRIPT = join(ROOT, 'scripts/generate-module-map.mjs');
const MAP = 'docs/architecture/module-map.md';
const read = relative => readFileSync(join(ROOT, relative), 'utf8');

/** A scratch copy of exactly the files the twin reads: lib.rs, module headers, the script and the map. */
function scratch(editMap = text => text) {
  const root = mkdtempSync(join(tmpdir(), 'module-map-'));
  mkdirSync(join(root, 'scripts'), { recursive: true });
  cpSync(join(ROOT, 'scripts/generate-module-map.rs'), join(root, 'scripts/generate-module-map.rs'));
  cpSync(join(ROOT, 'rust/src'), join(root, 'rust/src'), { recursive: true });
  mkdirSync(join(root, 'docs/architecture'), { recursive: true });
  writeFileSync(join(root, MAP), editMap(read(MAP)));
  return root;
}

test('the committed module map lists exactly the modules lib.rs declares', () => {
  const { declared, missing, stale } = coverage(ROOT);
  assert.ok(declared.length >= 150, `the crate documents a large module surface (found ${declared.length})`);
  assert.deepEqual({ missing, stale }, { missing: [], stale: [] });
  const result = spawnSync(process.execPath, [SCRIPT, '--check'], { encoding: 'utf8', timeout: 60_000 });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /^module map covers all \d+ declared modules\n$/u);
});

test('replay of main run 43730270231: a declared module without a row fails --check', () => {
  const root = scratch(text => text.split('\n').filter(line => !line.startsWith('| `standing_scope` |')).join('\n'));
  const writes = [];
  const original = process.stderr.write;
  try {
    assert.deepEqual(coverage(root).missing, ['standing_scope']);
    // main() takes the root, so the replay runs the CLI logic against the scratch copy.
    process.stderr.write = chunk => { writes.push(String(chunk)); return true; };
    const status = main(['--check'], root);
    process.stderr.write = original;
    assert.equal(status, 1);
    assert.match(writes.join(''), /missing \["standing_scope"\], stale \[\]; run --write/u);
  } finally {
    process.stderr.write = original;
    rmSync(root, { recursive: true, force: true });
  }
});

test('a row naming no declared module is reported stale', () => {
  const root = scratch(text => text.replace(END, '| `retired_module` | Gone. | §2 |\n' + END));
  try {
    assert.deepEqual(coverage(root), { ...coverage(ROOT), stale: ['retired_module'] });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('every committed row carries a responsibility and a real owning section, one per module', () => {
  const section = committedSection(read(MAP));
  let count = 0;
  for (const line of section.split('\n')) {
    const rest = line.trim();
    if (!rest.startsWith('|')) continue;
    const body = rest.slice(1).trim();
    if (body.startsWith('---') || body.startsWith('Module |')) continue;
    const cells = line.split('|');
    if (cells.length < 4) continue;
    assert.ok([...cells[2].trim()].length >= 8, `a one-sentence responsibility: ${line}`);
    const owning = cells[3].trim();
    assert.ok(owning.startsWith('§'), `the owning section is a §-number: ${line}`);
    const number = Number.parseFloat(owning.slice(1));
    assert.ok(number >= 2 && number <= 15, `a real section: ${line}`);
    count += 1;
  }
  assert.equal(count, rows(ROOT).length);
});

test('the twin renders the committed section byte for byte', () => {
  // The committed section is what the Rust generator (or this twin) wrote with --write.
  assert.equal(committedSection(read(MAP)), rendered(ROOT));
  assert.ok(rendered(ROOT).startsWith(BEGIN) && rendered(ROOT).endsWith(END));
});

test('tables are read from the Rust generator, not duplicated', () => {
  const { overrides, rules } = tables(read('scripts/generate-module-map.rs'));
  assert.ok(overrides.size > 0 && rules.length > 0);
  for (const rule of rules) assert.equal(sectionFor(`${rule.needles[0]}`, [rule]), rule.section);
  assert.equal(sectionFor('no_rule_matches_this_name', []), '2');
  const generator = read('scripts/generate-module-map.rs');
  for (const needle of ['--check', '--write', 'module-map:begin', 'pub mod ']) {
    assert.ok(generator.includes(needle), `the Rust generator still carries ${needle}`);
  }
});

test('tidy truncates without dangling words (ported from the Rust unit test)', () => {
  const sentence = tidy('rethinking *how* to solve the general failure-to-repair classifier when the meta algorithm and the');
  assert.ok([...sentence].length < 120);
  assert.ok(!sentence.endsWith(' and'));
  // Byte-for-byte with the Rust generator: one pass over the tails, so `and the` keeps `and`.
  assert.equal(sentence, 'rethinking *how* to solve the general failure-to-repair classifier when the meta algorithm and.');
  assert.ok(sentence.endsWith('.'));
  assert.equal(tidy('  Spaced   words...  '), 'Spaced words.');
});

test('mappedNames reads the first column back (ported from the Rust unit test)', () => {
  assert.deepEqual(
    mappedNames('| Module | Responsibility | Owning section |\n|---|---|---|\n| `solver` | The universal problem solver. | §7 |\n| not a row\n'),
    ['solver'],
  );
});
