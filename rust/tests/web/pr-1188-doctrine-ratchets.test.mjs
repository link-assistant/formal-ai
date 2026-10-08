// PR #1188 doctrine rows pinned by the ratchet gates they name: R997/R998 (no
// browser handler row is native-only) and R1005/R1010 (the meta reasoner, the
// JavaScript server and the agentic planner hold no natural-language literal;
// the worker's count only falls). The gates run in the layered CI js tier;
// this suite runs them again so a row's verdict rests on a test, and pins the
// zero ceilings themselves so a gate cannot pass by raising one.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');

function ceilings(relative) {
  const values = new Map();
  for (const line of readFileSync(join(ROOT, relative), 'utf8').split('\n')) {
    const match = /^\s+([\w-]+[-_]ceiling)\s+(\d+)\s*$/.exec(line);
    if (match) values.set(match[1], Number(match[2]));
  }
  return values;
}

function gate(script) {
  return execFileSync(process.execPath, [join(ROOT, 'scripts', script)], { cwd: ROOT, encoding: 'utf8' });
}

describe('R997/R998: every handler the Rust engine has is a JavaScript twin', () => {
  test('the native-only ceiling is zero', () => {
    assert.equal(ceilings('data/meta/js-parity-ratchet.lino').get('native-only-ceiling'), 0);
  });

  test('the registry measures no native-only row', () => {
    assert.match(gate('check-js-parity.mjs'), /native-only handler rows: 0 \(ceiling 0\)/);
  });
});

describe('R1005/R1010: wording is seed data, not code', () => {
  const held = ceilings('data/meta/js-literal-ratchet.lino');

  test('the meta reasoner, the server and the agentic planner are held at zero', () => {
    for (const key of ['meta-ceiling', 'server-ceiling', 'agentic-ceiling']) assert.equal(held.get(key), 0, key);
  });

  test('the worker has a finite ceiling the gate measures against', () => {
    assert.ok(Number.isInteger(held.get('worker-ceiling')) && held.get('worker-ceiling') > 0);
  });

  test('each zero-ceiling surface measures zero literals', () => {
    const report = gate('check-js-literals.mjs');
    for (const surface of ['meta', 'server', 'agentic']) {
      assert.match(report, new RegExp(`natural-language literals \\(${surface}\\): 0 \\(ceiling 0\\)`));
    }
  });
});
