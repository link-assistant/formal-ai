// PR #1188 (SCRIPTS-A): the requirement pipeline's JavaScript twins.
//
// `scripts/assemble-requirements.mjs`, `scripts/generate-requirement-status.mjs`,
// `scripts/render-status.mjs` and `scripts/check-requirement-status.mjs` replace
// their `rust-script` originals locally, so nobody compiles Rust to regenerate
// the register. They must print the same lines, exit with the same codes and
// write the same bytes. This suite pins:
//
// - the Rust originals' own unit tests (`#[cfg(test)]` in
//   `scripts/assemble-requirements.rs`), ported to the twin's functions;
// - the Rust standard-library semantics the twins reproduce (`str::lines`,
//   `str::trim`, `PathBuf` ordering, `{:?}` lists);
// - every success and failure line each CLI prints, on a synthetic tree, with
//   the exact text the Rust source formats;
// - the committed bytes: on a scratch copy of a commit whose Rust gates passed
//   in CI, every twin's check passes and every generated file, rebuilt from
//   nothing, equals the committed one (skipped when the commit is not fetched).
//
// CI runs the Rust originals beside the twins and compares them
// (`data/meta/ci-gates/check-*-js-twin.lino`).

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import * as assemble from '../../../scripts/assemble-requirements.mjs';
import { quoted, verdict, containsWord } from '../../../scripts/generate-requirement-status.mjs';
import { unquote } from '../../../scripts/check-requirement-status.mjs';
import { replaceRegion } from '../../../scripts/render-status.mjs';
import { comparePaths, debugList, lines, trim } from '../../../scripts/lib/requirements-rust-compat.mjs';
import { compareRuns } from '../../../scripts/lib/requirements-twin-parity.mjs';
import { headOverlay, PIPELINE_COMMITTED } from '../../../scripts/lib/requirements-head-overlay.mjs';

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const script = (name) => join(REPO, 'scripts', `${name}.mjs`);

function run(cwd, name, ...args) {
  const result = spawnSync(process.execPath, [script(name), ...args], { cwd, encoding: 'utf8' });
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

function put(root, path, text) {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), text);
}

describe('the Rust standard-library semantics the twins reproduce', () => {
  test('str::lines drops one final newline and a \\r only before \\n', () => {
    assert.deepEqual(lines(''), []);
    assert.deepEqual(lines('\n'), ['']);
    assert.deepEqual(lines('a\r\nb\r'), ['a', 'b\r']);
    assert.deepEqual(lines('a\n\n'), ['a', '']);
  });

  test('str::trim is Unicode White_Space, not JavaScript \\s', () => {
    assert.equal(trim('\u0085 x　'), 'x');
    assert.equal(trim('﻿x'), '﻿x');
  });

  test('a PathBuf orders by component, so a directory sorts before its .lino sibling', () => {
    assert.ok(comparePaths('data/meta/requirement-status-ledger/requirements-01.lino', 'data/meta/requirement-status-ledger.lino') < 0);
    assert.ok(comparePaths('/r/README.md', '/r/docs/status.md') < 0);
  });

  test('{:?} of a string list quotes and escapes like Rust', () => {
    assert.equal(debugList(['a', 'b"c\\d\n']), '["a", "b\\"c\\\\d\\n"]');
  });
});

