import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {delimiter, join} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const probe = join(root, 'experiments/issue-1580/probe-budget-process-lifetime.py');
const source = process.env.FORMAL_AI_CODEX_LIFETIME_SOURCE
  ?? join(root, 'experiments/issue_859_codex_e2e/run.sh');
const declaration = readFileSync(source, 'utf8').match(/^codex_cli\(\) \{\n[\s\S]*?^\}/mu)?.[0];
assert.ok(declaration, 'exercise the actual integration client launcher');
for (const route of ['binary', 'bunx']) for (const [fixture, status] of [
  ['expired-parent-success', 0], ['expired-parent-failure', 7],
  ['term-resistant', 0], ['short-success', 0], ['short-failure', 7],
]) {
  test(`Codex launcher stops descendants before returning: ${route} ${fixture}`, {
    skip: process.platform !== 'linux' && 'fixture needs the Linux child-subreaper API',
    timeout: 15000,
  }, () => {
    const directory = mkdtempSync(join(tmpdir(), 'codex-child-regression-'));
    try {
      const adapter = join(directory, 'adapter.sh'), output = join(directory, 'observation.json');
      const setup = route === 'binary' ? 'CODEX_BIN="$1"\nshift\n'
        : 'CODEX_BIN=\nCODEX_VERSION=fixture\n';
      writeFileSync(join(directory, 'bunx'), '#!/usr/bin/env bash\nshift 3\nexec "$@"\n', {mode: 0o700});
      writeFileSync(adapter, 'shift 2\nROOT="$CODEX_FIXTURE_ROOT"\n' + setup
        + declaration + '\ncodex_cli "$@"\n');
      const observed = spawnSync('python3', [probe, adapter, output, '--case', fixture], {
        env: {...process.env, CODEX_FIXTURE_ROOT: root, PATH: directory + delimiter + process.env.PATH}, encoding: 'utf8',
        timeout: 12000, maxBuffer: 65536,
      });
      assert.equal(observed.status, 0, observed.stderr);
      const result = JSON.parse(readFileSync(output, 'utf8'));
      assert.equal(result.exitCode, status, JSON.stringify(result));
      assert.equal(result.workerRunningWhenWrapperExited, false, JSON.stringify(result));
      assert.match(result.stdout, /fixture stdout/u);
      assert.match(result.stderr, /fixture stderr/u);
      assert.ok(result.pipeClosedSeconds < 3.5, JSON.stringify(result));
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  });
}
