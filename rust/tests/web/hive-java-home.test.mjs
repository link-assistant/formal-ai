import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { taskRuntimeDockerfile, taskRuntimeJavaHomeCommand } from '../../../scripts/build-hive-original-delivery-image.mjs';
function observe({
  executable = true,
  existingAlias = false
} = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-java-home-control-'));
  try {
    const bin = path.join(root, 'bin'),
      home = path.join(root, 'jdk'),
      alias = path.join(root, 'active'),
      calls = path.join(root, 'calls');
    fs.mkdirSync(bin);
    fs.mkdirSync(path.join(home, 'bin'), {
      recursive: true
    });
    fs.writeFileSync(path.join(home, 'bin/java'), '#!/bin/sh\nprintf "%s|%s" "$JAVA_HOME" "$*" > "$CALLS"\n', {
      mode: executable ? 0o700 : 0o600
    });
    fs.symlinkSync(path.join(home, 'bin/java'), path.join(bin, 'java'));
    fs.writeFileSync(path.join(bin, 'readlink'), '#!/bin/sh\n[ "$1" = -f ] && [ "$2" = "$EXPECTED_JAVA" ] || exit 41\nprintf "%s\\n" "$RESOLVED_JAVA"\n', {
      mode: 0o700
    });
    for (const [name, command] of [['dirname', '/usr/bin/dirname'], ['ln', '/bin/ln']]) fs.symlinkSync(command, path.join(bin, name));
    if (existingAlias) fs.writeFileSync(alias, 'caller-owned');
    const result = spawnSync('/bin/sh', ['-c', taskRuntimeJavaHomeCommand()], {
      encoding: 'utf8',
      timeout: 5000,
      env: {
        ...process.env,
        PATH: bin,
        JAVA_HOME: alias,
        CALLS: calls,
        EXPECTED_JAVA: path.join(bin, 'java'),
        RESOLVED_JAVA: path.join(home, 'bin/java')
      }
    });
    return {
      status: result.status,
      aliasTarget: fs.existsSync(alias) && fs.lstatSync(alias).isSymbolicLink() ? fs.readlinkSync(alias) : null,
      expectedHome: home,
      alias,
      call: fs.existsSync(calls) ? fs.readFileSync(calls, 'utf8') : null,
      retained: existingAlias ? fs.readFileSync(alias, 'utf8') : null
    };
  } finally {
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
}
test('runtime home derives from the resolved Java executable and is independently probed', () => {
  const result = observe();
  assert.equal(result.status, 0);
  assert.equal(result.aliasTarget, result.expectedHome);
  assert.equal(result.call, result.alias + '|-version');
});
test('unusable discovered runtime refuses before linking or probing', () => {
  const result = observe({
    executable: false
  });
  assert.notEqual(result.status, 0);
  assert.equal(result.aliasTarget, null);
  assert.equal(result.call, null);
});
test('a pre-existing runtime alias is not overwritten', () => {
  const result = observe({
    existingAlias: true
  });
  assert.notEqual(result.status, 0);
  assert.equal(result.retained, 'caller-owned');
  assert.equal(result.call, null);
});
test('task image persists the discovered JVM home before the ordinary user starts', () => {
  const text = taskRuntimeDockerfile('registry.invalid/heldout:runtime');
  assert.ok(text.includes('ENV JAVA_HOME=/usr/local/lib/formal-ai-jvm'));
  assert.ok(text.indexOf('RUN ' + taskRuntimeJavaHomeCommand()) < text.indexOf('USER box'));
  assert.ok(!text.includes('default-jdk'));
  assert.ok(!text.includes('--server=false'));
});