describe('assemble-requirements.rs unit tests, ported to the twin', () => {
  test('the preamble sorts first, then issues by number, then doctrine', () => {
    const names = ['doctrine-compiled-logic.md', 'issue-0991-how-to-synthesis.md', 'preamble-requirements-for-issue-1.md',
      'issue-0016-multilingual.md', 'issue-0016-follow-up.md'];
    const root = mkdtempSync(join(tmpdir(), 'assemble-order-'));
    for (const name of names) writeFileSync(join(root, name), '');
    assert.deepEqual(assemble.orderedShards(root).map((path) => path.slice(root.length + 1)), [
      'preamble-requirements-for-issue-1.md', 'issue-0016-follow-up.md', 'issue-0016-multilingual.md',
      'issue-0991-how-to-synthesis.md', 'doctrine-compiled-logic.md']);
    rmSync(root, { recursive: true, force: true });
  });

  test('an unprefixed shard name is rejected and a heading names its first issue', () => {
    assert.equal(assemble.shardOrder('current-scope-boundary.md'), null);
    assert.equal(assemble.shardOrder('issue-0991-notes.txt'), null);
    assert.equal(assemble.headingIssue('## Issue #16 Follow-Up: Universal Data/Seed (PR #17 reopen)'), 16);
    assert.equal(assemble.headingIssue('## Current Scope Boundary'), null);
  });

  test('a shard name is derived from its heading', () => {
    assert.equal(assemble.shardName('## Issue #991 Dynamic Multi-Source How-To Synthesis'), 'issue-0991-dynamic-multi-source-how-to-synthesis.md');
    assert.equal(assemble.shardName('## Standing Doctrine: Compiled Logic (2026-08-04)'), 'doctrine-standing-doctrine-compiled-logic-2026-08-04.md');
  });

  test('splitting keeps the preamble apart from the sections', () => {
    assert.deepEqual(assemble.sections('# Title\n\nIntro.\n\n## One\n\na\n\n## Two\n\nb\n'),
      ['# Title\n\nIntro.\n\n', ['## One\n\na\n\n', '## Two\n\nb\n']]);
  });

  test('a part header is exactly its counted lines; sections pack in order', () => {
    assert.equal(lines(assemble.partHeader(2, 3)).length, assemble.PART_HEADER_LINES);
    const half = assemble.PART_LINE_LIMIT / 2;
    assert.deepEqual(assemble.pack([['a', 10], ['b', half], ['c', half], ['d', 5]]), [[0, 1], [2, 3]]);
    assert.throws(() => assemble.pack([['issue-0001-huge.md', assemble.PART_LINE_LIMIT]]), /issue-0001-huge\.md/);
  });

  test('links rebase into the parts directory, to the root and back', () => {
    for (const [shard, part] of [['../upload-memory.md', '../../upload-memory.md'], ['../../scripts/close-total.py', '../../../scripts/close-total.py'],
      ['issue-0709-fusion.md', '../issue-0709-fusion.md'], ['https://example.org/a.md', 'https://example.org/a.md'], ['#anchor', '#anchor']]) {
      assert.equal(assemble.shardToPartRelative(shard), part);
    }
    for (const [shard, root] of [['../upload-memory.md', 'docs/upload-memory.md'], ['../../scripts/close-total.py', 'scripts/close-total.py'],
      ['issue-0709-fusion.md', 'docs/requirements/issue-0709-fusion.md']]) {
      assert.equal(assemble.toRootRelative(shard), root);
      assert.equal(assemble.toShardRelative(root), shard);
    }
    assert.equal(assemble.toShardRelative(assemble.SHARDS), '.');
    for (const target of ['https://example.org/a.md', '#an-anchor', '/docs/absolute.md', 'mailto:someone@example.org']) {
      assert.equal(assemble.toRootRelative(target), target);
    }
  });

  test('rewriting touches the target and nothing else, and keeps a title', () => {
    const shard = 'See [`docs/upload-memory.md`](../upload-memory.md) and [the spec](https://example.org) (a note) — cost: $1 (approx).\n';
    const root = 'See [`docs/upload-memory.md`](docs/upload-memory.md) and [the spec](https://example.org) (a note) — cost: $1 (approx).\n';
    assert.equal(assemble.rewriteLinks(shard, assemble.toRootRelative), root);
    assert.equal(assemble.rewriteLinks(root, assemble.toShardRelative), shard);
    assert.equal(assemble.rewriteLinks('[a](../b.md "Title")', assemble.toRootRelative), '[a](docs/b.md "Title")');
  });

  test('shard links resolve from the shard, and a shard is named for its own issue', () => {
    const shards = ['docs/requirements/issue-0018-memory.md'];
    const broken = assemble.linkViolations(shards, () => '## Issue #18\n\nSee [it](docs/upload-memory.md).\n', (path) => path === 'docs/upload-memory.md');
    assert.equal(broken.length, 1);
    assert.match(broken[0], /docs\/upload-memory\.md/);
    assert.deepEqual(assemble.linkViolations(shards, () => 'See [it](../upload-memory.md#top).\n', (path) => path === 'docs/requirements/../upload-memory.md'), []);
    assert.ok(assemble.shardViolations(['docs/requirements/issue-0991-copy.md'], () => '## Issue #709 Multi-Source Search Fusion\n')[0].includes('issue-0709-'));
    assert.deepEqual(assemble.shardViolations(['docs/requirements/issue-0709-fusion.md'], () => '## Issue #709 Multi-Source Search Fusion\n'), []);
  });
});

