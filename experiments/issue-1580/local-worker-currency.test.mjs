import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {MANIFEST, renderManifest, WORKER} from '../../scripts/wasm-worker-inputs.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const digest = value => createHash('sha256').update(value).digest('hex');
test('the local worker gate rejects changed inputs and binary bytes without compiling', {timeout: 20_000}, t => {
  const directory = mkdtempSync(join(tmpdir(), 'local-worker-currency-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const runner = 'experiments/formal_ai_subagent/local-gates.mjs';
  const gate = 'data/meta/ci-gates/check-wasm-worker-current.lino';
  const input = 'js/wasm-worker/src/lib.rs';
  for (const name of [runner, gate, input, WORKER, MANIFEST, 'scripts/wasm-worker-inputs.mjs']) {
    mkdirSync(join(directory, name, '..'), {recursive: true});
  }
  mkdirSync(join(directory, '.github/workflows'), {recursive: true});
  copyFileSync(join(root, runner), join(directory, runner));
  copyFileSync(join(root, gate), join(directory, gate));
  const checker = new URL('../../scripts/wasm-worker-inputs.mjs', import.meta.url).href;
  writeFileSync(join(directory, 'scripts/wasm-worker-inputs.mjs'),
    `import {main} from ${JSON.stringify(checker)};\nprocess.exitCode=main([...process.argv.slice(2),'--repository',${JSON.stringify(directory)}]);\n`);
  const source = 'pub fn value() {}\n', binary = 'bounded fixture worker bytes';
  writeFileSync(join(directory, input), source);
  writeFileSync(join(directory, WORKER), binary);
  writeFileSync(join(directory, MANIFEST), renderManifest({
    worker: {bytes: Buffer.byteLength(binary), sha256: digest(binary)},
    compiler: {release: 'fixture: no compiler executed', commit: 'a'.repeat(40)},
    inputs: [{path: input, sha256: digest(source)}],
  }));
  for (const args of [['init', '-q'], ['add', '.'], ['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'Worker fixture']]) {
    const result = spawnSync('git', args, {cwd: directory, encoding: 'utf8'});
    assert.equal(result.status, 0, result.stdout + result.stderr);
  }
  const run = () => spawnSync(process.execPath, [runner, '--only', 'check-wasm-worker-current'], {
    cwd: directory, encoding: 'utf8', maxBuffer: 128 * 1024,
    env: {...process.env, NODE_OPTIONS: '--max-old-space-size=128'},
  });
  let result = run();
  assert.equal(result.status, 0, result.stdout + result.stderr);
  writeFileSync(join(directory, input), 'pub fn changed_value() {}\n');
  result = run();
  assert.equal(result.status, 1, result.stdout + result.stderr);
  const log = join(directory, 'experiments/formal_ai_subagent/sandboxes/gates/check-wasm-worker-current.log');
  assert.match(readFileSync(log, 'utf8'), /changed since the worker was built/u);
  writeFileSync(join(directory, input), source);
  writeFileSync(join(directory, WORKER), 'changed worker bytes');
  result = run();
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(readFileSync(log, 'utf8'), /is not the binary the manifest records/u);
});
