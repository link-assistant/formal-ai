// Issue #1191 requirement 8: the single push gate is written down for people
// (CONTRIBUTING.md) and for agents (AGENTS.md, with CLAUDE.md pointing to it).
// The bounded local Rust build it prescribes really prunes a shared
// CARGO_TARGET_DIR.

import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const read = relative => readFileSync(join(ROOT, relative), 'utf8');

const RULES = [
  [/drafting sub-agents never push/iu, 'drafting sub-agents never push'],
  [/one CI\/CD agent pushes/iu, 'one CI/CD agent pushes'],
  [/every (actionable )?failure of the latest run/iu, 'after collecting every failure of the latest run'],
  [/never push while a run is in progress/iu, 'never push while a run is in progress'],
  [/run the JavaScript checks locally before the push/iu, 'JavaScript checks before the push'],
  [/no full local Rust build unless the change is Rust-only/iu, 'no full local Rust build'],
  [/CARGO_TARGET_DIR/u, 'a shared CARGO_TARGET_DIR'],
  [/CARGO_INCREMENTAL=0/u, 'CARGO_INCREMENTAL=0'],
  [/CARGO_(BUILD|TEST)_JOBS=2|limited `-j`/u, 'a limited -j'],
  [/scripts\/prune-build-cache\.sh/u, 'pruning old artifacts'],
];

for (const document of ['AGENTS.md', 'CONTRIBUTING.md', 'CLAUDE.md']) {
  test(`${document} states every push-gate rule`, () => {
    const text = read(document).replace(/\s+/gu, ' ');
    for (const [pattern, rule] of RULES) assert.match(text, pattern, `${document}: ${rule}`);
  });
}

test('CLAUDE.md defers to AGENTS.md and the agent file to CONTRIBUTING.md', () => {
  assert.match(read('CLAUDE.md'), /\[AGENTS\.md\]\(AGENTS\.md\)/u);
  assert.match(read('AGENTS.md'), /\[CONTRIBUTING\.md\]\(CONTRIBUTING\.md\)/u);
  assert.match(read('CONTRIBUTING.md'), /## The single push gate \(issue #1191\)/u);
});

test('the commands the push gate names exist', () => {
  for (const path of [
    'experiments/formal_ai_subagent/local-gates.mjs',
    'scripts/cargo-test.sh',
    'scripts/prune-build-cache.sh',
    'data/meta/main-only-regressions.lino',
    '.github/workflows/native-type-check.yml',
  ]) assert.ok(existsSync(join(ROOT, path)), path);
});

test('prune-build-cache.sh prunes the shared CARGO_TARGET_DIR, not rust/target', { skip: process.platform === 'win32' }, () => {
  const target = mkdtempSync(join(tmpdir(), 'formal-ai-shared-target-'));
  try {
    mkdirSync(join(target, 'debug', 'examples'), { recursive: true });
    writeFileSync(join(target, 'debug', 'examples', 'linked-example'), 'x'.repeat(4096));
    const output = execFileSync('bash', ['scripts/prune-build-cache.sh'], {
      cwd: ROOT,
      encoding: 'utf8',
      timeout: 60_000,
      env: { ...process.env, CARGO_TARGET_DIR: target, CARGO_TEST_NO_PRUNE: '', CARGO_TARGET_MAX_SIZE_MB: '0', PATH: '/usr/bin:/bin' },
    });
    assert.match(output, /prune-build-cache:/u);
    assert.ok(!existsSync(join(target, 'debug', 'examples')), 'linked example binaries are removed from the shared directory');
  } finally {
    rmSync(target, { recursive: true, force: true });
  }
});