describe('generate-requirement-status and check-requirement-status helpers', () => {
  test('a value with a double quote is single-quoted with \\x27 apostrophes', () => {
    assert.equal(quoted('say "it\'s"'), "'say \"it\\x27s\"'");
    assert.equal(quoted('a\\b'), '"a\\\\b"');
    assert.equal(unquote('"a""b\\\\c\\n"'), 'a"b\\c\\n');
  });

  test('a verdict reads words, not substrings, from the status cells', () => {
    assert.equal(containsWord('superseded_read_only_work', 'superseded'), false);
    assert.equal(verdict('| R1 | the planned search | Implemented. |', 'rust/tests/web/x.test.mjs'), 'implemented');
    assert.equal(verdict('| R1 | text | Implemented. |', ''), 'partial');
    assert.equal(verdict('| R1 | text | Pending. |', 'x'), 'not-delivered');
    assert.equal(verdict('- R1 withdrawn', ''), 'withdrawn');
  });
});

describe('render-status regions', () => {
  test('a region is replaced in place, appended when absent, and an unmatched marker fails', () => {
    assert.equal(replaceRegion('a <!-- status:begin x -->old<!-- status:end x --> b', 'x', 'new'), 'a <!-- status:begin x -->\nnew\n<!-- status:end x --> b');
    assert.equal(replaceRegion('a\n\n', 'x', 'new'), 'a\n\n<!-- status:begin x -->\nnew\n<!-- status:end x -->\n');
    assert.throws(() => replaceRegion('<!-- status:begin x -->', 'x', ''), /x: unmatched status region marker/);
  });
});

describe('the parity comparer', () => {
  test('equal runs agree; a different exit code or line is reported', () => {
    const run = { status: 0, stdout: 'a\nb\n', stderr: '' };
    assert.deepEqual(compareRuns(run, { ...run }), []);
    assert.deepEqual(compareRuns(run, { ...run, status: 1, stdout: 'a\nc\n' }),
      ['exit code: rust 0 / js 1', 'stdout line 2: rust "b" / js "c"']);
  });
});

