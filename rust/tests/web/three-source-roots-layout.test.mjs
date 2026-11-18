// Issue #1191: the layout gate of rust/tests/unit/issue_1138_three_source_roots.rs
// in the fast PR lane. Only main ran it, so PR #1559 added a top-level
// examples/scoped-standing-requirements.mjs with a green pull request and main
// then failed `the_crate_lives_under_rust`. The example now lives in
// rust/examples/ (the move shared with PR #1572); this test fails any pull
// request that recreates a crate directory at the top level.

import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));

/** The paths the Rust layout tests forbid or require (same lists, same order); `exists` is injectable for replays. */
function layoutProblems(root, exists = path => existsSync(path)) {
  const problems = [];
  for (const layer of ['js', 'ts', 'rust']) {
    const path = join(root, layer);
    if (!exists(path) || !statSync(path).isDirectory()) problems.push(`${layer}/ must exist as a source root`);
  }
  for (const required of ['Cargo.toml', 'src', 'tests']) {
    if (!exists(join(root, 'rust', required))) problems.push(`rust/${required} must move with the crate root`);
  }
  for (const entry of ['benches', 'examples', 'Cargo.toml', 'src']) {
    if (exists(join(root, entry))) problems.push(`${entry} still sits at the repository top level`);
  }
  for (const required of ['worker/formal_ai_worker.js', 'seed_loader.js', 'seed-files.js']) {
    if (!exists(join(root, 'js', required))) problems.push(`js/${required} must move with the src/web first citizens`);
  }
  if (exists(join(root, 'rust/src/web'))) problems.push('the crate\'s src/web must not survive inside rust/');
  return problems;
}

test('the repository keeps the three source roots and no crate directory at the top level', () => {
  assert.deepEqual(layoutProblems(ROOT), []);
});

test('replay of main\'s the_crate_lives_under_rust failure: a top-level examples/ is reported', () => {
  // The tree as PR #1559 left it: this checkout plus a top-level examples/.
  const added = join(ROOT, 'examples');
  const problems = layoutProblems(ROOT, path => path === added || existsSync(path));
  assert.deepEqual(problems, ['examples still sits at the repository top level']);
});
