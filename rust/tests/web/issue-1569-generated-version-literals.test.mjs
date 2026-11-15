// Issue #1569: PR #1498 added GitHub API job records under
// data/benchmarks/ci-phase-timing/baseline-runs/ whose step names quote
// `coursier/setup-action@<sha>`. check-generated-version-literals is a
// rust-script gate run only in the main lint job, so the PR passed and main
// failed. The gate now treats those captured records as evidence, and its
// JavaScript twin runs here, in the fast pull-request profile.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';

import { boundedCommand } from '../../../js/server/bounded-command.mjs';
import { CAPTURED_EVIDENCE, literalActionSha, literalToolchainVersion, scanData }
  from '../../../scripts/check-generated-version-literals.mjs';

const SHA = '648df969f41ef15fda2baba8b37f9fa3d16390a3';

function tree(t, files) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-version-literals-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const [name, body] of Object.entries(files)) {
    const file = path.join(root, 'data', name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, body);
  }
  return root;
}

test('the twin carries the same captured-evidence list as the Rust gate', () => {
  const rust = fs.readFileSync('scripts/check-generated-version-literals.rs', 'utf8');
  const list = rust.match(/const CAPTURED_EVIDENCE: \[&str; (\d+)\] = \[([^\]]*)\];/u);
  assert.ok(list, 'CAPTURED_EVIDENCE in the Rust gate');
  const entries = [...list[2].matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
  assert.equal(Number(list[1]), entries.length);
  assert.deepEqual(CAPTURED_EVIDENCE, entries);
});

test('pins and toolchain versions follow the Rust byte rules', () => {
  assert.equal(literalActionSha(`      - uses: actions/checkout@${SHA}`), 30);
  assert.equal(literalActionSha('      - uses: actions/checkout@v7'), null);
  assert.equal(literalActionSha(`sha256@${SHA}aa`), null);
  assert.equal(literalActionSha(`é@${SHA}`), 2);
  assert.equal(literalToolchainVersion("        java-version: '21'"), 'java-version');
  assert.equal(literalToolchainVersion('  version: {java}'), null);
  assert.equal(literalToolchainVersion('  latest_tag "v7.0.1"'), null);
});

test('captured CI run records are evidence, the same text elsewhere is a finding', (t) => {
  const job = `{"name":"Run coursier/setup-action@${SHA}"}\n`;
  const root = tree(t, {
    'benchmarks/ci-phase-timing/baseline-runs/40475832633-jobs.json': job,
    'benchmarks/issue-requirements/issue-1168.md': "        java-version: '21'\n",
    'cache/page.lino': job,
    'benchmarks/baseline-runs/jobs.json': job,
  });
  const findings = scanData(path.join(root, 'data'));
  assert.deepEqual(findings.map((finding) => `${finding.path}:${finding.line}`), ['data/benchmarks/baseline-runs/jobs.json:1']);
});

test('the repository data/ scans clean through the command-line twin', async () => {
  const result = await boundedCommand(process.execPath, ['scripts/check-generated-version-literals.mjs'], {
    timeout: 60000, maxBuffer: 4 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, String(result.stderr));
  assert.equal(result.status, 0, String(result.stdout));
  assert.match(String(result.stdout), /generated-version literals: clean/u);
});