function syntheticTree() {
  const root = mkdtempSync(join(tmpdir(), 'scripts-js-twins-'));
  put(root, 'docs/requirements/README.md', '# Shards\n');
  put(root, 'docs/requirements/preamble-requirements.md', '# Requirements\n\nSee [traceability](../requirements-traceability.md).\n');
  put(root, 'docs/requirements/issue-0001-alpha.md', '## Issue #1 Alpha\n\n| ID | Requirement | Status |\n| --- | --- | --- |\n'
    + '| R1-1 | Alpha works. | Implemented: pinned by `rust/tests/web/alpha.test.mjs`. |\n'
    + '| R1-2 | Beta "quoted". | Planned. |\n');
  put(root, 'docs/requirements/doctrine-zeta.md', '## Standing Doctrine: Zeta\n\n- R9 is superseded by R1-1.\n');
  put(root, 'docs/requirements-traceability.md', '| R1-1 | Alpha | 2026-10-08 | rust/tests/web/alpha.test.mjs | confirmed by hand |\n');
  put(root, 'rust/tests/web/alpha.test.mjs', '');
  put(root, 'data/seed/languages.lino', 'languages\n  language "en"\n    name "English"\n    status "complete"\n    uncovered_behavior "none"\n');
  put(root, 'data/benchmarks/external-results.lino', 'result\n  record_type external_benchmark_result\n  suite "s"\n  date "2026-10-01"\n  slice "2"\n  passed 1\n  total 2\n'
    + 'result\n  record_type external_benchmark_result\n  suite "s"\n  date "2026-10-01"\n  slice "10"\n  passed 3\n  total 4\n');
  put(root, 'data/meta/self-hosting-ledger.lino', 'self_hosting\n  release\n    tag "v1"\n    percentage_basis_points 10\n');
  for (const name of ['debt-ratchet', 'core-boundary-ledger', 'handler-migration-ledger', 'ladder-ratchet']) put(root, `data/meta/${name}.lino`, `${name}\n`);
  put(root, 'data/meta/worker-line-budget/a.lino', 'a\n');
  put(root, 'docs/benchmarks.md', '# Benchmarks\n\n<!-- status:begin benchmarks -->\nold\n<!-- status:end benchmarks -->\n');
  put(root, 'README.md', '# Readme\n');
  return root;
}

