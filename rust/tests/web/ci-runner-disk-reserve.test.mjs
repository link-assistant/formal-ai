import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';

const script = path.resolve('scripts/free-runner-disk.sh');
function observe(overrides = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-runner-disk-control-'));
  try {
    const bin = path.join(directory, 'bin'); fs.mkdirSync(bin);
    const operations = path.join(directory, 'operations'), cleaned = path.join(directory, 'cleaned');
    const df = [
      '#!/bin/bash',
      'if [ "$1" != -Pk ]; then echo "mock hosted disk"; exit 0; fi',
      'available="$MOCK_FREE_KIB"',
      'if [ -e "$MOCK_CLEANED" ]; then available="$MOCK_CLEAN_KIB"; fi',
      'printf "Filesystem 1024-blocks Used Available Capacity Mounted on\\nmock 100000000 1 %s 1%% /\\n" "$available"',
    ].join('\n') + '\n';
    const sudo = [
      '#!/bin/bash',
      'printf "%s\\n" "$*" >> "$MOCK_OPERATIONS"',
      ': > "$MOCK_CLEANED"',
      'exit 0',
    ].join('\n') + '\n';
    fs.writeFileSync(path.join(bin,'df'),df,{mode:0o700});
    fs.writeFileSync(path.join(bin,'sudo'),sudo,{mode:0o700});
    const environment = {
      ...process.env, PATH: bin + path.delimiter + '/usr/bin:/bin',
      GITHUB_ACTIONS: 'true', RUNNER_OS: 'Linux', RUNNER_DISK_DEBUG: '0',
      RUNNER_DISK_TARGET_MIB: '', RUNNER_DISK_MIN_MIB: '6144',
      MOCK_FREE_KIB: String(14 * 1024 * 1024), MOCK_CLEAN_KIB: String(32 * 1024 * 1024),
      MOCK_CLEANED: cleaned, MOCK_OPERATIONS: operations, ...overrides,
    };
    const result = spawnSync('/bin/bash',[script],{env:environment,encoding:'utf8',timeout:5000,maxBuffer:65536});
    if (result.error) throw result.error;
    return {...result,operations:fs.existsSync(operations)?fs.readFileSync(operations,'utf8'):''};
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
}

test('cleanup refuses local execution and skips non-Linux SDK deletion', () => {
  const local = observe({GITHUB_ACTIONS:''});
  assert.equal(local.status,2);assert.equal(local.operations,'');
  const mac = observe({RUNNER_OS:'macOS'});
  assert.equal(mac.status,0);assert.equal(mac.operations,'');
  assert.match(mac.stdout,/Linux SDK cleanup skipped/u);
});
test('prebuilt jobs avoid cleanup when the ordinary hosted disk already meets their reserve', () => {
  const result = observe({RUNNER_DISK_TARGET_MIB:'12288'});
  assert.equal(result.status,0);assert.equal(result.operations,'');
  assert.match(result.stdout,/Cleanup skipped/u);
});
test('heavy build reserve still triggers bounded cleanup and sufficient disk skips it', () => {
  const low = observe(); assert.equal(low.status,0);assert.notEqual(low.operations,'');
  assert.match(low.operations,/docker image prune --all --force/u);
  const sufficient = observe({RUNNER_DISK_TARGET_MIB:'24576',MOCK_FREE_KIB:String(32*1024*1024)});
  assert.equal(sufficient.status,0);assert.equal(sufficient.operations,'');
});
for (const value of ['text','0','1023','1048577','999999999999999999']) {
  test('invalid reserve is rejected before any deletion: '+value, () => {
    const result = observe({RUNNER_DISK_TARGET_MIB:value});
    assert.equal(result.status,2);assert.equal(result.operations,'');
  });
}
test('explicit decimal reserves do not acquire octal semantics', () => {
  const result = observe({RUNNER_DISK_TARGET_MIB:'012288'});
  assert.equal(result.status,0);assert.equal(result.operations,'');
});
