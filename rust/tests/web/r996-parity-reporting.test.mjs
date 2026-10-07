// R996 (doctrine 2026-09-24): the parity state is reported honestly wherever
// it is claimed, and no document claims more parity than is measured.
//
// docs/source-roots.md is the per-root statement. Each claim it makes is
// checked against the thing it describes:
// - the ts/ row says the tree is generated exactly when ts/ holds generated
//   TypeScript, and calls it a stub only while it holds none;
// - the js/ row's native-only handler count is the measured ceiling of
//   data/meta/js-parity-ratchet.lino, the number scripts/check-js-parity.mjs
//   enforces;
// - only the rust/ row calls its root complete, and no row claims full parity.
// The register's own R992 row is held to the same facts: no ts/ stub once
// ts/ is generated, and the js → rust counts of data/meta/js-rust-translation.lino.

import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const ROOTS = readFileSync(`${REPO_ROOT}/docs/source-roots.md`, 'utf8');

/** The table row of one root, by its backticked name. */
function row(root) {
  const line = ROOTS.split('\n').find((candidate) => candidate.startsWith(`| \`${root}\` |`));
  assert.ok(line, `docs/source-roots.md has a row for ${root}`);
  return line;
}

/** TypeScript files under a directory, recursively. */
function typescriptFiles(directory) {
  let count = 0;
  for (const entry of readdirSync(directory)) {
    const path = `${directory}/${entry}`;
    if (statSync(path).isDirectory()) count += typescriptFiles(path);
    else if (/\.(m?ts)$/.test(entry)) count += 1;
  }
  return count;
}

test('R996: the ts/ row matches what ts/ holds', () => {
  const generated = typescriptFiles(`${REPO_ROOT}/ts`);
  const ts = row('ts/');
  if (generated > 0) {
    assert.match(ts, /generated from `js\/`/, 'ts/ holds generated TypeScript, so the row says so');
    assert.doesNotMatch(ts, /\bstub\b/, `ts/ holds ${generated} TypeScript files; the row must not call it a stub`);
  } else {
    assert.match(ts, /\bstub\b/, 'ts/ holds no TypeScript, so the row must call it a stub');
  }
});

test('R996: the js/ row reports the measured native-only handler count', () => {
  const ratchet = readFileSync(`${REPO_ROOT}/data/meta/js-parity-ratchet.lino`, 'utf8');
  const ceiling = Number(/native_only_ceiling (\d+)/.exec(ratchet)?.[1]);
  assert.ok(Number.isInteger(ceiling), 'the parity ratchet records a ceiling');
  const js = row('js/');
  assert.match(js, new RegExp(`measures ${ceiling} native-only handler-registry rows`));
  assert.match(js, /data\/meta\/js-parity-ratchet\.lino/);
});

test('R996: only the rust/ root is called complete, and no row claims full parity', () => {
  assert.match(row('rust/'), /complete implementation root/);
  for (const root of ['js/', 'ts/']) {
    assert.doesNotMatch(row(root), /complete implementation|full parity/i, `${root} claims more than is measured`);
  }
  assert.doesNotMatch(ROOTS, /full three-root parity (is|has been) (reached|achieved|delivered)/i);
});

test('R996: the R992 register row reports the measured state of each leg', () => {
  const doctrine = readFileSync(
    `${REPO_ROOT}/docs/requirements/doctrine-standing-doctrine-three-roots-full-parity-through-the-meta-language-2026-09-24.md`,
    'utf8',
  );
  const r992 = doctrine.split('\n').find((line) => line.startsWith('| R992 |'));
  assert.ok(r992, 'the doctrine shard defines R992');
  const generated = typescriptFiles(`${REPO_ROOT}/ts`);
  if (generated > 0) assert.doesNotMatch(r992, /\bstub\b/, `ts/ holds ${generated} TypeScript files; R992 must not call it a stub`);
  // The js → rust leg: the counts are the translation ledger's, and the row
  // stays partial while any item is still carried by hand.
  const ledger = readFileSync(`${REPO_ROOT}/data/meta/js-rust-translation.lino`, 'utf8');
  const field = (name) => Number(new RegExp(`^ {2}${name} (\\d+)$`, 'mu').exec(ledger)?.[1]);
  const [translated, carried, modules] = [field('translated_items'), field('carried_items'), field('modules')];
  const items = (translated + carried).toLocaleString('en-US');
  assert.ok(r992.includes(`${translated} of ${items} top-level items across ${modules} modules translate`), r992);
  if (carried > 0) assert.match(r992, /\| Partial, /u, 'R992 cannot be delivered while items are carried by hand');
});