describe('the twins on a synthetic tree print what the Rust source formats', () => {
  const root = syntheticTree();
  const read = (path) => readFileSync(join(root, path), 'utf8');

  test('assemble-requirements: drift, write, current, misnamed shard, broken link, stale part', () => {
    let result = run(root, 'assemble-requirements');
    assert.equal(result.status, 1);
    assert.equal(result.stdout, '\nChecking REQUIREMENTS.md and docs/requirements/assembled/ against docs/requirements/ (3 shards)...\n\n'
      + '::error::REQUIREMENTS.md does not match docs/requirements/.\n'
      + '::error::docs/requirements/assembled/part-01.md does not match docs/requirements/.\n'
      + '\nRun: rust-script scripts/assemble-requirements.rs --write\n\n');
    result = run(root, 'assemble-requirements', '--write');
    assert.equal(result.stdout, '\nRebuilt REQUIREMENTS.md and 1 part(s) in docs/requirements/assembled/ from 3 shards.\n\n');
    assert.ok(read('docs/requirements/assembled/part-01.md').includes('[traceability](../../requirements-traceability.md)'));
    assert.ok(read('REQUIREMENTS.md').includes('1. [Part 1](docs/requirements/assembled/part-01.md)\n   - Requirements\n   - Issue #1 Alpha\n   - Standing Doctrine: Zeta\n'));
    result = run(root, 'assemble-requirements');
    assert.deepEqual([result.status, result.stdout.split('\n')[3]], [0, 'REQUIREMENTS.md and its 1 part(s) match the shards.']);
    assert.equal(run(root, 'assemble-requirements', '--write').stdout, '\nREQUIREMENTS.md and 1 part(s) are already current (3 shards).\n\n');

    put(root, 'docs/requirements/assembled/part-02.md', 'stale\n');
    assert.ok(run(root, 'assemble-requirements').stdout.includes('::error::docs/requirements/assembled/part-02.md (stale) does not match docs/requirements/.\n'));
    rmSync(join(root, 'docs/requirements/assembled/part-02.md'));

    put(root, 'docs/requirements/issue-0002-copy.md', '## Issue #3 Copy\n\nSee [x](missing.md).\n');
    result = run(root, 'assemble-requirements');
    assert.equal(result.status, 1);
    assert.equal(result.stdout, '::error::docs/requirements/issue-0002-copy.md starts with the heading `## Issue #3 Copy`, so it must be named `issue-0003-<slug>.md`\n'
      + '::error::docs/requirements/issue-0002-copy.md links to `missing.md`, which does not exist relative to docs/requirements/. '
      + 'Write shard links relative to the shard; assembly rebases them to `docs/requirements/missing.md`.\n');
    rmSync(join(root, 'docs/requirements/issue-0002-copy.md'));
    put(root, 'docs/requirements/notes.md', '');
    assert.equal(run(root, 'assemble-requirements').stdout, '::error::docs/requirements/notes.md must start with `preamble-`, `issue-NNNN-` or `doctrine-`\n');
    rmSync(join(root, 'docs/requirements/notes.md'));
  });

  test('generate-requirement-status: stale list in PathBuf order, write, obsolete shard removed', () => {
    let result = run(root, 'generate-requirement-status');
    assert.equal(result.status, 1);
    assert.equal(result.stderr, 'stale generated requirement-status files: ["data/meta/requirement-status-ledger/requirements-01.lino", "data/meta/requirement-status-ledger.lino"]\n'
      + 'run rust-script scripts/generate-requirement-status.rs --write\n');
    put(root, 'data/meta/requirement-status-ledger/requirements-07.lino', 'obsolete\n');
    result = run(root, 'generate-requirement-status', '--write');
    assert.deepEqual([result.status, result.stdout], [0, 'generated 2 requirement-status files\n']);
    assert.deepEqual(readdirSync(join(root, 'data/meta/requirement-status-ledger')), ['requirements-01.lino']);
    const shard = read('data/meta/requirement-status-ledger/requirements-01.lino');
    assert.ok(shard.includes('    id "R1-1"\n    shard "docs/requirements/issue-0001-alpha.md"\n    verdict "implemented"\n    delivered "2026-10-08"\n    issue "0001"\n    automated_test "rust/tests/web/alpha.test.mjs"\n    manual "confirmed by hand"\n'));
    assert.ok(shard.includes('    id "R1-2"\n    shard "docs/requirements/issue-0001-alpha.md"\n    verdict "not-delivered"\n'));
    assert.ok(shard.includes('    id "R9"\n    shard "docs/requirements/doctrine-zeta.md"\n    verdict "superseded"\n    delivered ""\n    issue ""\n    automated_test ""\n    manual "not yet confirmed"\n'));
    assert.ok(read('data/meta/requirement-status-ledger.lino').includes('  requirement_count 3\n  implemented_count 1\n  shard "data/meta/requirement-status-ledger/requirements-01.lino"\n'));
    assert.deepEqual(run(root, 'generate-requirement-status').stdout, 'requirement-status ledger is current (2 files)\n');
  });

  test('check-requirement-status: parity, then every failure line in Rust order', () => {
    assert.deepEqual(run(root, 'check-requirement-status').stdout, 'requirement-status parity holds for 3 assembled requirements\n');
    const path = 'data/meta/requirement-status-ledger/requirements-01.lino';
    const original = read(path);
    put(root, path, original.replace('verdict "not-delivered"', 'verdict "done"')
      .replace('automated_test "rust/tests/web/alpha.test.mjs"', 'automated_test "rust/tests/web/gone.test.mjs"')
      + '  requirement\n    id "R9"\n    shard "docs/requirements/issue-0001-alpha.md"\n    verdict "implemented"\n'
      + '  requirement\n    id "R77"\n    shard "docs/requirements/missing.md"\n    verdict "partial"\n');
    const result = run(root, 'check-requirement-status');
    assert.equal(result.status, 1);
    assert.equal(result.stderr, [
      'the status ledger contains a duplicate requirement id',
      'R77: absent from the assembled REQUIREMENTS.md parts',
      'R1-1: automated test `rust/tests/web/gone.test.mjs` does not exist',
      'R1-2: unknown verdict `done`',
      'R77: owning shard `docs/requirements/missing.md` does not exist',
      'R9: owning shard `docs/requirements/issue-0001-alpha.md` does not name it',
      'R9: implemented without an automated test',
    ].map((line) => `requirement-status: ${line}\n`).join(''));
    put(root, path, original);
  });

  test('render-status: unknown mode, stale surfaces in PathBuf order, write, latest slice wins', () => {
    assert.deepEqual(run(root, 'render-status', '--bogus'), { status: 2, stdout: '', stderr: 'render-status: unknown mode --bogus; expected --write or --check\n' });
    let result = run(root, 'render-status');
    assert.equal(result.status, 1);
    assert.equal(result.stderr, 'stale status surfaces: ["README.md", "docs/benchmarks.md", "docs/status.md"]\nrun rust-script scripts/render-status.rs --write\n');
    assert.equal(run(root, 'render-status', '--write').stdout, 'rendered 3 status surfaces\n');
    assert.equal(run(root, 'render-status', '--check').stdout, 'status surfaces are current\n');
    const status = read('docs/status.md');
    assert.ok(status.includes('| `s` | 2026-10-01 | 10 | 3 | 4 |  |\n'), status);
    assert.ok(status.includes('| `implemented` | 1 |\n| `not-delivered` | 1 |\n| `superseded` | 1 |\n'));
    assert.ok(status.includes('| `data/meta/worker-line-budget/*.lino` | 1 files |\n'));
    assert.ok(read('README.md').endsWith('\n\n<!-- status:begin self-hosting -->\nLatest ledger row: `v1`; release share `10` basis points, trailing share `` basis points, target `` basis points.\n<!-- status:end self-hosting -->\n'));
    put(root, 'docs/benchmarks.md', '<!-- status:end benchmarks -->\n');
    assert.deepEqual(run(root, 'render-status'), { status: 1, stdout: '', stderr: 'render-status: benchmarks: unmatched status region marker\n' });
    rmSync(root, { recursive: true, force: true });
  });
});

