import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {taskRuntimeDockerfile} from '../../../scripts/build-hive-original-delivery-image.mjs';

function observe({warm = false, installExit = 0, probeExit = 0} = {}) {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-hive-runtime-control-'));
  try {
    const calls = path.join(scratch, 'calls');
    const java = '#!/bin/sh\nprintf "java|%s\\n" "$*" >> "$CALLS"\nexit "$PROBE_EXIT"\n';
    const install = '#!/bin/sh\nprintf "install|%s\\n" "$*" >> "$CALLS"\n[ "$INSTALL_EXIT" = 0 ] || exit "$INSTALL_EXIT"\nprintf "%s" "$JAVA_STUB" > "$FAKE_BIN/java"\n/bin/chmod 700 "$FAKE_BIN/java"\n';
    const cleanup = '#!/bin/sh\nprintf "cleanup|%s\\n" "$*" >> "$CALLS"\n';
    for (const [name, source] of [['bash', install], ['rm', cleanup], ...(warm ? [['java', java]] : [])]) {
      fs.writeFileSync(path.join(scratch, name), source, {mode: 0o700});
    }
    const dockerfile = taskRuntimeDockerfile('registry.invalid/heldout:runtime');
    const command = dockerfile.split('\n').find(line => line.startsWith('RUN if ')).slice(4);
    const result = spawnSync('/bin/sh', ['-c', command], {encoding: 'utf8', timeout: 5000,
      env: {...process.env, PATH: scratch, FAKE_BIN: scratch, CALLS: calls,
        JAVA_STUB: java, INSTALL_EXIT: String(installExit), PROBE_EXIT: String(probeExit)}});
    return {status: result.status, calls: fs.existsSync(calls) ? fs.readFileSync(calls, 'utf8').trimEnd().split('\n') : [], dockerfile};
  } finally {
    fs.rmSync(scratch, {recursive: true, force: true});
  }
}

test('cold image bootstrap provides a verified runtime before the unchanged box-user solve', () => {
  const result = observe();
  assert.equal(result.status, 0);
  assert.match(result.calls[0], /^install\|.*apt-install-with-retry\.sh --no-install-recommends default-jre-headless$/u);
  assert.equal(result.calls[1], 'java|-version');
  assert.match(result.calls[2], /^cleanup\|-rf \/var\/lib\/apt\/lists\/\* \/usr\/local\/lib\/formal-ai-build$/u);
  assert.equal(result.calls.length, 3);
  assert.ok(result.dockerfile.indexOf('RUN if ') < result.dockerfile.indexOf('USER box'));
  assert.ok(result.dockerfile.includes('ENV FORMAL_AI_INSTALL_GRANT=workspace'));
  assert.ok(result.dockerfile.includes('formal-ai --version'));
  assert.ok(!/apt-install[^\n]*(?:kotlinc|scalac|rustc)/u.test(result.dockerfile));
});
test('existing runtime is probed without reinstalling', () => {
  const result = observe({warm: true});
  assert.equal(result.status, 0);
  assert.equal(result.calls[0], 'java|-version');
  assert.equal(result.calls.length, 2);
});
test('package installation failure retains its exit without probing or cleanup', () => {
  const result = observe({installExit: 37});
  assert.equal(result.status, 37);
  assert.equal(result.calls.length, 1);
  assert.ok(result.calls[0].startsWith('install|'));
});
test('an unusable runtime fails the image build and retains its diagnostic state', () => {
  const result = observe({warm: true, probeExit: 42});
  assert.equal(result.status, 42);
  assert.deepEqual(result.calls, ['java|-version']);
});
