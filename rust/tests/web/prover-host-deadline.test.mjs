import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, existsSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { runProverWith, createProverHost } from '../../../js/server/prover-host.mjs';

test('a nonterminating prover is killed and never reported successful', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-prover-deadline-'));
  const started = performance.now();
  const marker = join(directory, 'prover-started');
  try {
    const result = runProverWith(process.execPath,
      `import fs from 'node:fs'; fs.writeFileSync(${JSON.stringify(marker)}, 'started'); process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);`, directory, 'mjs', 1000);
    assert.equal(result.exit, null);
    assert.ok(performance.now() - started < 4000, 'prover must return within its finite deadline');
    assert.equal(readFileSync(marker, 'utf8'), 'started', 'prover must actually start before the deadline');
    assert.ok(existsSync(result.sourcePath));
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('bounded prover execution preserves successful and failed exit statuses', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-prover-exit-'));
  try {
    for (const code of [0, 7]) {
      const result = runProverWith(process.execPath, `process.exit(${code});`, directory, 'mjs', 2000);
      assert.equal(result.exit, code);
    }
    const absent = createProverHost({ path: '', tmpdir: directory, timeoutMs: 100 });
    assert.equal(absent('missing-prover', 'mjs', ''), null);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('invalid prover deadlines fail before compile-unit filesystem effects', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-prover-policy-'));
  const destination = join(directory, 'not-created');
  try {
    for (const timeout of [0, -1, NaN, Infinity, 0.5, 30001, '100']) {
      assert.throws(() => runProverWith(process.execPath, '', destination, 'mjs', timeout),
        { name: 'RangeError', message: 'InvalidProverTimeout' });
    }
    assert.equal(existsSync(destination), false);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