// A commit whose four Rust requirement gates passed in CI (run 37738783201):
// the twins must accept its tree and rebuild its outputs byte for byte.
const PASSING_REVISION = process.env.SCRIPTS_JS_TWINS_REVISION ?? 'a75bf3772';
const revisionAvailable = spawnSync('git', ['-C', REPO, 'cat-file', '-e', `${PASSING_REVISION}^{commit}`]).status === 0;

describe('the committed bytes', { skip: revisionAvailable ? false : `commit ${PASSING_REVISION} is not fetched` }, () => {
  test('every twin accepts the commit and rebuilds every output from nothing', () => {
    const root = mkdtempSync(join(tmpdir(), 'scripts-js-twins-commit-'));
    try {
      headOverlay(REPO, root, PIPELINE_COMMITTED, undefined, PASSING_REVISION);
      assert.equal(run(root, 'assemble-requirements').status, 0);
      assert.match(run(root, 'generate-requirement-status').stdout, /^requirement-status ledger is current \(\d+ files\)\n$/);
      assert.match(run(root, 'check-requirement-status').stdout, /^requirement-status parity holds for \d+ assembled requirements\n$/);
      assert.equal(run(root, 'render-status', '--check').stdout, 'status surfaces are current\n');

      const outputs = ['REQUIREMENTS.md', 'docs/status.md', 'data/meta/requirement-status-ledger.lino',
        ...readdirSync(join(root, 'docs/requirements/assembled')).map((name) => `docs/requirements/assembled/${name}`),
        ...readdirSync(join(root, 'data/meta/requirement-status-ledger')).map((name) => `data/meta/requirement-status-ledger/${name}`)];
      const committed = new Map(outputs.map((path) => [path, readFileSync(join(root, path))]));
      for (const path of outputs) rmSync(join(root, path));
      assert.equal(run(root, 'assemble-requirements', '--write').status, 0);
      assert.equal(run(root, 'generate-requirement-status', '--write').status, 0);
      assert.equal(run(root, 'render-status', '--write').status, 0);
      for (const [path, bytes] of committed) {
        assert.ok(existsSync(join(root, path)), `${path} rebuilt`);
        assert.ok(readFileSync(join(root, path)).equals(bytes), `${path} equals ${PASSING_REVISION}`);
      }
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
