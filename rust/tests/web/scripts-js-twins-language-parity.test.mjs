// PR #1188 (LEXEMES): `scripts/check-language-parity.mjs` is the JavaScript
// twin of `scripts/check-language-parity.rs`. These cases are the Rust
// script's own unit tests (scripts/language-parity-lib.rs `mod tests`) plus the
// command-line surface: each pins the exact text the Rust source formats, so a
// twin that drifts fails here before CI diffs both halves on the real tree
// (data/meta/ci-gates/check-language-parity-js-twin.lino).

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { checkDebt, renderDebt, validDate } from '../../../scripts/check-language-parity.mjs';
import { gapsFromDocuments } from '../../../scripts/lib/checks-language-parity.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

const CATALOG = [
  'catalog',
  '  asset ETH',
  '    lexeme en',
  '      surface',
  '        text ether',
  '    lexeme es',
  '      surface',
  '        text éter',
  '',
].join('\n');

function sampleGap() {
  return gapsFromDocuments([['data/seed/catalog.lino', CATALOG]])[0];
}

/** A temporary repository holding `files` (path -> text). */
function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), 'scripts-js-twins-language-parity-'));
  for (const [path, text] of Object.entries(files)) {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), text);
  }
  return root;
}

function run(args) {
  const result = spawnSync(process.execPath, [join(ROOT, 'scripts/check-language-parity.mjs'), ...args], {
    encoding: 'utf8',
  });
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

describe('the structural census', () => {
  test('derives lexeme owners from structure, not from the file name or root name', () => {
    const gap = sampleGap();
    assert.equal(gap.source, 'data/seed/catalog.lino');
    assert.equal(gap.meaning, 'asset ETH');
    assert.deepEqual(gap.present, ['en', 'es']);
    assert.deepEqual(gap.missing, ['ru', 'hi', 'zh']);
    assert.equal(gap.ownerPath, '7:catalog9:asset ETH');
  });

  test('a complete owner is no debt and a non-lexeme language row does not count', () => {
    const text = 'registry\n  language es\n    status partial\n  idea\n'
      + '    lexeme en\n    lexeme ru\n    lexeme hi\n    lexeme zh\n    lexeme es\n';
    assert.deepEqual(gapsFromDocuments([['data/seed/not-a-meanings-file.lino', text]]), []);
  });

  test('a duplicate language under one owner is rejected', () => {
    assert.throws(
      () => gapsFromDocuments([['data/seed/duplicate.lino', 'root\n  idea\n    lexeme en\n    lexeme en\n']]),
      /duplicate `lexeme en`/,
    );
  });
});

describe('the dated debt ledger', () => {
  test('an exact generated ledger is accepted', () => {
    const gaps = [sampleGap()];
    assert.deepEqual(checkDebt(gaps, renderDebt(gaps, '2026-09-17')), []);
  });

  test('a missing debt row is rejected', () => {
    const failures = checkDebt([sampleGap()], renderDebt([], '2026-09-17'));
    assert.deepEqual(failures, [
      'missing debt row for data/seed/catalog.lino meaning `asset ETH` (missing ru,hi,zh)',
    ]);
  });

  test('a duplicate debt row is rejected', () => {
    const gaps = [sampleGap()];
    const one = renderDebt(gaps, '2026-09-17');
    const row = one.split('\n').find((line) => line.startsWith('  uncovered_behavior '));
    const failures = checkDebt(gaps, `${one}${row}\n`);
    assert.deepEqual(failures, [
      'duplicate debt row for data/seed/catalog.lino at structural owner 7:catalog9:asset ETH',
    ]);
  });

  test('a debt row goes stale once its owner is complete', () => {
    const failures = checkDebt([], renderDebt([sampleGap()], '2026-09-17'));
    assert.deepEqual(failures, [
      'stale debt row for data/seed/catalog.lino meaning `asset ETH`; the structural owner is complete or absent',
    ]);
  });

  test('an invalid calendar date is rejected', () => {
    assert.deepEqual(checkDebt([sampleGap()], renderDebt([sampleGap()], '2026-02-30')), [
      'invalid observed_on date `2026-02-30`',
    ]);
    assert.equal(validDate('2028-02-29'), true);
    assert.equal(validDate('2100-02-29'), false);
  });
});

describe('the command line', () => {
  test('--write regenerates the ledger, the check then passes, and --count prints the measure', () => {
    const root = fixture({
      'data/seed/catalog.lino': CATALOG,
      'data/meta/language-parity-debt.lino': renderDebt([], '2026-09-17'),
    });
    try {
      const stale = run(['--repo', root]);
      assert.equal(stale.status, 1);
      assert.equal(stale.stdout, '');
      assert.equal(
        stale.stderr,
        'language parity check failed:\n'
          + 'missing debt row for data/seed/catalog.lino meaning `asset ETH` (missing ru,hi,zh)\n',
      );

      const written = run(['--repo', root, '--write', '--date', '2026-10-08']);
      assert.equal(written.stdout, 'recorded 1 language-parity gaps in data/meta/language-parity-debt.lino\n');
      assert.equal(
        readFileSync(join(root, 'data/meta/language-parity-debt.lino'), 'utf8'),
        renderDebt([sampleGap()], '2026-10-08'),
      );

      const exact = run(['--repo', root]);
      assert.deepEqual(exact, {
        status: 0,
        stdout: 'language parity: 1 explicit dated gaps; debt rows are exact\n',
        stderr: '',
      });
      assert.equal(run(['--repo', root, '--count']).stdout, '1\n');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  test('argument errors print what the Rust original prints', () => {
    assert.deepEqual(run(['--bogus']), {
      status: 1,
      stdout: '',
      stderr: 'language parity check failed:\nunknown argument: --bogus\n',
    });
    assert.equal(run(['--write']).stderr, 'language parity check failed:\n--write requires --date YYYY-MM-DD\n');
    assert.equal(
      run(['--count', '--write']).stderr,
      'language parity check failed:\n--count and --write are mutually exclusive\n',
    );
  });
});
