import { strict as assert } from 'node:assert';
import { spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const script = join(root, 'scripts/check-changelog-fragment.mjs');
const fixture = resolve(dirname(fileURLToPath(import.meta.url)), '../fixtures/changelog-fragment/git-output.mjs');
const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;

function observe(mode, installGit = true) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-changelog-output-'));
  try {
    if (installGit) {
      const fixturePath = join(directory, 'fixture.mjs');
      copyFileSync(fixture, fixturePath);
      const git = join(directory, 'git');
      writeFileSync(git, `#!/bin/sh\nexec ${quote(process.execPath)} ${quote(fixturePath)} "$@"\n`);
      chmodSync(git, 0o755);
    }
    return spawnSync(process.execPath, [script], {
      cwd: directory,
      env: { ...process.env, PATH: directory, RUST_ROOT: 'rust', GITHUB_BASE_REF: 'main', CHANGELOG_GIT_FIXTURE_MODE: mode },
      encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test('a Git diff larger than one MiB retains every path and its source/fragment decision', () => {
  const result = observe('large');
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  assert.ok(Buffer.byteLength(result.stdout) > 1024 * 1024);
  for (let index = 0; index < 11000; index += 1) {
    assert.ok(result.stdout.includes(`  data/evidence/${String(index).padStart(5, '0')}/${'x'.repeat(100)}.lino\n`));
  }
  assert.match(result.stdout, /Source files changed: 1\n  rust\/src\/fixture\.rs\n/);
  assert.match(result.stdout, /Changelog fragments added: 1\n  changelog\.d\/fixture\.md\n/);
  assert.match(result.stdout, /Changelog check passed \(source files changed: 1, fragments added: 1\)/);
  assert.doesNotMatch(result.stdout, /No changed files found/);
});

test('a failed Git command fails closed rather than claiming an empty diff', () => {
  const result = observe('failed');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Error executing git/);
  assert.match(result.stderr, /fixture git failure/);
  assert.doesNotMatch(result.stdout, /No changed files found|Changelog check passed/);
});

test('Git output beyond the explicit bound fails closed', () => {
  const result = observe('overflow');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /output exceeded 16777216 bytes/);
  assert.doesNotMatch(result.stdout, /No changed files found|Changelog check passed/);
});

test('an unavailable Git executable fails closed', () => {
  const result = observe('empty', false);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Failed to execute git/);
  assert.doesNotMatch(result.stdout, /No changed files found|Changelog check passed/);
});

test('a genuinely empty successful Git diff remains successful', () => {
  const result = observe('empty');
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /No changed files found/);
});
